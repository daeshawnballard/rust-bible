// Kokoro runs in a worker on the listener's device. Only static model files
// are downloaded; Scripture is never sent to a speech service.
export const NATURAL_VOICES = [
  {voiceURI:'kokoro:af_heart',name:'Heart · Natural American',lang:'en-US'},
  {voiceURI:'kokoro:am_michael',name:'Michael · Natural American',lang:'en-US'},
  {voiceURI:'kokoro:bm_george',name:'George · Natural British',lang:'en-GB'},
];
export const DEFAULT_NARRATOR = NATURAL_VOICES[0].voiceURI;
let worker, nextId=0, generated=false;
const requests=new Map(), cache=new Map();
function destroyWorker(message) {
  worker?.terminate();worker=null;generated=false;cache.clear();
  for(const pending of requests.values()){clearTimeout(pending.timer);pending.reject(new Error(message));}
  requests.clear();
}
function inference(text,voice,onProgress) {
  const key=`${voice}\n${text}`;
  if(cache.has(key)){const hit=cache.get(key);hit.progress=onProgress||hit.progress;return hit.promise;}
  if(!worker){
    worker=new Worker(new URL('./local-voice-worker.js',import.meta.url),{type:'module'});
    worker.onmessage=({data})=>{
      const pending=requests.get(data.id);if(!pending)return;
      if(data.type==='progress'){pending.entry.progress?.(data);return;}
      if(data.type!=='audio'&&data.type!=='error')return;
      clearTimeout(pending.timer);requests.delete(data.id);
      if(data.type==='error'){cache.delete(pending.key);pending.reject(new Error(data.message||'Natural voice could not load.'));}
      else if(!(data.samples instanceof Float32Array)||!data.samples.length){cache.delete(pending.key);pending.reject(new Error('Natural voice returned no audio.'));}
      else {generated=true;pending.resolve({samples:data.samples,sampleRate:data.sampleRate||24000});}
    };
    worker.onerror=()=>destroyWorker('Natural voice could not start. Try again, or choose Device default in Playback options.');
  }
  const id=++nextId,entry={progress:onProgress};
  entry.promise=new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>destroyWorker('Natural voice took too long to load. Try again, or choose Device default in Playback options.'),180000);
    requests.set(id,{resolve,reject,timer,key,entry});
    worker.postMessage({id,type:'generate',text,voice});
  });
  cache.set(key,entry);
  while(cache.size>5)cache.delete(cache.keys().next().value);
  return entry.promise;
}
function wav(samples,sampleRate) {
  const bytes=new ArrayBuffer(44+samples.length*2),v=new DataView(bytes);
  const str=(offset,text)=>{for(let i=0;i<text.length;i++)v.setUint8(offset+i,text.charCodeAt(i));};
  str(0,'RIFF');v.setUint32(4,36+samples.length*2,true);str(8,'WAVE');str(12,'fmt ');v.setUint32(16,16,true);v.setUint16(20,1,true);v.setUint16(22,1,true);v.setUint32(24,sampleRate,true);v.setUint32(28,sampleRate*2,true);v.setUint16(32,2,true);v.setUint16(34,16,true);str(36,'data');v.setUint32(40,samples.length*2,true);
  for(let i=0;i<samples.length;i++){const x=Math.max(-1,Math.min(1,samples[i]));v.setInt16(44+i*2,x<0?x*32768:x*32767,true);}
  return new Blob([bytes],{type:'audio/wav'});
}
export class LocalNarrator extends EventTarget {
  constructor({onProgress=()=>{}}={}) {
    super();this.native=window.speechSynthesis;this.onProgress=onProgress;this.token=0;this.current=null;this.deviceOwned=false;this.objectURL=null;
    this.audio=document.createElement('audio');this.audio.hidden=true;this.audio.preload='auto';this.audio.preservesPitch=true;this.audio.dataset.naturalAudio='';document.body.append(this.audio);
    this.forwardVoices=()=>this.dispatchEvent(new Event('voiceschanged'));this.native?.addEventListener('voiceschanged',this.forwardVoices);
  }
  get supported(){return !!(window.Worker&&window.WebAssembly&&this.audio.canPlayType('audio/wav'))||!!this.native;}
  get paused(){return this.mode==='device'?!!this.native?.paused:this.audio.paused&&!!this.current;}
  getVoices(){return [...NATURAL_VOICES,...(this.native?.getVoices()||[])];}
  isLocalVoice(voice){return voice?.voiceURI?.startsWith('kokoro:')===true;}
  unlock(){
    // Start this media element during the user's click so subsequent generated
    // chunks can play after asynchronous model/inference work on mobile.
    if(this.unlocked)return;
    this.unlocked=true;this.audio.src=URL.createObjectURL(wav(new Float32Array(240),24000));
    const url=this.audio.src;this.audio.play().then(()=>{if(!this.current)this.audio.pause();URL.revokeObjectURL(url);}).catch(()=>{URL.revokeObjectURL(url);});
  }
  prefetch(text,voice){if(text&&this.isLocalVoice(voice))inference(text,voice.voiceURI.slice(7)).catch(()=>{});}
  speak(utterance){
    this.current=utterance;const token=++this.token;
    if(!this.isLocalVoice(utterance.voice)){
      this.mode='device';
      if(!this.native||!window.SpeechSynthesisUtterance){utterance.onerror?.({error:'Device speech is unavailable. Choose a natural voice.'});return;}
      const native=new SpeechSynthesisUtterance(utterance.text);native.lang=utterance.lang;native.rate=utterance.rate;native.voice=utterance.voice||null;
      for(const event of ['start','end','pause','resume','error'])native[`on${event}`]=e=>{if(token===this.token)utterance[`on${event}`]?.(e);};
      this.deviceOwned=true;this.native.speak(native);return;
    }
    this.mode='local';this.audio.dataset.engine='kokoro';
    let started=false;this.pending=true;
    inference(utterance.text,utterance.voice.voiceURI.slice(7),progress=>{if(token===this.token)this.onProgress(progress);}).then(({samples,sampleRate})=>{
      if(token!==this.token)return;
      this.pending=false;
      if(this.objectURL)URL.revokeObjectURL(this.objectURL);
      this.objectURL=URL.createObjectURL(wav(samples,sampleRate));const url=this.objectURL;
      this.audio.onplaying=()=>{if(token!==this.token||started||this.audio.currentSrc!==url)return;started=true;this.audio.dataset.playback='playing';utterance.onstart?.();};
      this.audio.onended=()=>{if(token===this.token&&this.audio.currentSrc===url){this.audio.dataset.playback='ended';utterance.onend?.();}};
      this.audio.onerror=()=>{if(token===this.token&&this.audio.currentSrc===url)utterance.onerror?.({error:'Natural audio could not play. Try Listen again or select Device default.'});};
      this.audio.src=url;this.audio.playbackRate=utterance.rate||1;
      this.audio.dataset.samples=String(samples.length);this.audio.dataset.sampleRate=String(sampleRate);
      return this.audio.play();
    }).catch(error=>{if(token===this.token)utterance.onerror?.({error:error.message||'Natural voice could not load.'});});
  }
  pause(){if(this.mode==='device')this.native?.pause();else {this.audio.pause();this.audio.dataset.playback='paused';this.current?.onpause?.();}}
  resume(){if(this.mode==='device')this.native?.resume();else {const token=this.token;this.audio.play().then(()=>{if(token===this.token){this.audio.dataset.playback='playing';this.current?.onresume?.();}}).catch(error=>{if(token===this.token)this.current?.onerror?.({error:error.message});});}}
  cancel(){if(this.pending&&!generated)destroyWorker('Loading canceled.');this.pending=false;this.token++;this.current=null;this.audio.onplaying=this.audio.onended=this.audio.onerror=null;this.audio.pause();this.audio.removeAttribute('src');this.audio.load();this.audio.dataset.playback='stopped';if(this.objectURL){URL.revokeObjectURL(this.objectURL);this.objectURL=null;}if(this.deviceOwned){this.native?.cancel();if(this.native?.paused)this.native.resume();this.deviceOwned=false;}}
  destroy(){this.cancel();this.native?.removeEventListener('voiceschanged',this.forwardVoices);this.audio.remove();}
}
