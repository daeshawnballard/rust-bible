// Kokoro runs in a worker on the listener's device. Only static model files
// are downloaded; Scripture is never sent to a speech service.
export const NATURAL_VOICES = [
  {voiceURI:'kokoro:af_heart',name:'Heart · Natural American',lang:'en-US'},
  {voiceURI:'kokoro:am_michael',name:'Michael · Natural American',lang:'en-US'},
];
export const DEFAULT_NARRATOR = NATURAL_VOICES[0].voiceURI;
export const MAX_SPEECH_SEGMENT_LENGTH = 220;

const SAMPLE_RATE = 24000;
const CACHE_LIMIT = 3;
const CACHE_SAMPLE_LIMIT = SAMPLE_RATE * 60;

let worker;
let nextId = 0;
let activeRequest = null;
let cachedSamples = 0;
const requests = new Map();
const requestQueue = [];
const cache = new Map();

export function splitSpeech(text) {
  const source = String(text || '');
  const segments = [];
  let start = 0;

  while (start < source.length) {
    let end = Math.min(start + MAX_SPEECH_SEGMENT_LENGTH, source.length);
    if (end < source.length) {
      const bounded = source.slice(start, end);
      const sentenceBoundary = /[.!?;:](?:["'”’»\])}]*)?(?:\s+|$)/gu;
      let match;
      let preferredEnd = -1;
      while ((match = sentenceBoundary.exec(bounded)) !== null) {
        preferredEnd = start + match.index + match[0].length;
      }

      if (preferredEnd > start) {
        end = preferredEnd;
      } else {
        let whitespace = -1;
        for (let index = 0; index < bounded.length; index += 1) {
          if (/\s/u.test(bounded[index])) whitespace = index;
        }
        if (whitespace >= 0) end = start + whitespace + 1;
      }
    }

    // Keep surrogate pairs intact without dropping either code unit.
    if (end < source.length && end > start
      && /[\uD800-\uDBFF]/u.test(source[end - 1])
      && /[\uDC00-\uDFFF]/u.test(source[end])) end -= 1;
    if (end <= start) end = Math.min(start + MAX_SPEECH_SEGMENT_LENGTH, source.length);
    segments.push(source.slice(start, end));
    start = end;
  }

  if (segments.join('') !== source) throw new Error('Could not preserve the complete reading text.');
  return segments;
}

function removeCacheEntry(entry) {
  if (cache.get(entry.key) !== entry) return;
  cache.delete(entry.key);
  cachedSamples = Math.max(0, cachedSamples - (entry.cachedSamples || 0));
  entry.cachedSamples = 0;
}

function trimCache() {
  while (cache.size > CACHE_LIMIT || cachedSamples > CACHE_SAMPLE_LIMIT) {
    const oldest = [...cache.values()].find(entry => entry.status === 'ready');
    if (!oldest) break;
    removeCacheEntry(oldest);
  }
}

function touchCache(entry) {
  if (cache.get(entry.key) !== entry) return;
  cache.delete(entry.key);
  cache.set(entry.key, entry);
}

function settleError(entry, message, name = 'AbortError') {
  clearTimeout(entry.timer);
  entry.timer = null;
  requests.delete(entry.id);
  removeCacheEntry(entry);
  entry.status = 'failed';
  const error = new Error(message);
  error.name = name;
  entry.reject(error);
}

function abortActiveRequest(message, name = 'AbortError') {
  const entry = activeRequest;
  if (!entry) return;
  const instance = worker;
  activeRequest = null;
  worker = null;
  instance?.terminate();
  settleError(entry, message, name);
}

function cancelQueuedRequest(entry, message) {
  const index = requestQueue.indexOf(entry);
  if (index < 0) return;
  requestQueue.splice(index, 1);
  settleError(entry, message);
}

function getWorker() {
  if (worker) return worker;
  const instance = new Worker(new URL('./local-voice-worker.js', import.meta.url), {type:'module'});
  worker = instance;
  instance.onmessage = ({data}) => {
    if (worker !== instance) return;
    const entry = requests.get(data.id);
    if (!entry || activeRequest !== entry) return;
    if (data.type === 'progress') {
      entry.progress?.(data);
      return;
    }
    if (data.type !== 'audio' && data.type !== 'error') return;

    clearTimeout(entry.timer);
    entry.timer = null;
    requests.delete(entry.id);
    activeRequest = null;
    if (data.type === 'error') {
      settleError(entry, data.message || 'Natural voice could not load.', 'Error');
      pumpRequests();
      return;
    }
    if (!(data.samples instanceof Float32Array) || !data.samples.length) {
      settleError(entry, 'Natural voice returned no audio.', 'Error');
      pumpRequests();
      return;
    }

    const telemetry = data.telemetry || {};
    const result = {
      samples: data.samples,
      sampleRate: data.sampleRate || SAMPLE_RATE,
      cacheSource: telemetry.workerCacheHit ? 'worker-cache' : 'generated',
      workerCacheHit: Boolean(telemetry.workerCacheHit),
      generationMs: Number.isFinite(telemetry.generationMs) ? telemetry.generationMs : null,
      modelWaitMs: Number.isFinite(telemetry.modelWaitMs) ? telemetry.modelWaitMs : null,
      segmentId: entry.segmentId || null,
      textCodeUnits: entry.text.length,
    };
    entry.status = 'ready';
    entry.cachedSamples = result.samples.length <= Math.min(CACHE_SAMPLE_LIMIT, entry.maxCacheSamples)
      ? result.samples.length : 0;
    if (entry.cachedSamples) cachedSamples += entry.cachedSamples;
    else removeCacheEntry(entry);
    trimCache();
    entry.resolve(result);
    pumpRequests();
  };
  instance.onerror = () => {
    if (worker !== instance) return;
    abortActiveRequest('Natural voice could not start. Try again or choose the other natural voice.', 'Error');
    pumpRequests();
  };
  return instance;
}

function pumpRequests() {
  if (activeRequest) return;
  let entry = requestQueue.shift();
  while (entry && entry.status !== 'queued') entry = requestQueue.shift();
  if (!entry) return;

  activeRequest = entry;
  entry.status = 'running';
  let instance;
  try {
    instance = getWorker();
  } catch (error) {
    activeRequest = null;
    settleError(entry,error?.message || 'Natural voice could not start.','Error');
    pumpRequests();
    return;
  }
  entry.timer = setTimeout(() => {
    if (activeRequest !== entry) return;
    abortActiveRequest('Natural voice took too long to load. Try again or choose the other natural voice.', 'Error');
    pumpRequests();
  }, 180000);
  try {
    instance.postMessage({id:entry.id,type:'generate',text:entry.text,voice:entry.voice,
      segmentId:entry.segmentId,maxCacheSamples:entry.maxCacheSamples});
  } catch (error) {
    abortActiveRequest(error?.message || 'Natural voice could not start.','Error');
    pumpRequests();
  }
}

function inference(text, voice, onProgress, {priority='active',segmentId=null,maxCacheSamples=CACHE_SAMPLE_LIMIT}={}) {
  const cacheSamples = Math.max(0, Math.min(CACHE_SAMPLE_LIMIT,
    Number.isFinite(maxCacheSamples) ? Math.floor(maxCacheSamples) : CACHE_SAMPLE_LIMIT));
  const key = `${voice}\n${text}`;
  const existing = cache.get(key);
  if (existing) {
    const wasReady = existing.status === 'ready';
    if (priority === 'speculative' && !wasReady) {
      existing.maxCacheSamples = Math.min(existing.maxCacheSamples, cacheSamples);
    }
    touchCache(existing);
    if (onProgress) existing.progress = onProgress;
    if (priority === 'active' && !wasReady) {
      existing.priority = 'active';
      if (activeRequest?.priority === 'speculative' && activeRequest !== existing) {
        abortActiveRequest('Speculative narration was superseded.');
      }
      requestQueue.sort((left, right) => Number(right.priority === 'active') - Number(left.priority === 'active'));
      if (!activeRequest && !wasReady) pumpRequests();
      if (activeRequest === existing) return existing.promise.then(result => ({...result,
        cacheSource:'inflight',segmentId:segmentId || result.segmentId}));
    }
    return existing.promise.then(result => {
      if (priority === 'speculative' && wasReady && result.samples.length > cacheSamples) removeCacheEntry(existing);
      return {...result,cacheSource:wasReady ? 'client-cache' : 'inflight',segmentId:segmentId || result.segmentId};
    });
  }

  if (priority === 'active' && activeRequest?.priority === 'speculative') {
    abortActiveRequest('Speculative narration was superseded.');
  }

  const id = ++nextId;
  const entry = {id,key,text,voice,priority,segmentId,progress:onProgress,status:'queued',timer:null,
    cachedSamples:0,maxCacheSamples:cacheSamples};
  entry.promise = new Promise((resolve, reject) => { entry.resolve = resolve; entry.reject = reject; });
  cache.set(key, entry);
  requests.set(id, entry);
  requestQueue.push(entry);
  requestQueue.sort((left, right) => Number(right.priority === 'active') - Number(left.priority === 'active'));
  pumpRequests();
  return entry.promise;
}

function wav(samples, sampleRate) {
  const bytes = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(bytes);
  const write = (offset, text) => { for (let index=0; index<text.length; index+=1) view.setUint8(offset+index,text.charCodeAt(index)); };
  write(0,'RIFF');view.setUint32(4,36+samples.length*2,true);write(8,'WAVE');write(12,'fmt ');
  view.setUint32(16,16,true);view.setUint16(20,1,true);view.setUint16(22,1,true);
  view.setUint32(24,sampleRate,true);view.setUint32(28,sampleRate*2,true);view.setUint16(32,2,true);view.setUint16(34,16,true);
  write(36,'data');view.setUint32(40,samples.length*2,true);
  for (let index=0; index<samples.length; index+=1) {
    const value = Math.max(-1,Math.min(1,samples[index]));
    view.setInt16(44+index*2,value<0?value*32768:value*32767,true);
  }
  return new Blob([bytes],{type:'audio/wav'});
}

function segmentTelemetry(result, utterance, phase) {
  return {
    phase,
    segmentId:utterance.segmentId || result.segmentId || null,
    segmentIndex:Number.isInteger(utterance.segmentIndex) ? utterance.segmentIndex : null,
    segmentCount:Number.isInteger(utterance.segmentCount) ? utterance.segmentCount : null,
    voice:utterance.voice?.voiceURI || null,
    cacheSource:result.cacheSource || 'unknown',
    workerCacheHit:Boolean(result.workerCacheHit),
    generationMs:result.generationMs,
    modelWaitMs:result.modelWaitMs,
    textCodeUnits:result.textCodeUnits,
    sampleRate:result.sampleRate,
    sampleCount:result.samples.length,
    durationSeconds:result.samples.length / result.sampleRate,
    at:Date.now(),
  };
}

function reportSegment(result, utterance, phase) {
  const detail = segmentTelemetry(result, utterance, phase);
  window.dispatchEvent(new CustomEvent('bible:voice-segment', {detail}));
  return detail;
}

function mediaMetadataReady(media, expectedURL) {
  if (media.readyState >= 1 && media.currentSrc === expectedURL) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      media.removeEventListener('loadedmetadata', ready);
      media.removeEventListener('error', failed);
      media.removeEventListener('abort', failed);
    };
    const ready = () => {
      if (media.currentSrc !== expectedURL) return;
      cleanup();
      resolve();
    };
    const failed = () => { cleanup(); reject(new Error('Hosted narration metadata could not load.')); };
    media.addEventListener('loadedmetadata', ready, {once:true});
    media.addEventListener('error', failed, {once:true});
    media.addEventListener('abort', failed, {once:true});
    if (media.readyState >= 1 && media.currentSrc === expectedURL) ready();
  });
}

