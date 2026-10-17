const VOICES = new Set(['af_heart', 'am_michael']);
const MAX_MEDIA_BYTES = 12 * 1024 * 1024;
const MAX_CUES_BYTES = 256 * 1024;
const MAX_ENTRIES = 2;

export function hostedAudioUrl(baseURL, voice, book, chapter, extension) {
  if (!VOICES.has(voice) || !Number.isInteger(book) || book < 1 || book > 66
    || !Number.isInteger(chapter) || chapter < 1
    || !['mp3', 'json'].includes(extension)) throw new TypeError('Invalid hosted narration reference.');
  return new URL(`/audio/v1/${voice}/${book}/${chapter}.${extension}`, baseURL).href;
}

export function validateHostedCues(document, expected) {
  if (!document || typeof document !== 'object' || Array.isArray(document)) return 'invalid_cues_document';
  const required = ['version','voice','book','chapter','sample_rate','duration','corpus_sha256',
    'audio_sha256','verses','peaks'];
  if (required.some(key => !(key in document))) return 'missing_cues_field';
  if (document.version !== 1 || document.voice !== expected.voice
    || document.book !== expected.book || document.chapter !== expected.chapter) return 'cues_path_mismatch';
  if (document.sample_rate !== 24000 || !Number.isFinite(document.duration) || document.duration <= 0) {
    return 'invalid_audio_metadata';
  }
  const hash = /^[a-f0-9]{64}$/i;
  if (!hash.test(document.corpus_sha256) || !hash.test(document.audio_sha256)) return 'invalid_metadata_hash';
  if (!Array.isArray(document.verses) || !document.verses.length) return 'invalid_verse_cues';
  let end = 0;
  for (let index = 0; index < document.verses.length; index += 1) {
    const cue = document.verses[index];
    if (!cue || cue.verse !== index + 1 || !Number.isFinite(cue.start) || !Number.isFinite(cue.end)
      || cue.start < end || cue.end <= cue.start || cue.end > document.duration
      || typeof cue.text_sha256 !== 'string' || !hash.test(cue.text_sha256)) return 'invalid_verse_cue';
    end = cue.end;
  }
  if (!Array.isArray(document.peaks) || ![512,1024].includes(document.peaks.length)
    || document.peaks.some(peak => !Number.isFinite(peak) || peak < 0 || peak > 1)) {
    return 'invalid_waveform_peaks';
  }
  return null;
}

export function cropPeakBins(peaks, duration, start, end) {
  if (!Array.isArray(peaks) || !peaks.length || !Number.isFinite(duration) || duration <= 0
    || !Number.isFinite(start) || !Number.isFinite(end) || start < 0 || end <= start) return [];
  const first = Math.max(0, Math.min(peaks.length - 1, Math.floor(start / duration * peaks.length)));
  const afterLast = Math.max(first + 1, Math.min(peaks.length, Math.ceil(end / duration * peaks.length)));
  return peaks.slice(first, afterLast);
}

export async function sha256Hex(input, cryptoImpl = globalThis.crypto) {
  if (!cryptoImpl?.subtle?.digest) throw new Error('SHA-256 verification is unavailable in this browser.');
  const bytes = input instanceof ArrayBuffer ? input : input.buffer.slice(input.byteOffset, input.byteOffset + input.byteLength);
  const digest = await cryptoImpl.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map(value => value.toString(16).padStart(2, '0')).join('');
}

export async function verifyVerseText(cue, text, cryptoImpl = globalThis.crypto) {
  if (!cue || typeof text !== 'string' || !text.trim()) throw new Error('The hosted verse cue has no matching text.');
  const digest = await sha256Hex(new TextEncoder().encode(text), cryptoImpl);
  if (digest.toLowerCase() !== cue.text_sha256.toLowerCase()) {
    throw new Error('Hosted narration verse text does not match this KJV passage.');
  }
  return true;
}

