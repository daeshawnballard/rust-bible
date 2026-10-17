export const VOICES = Object.freeze(["af_heart", "am_michael"]);

export const CHAPTER_COUNTS = Object.freeze([
  50, 40, 27, 36, 34, 24, 21, 4, 31, 24, 22, 25, 29, 36, 10, 13, 10, 42, 150,
  31, 12, 8, 66, 52, 5, 48, 12, 14, 3, 9, 1, 4, 7, 3, 3, 3, 2, 14, 4, 28, 16,
  24, 21, 28, 16, 16, 13, 6, 6, 4, 4, 5, 3, 6, 4, 3, 1, 13, 5, 5, 3, 5,
  1, 1, 1, 22,
]);

export const AUDIO_MAX_BYTES = 12 * 1024 * 1024;
export const CUES_MAX_BYTES = 256 * 1024;
export const AUDIO_PREFIX = "kokoro-v1";
export const NARRATION_SCRIPT_VERSION = 3;

const MODEL_REVISION = "a71e4d38b236d968966a2002c4c895dbd12b1c3c";
const VOICE_FILE_SHA256 = Object.freeze({
  af_heart: "2c1c733b0e6576c810e268d3e440c21dea4e0f0131a3ba4cfc98d7fe6136d094",
  am_michael: "3940147ded35deba0bb52e8132f89b719298e0520258c34584358aa5a24da2ea",
});
const MP3_EXTRA_TAIL_TOLERANCE_SAMPLES = 24;
const ALIGNMENT_MIN_CORRELATION = 0.9;
const ALIGNMENT_MIN_MARGIN = 0.005;

export function parseNarrationPath(pathname, kind) {
  const match = pathname.match(
    /^\/audio\/v1\/(af_heart|am_michael)\/([1-9][0-9]?)\/([1-9][0-9]{0,2})\.(mp3|json)$/,
  );
  if (!match) return null;

  const [, voice, rawBook, rawChapter, extension] = match;
  const book = Number(rawBook);
  const chapter = Number(rawChapter);
  if (!VOICES.includes(voice)) return null;
  if (book < 1 || book > CHAPTER_COUNTS.length) return null;
  if (chapter < 1 || chapter > CHAPTER_COUNTS[book - 1]) return null;
  if (kind && extension !== kind) return null;

  return { voice, book, chapter, extension };
}

export function parseImportPath(pathname) {
  const match = pathname.match(
    /^\/_narration\/import\/(af_heart|am_michael)\/([1-9][0-9]?)\/([1-9][0-9]{0,2})\.(mp3|json)$/,
  );
  if (!match) return null;
  const [, voice, rawBook, rawChapter, extension] = match;
  const book = Number(rawBook);
  const chapter = Number(rawChapter);
  if (book < 1 || book > CHAPTER_COUNTS.length) return null;
  if (chapter < 1 || chapter > CHAPTER_COUNTS[book - 1]) return null;
  return { voice, book, chapter, extension };
}

export function r2Key({ voice, book, chapter, extension }) {
  return `${AUDIO_PREFIX}/${voice}/${book}/${chapter}.${extension}`;
}

export function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