export class LocalNarrator extends EventTarget {
  constructor({onProgress=()=>{}}={}) {
    super();
    this.onProgress = onProgress;
    this.token = 0;
    this.current = null;
    this.objectURL = null;
    this.hostedSession = null;
    this.hostedStorePromise = import('./hosted-voice.js').then(({HostedNarrationStore}) => {
      // Use the module directory so mounted readers keep their URL prefix.
      this.hostedStore = new HostedNarrationStore();
      return this.hostedStore;
    });
    this.hostedStorePromise.catch(() => {});
    this.audio = document.createElement('audio');
    this.audio.hidden = true;
    this.audio.preload = 'auto';
    this.audio.preservesPitch = true;
    this.audio.dataset.naturalAudio = '';
    document.body.append(this.audio);
    for (const event of ['timeupdate','ratechange','seeked']) {
      this.audio.addEventListener(event, () => this.updateHostedSession());
    }
  }

  get supported() {
    return Boolean(window.fetch && this.audio.canPlayType('audio/mpeg'))
      || Boolean(window.Worker && window.WebAssembly && this.audio.canPlayType('audio/wav'));
  }
  get paused() { return this.audio.paused && Boolean(this.current); }
  get duration() {
    if (this.mode === 'hosted' && this.current?.route?.cue) {
      const cue = this.current.route.cue;
      return Math.max(0, cue.end - cue.start);
    }
    return Number.isFinite(this.audio.duration) ? this.audio.duration : 0;
  }
  get currentTime() {
    const time = Number.isFinite(this.audio.currentTime) ? this.audio.currentTime : 0;
    if (this.mode === 'hosted' && this.current?.route?.cue) {
      const cue = this.current.route.cue;
      return Math.max(0, Math.min(cue.end - cue.start, time - cue.start));
    }
    return time;
  }
  get seekable() { return ['local','hosted'].includes(this.mode) && !this.pending && !!this.current && this.duration > 0; }
  getVoices() { return NATURAL_VOICES; }
  isLocalVoice(voice) { return NATURAL_VOICES.some(item => item.voiceURI === voice?.voiceURI); }

