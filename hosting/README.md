# Narration storage Worker

This is a small Cloudflare Worker transport for the existing Rust/WASM site. Static files are copied from the existing `dist/` build into `dist/client`; the Worker serves those requests through the `ASSETS` binding and serves chapter audio/cues from the `BUCKET` R2 binding. It does not generate narration or call an inference provider.

`npm run build:hosted` first runs `bash scripts/build-wasm.sh`, then builds the Worker and client assets in a fresh temporary staging directory. It preserves the current `dist/` tree and copies complete outputs into `dist/client`, `dist/server`, and `dist/.openai` without clearing existing directories. The staging path is printed and retained for recovery.

The upload tool expects files laid out as `<audio-root>/<voice>/<book>/<chapter>.mp3` and `.json`. It verifies the local cue contract and audio digest, imports audio before cues, then checks HEAD metadata and streams each stored object back through SHA-256. For example:

```sh
node hosting/upload-narration.mjs https://<site-origin> /path/to/narration
```

Enter the upload token at the hidden terminal prompt, or pipe it through stdin. The token is never accepted as a command-line argument or read from a file. The deployed Worker requires the `NARRATION_UPLOAD_TOKEN` secret and an R2 binding named `BUCKET`; keep both configured through the host's secret and storage controls.

For a long-running full-corpus upload, use the opt-in watcher and choose a private receipt path outside the narration directory:

```sh
node hosting/upload-narration.mjs --watch https://<site-origin> /path/to/narration --receipt /private/path/narration-progress.json
```

The watcher checks the live health endpoint, scans for complete chapter pairs every 15 seconds, waits for each pair to remain stable across two scans, and uploads pairs sequentially. It retries a failed pair at most three times with backoff, then leaves it marked `FAIL` until its source changes. The receipt is mode `0600`, contains no token or source paths, and records only safe object keys, hashes, byte counts, timestamps, and statuses. A second watcher using the same receipt is rejected while the first is active. When a prior receipt names the same known Worker revision, a resumed run re-reads its prior objects from the current server before counting them again; otherwise it uploads and reads them back again. `COMPLETE` requires fresh readbacks for all 2,378 voice/chapter pairs; missing or failed pairs remain listed in `missing_pair_keys`. Remote uploads require HTTPS; plain HTTP is accepted only for `localhost`, `127.0.0.1`, or `::1` local testing.

Public URLs are `/audio/v1/{voice}/{book}/{chapter}.mp3` and `.json`, using one-based numeric book and chapter identifiers. R2 keys are `kokoro-v1/{voice}/{book}/{chapter}.mp3` and `.json`. MP3 objects use a one-hour cache lifetime and ETags; cue JSON is `no-store` while chapter audio is still changing. Uploads are limited to those two voices and valid KJV chapter ranges.