export async function resolveVerseCues(chapterAsset, records, cryptoImpl = globalThis.crypto) {
  if (!chapterAsset || chapterAsset.missing) return new Map();
  const routes = new Map();
  for (const record of records) {
    const cue = chapterAsset.metadata.verses[Number(record.verse) - 1];
    if (!cue || cue.verse !== Number(record.verse)) throw new Error('Hosted narration has no cue for this verse.');
    await verifyVerseText(cue, record.text, cryptoImpl);
    routes.set(Number(record.verse), {
      kind:'hosted',
      assetKey:chapterAsset.key,
      asset:chapterAsset,
      cue,
      peaks:cropPeakBins(chapterAsset.metadata.peaks,chapterAsset.metadata.duration,cue.start,cue.end),
      voice:chapterAsset.metadata.voice,
      url:chapterAsset.url,
    });
  }
  return routes;
}

async function limitedBytes(response, maxBytes, description) {
  const rawLength = response.headers?.get?.('content-length');
  const length = rawLength === null || rawLength === undefined ? null : Number(rawLength);
  if (length !== null && (!Number.isSafeInteger(length) || length < 0 || length > maxBytes)) {
    throw new Error(`${description} exceeds its size limit.`);
  }
  if (response.body?.getReader) {
    const reader = response.body.getReader();
    const chunks = [];
    let size = 0;
    try {
      while (true) {
        const {done, value} = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > maxBytes) {
          try { await reader.cancel(); } catch {}
          throw new Error(`${description} exceeds its size limit.`);
        }
        chunks.push(value);
      }
    } finally {
      reader.releaseLock?.();
    }
    if (!size) throw new Error(`${description} is empty or exceeds its size limit.`);
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes.buffer;
  }
  const bytes = await response.arrayBuffer();
  if (!bytes.byteLength || bytes.byteLength > maxBytes) throw new Error(`${description} is empty or exceeds its size limit.`);
  return bytes;
}

export class HostedNarrationStore {
  constructor({baseURL = globalThis.location?.origin || 'http://localhost', fetchImpl = globalThis.fetch,
    cryptoImpl = globalThis.crypto, maxBytes = MAX_MEDIA_BYTES, maxEntries = MAX_ENTRIES} = {}) {
    if (typeof fetchImpl !== 'function') throw new TypeError('Fetch is required for hosted narration.');
    this.baseURL = baseURL;
    this.fetch = fetchImpl;
    this.crypto = cryptoImpl;
    this.maxBytes = Number.isFinite(maxBytes) ? Math.max(1, Math.min(MAX_MEDIA_BYTES, Math.floor(maxBytes))) : MAX_MEDIA_BYTES;
    this.maxEntries = Number.isFinite(maxEntries) ? Math.max(1, Math.min(MAX_ENTRIES, Math.floor(maxEntries))) : MAX_ENTRIES;
    this.entries = new Map();
  }

  has(key) {
    const entry = this.entries.get(key);
    return Boolean(entry?.result && !entry.result.missing && entry.result.objectURL);
  }

  activate(key) {
    for (const entry of this.entries.values()) entry.active = entry.key === key;
  }

  load(book, chapter, voice, {priority = 'active'} = {}) {
    const metadataURL = hostedAudioUrl(this.baseURL, voice, book, chapter, 'json');
    const audioURL = hostedAudioUrl(this.baseURL, voice, book, chapter, 'mp3');
    const key = `${voice}:${book}:${chapter}`;
    const active = priority === 'active';
    if (active) this.activate(key);
    const existing = this.entries.get(key);
    if (existing) {
      existing.active ||= active;
      existing.prefetched &&= !active;
      this.touch(existing);
      return existing.promise;
    }

    const entry = {key,metadataURL,audioURL,active,prefetched:!active,status:'pending',controller:new AbortController(),result:null,promise:null};
    this.entries.set(key, entry);
    entry.promise = this.fetchChapter(entry, {book,chapter,voice}).then(result => {
      if (this.entries.get(key) === entry) {
        entry.status = 'ready';
        entry.result = result;
        this.trim();
      } else if (result.objectURL) {
        URL.revokeObjectURL(result.objectURL);
      }
      return result;
    }).catch(error => {
      if (this.entries.get(key) === entry) this.entries.delete(key);
      throw error;
    });
    return entry.promise;
  }