  async prepareVerses(records, voiceURI, {priority='active'}={}) {
    const voice = NATURAL_VOICES.find(item => item.voiceURI === voiceURI);
    if (!voice || !Array.isArray(records) || !records.length) throw new Error('A supported natural voice and verses are required.');
    const first = records[0];
    const book = Number(first.bookIndex) + 1;
    const chapter = Number(first.chapter);
    if (!Number.isInteger(book) || book < 1 || book > 66 || !Number.isInteger(chapter) || chapter < 1
      || records.some(record => Number(record.bookIndex) + 1 !== book || Number(record.chapter) !== chapter)) {
      throw new Error('Hosted narration needs one known Bible book and chapter.');
    }
    const {HostedNarrationStore, resolveVerseCues} = await import('./hosted-voice.js');
    const store = await this.hostedStorePromise;
    const asset = await store.load(book, chapter, voiceURI.slice(7), {priority});
    if (asset.missing) {
      return new Map(records.map(record => [Number(record.verse), {
        kind:'local',fallbackFromHosted:true,missingAsset:asset.reason,voice,
      }]));
    }
    const hosted = await resolveVerseCues(asset, records);
    for (const route of hosted.values()) route.voice = voice;
    return hosted;
  }

  async prefetchChapter(bookIndex, chapter, voiceURI) {
    const voice = NATURAL_VOICES.find(item => item.voiceURI === voiceURI);
    const book = Number(bookIndex) + 1;
    if (!voice || !Number.isInteger(book) || book < 1 || book > 66 || !Number.isInteger(Number(chapter))) return null;
    const store = await this.hostedStorePromise;
    return store.prefetch(book, Number(chapter), voiceURI.slice(7));
  }

