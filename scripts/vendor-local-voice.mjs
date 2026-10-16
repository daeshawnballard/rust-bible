#!/usr/bin/env node

// Rebuild the local Kokoro browser runtime and small voice bins from pinned
// sources. All source and npm dependencies stay in a fresh retained temp dir;
// npm lifecycle scripts are disabled. The GPL-licensed eSpeak engine is loaded
// as a separately version-pinned npm-CDN module, never embedded in runtime.js.

import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const VENDOR = path.join(ROOT, 'web/vendor/kokoro');
const VOICES = path.join(VENDOR, 'voices');
const LICENSES = path.join(VENDOR, 'licenses');
const MODEL_ID = 'onnx-community/Kokoro-82M-v1.0-ONNX';
const MODEL_REVISION = '1939ad2a8e416c0acfeecc08a694d14ef25f2231';
const KOKORO_SOURCE = {
  url: 'https://codeload.github.com/hexgrad/kokoro/tar.gz/664c76a704021239ba59c84dcbaa4d3dece01fe9',
  commit: '664c76a704021239ba59c84dcbaa4d3dece01fe9',
  sha256: '7fb00d418195be678663f7844380abaf791aba4749137ed402d0da197bfa00cb',
};
const PHONEMIZER = {
  version: '1.2.1',
  cdnUrl: 'https://cdn.jsdelivr.net/npm/phonemizer@1.2.1/+esm',
  npmUrl: 'https://registry.npmjs.org/phonemizer/-/phonemizer-1.2.1.tgz',
  integrity: 'sha512-v0KJ4mi2T4Q7eJQ0W15Xd4G9k4kICSXE8bpDeJ8jisL4RyJhNWsweKTOi88QXFc4r4LZlz5jVL5lCHhkpdT71A==',
  sourceCommit: '6835144b7ee9043129222549c1ed2f6a27216278',
};
const LICENSE_SOURCES = {
  'Apache-2.0.txt': 'https://www.apache.org/licenses/LICENSE-2.0.txt',
  'MIT.txt': 'https://raw.githubusercontent.com/microsoft/onnxruntime/v1.22.0/LICENSE',
  'GPL-3.0.txt': 'https://raw.githubusercontent.com/spdx/license-list-data/v3.26.0/text/GPL-3.0-only.txt',
  'BSD-3-Clause.txt': 'https://raw.githubusercontent.com/spdx/license-list-data/v3.26.0/text/BSD-3-Clause.txt',
  'ISC.txt': 'https://raw.githubusercontent.com/spdx/license-list-data/v3.26.0/text/ISC.txt',
};
const ONNX_SOURCE = {
  url: 'https://registry.npmjs.org/onnxruntime-web/-/onnxruntime-web-1.22.0-dev.20250409-89f8206ba4.tgz',
  integrity: 'sha512-0uS76OPgH0hWCPrFKlL8kYVV7ckM7t/36HfbgoFw6Nd0CZVVbQC4PkrR8mBX8LtNUFZO25IQBqV2Hx2ho3FlbQ==',
  version: '1.22.0-dev.20250409-89f8206ba4',
};
const VOICE_HASHES = {
  af_heart: 'd583ccff3cdca2f7fae535cb998ac07e9fcb90f09737b9a41fa2734ec44a8f0b',
  am_michael: '1d1f21dd8da39c30705cd4c75d039d265e9bc4a2a93ed09bc9e1b1225eb95ba1',
  bm_george: 'c4b235a4c1f2cd3b939fed08b899ce9385638b763f7b73a59616c4fc9bd6c9bc',
};

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

