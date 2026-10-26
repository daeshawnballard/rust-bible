# Narration storage Worker

This is a small Cloudflare Worker transport for the existing Rust/WASM site. Core files are copied from the `dist/` build into `dist/client`; the Worker serves those requests through `ASSETS` and serves chapter audio/cues from `BUCKET`. Large public artwork, fonts, and vendor files use the same URLs through content-addressed Site R2 storage. It does not generate narration or call an inference provider.

`npm run build:hosted` first runs `bash scripts/build-wasm.sh`, then builds the Worker and core client in fresh staging. It moves the complete static preview into that retained staging directory and writes the compact deployment to `dist/client`, `dist/server`, and `dist/.openai`. No output directory is deleted; both staging and preview paths are printed for recovery.

The build catalogs each stored asset against its exact committed `web/` bytes. Dirty or untracked assets fail the build. Push that same commit to GitHub and Sites before saving the version. On a cold read, the Worker fetches only the cataloged path from the commit-pinned public GitHub source, checks byte length and SHA-256, then stores verified bytes under `public-assets/v1/<sha256>`. Subsequent reads use Site R2. There is no arbitrary upstream proxy or public write endpoint. Until an asset is stored, delivery depends on the public source being reachable; run the verifier after publication to populate storage and verify all assets before acceptance.

```sh
npm run test:assets
node hosting/verify-public-assets.mjs https://daeshawnballard.com/projects/bibleinrust/ <saved-source-sha> /private/path/assets-check.json
```

The verifier checks each asset's complete public response, fresh Worker nonce, byte digest, and a second read from R2. It records failures rather than treating HTTP success alone as acceptance. Reader interactions and narration still need their own live browser checks.

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

## Read-only public verification

`verify-narration.mjs` checks the exact 2,378 Heart/Michael chapter pairs from `audio-contract.mjs`. It enumerates the expected KJV chapter ranges and validates each complete local cue/audio pair and its SHA-256 values. At the start of each scan it sends one lightweight public audio `HEAD` with a fresh UUID in `?verify=<UUID>` and requires the exact `X-Narration-Verification` echo before starting per-object reads. If the Worker does not echo the UUID, locally valid pairs stay `UNVERIFIED`, bulk object reads are skipped, and watch mode retries the gate on the next scan. Once the gate passes, every `.mp3` and `.json` `HEAD` and `GET` gets its own fresh UUID and must echo it exactly. Requests use `Cache-Control: no-cache, no-store` and the fetch client's `no-store` mode. Cache headers are recorded separately; they do not replace the request-specific Worker echo. Audio responses are hashed as streams so the corpus is not held in memory.

Run one pass with a new absolute private receipt path outside the narration source root:

```sh
node hosting/verify-narration.mjs https://<site-origin> /path/to/narration --receipt /private/path/narration-public-check-<run-id>.json
```

To keep checking pairs as the existing generator and uploader publish them, use watch mode:

```sh
node hosting/verify-narration.mjs --watch https://<site-origin> /path/to/narration --receipt /private/path/narration-public-watch-<run-id>.json --interval-seconds 15
```

The verifier is read-only against the site: it has no upload route, accepts no token, and only performs public `GET`/`HEAD` requests. It does not change source files, credentials, or hosting configuration. Concurrency is capped at four chapter pairs. Missing local pairs and public 404s with an exact Worker echo remain pending during publication; missing or mismatched echoes remain `UNVERIFIED`, never terminal failures. Other contract, hash, metadata, HTTP, or network failures are retained with their route stage and safe status details. Transient failures get at most two attempts for unchanged evidence. A mismatch is reopened only after a changed source signature or measured, echoed remote `HEAD` metadata/ETag signature; a fresh run starts with no passes imported from an earlier receipt.

The receipt and exclusive lock are mode `0600`, contain no source paths or secrets, and are written outside the source root. Choose a new receipt path for each run. A normal stop renames the lock to a run-specific `.closed-<run-id>` file. If a process is interrupted before it closes the lock, use a new private receipt path. `COMPLETE` means every expected pair had local contract evidence and fresh public `HEAD`/`GET` responses with exact request-specific Worker echoes during this run. Health data, including any reported Worker revision, is recorded as observed; release identity remains `UNKNOWN` and must be bound separately to the appropriate Sites archive/native API and the actual public assets.