  hasHostedRoute(route) {
    return Boolean(route?.assetKey && this.hostedStore?.has(route.assetKey));
  }

  cancelPrefetch() {
    void this.hostedStorePromise.then(store => store.cancelPrefetch()).catch(() => {});
    for (const entry of [...requestQueue]) {
      if (entry.priority === 'speculative') cancelQueuedRequest(entry,'Narration prefetch canceled.');
    }
    if (activeRequest?.priority === 'speculative') {
      abortActiveRequest('Narration prefetch canceled.');
      pumpRequests();
    }
  }

  setRate(rate) {
    const value = Number(rate);
    if (Number.isFinite(value) && value >= 0.5 && value <= 2) {
      this.audio.defaultPlaybackRate = value;
      this.audio.playbackRate = value;
    }
  }
  setVolume(volume) {
    const value = Number(volume);
    if (Number.isFinite(value)) this.audio.volume = Math.max(0,Math.min(1,value));
  }
  seek(seconds) {
    const value = Number(seconds);
    if (!this.seekable || !Number.isFinite(value)) return false;
    const offset = Math.max(0,Math.min(this.duration,value));
    this.audio.currentTime = this.mode === 'hosted' ? this.current.route.cue.start + offset : offset;
    return true;
  }

  unlock() {
    // Start this media element during the user's click so subsequent generated
    // chunks can play after asynchronous model/inference work on mobile.
    if (this.unlocked) return;
    this.unlocked = true;
    this.audio.src = URL.createObjectURL(wav(new Float32Array(240),SAMPLE_RATE));
    const url = this.audio.src;
    this.audio.play().then(() => { if (!this.current) this.audio.pause(); URL.revokeObjectURL(url); })
      .catch(() => URL.revokeObjectURL(url));
  }

  prefetch(text, voice, metadata={}) {
    if (!text || !this.isLocalVoice(voice)) return;
    return inference(text,voice.voiceURI.slice(7),null,{priority:'speculative',segmentId:metadata.segmentId,
      maxCacheSamples:metadata.maxCacheSamples}).catch(() => null);
  }

