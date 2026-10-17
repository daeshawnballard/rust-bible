#!/usr/bin/env python3
"""Prepare offline Kokoro chapter narration and run a bounded benchmark.

All model/voice inputs must already be local. HF and Transformers offline mode
are enabled before importing MLX Audio; English G2P's spaCy model is required
locally to prevent Misaki from auto-downloading it on first use.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import math
import os
import platform
import shutil
import statistics
import subprocess
import sys
import time
import uuid
import traceback
from pathlib import Path
from typing import Any

SCRIPT_VERSION = 3
MODEL_REPO = "mlx-community/Kokoro-82M-bf16"
MODEL_REVISION = "a71e4d38b236d968966a2002c4c895dbd12b1c3c"
MODEL_FILES = {
    "kokoro-v1_0.safetensors": "4e9ecdf03b8b6cf906070390237feda473dc13327cb8d56a43deaa374c02acd8",
    "voices/af_heart.safetensors": "2c1c733b0e6576c810e268d3e440c21dea4e0f0131a3ba4cfc98d7fe6136d094",
    "voices/am_michael.safetensors": "3940147ded35deba0bb52e8132f89b719298e0520258c34584358aa5a24da2ea",
}
SUPPORTED_VOICES = ("af_heart", "am_michael")
SAMPLE_RATE = 24_000
MP3_BITRATE = "64k"
MP3_EXTRA_TAIL_TOLERANCE_SAMPLES = 24  # exactly 1 ms at 24 kHz
ALIGNMENT_SEARCH_SAMPLES = 128
ALIGNMENT_MIN_CORRELATION = 0.90
ALIGNMENT_MIN_MARGIN = 0.005


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_json_atomic(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = path.with_name(f".{path.name}.tmp-{uuid.uuid4().hex}")
    temp_path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    os.replace(temp_path, path)


def require_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise SystemExit(f"Required {label} is missing: {path}")


def preflight(model_dir: Path, corpus_path: Path, require_ffmpeg: bool = False) -> dict[str, Any]:
    if not model_dir.is_dir() or model_dir.name != "Kokoro-82M-bf16":
        raise SystemExit(
            "--model-dir must name the local directory `Kokoro-82M-bf16`; "
            "the MLX Audio loader infers model type from the directory name."
        )
    require_file(model_dir / "config.json", "Kokoro config")
    for relpath, expected_hash in MODEL_FILES.items():
        path = model_dir / relpath
        require_file(path, f"pinned model file {relpath}")
        observed_hash = sha256_file(path)
        if observed_hash != expected_hash:
            raise SystemExit(f"Pinned model hash mismatch for {path}: {observed_hash}")
    require_file(corpus_path, "KJV corpus")
    if require_ffmpeg and (not shutil.which("ffmpeg") or not shutil.which("ffprobe")):
        raise SystemExit("Both ffmpeg and ffprobe must be on PATH for chapter generation.")
    return {
        "model_repo": MODEL_REPO,
        "model_revision": MODEL_REVISION,
        "model_files_sha256": {name: value for name, value in MODEL_FILES.items()},
        "model_dir": str(model_dir.resolve()),
        "corpus_sha256": sha256_file(corpus_path),
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "machine": platform.machine(),
    }


def load_corpus(path: Path) -> tuple[list[dict[str, Any]], str]:
    raw = path.read_bytes()
    books = json.loads(raw)
    if not isinstance(books, list) or len(books) != 66:
        raise SystemExit(f"Expected 66 canonical book entries, received {len(books) if isinstance(books, list) else 'non-list'}.")
    for b_idx, book in enumerate(books, start=1):
        if not isinstance(book.get("chapters"), list) or not book["chapters"]:
            raise SystemExit(f"Book index {b_idx} has no chapters.")
        for c_idx, chapter in enumerate(book["chapters"], start=1):
            if not isinstance(chapter, list) or not chapter or not all(isinstance(v, str) and v for v in chapter):
                raise SystemExit(f"Invalid verse list at book {b_idx}, chapter {c_idx}.")
    return books, sha256_bytes(raw)


def get_chapter(books: list[dict[str, Any]], book_no: int, chapter_no: int) -> list[str]:
    if not 1 <= book_no <= len(books):
        raise SystemExit(f"Book must be in 1..{len(books)}.")
    chapters = books[book_no - 1]["chapters"]
    if not 1 <= chapter_no <= len(chapters):
        raise SystemExit(f"Book {book_no} has {len(chapters)} chapters; requested {chapter_no}.")
    return chapters[chapter_no - 1]


def choose_benchmark_verses(books: list[dict[str, Any]], count: int = 12) -> list[dict[str, Any]]:
    all_verses: list[dict[str, Any]] = []
    for book_no, book in enumerate(books, start=1):
        for chapter_no, chapter in enumerate(book["chapters"], start=1):
            for verse_no, text in enumerate(chapter, start=1):
                all_verses.append({
                    "book": book_no,
                    "chapter": chapter_no,
                    "verse": verse_no,
                    "text": text,
                    "chars": len(text),
                    "words": len(text.split()),
                })
    ranked = sorted(all_verses, key=lambda item: (item["chars"], item["book"], item["chapter"], item["verse"]))
    selected: list[dict[str, Any]] = []
    for i in range(count):
        index = round(i * (len(ranked) - 1) / max(count - 1, 1))
        item = dict(ranked[index])
        item["text_sha256"] = sha256_bytes(item["text"].encode("utf-8"))
        selected.append(item)
    return selected


def load_engine(model_dir: Path):
    # Misaki's spaCy model bootstrap calls uv itself; point it at this active
    # interpreter, then refuse a missing model before it could make a request.
    os.environ["VIRTUAL_ENV"] = sys.prefix
    os.environ["HF_HUB_OFFLINE"] = "1"
    os.environ["TRANSFORMERS_OFFLINE"] = "1"
    os.environ["HF_HUB_DISABLE_TELEMETRY"] = "1"
    try:
        import spacy
        if not spacy.util.is_package("en_core_web_sm"):
            raise SystemExit(
                "Missing spaCy en_core_web_sm 3.8.0. Install the pinned wheel "
                "from narration-requirements.txt before running; the script will not fetch it."
            )
    except ImportError as exc:
        raise SystemExit("Missing spaCy. Install the pinned narration requirements first.") from exc
    from mlx_audio.tts.utils import load_model
    import mlx.core as mx
    return load_model(model_dir), mx


def generate_verse(model, mx, text: str, voice: str, voice_path: Path) -> tuple[Any, int, float]:
    started = time.perf_counter()
    results = list(model.generate(text=text, voice=str(voice_path), lang_code="a", speed=1.0))
    if not results:
        raise RuntimeError(f"Kokoro produced no segments for {voice}.")
    mx.eval(*[result.audio for result in results])
    import numpy as np
    arrays = [np.asarray(result.audio, dtype=np.float32).reshape(-1) for result in results]
    pcm = np.concatenate(arrays) if len(arrays) > 1 else arrays[0]
    if not len(pcm):
        raise RuntimeError(f"Kokoro produced empty PCM for {voice}.")
    elapsed = time.perf_counter() - started
    return pcm, len(pcm), elapsed


def run_benchmark(args: argparse.Namespace) -> dict[str, Any]:
    books, corpus_hash = load_corpus(args.corpus)
    identity = preflight(args.model_dir, args.corpus)
    verses = choose_benchmark_verses(books)
    model, mx = load_engine(args.model_dir)
    records: list[dict[str, Any]] = []
    for voice in SUPPORTED_VOICES:
        voice_path = args.model_dir / "voices" / f"{voice}.safetensors"
        # A separate warm-up per voice keeps tokenizer/graph setup out of the
        # 12 measured samples and also proves each local voice file loads.
        _, warmup_samples, warmup_seconds = generate_verse(model, mx, verses[0]["text"], voice, voice_path)
        voice_records = []
        for verse in verses:
            pcm, sample_count, elapsed = generate_verse(model, mx, verse["text"], voice, voice_path)
            duration = sample_count / SAMPLE_RATE
            row = {
                **{k: verse[k] for k in ("book", "chapter", "verse", "chars", "words", "text_sha256")},
                "sample_rate": SAMPLE_RATE,
                "samples": sample_count,
                "audio_seconds": round(duration, 6),
                "wall_seconds": round(elapsed, 6),
                "realtime_factor": round(elapsed / duration, 6),
            }
            voice_records.append(row)
        durations = [row["audio_seconds"] for row in voice_records]
        rtfs = [row["realtime_factor"] for row in voice_records]
        words = sum(row["words"] for row in voice_records)
        total_corpus_words = sum(len(text.split()) for book in books for chapter in book["chapters"] for text in chapter)
        median_seconds_per_word = statistics.median(row["audio_seconds"] / max(row["words"], 1) for row in voice_records)
        median_rtf = statistics.median(rtfs)
        estimated_audio_seconds = total_corpus_words * median_seconds_per_word
        records.append({
            "voice": voice,
            "warmup": {
                "samples": warmup_samples,
                "audio_seconds": round(warmup_samples / SAMPLE_RATE, 6),
                "wall_seconds": round(warmup_seconds, 6),
            },
            "measured": voice_records,
            "summary": {
                "measured_verses": len(voice_records),
                "median_realtime_factor_excluding_warmup": round(median_rtf, 6),
                "median_audio_seconds_per_word": round(median_seconds_per_word, 6),
                "estimated_full_corpus_words": total_corpus_words,
                "estimated_full_corpus_audio_hours": round(estimated_audio_seconds / 3600, 3),
                "estimated_full_corpus_wall_hours_at_median_rtf": round(estimated_audio_seconds * median_rtf / 3600, 3),
                "estimate_method": "Exact corpus whitespace word count multiplied by median seconds/word and median measured RTF across the 12 length-stratified verses; planning estimate only.",
            },
        })
    result = {
        "version": 1,
        "run_kind": "bounded_offline_benchmark",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "script_version": SCRIPT_VERSION,
        **identity,
        "corpus_sha256": corpus_hash,
        "sample_rate": SAMPLE_RATE,
        "speed": 1.0,
        "selection": "12 corpus verses selected at evenly spaced ranks after sorting by Unicode character count; includes the shortest and longest verse.",
        "voices": records,
    }
    args.output_dir.mkdir(parents=True, exist_ok=True)
    output_path = args.output_dir / "benchmark.json"
    write_json_atomic(output_path, result)
    result["output_path"] = str(output_path)
    return result


def run_ffmpeg_encoder(temp_mp3: Path):
    command = [
        "ffmpeg", "-hide_banner", "-loglevel", "error",
        "-f", "f32le", "-ar", str(SAMPLE_RATE), "-ac", "1", "-i", "pipe:0",
        "-c:a", "libmp3lame", "-b:a", MP3_BITRATE, "-write_xing", "1",
        "-id3v2_version", "3", str(temp_mp3),
    ]
    return subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


def mp3_decoded_pcm(path: Path):
    import numpy as np
    command = [
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-i", str(path),
        "-map", "0:a:0", "-f", "f32le", "-acodec", "pcm_f32le",
        "-ac", "1", "-ar", str(SAMPLE_RATE), "pipe:1",
    ]
    result = subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if len(result.stdout) % 4:
        raise RuntimeError(f"ffmpeg returned a partial float32 PCM sample ({len(result.stdout)} bytes)")
    return np.frombuffer(result.stdout, dtype="<f4")


def first_sample_alignment(source_pcm, decoded_pcm) -> dict[str, Any]:
    """Measure the MP3's leading alignment against a high-energy source window."""
    import numpy as np
    source = np.asarray(source_pcm, dtype=np.float64).reshape(-1)
    decoded = np.asarray(decoded_pcm, dtype=np.float64).reshape(-1)
    window = min(SAMPLE_RATE, len(source), len(decoded))
    if window < 2048:
        raise RuntimeError(f"Too little PCM to verify MP3 leading alignment: {window} samples")

    # Search within the first eight seconds for a one-second (or shorter) window
    # with clear speech energy. This avoids false offsets caused by initial silence.
    search_end = min(len(source) - window, SAMPLE_RATE * 8)
    step = max(1, min(SAMPLE_RATE // 20, window // 8))
    starts = list(range(0, search_end + 1, step))
    if not starts or starts[-1] != search_end:
        starts.append(search_end)
    squared = source * source
    cumulative = np.concatenate(([0.0], np.cumsum(squared)))
    start = max(starts, key=lambda pos: cumulative[pos + window] - cumulative[pos])
    source_window = source[start:start + window]
    source_window = source_window - np.mean(source_window)
    source_norm = float(np.linalg.norm(source_window))
    if source_norm <= 1e-8:
        raise RuntimeError("Cannot verify MP3 leading alignment: source window is silent")

    rows: list[tuple[float, int]] = []
    for lag in range(-ALIGNMENT_SEARCH_SAMPLES, ALIGNMENT_SEARCH_SAMPLES + 1):
        decoded_start = start + lag
        if decoded_start < 0 or decoded_start + window > len(decoded):
            continue
        decoded_window = decoded[decoded_start:decoded_start + window]
        decoded_window = decoded_window - np.mean(decoded_window)
        denominator = source_norm * float(np.linalg.norm(decoded_window))
        score = float(np.dot(source_window, decoded_window) / denominator) if denominator > 1e-8 else 0.0
        rows.append((score, lag))
    if not rows:
        raise RuntimeError("Cannot verify MP3 leading alignment: no overlapping decode window")
    best_score, best_lag = max(rows)
    zero_score = next((score for score, lag in rows if lag == 0), None)
    next_score = max((score for score, lag in rows if lag != 0), default=-1.0)
    margin = (zero_score - next_score) if zero_score is not None else -1.0
    if best_lag != 0 or zero_score is None or zero_score < ALIGNMENT_MIN_CORRELATION or margin < ALIGNMENT_MIN_MARGIN:
        raise RuntimeError(
            "MP3 first-sample alignment is not established: "
            f"best_lag={best_lag}, correlation_at_zero={zero_score}, competing_lag_margin={margin}"
        )
    return {
        "lag_samples": 0,
        "source_window_start_sample": start,
        "window_samples": window,
        "correlation_at_zero": round(zero_score, 6),
        "margin_over_best_nonzero_lag": round(margin, 6),
        "search_range_samples": ALIGNMENT_SEARCH_SAMPLES,
    }


def ffprobe_gapless_metadata(path: Path) -> dict[str, Any]:
    """Read FFmpeg's Xing/LAME gapless skip/padding evidence for a duration delta."""
    command = [
        "ffprobe", "-v", "error", "-select_streams", "a:0", "-show_packets",
        "-show_entries", "packet=side_data_list", "-of", "json", str(path),
    ]
    result = subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    packets = json.loads(result.stdout).get("packets", [])
    if not packets:
        raise RuntimeError("ffprobe found no MP3 packets for gapless metadata validation")

    def skip_values(packet: dict[str, Any]) -> tuple[int, int] | None:
        for entry in packet.get("side_data_list", []):
            if entry.get("side_data_type") == "Skip Samples":
                return int(entry.get("skip_samples", 0)), int(entry.get("discard_padding", 0))
        return None

    first = skip_values(packets[0])
    last = skip_values(packets[-1])
    if first is None or last is None:
        raise RuntimeError("MP3 lacks FFprobe Xing/LAME skip-samples or discard-padding metadata")
    return {
        "packet_count": len(packets),
        "first_packet_skip_samples": first[0],
        "first_packet_discard_padding_samples": first[1],
        "last_packet_skip_samples": last[0],
        "last_packet_discard_padding_samples": last[1],
        "xing_gapless_metadata_present": first[0] > 0 and last[1] > 0,
    }


def ffprobe_metadata(path: Path) -> dict[str, Any]:
    command = [
        "ffprobe", "-v", "error", "-show_entries",
        "stream=codec_name,sample_rate,channels,start_time,duration:format=duration,size",
        "-of", "json", str(path),
    ]
    result = subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    parsed = json.loads(result.stdout)
    if not parsed.get("streams"):
        raise RuntimeError("ffprobe found no audio stream")
    stream = parsed["streams"][0]
    fmt = parsed.get("format", {})
    return {
        "codec_name": stream.get("codec_name"),
        "sample_rate": int(stream["sample_rate"]) if stream.get("sample_rate") else None,
        "channels": stream.get("channels"),
        "stream_start_time": float(stream["start_time"]) if stream.get("start_time") is not None else None,
        "stream_duration": float(stream["duration"]) if stream.get("duration") is not None else None,
        "format_duration": float(fmt["duration"]) if fmt.get("duration") is not None else None,
        "format_size": int(fmt["size"]) if fmt.get("size") is not None else None,
    }


def chapter_peaks(pcm, duration_samples: int) -> tuple[int, list[float]]:
    import numpy as np
    # Compact normalized envelope: 512 bins up to 5 minutes, 1024 above.
    bins = 512 if duration_samples <= SAMPLE_RATE * 300 else 1024
    edges = np.linspace(0, duration_samples, bins + 1, dtype=np.int64)
    absolute = np.abs(pcm)
    peaks = [round(float(np.max(absolute[edges[i]:edges[i + 1]])), 5) for i in range(bins)]
    return bins, peaks


def chapter_is_valid(mp3_path: Path, json_path: Path, corpus_hash: str, voice: str, book: int, chapter: int, verses: list[str]) -> bool:
    if not mp3_path.is_file() or not json_path.is_file():
        return False
    try:
        manifest = json.loads(json_path.read_text(encoding="utf-8"))
        verses_meta = manifest.get("verses", [])
        peaks = manifest.get("peaks", [])
        mp3_validation = manifest.get("mp3_validation", {})
        generator = manifest.get("generator", {})
        peak_count = generator.get("peak_bin_count")
        decoded_samples = mp3_validation.get("decoded_samples_at_24khz")
        source_samples = mp3_validation.get("source_pcm_samples")
        delta = decoded_samples - source_samples if isinstance(decoded_samples, int) and isinstance(source_samples, int) else None
        xing = mp3_validation.get("xing_gapless_metadata", {})
        alignment = mp3_validation.get("first_sample_alignment", {})
        duration_ok = delta == 0 or (
            isinstance(delta, int)
            and 0 < delta <= MP3_EXTRA_TAIL_TOLERANCE_SAMPLES
            and xing.get("xing_gapless_metadata_present") is True
        )
        return (
            manifest.get("version") == 1
            and manifest.get("script_version") == SCRIPT_VERSION
            and manifest.get("voice") == voice
            and manifest.get("book") == book
            and manifest.get("chapter") == chapter
            and manifest.get("corpus_sha256") == corpus_hash
            and manifest.get("audio_sha256") == sha256_file(mp3_path)
            and manifest.get("sample_rate") == SAMPLE_RATE
            and generator.get("model_revision") == MODEL_REVISION
            and generator.get("voice_file_sha256") == MODEL_FILES[f"voices/{voice}.safetensors"]
            and peak_count in (512, 1024)
            and len(peaks) == peak_count
            and all(isinstance(value, (int, float)) and math.isfinite(value) and 0 <= value <= 1 for value in peaks)
            and manifest.get("peak_count") == peak_count
            and duration_ok
            and mp3_validation.get("duration_tolerance_samples") == MP3_EXTRA_TAIL_TOLERANCE_SAMPLES
            and alignment.get("lag_samples") == 0
            and isinstance(alignment.get("correlation_at_zero"), (int, float))
            and alignment.get("correlation_at_zero") >= ALIGNMENT_MIN_CORRELATION
            and isinstance(alignment.get("margin_over_best_nonzero_lag"), (int, float))
            and alignment.get("margin_over_best_nonzero_lag") >= ALIGNMENT_MIN_MARGIN
            and mp3_validation.get("media_cue_offset_seconds") == 0.0
            and [item.get("text_sha256") for item in verses_meta]
            == [sha256_bytes(text.encode("utf-8")) for text in verses]
            and len(verses_meta) == len(verses)
        )
    except (AttributeError, OSError, ValueError, TypeError):
        return False


def run_chapter(args: argparse.Namespace, engine=None, prepared=None) -> dict[str, Any]:
    if prepared is None:
        books, corpus_hash = load_corpus(args.corpus)
        identity = preflight(args.model_dir, args.corpus, require_ffmpeg=True)
    else:
        books, corpus_hash, identity = prepared
    if args.voice not in SUPPORTED_VOICES:
        raise SystemExit(f"Voice must be one of: {', '.join(SUPPORTED_VOICES)}")
    verses = get_chapter(books, args.book, args.chapter)
    out_dir = args.output_dir / args.voice / str(args.book)
    mp3_path = out_dir / f"{args.chapter}.mp3"
    json_path = out_dir / f"{args.chapter}.json"
    if chapter_is_valid(mp3_path, json_path, corpus_hash, args.voice, args.book, args.chapter, verses):
        return {"status": "SKIPPED_VALID", "mp3": str(mp3_path), "manifest": str(json_path)}

    model, mx = engine if engine is not None else load_engine(args.model_dir)
    voice_path = args.model_dir / "voices" / f"{args.voice}.safetensors"
    out_dir.mkdir(parents=True, exist_ok=True)
    temp_mp3 = out_dir / f".{args.chapter}.mp3.tmp-{uuid.uuid4().hex}.mp3"
    verse_rows: list[dict[str, Any]] = []
    pcm_parts = []
    total_samples = 0
    for verse_no, text in enumerate(verses, start=1):
        pcm, sample_count, _ = generate_verse(model, mx, text, args.voice, voice_path)
        pcm_parts.append(pcm)
        start_sample = total_samples
        total_samples += sample_count
        verse_rows.append({
            "verse": verse_no,
            "start": round(start_sample / SAMPLE_RATE, 6),
            "end": round(total_samples / SAMPLE_RATE, 6),
            "text_sha256": sha256_bytes(text.encode("utf-8")),
        })
    import numpy as np
    chapter_pcm = np.concatenate(pcm_parts) if len(pcm_parts) > 1 else pcm_parts[0]
    if len(chapter_pcm) != total_samples:
        raise RuntimeError(f"PCM concatenation length mismatch: {len(chapter_pcm)} != {total_samples}")
    if not np.all(np.isfinite(chapter_pcm)):
        raise RuntimeError("Kokoro source PCM contains NaN or infinite samples")
    source_peak_abs = float(np.max(np.abs(chapter_pcm)))
    clipped_sample_count = int(np.count_nonzero(np.abs(chapter_pcm) > 1.0))
    # Clip only the PCM that is actually encoded, and derive waveform peaks from
    # that same PCM. Record the pre-clip level and count so normalization is honest.
    chapter_pcm = np.clip(chapter_pcm, -1.0, 1.0)
    encoded_peak_abs = float(np.max(np.abs(chapter_pcm)))
    encoder = run_ffmpeg_encoder(temp_mp3)
    if encoder.stdin is None:
        raise RuntimeError("ffmpeg encoder pipe did not open")
    try:
        encoder.stdin.write(chapter_pcm.astype("<f4", copy=False).tobytes())
        encoder.stdin.close()
        stderr = encoder.stderr.read().decode("utf-8", errors="replace") if encoder.stderr else ""
        code = encoder.wait()
        if code != 0:
            raise RuntimeError(f"ffmpeg exited {code}: {stderr[-2000:]}")
    except Exception:
        if encoder.stdin and not encoder.stdin.closed:
            encoder.stdin.close()
        encoder.wait()
        raise
    if not temp_mp3.is_file() or temp_mp3.stat().st_size == 0:
        raise RuntimeError("ffmpeg produced no MP3 file")
    decoded_pcm = mp3_decoded_pcm(temp_mp3)
    decoded_samples = len(decoded_pcm)
    decoded_minus_source_samples = decoded_samples - total_samples
    if decoded_minus_source_samples < 0 or decoded_minus_source_samples > MP3_EXTRA_TAIL_TOLERANCE_SAMPLES:
        raise RuntimeError(
            "MP3 decoded sample count outside the explicit 0..24 sample tail-padding tolerance: "
            f"decoded={decoded_samples}, source={total_samples}, delta={decoded_minus_source_samples}"
        )
    alignment = first_sample_alignment(chapter_pcm, decoded_pcm)
    gapless = ffprobe_gapless_metadata(temp_mp3) if decoded_minus_source_samples else None
    if decoded_minus_source_samples and not gapless["xing_gapless_metadata_present"]:
        raise RuntimeError(
            "MP3 has a positive duration delta but FFprobe did not confirm Xing/LAME gapless metadata: "
            f"{gapless}"
        )
    probe = ffprobe_metadata(temp_mp3)
    peak_bin_count, peaks = chapter_peaks(chapter_pcm, total_samples)
    os.replace(temp_mp3, mp3_path)
    manifest = {
        "version": 1,
        "script_version": SCRIPT_VERSION,
        "voice": args.voice,
        "book": args.book,
        "chapter": args.chapter,
        "sample_rate": SAMPLE_RATE,
        "duration": round(total_samples / SAMPLE_RATE, 6),
        "corpus_sha256": corpus_hash,
        "audio_sha256": sha256_file(mp3_path),
        "verses": verse_rows,
        "peaks": peaks,
        "peak_count": peak_bin_count,
        "peak_bin_duration_seconds": round((total_samples / SAMPLE_RATE) / peak_bin_count, 6),
        "peak_semantics": "Equal-duration max-absolute-amplitude bins spanning the full chapter source PCM.",
        "pcm_validation": {
            "all_samples_finite": True,
            "source_peak_abs_before_clip": round(source_peak_abs, 8),
            "encoded_peak_abs_after_clip": round(encoded_peak_abs, 8),
            "clipped_sample_count": clipped_sample_count,
            "clipping_applied": clipped_sample_count > 0,
        },
        "generator": {
            "library": "mlx-audio==0.5.9",
            "model_repo": MODEL_REPO,
            "model_revision": MODEL_REVISION,
            "voice_file_sha256": MODEL_FILES[f"voices/{args.voice}.safetensors"],
            "speed": 1.0,
            "mp3_encoder": "libmp3lame",
            "mp3_bitrate": MP3_BITRATE,
            "mp3_xing_header": True,
            "peak_bin_count": peak_bin_count,
            "peak_bin_duration_seconds": round((total_samples / SAMPLE_RATE) / peak_bin_count, 6),
        },
        "mp3_validation": {
            "ffprobe_codec": probe["codec_name"],
            "ffprobe_sample_rate": probe["sample_rate"],
            "ffprobe_channels": probe["channels"],
            "ffprobe_stream_start_time_seconds": round(probe["stream_start_time"], 6) if probe["stream_start_time"] is not None else None,
            "ffprobe_duration": round(probe["format_duration"], 6) if probe["format_duration"] is not None else None,
            "decoded_samples_at_24khz": decoded_samples,
            "source_pcm_samples": total_samples,
            "decoded_minus_source_samples": decoded_samples - total_samples,
            "decoded_duration_minus_source_seconds": round((decoded_samples - total_samples) / SAMPLE_RATE, 6),
            "duration_tolerance_samples": MP3_EXTRA_TAIL_TOLERANCE_SAMPLES,
            "duration_tolerance_seconds": round(MP3_EXTRA_TAIL_TOLERANCE_SAMPLES / SAMPLE_RATE, 6),
            "duration_delta_contract": "FFmpeg may decode up to 24 additional tail samples (1 ms at 24 kHz) only when Xing/LAME gapless metadata is present; fewer decoded samples fail.",
            "xing_gapless_metadata": gapless,
            "first_sample_alignment": alignment,
            "media_cue_offset_seconds": 0.0,
        },
    }
    write_json_atomic(json_path, manifest)
    return {
        "status": "GENERATED",
        "mp3": str(mp3_path),
        "manifest": str(json_path),
        "mp3_bytes": mp3_path.stat().st_size,
        "duration_seconds": manifest["duration"],
        "source_pcm_samples": total_samples,
        "decoded_samples": decoded_samples,
        "decoded_minus_source_samples": decoded_samples - total_samples,
        "peak_bins": len(manifest["peaks"]),
        "peak_bin_duration_seconds": manifest["peak_bin_duration_seconds"],
        "ffprobe_stream_start_time_seconds": manifest["mp3_validation"]["ffprobe_stream_start_time_seconds"],
        "decoded_duration_minus_source_seconds": manifest["mp3_validation"]["decoded_duration_minus_source_seconds"],
        "verse_count": len(verse_rows),
        "corpus_sha256": corpus_hash,
        "identity": identity,
    }


PRIORITY_CHAPTERS = [
    (1, 2), (1, 3), (19, 23), (19, 24), (2, 25), (43, 4), (43, 17),
    (43, 18), (44, 16), (44, 17), (39, 4), (40, 1), (66, 22),
]


def all_chapter_order(books: list[dict[str, Any]]) -> list[tuple[int, int]]:
    all_chapters = [
        (book_no, chapter_no)
        for book_no, book in enumerate(books, start=1)
        for chapter_no in range(1, len(book["chapters"]) + 1)
    ]
    priority = [item for item in PRIORITY_CHAPTERS if item in all_chapters]
    priority_set = set(priority)
    first_chapters = [item for item in all_chapters if item[1] == 1 and item not in priority_set]
    first_set = priority_set | set(first_chapters)
    remaining = [item for item in all_chapters if item not in first_set]
    return priority + first_chapters + remaining


def acquire_job_lock(output_dir: Path):
    output_dir.mkdir(parents=True, exist_ok=True)
    lock_path = output_dir / ".narration-job.lock"
    handle = lock_path.open("a+", encoding="utf-8")
    try:
        fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as exc:
        handle.close()
        raise SystemExit(f"Another narration job holds {lock_path}; refusing a duplicate owner.") from exc
    handle.write(f"pid={os.getpid()} started={time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\n")
    handle.flush()
    return handle


def run_all(args: argparse.Namespace) -> dict[str, Any]:
    books, corpus_hash = load_corpus(args.corpus)
    identity = preflight(args.model_dir, args.corpus, require_ffmpeg=True)
    chapter_order = all_chapter_order(books)
    jobs = [(voice, book_no, chapter_no) for book_no, chapter_no in chapter_order for voice in SUPPORTED_VOICES]
    progress_path = args.progress_file or (args.output_dir / "progress.json")
    lock = acquire_job_lock(args.output_dir)
    started = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    progress: dict[str, Any] = {
        "version": 1,
        "script_version": SCRIPT_VERSION,
        "state": "RUNNING",
        "pid": os.getpid(),
        "started_utc": started,
        "updated_utc": started,
        "corpus_sha256": corpus_hash,
        "model_repo": MODEL_REPO,
        "model_revision": MODEL_REVISION,
        "voices": list(SUPPORTED_VOICES),
        "total_items": len(jobs),
        "completed_items": 0,
        "generated_this_run": 0,
        "skipped_valid_this_run": 0,
        "counts_by_voice": {voice: {"generated": 0, "skipped_valid": 0} for voice in SUPPORTED_VOICES},
        "schedule": "13 priority chapters, then every canonical chapter 1, then remaining chapters in canonical order; each chapter runs af_heart then am_michael.",
        "priority_chapters": [{"book": book_no, "chapter": chapter_no} for book_no, chapter_no in PRIORITY_CHAPTERS],
        "last_item": None,
        "failure": None,
        "output_dir": str(args.output_dir.resolve()),
        "progress_file": str(progress_path.resolve()),
        "runtime_identity": identity,
    }
    write_json_atomic(progress_path, progress)
    current_job = None
    try:
        model, mx = load_engine(args.model_dir)
        for index, (voice, book_no, chapter_no) in enumerate(jobs, start=1):
            current_job = {"index": index, "voice": voice, "book": book_no, "chapter": chapter_no}
            chapter_args = argparse.Namespace(**vars(args))
            chapter_args.command = "chapter"
            chapter_args.voice = voice
            chapter_args.book = book_no
            chapter_args.chapter = chapter_no
            result = run_chapter(chapter_args, engine=(model, mx), prepared=(books, corpus_hash, identity))
            progress["completed_items"] = index
            progress["generated_this_run"] += int(result["status"] == "GENERATED")
            progress["skipped_valid_this_run"] += int(result["status"] == "SKIPPED_VALID")
            progress["counts_by_voice"][voice]["generated"] += int(result["status"] == "GENERATED")
            progress["counts_by_voice"][voice]["skipped_valid"] += int(result["status"] == "SKIPPED_VALID")
            progress["last_item"] = {
                "voice": voice,
                "book": book_no,
                "chapter": chapter_no,
                "status": result["status"],
                "mp3": result.get("mp3"),
                "manifest": result.get("manifest"),
                "mp3_bytes": result.get("mp3_bytes"),
                "duration_seconds": result.get("duration_seconds"),
                "decoded_minus_source_samples": result.get("decoded_minus_source_samples"),
                "peak_bins": result.get("peak_bins"),
            }
            progress["updated_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
            write_json_atomic(progress_path, progress)
            print(json.dumps({"progress": progress["completed_items"], "total": progress["total_items"], **progress["last_item"]}), flush=True)
        progress["state"] = "COMPLETE"
        progress["updated_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        write_json_atomic(progress_path, progress)
        return progress
    except KeyboardInterrupt:
        progress["state"] = "INTERRUPTED"
        progress["updated_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        write_json_atomic(progress_path, progress)
        raise
    except BaseException as exc:
        progress.update({
            "state": "FAILED",
            "updated_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "failure": {
                **(current_job or {}),
                "error_type": type(exc).__name__,
                "error": str(exc),
                "traceback": traceback.format_exc(),
            },
            "last_item": {**(current_job or {}), "status": "FAILED"},
        })
        write_json_atomic(progress_path, progress)
        raise
    finally:
        lock.close()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True, help="Path to canonical data/kjv.json")
    parser.add_argument("--model-dir", type=Path, required=True, help="Pinned local Kokoro-82M-bf16 directory")
    parser.add_argument("--output-dir", type=Path, required=True, help="Private output directory outside the repository")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("benchmark", help="Generate 12 corpus verses per selected voice; write benchmark.json only")
    chapter = sub.add_parser("chapter", help="Generate one chapter with per-verse cues and MP3 waveform peaks")
    chapter.add_argument("--voice", required=True, choices=SUPPORTED_VOICES)
    chapter.add_argument("--book", required=True, type=int, help="Canonical 1-based KJV book index")
    chapter.add_argument("--chapter", required=True, type=int, help="1-based chapter number")
    all_cmd = sub.add_parser("all", help="Generate/resume both voices, prioritizing selected chapters then all chapter 1s then canonical remainder")
    all_cmd.add_argument("--progress-file", type=Path, help="Progress JSON path; defaults to OUTPUT_DIR/progress.json")
    return parser


def main() -> int:
    # All external model access is deliberately disabled. Inputs are pinned,
    # local files and the optional chapter command uses the local ffmpeg binary.
    os.environ["HF_HUB_OFFLINE"] = "1"
    os.environ["TRANSFORMERS_OFFLINE"] = "1"
    args = build_parser().parse_args()
    if args.command == "benchmark":
        result = run_benchmark(args)
    elif args.command == "chapter":
        result = run_chapter(args)
    elif args.command == "all":
        result = run_all(args)
    else:  # pragma: no cover - argparse enforces known commands
        raise SystemExit("Unknown command")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
