import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {
  HostedNarrationStore,
  cropPeakBins,
  hostedAudioUrl,
  resolveVerseCues,
  validateHostedCues,
} from '../web/hosted-voice.js';
import {NATURAL_VOICES} from '../web/local-voice.js';

const digest = value => createHash('sha256').update(value).digest('hex');
const voice = 'af_heart';
const verseText = 'The Lord is my shepherd.';
const audioBytes = new TextEncoder().encode('bounded fake mp3 bytes');
const chapter = (book=1, chapterNo=1, bytes=audioBytes) => ({
  version:1,
  voice,
  book,
  chapter:chapterNo,
  sample_rate:24000,
  duration:2,
  corpus_sha256:digest('corpus'),
  audio_sha256:digest(bytes),
  verses:[{verse:1,start:0.25,end:1.5,text_sha256:digest(verseText)}],
  peaks:Array.from({length:512},(_,index) => index / 512),
});
const expectedRecord = {bookIndex:0,chapter:1,verse:1,text:verseText};

assert.deepEqual(NATURAL_VOICES.map(item => item.voiceURI), ['kokoro:af_heart','kokoro:am_michael']);
assert.equal(hostedAudioUrl('https://example.test',voice,1,2,'mp3'),
  'https://example.test/audio/v1/af_heart/1/2.mp3');
assert.equal(hostedAudioUrl('https://example.test/projects/bibleinrust/',voice,1,2,'mp3'),
  'https://example.test/projects/bibleinrust/audio/v1/af_heart/1/2.mp3');
assert.throws(() => hostedAudioUrl('https://example.test','bm_george',1,1,'mp3'), TypeError);
assert.deepEqual(cropPeakBins([0,1,2,3],4,1,3), [1,2]);
assert.equal(validateHostedCues(chapter(),{voice,book:1,chapter:1}), null);
assert.equal(validateHostedCues({...chapter(),book:2},{voice,book:1,chapter:1}), 'cues_path_mismatch');

const calls = [];
const fetchChapter = async (url, options) => {
  calls.push({url:String(url),options});
  if (String(url).endsWith('.json')) return new Response(JSON.stringify(chapter()),{status:200});
  return new Response(audioBytes,{status:200,headers:{'x-content-sha256':digest(audioBytes)}});
};
const store = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:fetchChapter});
const asset = await store.load(1,1,voice);
assert.equal(asset.missing,false);
assert(calls.every(call => call.options.method === 'GET' && call.options.cache === 'no-store'));
assert(calls.every(call => call.options.credentials === 'same-origin'));
const routes = await resolveVerseCues(asset,[expectedRecord]);
assert.equal(routes.get(1).kind,'hosted');
assert.equal(routes.get(1).cue.start,0.25);
assert(routes.get(1).peaks.length > 0);
await assert.rejects(resolveVerseCues(asset,[{...expectedRecord,text:'different passage'}]),/does not match/);
store.clear();

const metadata404 = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:async () => new Response('',{status:404})});
assert.equal((await metadata404.load(1,1,voice)).missing,true);
metadata404.clear();

let audioRequest = false;
const audio404 = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:async url => {
  if (String(url).endsWith('.json')) return new Response(JSON.stringify(chapter()),{status:200});
  audioRequest = true;
  return new Response('',{status:404});
}});
assert.equal((await audio404.load(1,1,voice)).missing,true);
assert.equal(audioRequest,true);
audio404.clear();

const unavailable = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:async () => new Response('',{status:503})});
await assert.rejects(unavailable.load(1,1,voice),/503/);
unavailable.clear();

const wrongBytes = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:async url => {
  if (String(url).endsWith('.json')) return new Response(JSON.stringify(chapter()),{status:200});
  return new Response(new TextEncoder().encode('tampered audio'),{status:200});
}});
await assert.rejects(wrongBytes.load(1,1,voice),/SHA-256/);
wrongBytes.clear();

const oversizedBytes = new HostedNarrationStore({baseURL:'https://example.test',maxBytes:4,fetchImpl:async url => {
  if (String(url).endsWith('.json')) return new Response(JSON.stringify(chapter(1,1,new Uint8Array([1,2,3,4,5]))),{status:200});
  return new Response(new Uint8Array([1,2,3,4,5]),{status:200});
}});
await assert.rejects(oversizedBytes.load(1,1,voice),/size limit/);
oversizedBytes.clear();

const boundedCache = new HostedNarrationStore({baseURL:'https://example.test',fetchImpl:async url => {
  const [,chapterNo,extension] = /\/([0-9]+)\.(json|mp3)$/.exec(String(url));
  if (extension === 'json') return new Response(JSON.stringify(chapter(1,Number(chapterNo))),{status:200});
  return new Response(audioBytes,{status:200});
}});
for (let chapterNo=1; chapterNo<=3; chapterNo+=1) await boundedCache.load(1,chapterNo,voice);
assert.equal(boundedCache.entries.size,2);
assert.equal(boundedCache.has(`${voice}:1:3`),true);
boundedCache.clear();

console.log('PASS: hosted URL and cue validation, verse matching, no-store requests, 404-only miss classification, digest/size rejection, and two-chapter cache bound');