  speak(utterance) {
    if (utterance.route?.kind === 'hosted') return this.speakHosted(utterance);
    if (utterance.route?.kind !== 'local') {
      utterance.onerror?.({error:'Hosted narration was not resolved for this verse.'});
      return;
    }
    this.current = utterance;
    const token = ++this.token;
    if (this.mode === 'hosted') {
      this.audio.pause();
      this.audio.onplaying = this.audio.onended = this.audio.onerror = null;
      this.audio.removeAttribute('src');
      this.audio.load();
    }
    this.mode = 'local';
    this.hostedSession = null;
    this.audio.dataset.engine = 'kokoro';
    let started = false;
    this.pending = true;
    inference(utterance.text,utterance.voice.voiceURI.slice(7),progress => {
      if (token === this.token) this.onProgress(progress);
    },{priority:'active',segmentId:utterance.segmentId}).then(result => {
      if (token !== this.token) return;
      this.pending = false;
      if (this.objectURL) URL.revokeObjectURL(this.objectURL);
      this.objectURL = URL.createObjectURL(wav(result.samples,result.sampleRate));
      const url = this.objectURL;
      this.audio.onplaying = () => {
        if (token !== this.token || started || this.audio.currentSrc !== url) return;
        started = true;
        this.audio.dataset.playback = 'playing';
        reportSegment(result,utterance,'started');
        utterance.onstart?.();
      };
      this.audio.onended = () => {
        if (token === this.token && this.audio.currentSrc === url) {
          this.audio.dataset.playback = 'ended';
          utterance.onend?.();
        }
      };
      this.audio.onerror = () => {
        if (token === this.token && this.audio.currentSrc === url) {
          utterance.onerror?.({error:'Natural audio could not play. Try Listen again or choose the other natural voice.'});
        }
      };
      this.audio.src = url;
      this.audio.playbackRate = utterance.rate || 1;
      this.audio.dataset.samples = String(result.samples.length);
      this.audio.dataset.sampleRate = String(result.sampleRate);
      this.dispatchEvent(new CustomEvent('audiobuffer',{detail:{samples:result.samples,sampleRate:result.sampleRate,
        duration:result.samples.length/result.sampleRate,text:utterance.text}}));
      utterance.onbuffer?.(result);
      return Promise.resolve(utterance.beforePlay).then(() => {
        if (token !== this.token) return undefined;
        this.setRate(utterance.rate || 1);
        return this.audio.play();
      });
    }).catch(error => {
      if (token === this.token) utterance.onerror?.({error:error.message||'Natural voice could not load.'});
    });
  }

  speakHosted(utterance) {
    const route = utterance.route;
    const previous = this.hostedSession;
    clearTimeout(previous?.cutoffTimer);
    const continuing = this.mode === 'hosted' && previous?.route.assetKey === route.assetKey
      && this.audio.currentSrc === route.asset.objectURL && !this.audio.paused;
    this.current = utterance;
    const token = ++this.token;
    this.mode = 'hosted';
    this.pending = !continuing;
    this.hostedStore?.activate(route.assetKey);
    const session = {token,route,utterance,started:false,ended:false};
    this.hostedSession = session;
    this.audio.dataset.engine = 'hosted';
    this.audio.playbackRate = utterance.rate || 1;
    delete this.audio.dataset.samples;
    delete this.audio.dataset.sampleRate;
    this.dispatchEvent(new CustomEvent('audiobuffer', {detail:{peaks:route.peaks,
      duration:route.cue.end-route.cue.start,cueStart:route.cue.start,cueEnd:route.cue.end}}));
    utterance.onbuffer?.({hosted:true,duration:route.cue.end-route.cue.start,peaks:route.peaks});
    this.audio.onplaying = () => {
      if (this.hostedSession === session) this.updateHostedSession();
    };
    this.audio.onended = () => {
      if (this.hostedSession !== session || session.ended) return;
      if (this.audio.currentTime + 0.05 >= route.cue.end) this.endHostedSession(session);
      else utterance.onerror?.({error:'Hosted narration ended before this verse cue finished.'});
    };
    this.audio.onerror = () => {
      if (this.hostedSession === session) utterance.onerror?.({error:'Hosted narration could not play.'});
    };
    if (continuing) {
      this.updateHostedSession();
      return;
    }
    this.audio.pause();
    this.audio.src = route.asset.objectURL;
    this.audio.load();
    this.audio.dataset.playback = 'loading';
    return Promise.resolve(utterance.beforePlay).then(() => {
      if (token !== this.token) return undefined;
      return mediaMetadataReady(this.audio, route.asset.objectURL);
    }).then(() => {
      if (token !== this.token) return undefined;
      this.audio.currentTime = route.cue.start;
      this.pending = false;
      // Loading a new media resource resets playbackRate to its default.
      // Apply the user's rate after metadata is ready, including verse one.
      this.setRate(utterance.rate || 1);
      return this.audio.play();
    }).catch(error => {
      if (token === this.token) utterance.onerror?.({error:error.message || 'Hosted narration could not start.'});
    });
  }

