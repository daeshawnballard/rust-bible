# Third-party notices

This directory contains a patched browser build of Kokoro and Transformers.js, ONNX Runtime Web WASM assets, and three voice embedding files. Complete license texts are grouped by SPDX identifier under `licenses/`.

| Component | Version or revision | License | Included use and source |
| --- | --- | --- | --- |
| Kokoro.js | 1.2.1 source, commit `664c76a704021239ba59c84dcbaa4d3dece01fe9` | Apache-2.0 | Patched browser inference wrapper; source archive and SHA-256 are recorded in `provenance.json`. |
| Hugging Face Transformers.js | 3.5.1 | Apache-2.0 | Bundled model and tokenizer runtime; resolved by Kokoro's pinned `package-lock.json`. |
| Phonemizer.js | 1.2.1 | Apache-2.0 | Loaded externally from the exact [jsDelivr npm URL](https://cdn.jsdelivr.net/npm/phonemizer@1.2.1/+esm); [NPM package](https://www.npmjs.com/package/phonemizer/v/1.2.1) and exact [source commit](https://github.com/xenova/phonemizer.js/tree/6835144b7ee9043129222549c1ed2f6a27216278). It is not embedded in `runtime.js`. |
| eSpeak NG payload served through Phonemizer.js | Original source revision not identified in the Phonemizer.js package | GPL-3.0 | Loaded from the external Phonemizer CDN package at runtime; not self-hosted in this repository. See the corresponding-source note below and `licenses/GPL-3.0.txt`. |
| ONNX Runtime Web | 1.22.0-dev.20250409-89f8206ba4 | MIT | WASM runtime and JavaScript loader; [NPM package](https://www.npmjs.com/package/onnxruntime-web/v/1.22.0-dev.20250409-89f8206ba4), NPM integrity `sha512-0uS76OPgH0hWCPrFKlL8kYVV7ckM7t/36HfbgoFw6Nd0CZVVbQC4PkrR8mBX8LtNUFZO25IQBqV2Hx2ho3FlbQ==`. |
| ONNX Runtime Common | 1.22.0-dev.20250409-89f8206ba4 | MIT | JavaScript support component of ONNX Runtime Web. |
| `@huggingface/jinja` | 0.4.1 dependency line | Apache-2.0 | Template support bundled through Transformers.js; [NPM package](https://www.npmjs.com/package/@huggingface/jinja/v/0.4.1). |
| `flatbuffers` | ONNX Runtime Web dependency | Apache-2.0 | ONNX schema/runtime dependency. |
| `long` | ONNX Runtime Web dependency | Apache-2.0 | Integer support dependency. |
| `platform` | ONNX Runtime Web dependency | MIT | Runtime platform detection dependency. |
| `protobufjs` | ONNX Runtime Web dependency | BSD-3-Clause | Protobuf support dependency. |
| `guid-typescript` | ONNX Runtime Web dependency | ISC | Identifier support dependency. |
| Kokoro-82M model and voice embeddings | HF revision `1939ad2a8e416c0acfeecc08a694d14ef25f2231` | Apache-2.0 | Model weights are fetched from [the pinned model repository](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/tree/1939ad2a8e416c0acfeecc08a694d14ef25f2231); `af_heart.bin`, `am_michael.bin`, and `bm_george.bin` are vendored from the same revision. |

`Apache-2.0.txt`, `MIT.txt`, `GPL-3.0.txt`, `BSD-3-Clause.txt`, and `ISC.txt` contain complete license texts. `provenance.json` records SHA-256 values and byte sizes for the vendored runtime, WASM pair, voice files, and license texts, plus the external package pins.

## eSpeak NG corresponding source

Phonemizer.js 1.2.1 describes itself as Apache-2.0 and documents that it uses eSpeak NG. Its published source at commit `6835144b7ee9043129222549c1ed2f6a27216278` includes the Emscripten worker output and data, but does not identify the original eSpeak NG C-source revision or include the C source and build scripts. The eSpeak NG upstream project is at [github.com/espeak-ng/espeak-ng](https://github.com/espeak-ng/espeak-ng), but no exact revision can be tied to this binary from the published artifact. The repository runtime externalizes Phonemizer and does not redistribute its Emscripten payload; the browser still requests that package from jsDelivr. The GPL-3.0 text is present, but that does not establish corresponding-source compliance for the CDN-served build. Keep this provenance status unresolved unless exact source/build materials are obtained or the dependency is replaced.