export function validateCueDocument(document, expected) {
  if (!document || typeof document !== "object" || Array.isArray(document)) {
    return "invalid_cues_document";
  }
  const requiredKeys = [
    "version",
    "script_version",
    "voice",
    "book",
    "chapter",
    "sample_rate",
    "duration",
    "corpus_sha256",
    "audio_sha256",
    "verses",
    "peaks",
    "peak_count",
    "generator",
    "mp3_validation",
  ];
  if (requiredKeys.some((key) => !(key in document))) return "missing_cues_field";
  if (document.version !== 1) return "unsupported_cues_version";
  if (document.script_version !== NARRATION_SCRIPT_VERSION) return "unsupported_generator_script_version";
  if (
    document.voice !== expected.voice ||
    document.book !== expected.book ||
    document.chapter !== expected.chapter
  ) {
    return "cues_path_mismatch";
  }
  if (document.sample_rate !== 24000) return "invalid_sample_rate";
  if (
    typeof document.duration !== "number" ||
    !Number.isFinite(document.duration) ||
    document.duration <= 0
  ) {
    return "invalid_duration";
  }
  if (!isSha256(document.corpus_sha256)) return "invalid_corpus_sha256";
  if (!isSha256(document.audio_sha256)) return "invalid_audio_sha256";
  if (
    expected.audioSha256 &&
    document.audio_sha256.toLowerCase() !== expected.audioSha256.toLowerCase()
  ) {
    return "audio_sha256_mismatch";
  }

  if (!Array.isArray(document.verses) || document.verses.length === 0) {
    return "invalid_verse_cues";
  }
  let previousEnd = 0;
  for (let index = 0; index < document.verses.length; index += 1) {
    const cue = document.verses[index];
    if (!cue || typeof cue !== "object" || Array.isArray(cue)) {
      return "invalid_verse_cue";
    }
    if (
      cue.verse !== index + 1 ||
      typeof cue.start !== "number" ||
      typeof cue.end !== "number" ||
      !Number.isFinite(cue.start) ||
      !Number.isFinite(cue.end) ||
      cue.start < previousEnd ||
      cue.end <= cue.start ||
      cue.end > document.duration
    ) {
      return "invalid_verse_timing";
    }
    if (!isSha256(cue.text_sha256)) return "invalid_text_sha256";
    previousEnd = cue.end;
  }

  if (
    !Array.isArray(document.peaks) ||
    (document.peaks.length !== 512 && document.peaks.length !== 1024)
  ) {
    return "invalid_waveform_peaks";
  }
  if (
    document.peaks.some(
      (peak) => typeof peak !== "number" || !Number.isFinite(peak) || peak < 0 || peak > 1,
    )
  ) {
    return "invalid_waveform_peak_value";
  }

  const generator = document.generator;
  if (!generator || typeof generator !== "object" || Array.isArray(generator)) {
    return "invalid_generator_metadata";
  }
  if (generator.model_revision !== MODEL_REVISION) return "unsupported_model_revision";
  if (generator.voice_file_sha256 !== VOICE_FILE_SHA256[expected.voice]) return "unsupported_voice_file";
  if (
    ![512, 1024].includes(generator.peak_bin_count) ||
    generator.peak_bin_count !== document.peak_count ||
    document.peak_count !== document.peaks.length
  ) {
    return "invalid_generator_peak_count";
  }

  const validation = document.mp3_validation;
  if (!validation || typeof validation !== "object" || Array.isArray(validation)) {
    return "invalid_mp3_validation";
  }
  if (
    validation.ffprobe_codec !== "mp3" ||
    validation.ffprobe_sample_rate !== 24000 ||
    validation.ffprobe_channels !== 1
  ) {
    return "invalid_mp3_format_validation";
  }
  const decodedSamples = validation.decoded_samples_at_24khz;
  const sourceSamples = validation.source_pcm_samples;
  if (!Number.isSafeInteger(decodedSamples) || !Number.isSafeInteger(sourceSamples) || sourceSamples <= 0) {
    return "invalid_mp3_sample_validation";
  }
  const sampleDelta = decodedSamples - sourceSamples;
  const gapless = validation.xing_gapless_metadata;
  const durationMatches = sampleDelta === 0 || (
    sampleDelta > 0 &&
    sampleDelta <= MP3_EXTRA_TAIL_TOLERANCE_SAMPLES &&
    gapless &&
    typeof gapless === "object" &&
    gapless.xing_gapless_metadata_present === true
  );
  if (
    !durationMatches ||
    validation.duration_tolerance_samples !== MP3_EXTRA_TAIL_TOLERANCE_SAMPLES
  ) {
    return "invalid_mp3_duration_validation";
  }
  const alignment = validation.first_sample_alignment;
  if (
    !alignment ||
    typeof alignment !== "object" ||
    alignment.lag_samples !== 0 ||
    typeof alignment.correlation_at_zero !== "number" ||
    !Number.isFinite(alignment.correlation_at_zero) ||
    alignment.correlation_at_zero < ALIGNMENT_MIN_CORRELATION ||
    typeof alignment.margin_over_best_nonzero_lag !== "number" ||
    !Number.isFinite(alignment.margin_over_best_nonzero_lag) ||
    alignment.margin_over_best_nonzero_lag < ALIGNMENT_MIN_MARGIN ||
    validation.media_cue_offset_seconds !== 0
  ) {
    return "invalid_mp3_alignment_validation";
  }
  return null;
}