async function fetchBytes(url, expectedSha256) {
  const response = await fetch(url, { redirect: 'follow' });
  if (!response.ok) throw new Error(`Download failed (${response.status}): ${url}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  if (expectedSha256 && sha256(bytes) !== expectedSha256) throw new Error(`SHA-256 mismatch: ${url}`);
  return bytes;
}

function extractTgz(bytes, member) {
  const result = spawnSync('tar', ['-xOzf', '-', member], {
    input: bytes,
    encoding: null,
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0) throw new Error(`Could not extract ${member}: ${result.stderr.toString()}`);
  return result.stdout;
}

function replaceExactly(source, before, after, label, expected = 1) {
  const count = source.split(before).length - 1;
  if (count !== expected) throw new Error(`${label}: expected ${expected} source match(es), found ${count}`);
  return source.split(before).join(after);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: 'utf8', maxBuffer: 8 * 1024 * 1024, ...options });
  if (result.status !== 0) throw new Error(`${command} failed (${result.status ?? result.signal}): ${result.stderr || result.stdout}`);
  return result.stdout;
}

await Promise.all([
  mkdir(VENDOR, { recursive: true }),
  mkdir(VOICES, { recursive: true }),
  mkdir(LICENSES, { recursive: true }),
]);

// Keep build inputs and intermediates for inspection; this script intentionally
// has no cleanup or deletion path.
const BUILD_DIR = await mkdtemp(path.join(tmpdir(), 'kokoro-local-voice-'));
const archivePath = path.join(BUILD_DIR, 'kokoro-source.tar.gz');
const archive = await fetchBytes(KOKORO_SOURCE.url, KOKORO_SOURCE.sha256);
await writeFile(archivePath, archive);
run('tar', ['-xzf', archivePath, '-C', BUILD_DIR], { timeout: 120_000 });
const SOURCE_DIR = path.join(BUILD_DIR, `kokoro-${KOKORO_SOURCE.commit}`, 'kokoro.js');

const phonemizePath = path.join(SOURCE_DIR, 'src/phonemize.js');
let phonemize = await readFile(phonemizePath, 'utf8');
phonemize = replaceExactly(
  phonemize,
  'import { phonemize as espeakng } from "phonemizer";',
  `const PHONEMIZER_CDN = "${PHONEMIZER.cdnUrl}";\nlet espeakngPromise;\nasync function getEspeakng() { espeakngPromise ??= import(PHONEMIZER_CDN).then((module) => module.phonemize); return espeakngPromise; }`,
  'external pinned Phonemizer import',
);
phonemize = replaceExactly(
  phonemize,
  '  const ps = (await Promise.all(sections.map(async ({ match, text }) => (match ? text : (await espeakng(text, lang)).join(" "))))).join("");',
  '  const espeakng = await getEspeakng();\n  const ps = (await Promise.all(sections.map(async ({ match, text }) => (match ? text : (await espeakng(text, lang)).join(" "))))).join("");',
  'lazy external phonemizer load',
);
await writeFile(phonemizePath, phonemize);

const voicesPath = path.join(SOURCE_DIR, 'src/voices.js');
let voices = await readFile(voicesPath, 'utf8');
voices = replaceExactly(voices, 'import path from "path";\nimport fs from "fs/promises";\n', '', 'browser-only voices source');
const voiceStart = voices.indexOf('async function getVoiceFile(id) {');
const voiceEnd = voices.indexOf('\n\nconst VOICE_CACHE', voiceStart);
if (voiceStart < 0 || voiceEnd < 0 || voices.indexOf('async function getVoiceFile(id) {', voiceStart + 1) >= 0) {
  throw new Error('Could not isolate the upstream voice-file loader.');
}
voices = `${voices.slice(0, voiceStart)}async function getVoiceFile(id) {\n  const url = new URL(\`./voices/${'${id}'}.bin\`, import.meta.url);\n  const response = await fetch(url);\n  if (!response.ok) throw new Error(\`Could not load local voice ${'${id}'}: ${'${response.status}'}\`);\n  return await response.arrayBuffer();\n}${voices.slice(voiceEnd)}`;
await writeFile(voicesPath, voices);

const kokoroPath = path.join(SOURCE_DIR, 'src/kokoro.js');
let kokoro = await readFile(kokoroPath, 'utf8');
kokoro = replaceExactly(
  kokoro,
  'static async from_pretrained(model_id, { dtype = "fp32", device = null, progress_callback = null } = {}) {\n    const model = StyleTextToSpeech2Model.from_pretrained(model_id, { progress_callback, dtype, device });\n    const tokenizer = AutoTokenizer.from_pretrained(model_id, { progress_callback });',
  'static async from_pretrained(model_id, { dtype = "fp32", device = null, progress_callback = null, revision = "1939ad2a8e416c0acfeecc08a694d14ef25f2231" } = {}) {\n    const model = StyleTextToSpeech2Model.from_pretrained(model_id, { progress_callback, dtype, device, revision });\n    const tokenizer = AutoTokenizer.from_pretrained(model_id, { progress_callback, revision });',
  'revision-pinned model loader',
);
kokoro = replaceExactly(
  kokoro,
  'export const env = {\n  set wasmPaths(value) {\n    hf.backends.onnx.wasm.wasmPaths = value;\n  },\n  get wasmPaths() {\n    return hf.backends.onnx.wasm.wasmPaths;\n  },\n};',
  'export { hf as env };',
  'direct ONNX environment export',
);
await writeFile(kokoroPath, kokoro);

run('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: SOURCE_DIR, timeout: 120_000 });
const runtimeBuildPath = path.join(BUILD_DIR, 'runtime.js');
const metafilePath = path.join(BUILD_DIR, 'esbuild-meta.json');
const esbuildPath = path.join(SOURCE_DIR, 'node_modules/esbuild/bin/esbuild');
const transformersSource = path.join(SOURCE_DIR, 'node_modules/@huggingface/transformers/src/transformers.js');
run(process.execPath, [
  esbuildPath,
  'src/kokoro.js', '--bundle', '--format=esm', '--platform=browser', '--target=es2022', '--minify',
  `--alias:@huggingface/transformers=${transformersSource}`,
  `--outfile=${runtimeBuildPath}`, `--metafile=${metafilePath}`,
], { cwd: SOURCE_DIR, timeout: 120_000 });
const runtime = await readFile(runtimeBuildPath);
const runtimeText = runtime.toString('utf8');
if (!runtimeText.includes(PHONEMIZER.cdnUrl)) throw new Error('Built runtime is missing the pinned external Phonemizer CDN URL.');
if (runtimeText.includes('eSpeakNG') || runtimeText.includes('espeakng.worker')) throw new Error('Built runtime unexpectedly contains the eSpeak worker payload.');
await writeFile(path.join(VENDOR, 'runtime.js'), runtime);

const onnxTgz = await fetchBytes(ONNX_SOURCE.url);
const onnxIntegrity = `sha512-${createHash('sha512').update(onnxTgz).digest('base64')}`;
if (onnxIntegrity !== ONNX_SOURCE.integrity) throw new Error('ONNX Runtime NPM integrity mismatch.');
for (const member of ['package/dist/ort-wasm-simd-threaded.mjs', 'package/dist/ort-wasm-simd-threaded.wasm']) {
  await writeFile(path.join(VENDOR, path.basename(member)), extractTgz(onnxTgz, member));
}

for (const [voice, digest] of Object.entries(VOICE_HASHES)) {
  const url = `https://huggingface.co/${MODEL_ID}/resolve/${MODEL_REVISION}/voices/${voice}.bin`;
  await writeFile(path.join(VOICES, `${voice}.bin`), await fetchBytes(url, digest));
}
for (const [name, url] of Object.entries(LICENSE_SOURCES)) {
  await writeFile(path.join(LICENSES, name), await fetchBytes(url));
}

const files = [
  'runtime.js', 'ort-wasm-simd-threaded.mjs', 'ort-wasm-simd-threaded.wasm',
  'voices/af_heart.bin', 'voices/am_michael.bin', 'voices/bm_george.bin',
  ...Object.keys(LICENSE_SOURCES).map((name) => `licenses/${name}`),
];
const manifest = {
  model: { id: MODEL_ID, revision: MODEL_REVISION, inference: 'public Hugging Face files; no API key or inference endpoint' },
  worker: null,
  build: {
    sourceArchive: KOKORO_SOURCE.url,
    sourceCommit: KOKORO_SOURCE.commit,
    sourceArchiveSha256: KOKORO_SOURCE.sha256,
    command: 'esbuild --bundle --format=esm --platform=browser --target=es2022 --minify',
    esbuild: '0.25.2 (pinned by upstream package-lock.json)',
    transformers: '3.5.1 (pinned by upstream package-lock.json)',
    install: 'npm ci --ignore-scripts --no-audit --no-fund',
  },
  phonemizer: {
    package: `phonemizer@${PHONEMIZER.version}`,
    npmUrl: PHONEMIZER.npmUrl,
    integrity: PHONEMIZER.integrity,
    runtimeCdnUrl: PHONEMIZER.cdnUrl,
    sourceCommit: PHONEMIZER.sourceCommit,
    externalized: true,
    espeakNgOriginalSourceRevision: null,
    note: 'The eSpeak NG payload is fetched by the browser from the pinned npm CDN URL and is not embedded in the self-hosted runtime.js. The published Phonemizer source snapshot does not identify the original eSpeak NG revision or provide corresponding C source/build inputs.',
  },
  packages: {
    'onnxruntime-web': { version: ONNX_SOURCE.version, url: ONNX_SOURCE.url, integrity: ONNX_SOURCE.integrity },
  },
  files: Object.fromEntries(await Promise.all(files.map(async (name) => {
    const bytes = await readFile(path.join(VENDOR, name));
    return [name, { bytes: bytes.length, sha256: sha256(bytes) }];
  }))),
};
const workerBytes = await readFile(path.join(ROOT, 'web/local-voice-worker.js'));
manifest.worker = { path: 'web/local-voice-worker.js', bytes: workerBytes.length, sha256: sha256(workerBytes) };
await writeFile(path.join(VENDOR, 'provenance.json'), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(JSON.stringify({ ...manifest, retainedBuildDirectory: BUILD_DIR }, null, 2));