  updateHostedSession() {
    const session = this.hostedSession;
    // Seeking fires timeupdate even while paused; it must not advance or restart narration.
    if (this.mode !== 'hosted' || !session || session.ended || this.audio.paused) return;
    const {cue} = session.route;
    const time = this.audio.currentTime;
    if (!session.started && time + 0.01 >= cue.start && time < cue.end) {
      session.started = true;
      this.audio.dataset.playback = 'playing';
      const detail = {book:session.route.asset.metadata.book,chapter:session.route.asset.metadata.chapter,
        verse:cue.verse,voice:session.route.voice.voiceURI.slice(7),url:session.route.url,
        cue:{start:cue.start,end:cue.end},currentTime:time,mediaTime:time};
      window.dispatchEvent(new CustomEvent('bible:hosted-verse',{detail}));
      session.utterance.onstart?.();
    }
    if (session.started && time + 0.005 >= cue.end) {
      this.endHostedSession(session);
    } else if (session.started && session.utterance.stopAtCueEnd) {
      // timeupdate can arrive a quarter-second apart. A selected range needs
      // its own deadline so the next verse's opening is not played meanwhile.
      clearTimeout(session.cutoffTimer);
      const remaining=(cue.end-time)/this.audio.playbackRate;
      session.cutoffTimer=setTimeout(()=>{
        if(this.hostedSession===session&&!session.ended&&!this.audio.paused)this.updateHostedSession();
      },Math.max(4,remaining*1000));
    }
  }

  endHostedSession(session) {
    if (this.hostedSession !== session || session.ended || this.audio.dataset.playback === 'paused') return;
    session.ended = true;
    clearTimeout(session.cutoffTimer);
    if(session.utterance.stopAtCueEnd)this.audio.pause();
    this.audio.dataset.playback = 'ended';
    session.utterance.onend?.();
  }

  pause({notify=true}={}) {
    clearTimeout(this.hostedSession?.cutoffTimer);
    this.audio.pause();
    this.audio.dataset.playback = 'paused';
    if (notify) this.current?.onpause?.();
  }

  resume() {
    const session = this.hostedSession;
    if (this.mode === 'hosted' && session && !session.ended
      && (this.audio.ended || this.audio.currentTime + 0.02 >= session.route.cue.end)) {
      // play() on an ended chapter restarts the entire file. Finish this cue
      // deliberately so Resume advances to the next verse or chapter instead.
      this.audio.dataset.playback = 'playing';
      this.endHostedSession(session);
      return;
    }
    const token = this.token;
    this.audio.play().then(() => {
      if (token === this.token) { this.audio.dataset.playback = 'playing'; this.current?.onresume?.(); }
    }).catch(error => { if (token === this.token) this.current?.onerror?.({error:error.message}); });
  }

  cancel() {
    if (activeRequest || requestQueue.length) {
      for (const entry of [...requestQueue]) cancelQueuedRequest(entry,'Loading canceled.');
      if (activeRequest) abortActiveRequest('Loading canceled.');
    }
    this.pending = false;
    this.token += 1;
    clearTimeout(this.hostedSession?.cutoffTimer);
    this.hostedSession = null;
    void this.hostedStorePromise.then(store => store.cancelPending()).catch(() => {});
    this.current = null;
    this.audio.onplaying = this.audio.onended = this.audio.onerror = null;
    this.audio.pause();
    this.audio.removeAttribute('src');
    this.audio.load();
    delete this.audio.dataset.samples;
    delete this.audio.dataset.sampleRate;
    this.audio.dataset.playback = 'stopped';
    if (this.objectURL) { URL.revokeObjectURL(this.objectURL); this.objectURL = null; }
  }

  destroy() {
    this.cancel();
    void this.hostedStorePromise.then(store => store.clear()).catch(() => {});
    this.audio.remove();
  }
}
