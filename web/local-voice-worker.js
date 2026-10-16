import { KokoroTTS, env } from './vendor/kokoro/runtime.js';

const MODEL_ID = 'onnx-community/Kokoro-82M-v1.0-ONNX';
const MODEL_REVISION = '1939ad2a8e416c0acfeecc08a694d14ef25f2231';
const SAMPLE_RATE = 24000;
const MAX_CHUNK_LENGTH = 180;
const CACHE_LIMIT = 2;
const CACHE_SAMPLE_LIMIT = SAMPLE_RATE * 60 * 2;

env.backends.onnx.wasm.wasmPaths = {
  mjs: new URL('./vendor/kokoro/ort-wasm-simd-threaded.mjs', import.meta.url),
  wasm: new URL('./vendor/kokoro/ort-wasm-simd-threaded.wasm', import.meta.url),
};
env.backends.onnx.wasm.numThreads = 1;

let modelPromise;
let inferenceQueue = Promise.resolve();
let cachedSamples = 0;
const audioCache = new Map();

function postError(id, error) {
  const message = error instanceof Error ? error.message : String(error);
  self.postMessage({ id, type: 'error', message: message || 'Natural voice inference failed.' });
}

function sendProgress(id, progress) {
  const file = typeof progress?.file === 'string' ? progress.file.split('/').pop() : '';
  self.postMessage({
    id,
    type: 'progress',
    message: file ? `Loading ${file}` : 'Downloading natural voice model…',
    loaded: Number.isFinite(progress?.loaded) ? progress.loaded : undefined,
    total: Number.isFinite(progress?.total) ? progress.total : undefined,
  });
}

function getModel(id) {
  if (!modelPromise) {
    modelPromise = KokoroTTS.from_pretrained(MODEL_ID, {
      dtype: 'q8',
      device: 'wasm',
      revision: MODEL_REVISION,
      progress_callback: (progress) => sendProgress(id, progress),
    }).catch((error) => {
      modelPromise = undefined;
      throw error;
    });
  }
  return modelPromise;
}

function splitTextLosslessly(text) {
  const chunks = [];
  let start = 0;
  while (start < text.length) {
    let end = Math.min(start + MAX_CHUNK_LENGTH, text.length);
    if (end < text.length) {
      const bounded = text.slice(start, end);
      const sentenceBoundary = /[.!?;:](?:["'”’»\])}]*)?(?:\s|$)/g;
      let match;
      let preferredEnd = -1;
      while ((match = sentenceBoundary.exec(bounded)) !== null) preferredEnd = start + match.index + match[0].length;
      if (preferredEnd > start) {
        end = preferredEnd;
      } else {
        let anyWhitespace = -1;
        for (let index = 0; index < bounded.length; index += 1) {
          if (/\s/u.test(bounded[index])) anyWhitespace = index;
        }
        if (anyWhitespace >= 0) {
          end = start + anyWhitespace + 1;
          while (end < text.length && end < start + MAX_CHUNK_LENGTH && /\s/.test(text[end])) end += 1;
        }
      }
      if (end > start && end < text.length && /[\uD800-\uDBFF]/.test(text[end - 1]) && /[\uDC00-\uDFFF]/.test(text[end])) end += 1;
    }
    chunks.push(text.slice(start, end));
    start = end;
  }
  if (chunks.join('') !== text) throw new Error('Could not preserve the complete reading text.');
  return chunks;
}

function cacheKey(text, voice) {
  return `${voice}\u0000${text}`;
}

function cacheGet(key) {
  const entry = audioCache.get(key);
  if (!entry) return undefined;
  audioCache.delete(key);
  audioCache.set(key, entry);
  return entry;
}

function cachePut(key, samples, sampleRate) {
  if (samples.length > CACHE_SAMPLE_LIMIT) return;
  const copy = samples.slice();
  const prior = audioCache.get(key);
  if (prior) cachedSamples -= prior.samples.length;
  audioCache.delete(key);
  audioCache.set(key, { samples: copy, sampleRate });
  cachedSamples += copy.length;
  while (audioCache.size > CACHE_LIMIT || cachedSamples > CACHE_SAMPLE_LIMIT) {
    const [oldestKey, oldest] = audioCache.entries().next().value;
    audioCache.delete(oldestKey);
    cachedSamples -= oldest.samples.length;
  }
}

function postAudio(id, samples, sampleRate) {
  const transferable = samples.slice();
  self.postMessage({ id, type: 'audio', samples: transferable, sampleRate }, [transferable.buffer]);
}

async function generate(id, text, voice) {
  if (typeof text !== 'string' || !text.trim()) throw new Error('Reading text is empty.');
  if (!['af_heart', 'am_michael', 'bm_george'].includes(voice)) throw new Error('The selected natural voice is unavailable.');
  const key = cacheKey(text, voice);
  const hit = cacheGet(key);
  if (hit) {
    postAudio(id, hit.samples, hit.sampleRate);
    return;
  }

  const tts = await getModel(id);
  const chunks = splitTextLosslessly(text);
  const pieces = [];
  let totalSamples = 0;
  let sampleRate = SAMPLE_RATE;
  for (let index = 0; index < chunks.length; index += 1) {
    self.postMessage({
      id,
      type: 'progress',
      message: chunks.length > 1 ? `Generating section ${index + 1} of ${chunks.length}` : 'Generating natural voice…',
    });
    const audio = await tts.generate(chunks[index], { voice });
    const samples = audio?.audio;
    const rate = audio?.sampling_rate ?? audio?.sampleRate ?? SAMPLE_RATE;
    if (!(samples instanceof Float32Array) || samples.length === 0) throw new Error('Natural voice returned no audio.');
    if (!Number.isFinite(rate) || rate !== SAMPLE_RATE || (index > 0 && rate !== sampleRate)) {
      throw new Error(`Unexpected natural voice sample rate: ${rate}.`);
    }
    sampleRate = rate;
    pieces.push(samples);
    totalSamples += samples.length;
  }

  const combined = new Float32Array(totalSamples);
  let offset = 0;
  for (const piece of pieces) {
    combined.set(piece, offset);
    offset += piece.length;
  }
  cachePut(key, combined, sampleRate);
  postAudio(id, combined, sampleRate);
}

async function handle(message) {
  const { id, type } = message ?? {};
  if (type === 'prepare') {
    await getModel(id);
    self.postMessage({ id, type: 'ready' });
    return;
  }
  if (type !== 'generate') throw new Error('Unsupported worker request.');
  await generate(id, message.text, message.voice);
}

self.onmessage = (event) => {
  const message = event.data;
  inferenceQueue = inferenceQueue
    .then(() => handle(message))
    .catch((error) => postError(message?.id, error));
};
