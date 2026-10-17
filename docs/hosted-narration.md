# Prepared Heart and Michael narration

The selectable voices are Kokoro `af_heart` (Heart) and `am_michael` (Michael). There is no Apple/device voice or paid inference API. Both are synthetic; the upstream catalog does not establish the speakers' race.

## Playback

`web/hosted-voice.js` fetches same-origin chapter MP3s and cue JSON from `/audio/v1/{voice}/{book}/{chapter}.{mp3,json}`. It checks path identity, cue order and bounds, audio SHA-256, and each narrated verse's exact text SHA-256. Chapters use one media element and continuous media time across their verse cues. Verse start events update the existing reader highlight and saved reading position. The waveform uses real max-amplitude PCM bins, cropped to the current verse. Rate, volume, pause/resume and verse-relative seeking stay in the shared dock.

At most two chapter assets are held; the next chapter is prefetched. A real 404 selects the existing local Kokoro runtime with the same voice. Other request or integrity failures stop with a visible error. Unpublished chapters and slow networks can still pause; this is not a guarantee of gapless audio. `bible:hosted-verse` reports the current recording URL, voice, reference and cue/media times without Scripture text.

## Offline preparation

`scripts/prepare-narration.py` uses Apple Silicon MLX and FFmpeg. Install `scripts/narration-requirements.txt` into a private virtual environment and download the model before running it. The runner requires local model files and forces Hugging Face/Transformers offline mode. Do not put generated recordings or job logs in the repository.

- Library: [MLX Audio](https://github.com/Blaizzy/mlx-audio), version `0.5.9` (MIT).
- Model: [mlx-community/Kokoro-82M-bf16](https://huggingface.co/mlx-community/Kokoro-82M-bf16/tree/a71e4d38b236d968966a2002c4c895dbd12b1c3c), revision `a71e4d38b236d968966a2002c4c895dbd12b1c3c` (Apache-2.0). This is the same Kokoro 1.0 voice family as the browser runtime, using bf16 weights rather than browser q8 weights; waveforms need not be identical.
- Model SHA-256: `4e9ecdf03b8b6cf906070390237feda473dc13327cb8d56a43deaa374c02acd8`.
- Heart embedding SHA-256: `2c1c733b0e6576c810e268d3e440c21dea4e0f0131a3ba4cfc98d7fe6136d094`.
- Michael embedding SHA-256: `3940147ded35deba0bb52e8132f89b719298e0520258c34584358aa5a24da2ea`.
- Embedded KJV corpus SHA-256: `fc99486e7d3b86e4ad1f0f424b36ab41b4ec4db858a776bd13aee1b7910f136b`.

Example (paths are placeholders):

```sh
/path/to/venv/bin/python scripts/prepare-narration.py \
  --model-dir /private/Kokoro-82M-bf16 \
  --output-dir /private/narration all
```

`all` is resumable and locks one writer. It produces 2,378 voice/chapter pairs, prioritizing selected reader passages and then the first chapter of every book. Existing pairs are reused only after validation against corpus, model pins, MP3 hash, cues and generation schema. Each verse is synthesized separately, then concatenated into a mono 24 kHz chapter and encoded at 64 kbps. Cue boundaries come from source PCM sample counts. JSON includes generator pins, exact verse text hashes, 512 or 1,024 equal-duration amplitude bins, clipping counts, and MP3 decoding checks.

The generator verifies zero-sample start alignment. FFmpeg decoding may add 0–24 tail samples (at most 1 ms) only when the positive delta has Xing/LAME gapless metadata. Negative or larger deltas fail. This tolerance is reported rather than rewritten as zero. Atomic chapter publication writes audio before its cue file; partial outputs are not ready pairs.

## Hosting and verification boundary

The Sites Worker and upload tool are described in [hosting/README.md](../hosting/README.md). Audio belongs in the R2 `BUCKET` binding, not relational database rows. Upload credentials are host secrets and enter the CLI through hidden stdin. Audio uploads precede cues; the CLI checks actual remote object hashes and supports a resumable watcher with private receipts.

A generation progress count, unit test, successful build, local R2 emulator, or upload HTTP status alone does not prove public playback. Release checks must cover the actual public reader, live media/cue requests, verse highlighting, view changes, pause/resume, seeking, chapter/book boundaries, finite daily passages, and failure behavior. Corpus preparation and public delivery remain separate progress measures.