  prefetch(book, chapter, voice) {
    return this.load(book, chapter, voice, {priority:'prefetch'});
  }

  cancelPrefetch() {
    for (const [key, entry] of this.entries) {
      if (!entry.prefetched || entry.active) continue;
      if (entry.status === 'pending') entry.controller.abort();
      if (entry.result?.objectURL) URL.revokeObjectURL(entry.result.objectURL);
      this.entries.delete(key);
    }
  }

  cancelPending() {
    for (const [key, entry] of this.entries) {
      if (entry.status !== 'pending') continue;
      entry.controller.abort();
      this.entries.delete(key);
    }
    this.cancelPrefetch();
    this.activate(null);
  }

  clear() {
    this.cancelPending();
    for (const entry of this.entries.values()) {
      if (entry.result?.objectURL) URL.revokeObjectURL(entry.result.objectURL);
    }
    this.entries.clear();
  }

  touch(entry) {
    if (this.entries.get(entry.key) !== entry) return;
    this.entries.delete(entry.key);
    this.entries.set(entry.key, entry);
  }

  trim() {
    while (this.entries.size > this.maxEntries) {
      const candidate = [...this.entries.values()].find(entry => !entry.active);
      if (!candidate) return;
      if (candidate.status === 'pending') candidate.controller.abort();
      if (candidate.result?.objectURL) URL.revokeObjectURL(candidate.result.objectURL);
      this.entries.delete(candidate.key);
    }
  }

  async fetchChapter(entry, expected) {
    const {signal} = entry.controller;
    const metadataResponse = await this.fetch(entry.metadataURL, {
      method:'GET',cache:'no-store',credentials:'same-origin',headers:{Accept:'application/json'},signal,
    });
    if (metadataResponse.status === 404) return {missing:true,reason:'cues',key:entry.key,url:entry.audioURL};
    if (!metadataResponse.ok) throw new Error(`Hosted narration metadata request failed (${metadataResponse.status}).`);
    const metadataBytes = await limitedBytes(metadataResponse, MAX_CUES_BYTES, 'Hosted narration metadata');
    let metadata;
    try { metadata = JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(metadataBytes)); }
    catch { throw new Error('Hosted narration metadata is not valid JSON.'); }
    const invalid = validateHostedCues(metadata, expected);
    if (invalid) throw new Error(`Hosted narration metadata is invalid (${invalid}).`);

    const audioResponse = await this.fetch(entry.audioURL, {
      method:'GET',cache:'no-store',credentials:'same-origin',headers:{Accept:'audio/mpeg'},signal,
    });
    if (audioResponse.status === 404) return {missing:true,reason:'audio',key:entry.key,url:entry.audioURL,metadata};
    if (!audioResponse.ok) throw new Error(`Hosted narration audio request failed (${audioResponse.status}).`);
    const audioBytes = await limitedBytes(audioResponse, this.maxBytes, 'Hosted narration audio');
    const headerHash = audioResponse.headers?.get?.('x-content-sha256');
    if (headerHash && headerHash.toLowerCase() !== metadata.audio_sha256.toLowerCase()) {
      throw new Error('Hosted narration audio digest does not match its metadata.');
    }
    const actualHash = await sha256Hex(audioBytes, this.crypto);
    if (actualHash.toLowerCase() !== metadata.audio_sha256.toLowerCase()) {
      throw new Error('Hosted narration audio failed its SHA-256 check.');
    }
    const blob = new Blob([audioBytes],{type:'audio/mpeg'});
    const objectURL = URL.createObjectURL(blob);
    return {missing:false,key:entry.key,url:entry.audioURL,objectURL,blob,metadata};
  }
}

export {MAX_MEDIA_BYTES as HOSTED_AUDIO_MAX_BYTES, MAX_ENTRIES as HOSTED_AUDIO_CACHE_CHAPTERS};
