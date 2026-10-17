#!/usr/bin/env node

import { createHash, randomUUID } from "node:crypto";
import { createReadStream } from "node:fs";
import { chmod, lstat, mkdir, open, readFile, readdir, rename, stat } from "node:fs/promises";
import { hostname } from "node:os";
import path from "node:path";
import { Readable } from "node:stream";
import {
  AUDIO_MAX_BYTES,
  CHAPTER_COUNTS,
  CUES_MAX_BYTES,
  VOICES,
  isSha256,
  parseNarrationPath,
  validateCueDocument,
} from "./audio-contract.mjs";

const EXPECTED_PAIR_COUNT = VOICES.length * CHAPTER_COUNTS.reduce((sum, count) => sum + count, 0);
const MAX_PAIR_ATTEMPTS = 3;
const PAIR_BACKOFF_MS = [1_000, 4_000];
const STABLE_SCAN_COUNT = 2;
const ROOT_CONTROL_FILES = new Set(["benchmark.json", "progress.json"]);
const GENERATOR_TEMP_MP3 = /^(af_heart|am_michael)\/[1-9][0-9]?\/\.[1-9][0-9]{0,2}\.mp3\.tmp-[a-f0-9]{32}\.mp3$/;

const cli = parseArguments(process.argv.slice(2));
if (cli.help) {
  printUsage();
  process.exit(0);
}
if (cli.error) {
  console.error(cli.error);
  printUsage();
  process.exit(2);
}

const siteOrigin = parseSiteOrigin(cli.origin);
const sourceRoot = path.resolve(cli.audioRoot);
const credentials = await readCredential();
if (!credentials || Buffer.byteLength(credentials, "utf8") > 512) {
  console.error("No narration upload token was supplied on stdin.");
  process.exit(2);
}

if (cli.watch) {
  await watchNarration({ receiptPath: cli.receiptPath, intervalMs: cli.intervalSeconds * 1000 });
} else {
  await uploadOnce(sourceRoot);
}

function parseSiteOrigin(value) {
  const url = new URL(value);
  const localHosts = new Set(["localhost", "127.0.0.1", "[::1]"]);
  const secureRemote = url.protocol === "https:";
  const localHttp = url.protocol === "http:" && localHosts.has(url.hostname.toLowerCase());
  if ((!secureRemote && !localHttp) || url.username || url.password || url.search || url.hash || url.pathname !== "/") {
    throw new Error("site-origin must be HTTPS; plain HTTP is allowed only for localhost, 127.0.0.1, or ::1.");
  }
  return url.origin;
}

function parseArguments(args) {
  if (args.length === 0 || args.includes("--help") || args.includes("-h")) return { help: true };
  let watch = false;
  let receiptPath = null;
  let intervalSeconds = 15;
  const positional = [];
  for (let index = 0; index < args.length; index += 1) {
    const value = args[index];
    if (value === "--watch") {
      watch = true;
    } else if (value === "--receipt") {
      receiptPath = args[index + 1] ?? null;
      if (!receiptPath || receiptPath.startsWith("--")) return { error: "--receipt needs a private progress JSON path." };
      index += 1;
    } else if (value === "--interval-seconds") {
      const raw = args[index + 1];
      if (!raw || !/^\d+$/.test(raw)) return { error: "--interval-seconds needs an integer from 15 to 3600." };
      intervalSeconds = Number(raw);
      if (intervalSeconds < 15 || intervalSeconds > 3600) return { error: "--interval-seconds must be from 15 to 3600." };
      index += 1;
    } else if (value.startsWith("--")) {
      return { error: `Unknown option: ${value}` };
    } else {
      positional.push(value);
    }
  }

  if (positional.length !== 2) return { error: "Provide exactly <site-origin> and <audio-root>." };
  if (watch && !receiptPath) return { error: "--watch requires --receipt <private-progress.json>." };
  if (!watch && (receiptPath || intervalSeconds !== 15)) return { error: "--receipt and --interval-seconds require --watch." };
  return { watch, receiptPath, intervalSeconds, origin: positional[0], audioRoot: positional[1] };
}

function printUsage() {
  console.error("Usage:");
  console.error("  node hosting/upload-narration.mjs <site-origin> <audio-root>");
  console.error("  node hosting/upload-narration.mjs --watch <site-origin> <audio-root> --receipt <private-progress.json> [--interval-seconds 15]");
  console.error("The upload token is read only from hidden terminal input or stdin; remote origins must use HTTPS.");
}

async function uploadOnce(root) {
  const chapters = await collectChapters(root);
  if (chapters.length === 0) {
    console.error("No narration chapter pairs found under <audio-root>/<voice>/<book>/<chapter>.{mp3,json}.");
    process.exit(2);
  }

  let uploaded = 0;
  for (const chapter of chapters) {
    const prepared = await prepareChapter(chapter);
    await uploadOne(chapter, "mp3", chapter.mp3, prepared.audioStat.size, prepared.audioSha);
    await readbackOne(chapter, "mp3", prepared.audioSha, prepared.audioStat.size);
    uploaded += 1;
    console.log(`PASS uploaded and read back ${chapter.voice}/${chapter.book}/${chapter.chapter}.mp3 (${prepared.audioStat.size} bytes)`);

    await uploadOne(chapter, "json", chapter.json, prepared.cuesStat.size, prepared.cuesSha);
    await readbackOne(chapter, "json", prepared.cuesSha, prepared.cuesStat.size);
    uploaded += 1;
    console.log(`PASS uploaded and read back ${chapter.voice}/${chapter.book}/${chapter.chapter}.json (${prepared.cuesStat.size} bytes)`);
  }

  console.log(`Completed ${uploaded} narration objects across ${chapters.length} chapters.`);
}

async function prepareChapter(chapter) {
  const audioStat = await checkedStat(chapter.mp3, AUDIO_MAX_BYTES);
  const cuesStat = await checkedStat(chapter.json, CUES_MAX_BYTES);
  const audioSha = await hashFile(chapter.mp3, audioStat.size, AUDIO_MAX_BYTES);
  const cuesBytes = await readSmallFile(chapter.json, cuesStat.size, CUES_MAX_BYTES);
  const cuesSha = sha256(cuesBytes);
  let cueDocument;
  try {
    cueDocument = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(cuesBytes));
  } catch {
    throw new Error("invalid_cues_json");
  }

  const cueError = validateCueDocument(cueDocument, {
    voice: chapter.voice,
    book: chapter.book,
    chapter: chapter.chapter,
    audioSha256: audioSha,
  });
  if (cueError) throw new Error(cueError);
  return { audioStat, cuesStat, audioSha, cuesSha };
}

async function watchNarration({ receiptPath, intervalMs }) {
  const absoluteReceipt = path.resolve(receiptPath);
  if (isPathWithin(sourceRoot, absoluteReceipt)) {
    throw new Error("The private progress receipt must be outside the narration source directory.");
  }
  const rootStat = await lstat(sourceRoot);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) {
    throw new Error("audio-root must be a real directory, not a symlink.");
  }

  await mkdir(path.dirname(absoluteReceipt), { recursive: true, mode: 0o700 });
  const sourceRootId = sha256(Buffer.from(sourceRoot, "utf8"));
  const previous = await readProgress(absoluteReceipt);
  if (previous && (previous.target_origin !== siteOrigin || previous.source_root_id !== sourceRootId)) {
    throw new Error("The existing progress receipt belongs to a different origin or source directory.");
  }

  const runId = randomUUID();
  const lock = await acquireLock(`${absoluteReceipt}.lock`, runId);
  const historicalPairs = previous?.pair_states ?? {};
  const retainedStates = Object.fromEntries(
    Object.entries(historicalPairs).map(([key, state]) => [
      key,
      state?.status === "PASS"
        ? { ...state, status: "UNVERIFIED", revalidation_reason: "new_run" }
        : state,
    ]),
  );
  const runStartedAt = new Date().toISOString();
  const progress = {
    schema_version: 1,
    run_id: runId,
    previous_run_id: previous?.run_id ?? null,
    status: "RUNNING",
    started_at: runStartedAt,
    updated_at: runStartedAt,
    completed_at: null,
    target_origin: siteOrigin,
    worker_revision: null,
    release_identity: "UNKNOWN",
    r2_binding_available: false,
    source_root_id: sourceRootId,
    expected_pairs: EXPECTED_PAIR_COUNT,
    verified_pairs: 0,
    missing_pairs: EXPECTED_PAIR_COUNT,
    present_source_pairs: 0,
    incomplete_source_pairs: 0,
    failed_pairs: 0,
    pending_pairs: EXPECTED_PAIR_COUNT,
    attempts_total: 0,
    last_health_check: null,
    missing_pair_keys: expectedPairKeys(),
    pair_states: retainedStates,
  };
  const historyRevision = previous?.worker_revision ?? null;
  const pendingRevalidation = new Map();
  const observations = new Map();
  let lastSummary = "";
  let lastPersistAt = 0;
  let stopRequested = false;
  const stop = () => { stopRequested = true; };
  process.on("SIGINT", stop);
  process.on("SIGTERM", stop);

  try {
    await writeProgress(absoluteReceipt, progress);
    console.log(`RUNNING ${progress.started_at} expected_pairs=${EXPECTED_PAIR_COUNT} interval_seconds=${intervalMs / 1000}`);

    while (!stopRequested) {
      const scan = await collectChapters(sourceRoot, { allowIncomplete: true });
      const chapterByKey = new Map(scan.chapters.map((chapter) => [chapterKey(chapter), chapter]));
      progress.present_source_pairs = chapterByKey.size;
      progress.incomplete_source_pairs = scan.incompleteKeys.length;

      const health = await checkHealth();
      progress.last_health_check = new Date().toISOString();
      if (health.available) {
        progress.r2_binding_available = true;
        if (progress.worker_revision !== health.revision) {
          if (progress.worker_revision !== null) {
            for (const [key, state] of Object.entries(progress.pair_states)) {
              if (state.status === "PASS") {
                pendingRevalidation.set(key, state);
                state.status = "UNVERIFIED";
                state.revalidation_reason = "worker_revision_changed";
              }
            }
          }
          progress.worker_revision = health.revision;
          progress.release_identity = health.revision ?? "UNKNOWN";
        }
      } else {
        progress.r2_binding_available = false;
        for (const [key, state] of Object.entries(progress.pair_states)) {
          if (state.status === "PASS") {
            pendingRevalidation.set(key, state);
            state.status = "UNVERIFIED";
            state.revalidation_reason = "health_unavailable";
          }
        }
      }

      for (const chapter of scan.chapters) {
        if (stopRequested) break;
        const key = chapterKey(chapter);
        let signature;
        try {
          signature = await sourceSignature(chapter);
        } catch {
          continue;
        }
        const state = progress.pair_states[key];
        if (state?.status === "PASS" && state.source_signature === signature) continue;
        if (state?.status === "PASS" && state.source_signature !== signature) {
          state.status = "UNVERIFIED";
          state.revalidation_reason = "source_changed";
          pendingRevalidation.delete(key);
        }
        if (state?.status === "FAIL" && state.source_signature === signature && state.attempts >= MAX_PAIR_ATTEMPTS) continue;

        const observation = observations.get(key);
        const stableCount = observation?.signature === signature ? observation.stable_count + 1 : 1;
        observations.set(key, { signature, stable_count: stableCount });
        if (stableCount < STABLE_SCAN_COUNT || !health.available) continue;

        let previousPass = pendingRevalidation.get(key);
        if (!previousPass && historyRevision === health.revision && health.revision !== null) {
          const candidate = historicalPairs[key];
          if (candidate?.status === "PASS" && candidate.source_signature === signature) previousPass = candidate;
        }
        if (previousPass?.source_signature === signature && health.revision !== null) {
          try {
            const prepared = await prepareChapter(chapter);
            const matchesPreviousPass =
              prepared.audioSha === previousPass.audio_sha256 &&
              prepared.cuesSha === previousPass.cues_sha256 &&
              prepared.audioStat.size === previousPass.audio_bytes &&
              prepared.cuesStat.size === previousPass.cues_bytes;
            if (matchesPreviousPass) {
              await readbackOne(chapter, "mp3", previousPass.audio_sha256, previousPass.audio_bytes);
              await readbackOne(chapter, "json", previousPass.cues_sha256, previousPass.cues_bytes);
              progress.pair_states[key] = {
                ...previousPass,
                status: "PASS",
                source_signature: signature,
                verified_at: new Date().toISOString(),
                reverified_this_run: true,
                attempts: 0,
              };
              pendingRevalidation.delete(key);
              await writeProgress(absoluteReceipt, progress);
              console.log(`PASS reverified current server objects ${key}`);
              continue;
            }
          } catch {
            // Run the full upload attempt below so invalid or stale local data
            // cannot inherit a historical PASS through server readback alone.
          }
          pendingRevalidation.delete(key);
        }

        const priorState = progress.pair_states[key];
        const firstAttempt = priorState?.source_signature === signature ? priorState.attempts : 0;
        let pairPassed = false;
        for (let attempt = firstAttempt + 1; attempt <= MAX_PAIR_ATTEMPTS; attempt += 1) {
          if (stopRequested) break;
          progress.attempts_total += 1;
          try {
            const prepared = await prepareChapter(chapter);
            if (await sourceSignature(chapter) !== signature) throw new Error("source_changed");
            await uploadOne(chapter, "mp3", chapter.mp3, prepared.audioStat.size, prepared.audioSha);
            await readbackOne(chapter, "mp3", prepared.audioSha, prepared.audioStat.size);
            await uploadOne(chapter, "json", chapter.json, prepared.cuesStat.size, prepared.cuesSha);
            await readbackOne(chapter, "json", prepared.cuesSha, prepared.cuesStat.size);
            if (await sourceSignature(chapter) !== signature) throw new Error("source_changed");

            progress.pair_states[key] = {
              status: "PASS",
              source_signature: signature,
              audio_sha256: prepared.audioSha,
              cues_sha256: prepared.cuesSha,
              audio_bytes: prepared.audioStat.size,
              cues_bytes: prepared.cuesStat.size,
              attempts: attempt,
              verified_at: new Date().toISOString(),
              reverified_this_run: false,
            };
            pairPassed = true;
            pendingRevalidation.delete(key);
            await writeProgress(absoluteReceipt, progress);
            console.log(`PASS uploaded and read back current server objects ${key} attempt=${attempt}`);
            break;
          } catch (error) {
            const failureCode = sanitizeFailureCode(error);
            progress.pair_states[key] = {
              status: "FAIL",
              source_signature: signature,
              attempts: attempt,
              failure_code: failureCode,
              failed_at: new Date().toISOString(),
            };
            await writeProgress(absoluteReceipt, progress);
            console.error(`FAIL ${key} attempt=${attempt}/${MAX_PAIR_ATTEMPTS} code=${failureCode}`);
            if (attempt < MAX_PAIR_ATTEMPTS) await delay(PAIR_BACKOFF_MS[attempt - 1]);
          }
        }
        if (!pairPassed && firstAttempt + 1 > MAX_PAIR_ATTEMPTS) {
          progress.pair_states[key] = {
            status: "FAIL",
            source_signature: signature,
            attempts: MAX_PAIR_ATTEMPTS,
            failure_code: priorState?.failure_code ?? "attempts_exhausted",
            failed_at: priorState?.failed_at ?? new Date().toISOString(),
          };
        }
      }

      refreshProgress(progress, chapterByKey, scan.incompleteKeys);
      const summary = [
        progress.verified_pairs,
        progress.missing_pairs,
        progress.present_source_pairs,
        progress.incomplete_source_pairs,
        progress.failed_pairs,
        progress.r2_binding_available,
        progress.worker_revision ?? "UNKNOWN",
      ].join(":");
      const now = Date.now();
      if (summary !== lastSummary || now - lastPersistAt >= 5 * 60 * 1000) {
        progress.updated_at = new Date(now).toISOString();
        progress.last_scan_at = progress.updated_at;
        await writeProgress(absoluteReceipt, progress);
        lastSummary = summary;
        lastPersistAt = now;
        console.log(
          `${progress.status} ${progress.updated_at} verified=${progress.verified_pairs}/${progress.expected_pairs}` +
          ` missing=${progress.missing_pairs} pending=${progress.pending_pairs} failed=${progress.failed_pairs}`,
        );
      }
      if (progress.verified_pairs === EXPECTED_PAIR_COUNT) {
        progress.status = "COMPLETE";
        progress.completed_at = new Date().toISOString();
        progress.updated_at = progress.completed_at;
        await writeProgress(absoluteReceipt, progress);
        console.log(`COMPLETE ${progress.completed_at} verified_pairs=${progress.verified_pairs}`);
        return;
      }
      if (!stopRequested) await delay(intervalMs);
    }

    progress.status = "INTERRUPTED";
    progress.updated_at = new Date().toISOString();
    await writeProgress(absoluteReceipt, progress);
    console.log(`INTERRUPTED ${progress.updated_at} verified=${progress.verified_pairs}/${progress.expected_pairs}`);
  } catch (error) {
    progress.status = "BLOCKED";
    progress.blocker_code = sanitizeFailureCode(error);
    progress.updated_at = new Date().toISOString();
    await writeProgress(absoluteReceipt, progress).catch(() => {});
    console.error(`BLOCKED ${progress.updated_at} code=${progress.blocker_code}`);
    process.exitCode = 1;
  } finally {
    process.off("SIGINT", stop);
    process.off("SIGTERM", stop);
    await finishLock(lock, runId, progress.status);
  }
}

async function checkHealth() {
  const url = new URL("/api/narration-health", siteOrigin);
  url.searchParams.set("watch", randomUUID());
  try {
    const response = await fetch(url, {
      method: "GET",
      cache: "no-store",
      redirect: "error",
      headers: { "Cache-Control": "no-cache" },
    });
    if (!response.ok) return { available: false, revision: null };
    const health = await response.json();
    if (health.service !== "narration-storage" || health.r2_binding_available !== true) {
      return { available: false, revision: null };
    }
    return {
      available: true,
      revision: typeof health.revision === "string" && health.revision.length > 0 ? health.revision : null,
    };
  } catch {
    return { available: false, revision: null };
  }
}

function expectedPairKeys() {
  const keys = [];
  for (const voice of VOICES) {
    for (let book = 1; book <= CHAPTER_COUNTS.length; book += 1) {
      for (let chapter = 1; chapter <= CHAPTER_COUNTS[book - 1]; chapter += 1) {
        keys.push(`${voice}/${book}/${chapter}`);
      }
    }
  }
  return keys;
}

function refreshProgress(progress, chapterByKey, incompleteKeys) {
  const expected = new Set(expectedPairKeys());
  const verified = Object.entries(progress.pair_states)
    .filter(([key, state]) => expected.has(key) && state.status === "PASS")
    .map(([key]) => key);
  const verifiedSet = new Set(verified);
  const missing = [...expected].filter((key) => !verifiedSet.has(key));
  progress.verified_pairs = verified.length;
  progress.missing_pairs = missing.length;
  progress.pending_pairs = missing.length;
  progress.failed_pairs = Object.values(progress.pair_states).filter((state) => state.status === "FAIL").length;
  progress.missing_pair_keys = missing;
  progress.present_source_pairs = chapterByKey.size;
  progress.incomplete_source_pairs = incompleteKeys.length;
}

async function sourceSignature(chapter) {
  const audio = await lstat(chapter.mp3);
  const cues = await lstat(chapter.json);
  if (!audio.isFile() || audio.isSymbolicLink() || !cues.isFile() || cues.isSymbolicLink()) {
    throw new Error("not_regular_file");
  }
  return [audio, cues]
    .map((info) => `${info.size}:${info.mtimeMs}:${info.ctimeMs}:${info.ino}`)
    .join("|");
}

function chapterKey(chapter) {
  return `${chapter.voice}/${chapter.book}/${chapter.chapter}`;
}

function sanitizeFailureCode(error) {
  const message = typeof error?.message === "string" ? error.message : "";
  const responseError = message.match(/HTTP (\d{3}) ([a-z][a-z0-9_]+)/i);
  if (responseError) return `http_${responseError[1]}_${responseError[2]}`;
  if (/source_changed/i.test(message)) return "source_changed";
  if (/size|too_large/i.test(message)) return "invalid_size";
  if (/sha-?256|digest/i.test(message)) return "digest_mismatch";
  if (/cue|json/i.test(message)) return "invalid_cues";
  return "operation_failed";
}

async function readProgress(filename) {
  try {
    const info = await lstat(filename);
    if (info.isSymbolicLink() || !info.isFile()) throw new Error("progress_receipt_not_regular_file");
    const value = JSON.parse(await readFile(filename, "utf8"));
    if (value.schema_version !== 1 || !value.pair_states || typeof value.pair_states !== "object") {
      throw new Error("progress_receipt_invalid");
    }
    await chmod(filename, 0o600);
    return value;
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
}

async function writeProgress(filename, value) {
  const temp = `${filename}.tmp-${randomUUID()}`;
  const handle = await open(temp, "wx", 0o600);
  try {
    const data = Buffer.from(`${JSON.stringify(value)}\n`, "utf8");
    await handle.write(data, 0, data.length, 0);
    await handle.sync();
  } finally {
    await handle.close();
  }
  await chmod(temp, 0o600);
  try {
    const existing = await lstat(filename);
    if (existing.isSymbolicLink() || !existing.isFile()) throw new Error("progress_receipt_not_regular_file");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  await rename(temp, filename);
}

async function acquireLock(filename, runId) {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const handle = await open(filename, "wx", 0o600);
      const data = Buffer.from(`${JSON.stringify({
        run_id: runId,
        status: "RUNNING",
        started_at: new Date().toISOString(),
        pid: process.pid,
        hostname: hostname(),
      })}\n`, "utf8");
      await handle.write(data, 0, data.length, 0);
      await handle.sync();
      return { filename, handle, runId };
    } catch (error) {
      if (error?.code !== "EEXIST") throw error;
      const info = await lstat(filename);
      if (info.isSymbolicLink() || !info.isFile()) throw new Error("watch_lock_not_regular_file");
      let owner;
      try { owner = JSON.parse(await readFile(filename, "utf8")); } catch { owner = null; }
      if (!owner || typeof owner.status !== "string") throw new Error("watch_lock_unverified");
      if (owner?.status === "RUNNING") {
        if (owner.hostname !== hostname()) throw new Error("watch_owned_by_another_host");
        if (isProcessAlive(owner.pid)) throw new Error("watch_already_running_for_receipt");
      }
      await rename(filename, `${filename}.previous-${randomUUID()}`);
    }
  }
  throw new Error("watch_lock_unavailable");
}

async function finishLock(lock, runId, status) {
  if (!lock?.handle) return;
  const data = Buffer.from(`${JSON.stringify({
    run_id: runId,
    status: "FINISHED",
    outcome: status,
    finished_at: new Date().toISOString(),
    pid: process.pid,
    hostname: hostname(),
  })}\n`, "utf8");
  try {
    await lock.handle.truncate(0);
    await lock.handle.write(data, 0, data.length, 0);
    await lock.handle.sync();
  } finally {
    await lock.handle.close();
  }
}

function isProcessAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === "EPERM";
  }
}

function isPathWithin(parent, child) {
  const relative = path.relative(parent, child);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function readCredential() {
  if (!process.stdin.isTTY || typeof process.stdin.setRawMode !== "function") {
    const chunks = [];
    let total = 0;
    for await (const chunk of process.stdin) {
      total += chunk.byteLength;
      if (total > 512) throw new Error("Narration upload token exceeded the 512-byte limit.");
      chunks.push(Buffer.from(chunk));
    }
    return Buffer.concat(chunks).toString("utf8").trim();
  }

  process.stderr.write("Narration upload token (hidden): ");
  const stdin = process.stdin;
  stdin.setRawMode(true);
  stdin.resume();
  const chars = [];
  try {
    for await (const chunk of stdin) {
      for (const byte of chunk) {
        if (byte === 3) throw new Error("Interrupted");
        if (byte === 13 || byte === 10) {
          process.stderr.write("\n");
          return Buffer.from(chars).toString("utf8");
        }
        if (byte === 127 || byte === 8) chars.pop();
        else chars.push(byte);
      }
    }
    return Buffer.from(chars).toString("utf8");
  } finally {
    stdin.setRawMode(false);
    stdin.pause();
  }
}

async function collectChapters(root, { allowIncomplete = false } = {}) {
  const rootStat = await lstat(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) {
    throw new Error("audio-root must be a real directory, not a symlink.");
  }
  const files = [];
  await walk(root, root, files);
  const pairs = new Map();
  for (const filename of files) {
    const relative = path.relative(root, filename).split(path.sep).join("/");
    if (ROOT_CONTROL_FILES.has(relative) || GENERATOR_TEMP_MP3.test(relative)) continue;
    const match = relative.match(/^(af_heart|am_michael)\/([1-9][0-9]?)\/([1-9][0-9]{0,2})\.(mp3|json)$/);
    if (!match) {
      if (/\.(mp3|json)$/i.test(relative)) throw new Error(`Unexpected narration file path: ${relative}`);
      continue;
    }
    const publicPath = `/audio/v1/${match[1]}/${match[2]}/${match[3]}.${match[4]}`;
    const parsed = parseNarrationPath(publicPath);
    if (!parsed) throw new Error(`Invalid narration chapter path: ${relative}`);
    const key = `${parsed.voice}/${parsed.book}/${parsed.chapter}`;
    const pair = pairs.get(key) ?? { ...parsed };
    if (pair[parsed.extension]) throw new Error(`Duplicate narration object: ${relative}`);
    pair[parsed.extension] = filename;
    pairs.set(key, pair);
  }
  const result = [...pairs.values()].sort(
    (left, right) => left.voice.localeCompare(right.voice) || left.book - right.book || left.chapter - right.chapter,
  );
  const incompleteKeys = [];
  const complete = [];
  for (const pair of result) {
    if (!pair.mp3 || !pair.json) {
      const key = chapterKey(pair);
      if (!allowIncomplete) throw new Error(`Chapter pair is incomplete: ${key}`);
      incompleteKeys.push(key);
      continue;
    }
    complete.push(pair);
  }
  return allowIncomplete ? { chapters: complete, incompleteKeys } : complete;
}

async function walk(root, current, output) {
  const entries = await readdir(current, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(current, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`Symlinks are not accepted in the narration directory: ${fullPath}`);
    if (entry.isDirectory()) await walk(root, fullPath, output);
    else if (entry.isFile()) output.push(fullPath);
  }
}

async function checkedStat(filename, maxBytes) {
  const info = await lstat(filename);
  if (!info.isFile() || info.isSymbolicLink()) throw new Error(`Not a regular file: ${filename}`);
  if (info.size <= 0 || info.size > maxBytes) throw new Error(`File size outside allowed bounds: ${filename}`);
  return info;
}

async function hashFile(filename, expectedSize, maxBytes) {
  const hash = createHash("sha256");
  let total = 0;
  const stream = createReadStream(filename);
  for await (const chunk of stream) {
    total += chunk.byteLength;
    if (total > maxBytes) throw new Error(`File grew beyond its size limit: ${filename}`);
    hash.update(chunk);
  }
  if (total !== expectedSize) throw new Error(`File changed while hashing: ${filename}`);
  return hash.digest("hex");
}

async function readSmallFile(filename, expectedSize, maxBytes) {
  const chunks = [];
  let total = 0;
  for await (const chunk of createReadStream(filename)) {
    total += chunk.byteLength;
    if (total > maxBytes) throw new Error(`File grew beyond its size limit: ${filename}`);
    chunks.push(Buffer.from(chunk));
  }
  if (total !== expectedSize) throw new Error(`File changed while reading: ${filename}`);
  return Buffer.concat(chunks, total);
}

async function uploadOne(chapter, extension, filename, size, digest) {
  const endpoint = new URL(
    `/_narration/import/${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}`,
    siteOrigin,
  );
  const contentType = extension === "mp3" ? "audio/mpeg" : "application/json";
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${credentials}`,
      "Content-Type": contentType,
      "Content-Length": String(size),
      "X-Content-SHA256": digest,
    },
    body: createReadStream(filename),
    duplex: "half",
  });
  const result = await readJsonResponse(response, "upload");
  if (!response.ok || result.sha256 !== digest || result.bytes !== size) {
    throw new Error(`Upload failed for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}: HTTP ${response.status} ${result.error ?? "unexpected receipt"}`);
  }
}

async function readbackOne(chapter, extension, digest, size) {
  const url = new URL(
    `/audio/v1/${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}`,
    siteOrigin,
  );
  url.searchParams.set("readback", randomUUID());
  const contentType = extension === "mp3" ? "audio/mpeg" : "application/json";
  // Compare stored byte lengths, not a CDN's compressed transfer size. Node
  // otherwise advertises gzip, and Cloudflare omits Content-Length on HEAD.
  const head = await fetch(url, { method: "HEAD", cache: "no-store", headers: { "Accept-Encoding": "identity" } });
  if (!head.ok || head.headers.get("x-content-sha256") !== digest || Number(head.headers.get("content-length")) !== size) {
    throw new Error(`HEAD readback mismatch for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}: HTTP ${head.status}`);
  }
  if (head.headers.get("content-type")?.split(";", 1)[0] !== contentType) {
    throw new Error(`HEAD readback content type mismatch for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}`);
  }

  const response = await fetch(url, { method: "GET", cache: "no-store", headers: { "Cache-Control": "no-cache", "Accept-Encoding": "identity" } });
  if (!response.ok || !response.body) {
    throw new Error(`GET readback failed for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}: HTTP ${response.status}`);
  }
  const responseLength = response.headers.get("content-length");
  if (responseLength !== null && Number(responseLength) !== size) {
    throw new Error(`GET readback length mismatch for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}`);
  }
  const actual = await hashResponse(response, extension === "mp3" ? AUDIO_MAX_BYTES : CUES_MAX_BYTES);
  if (actual.bytes !== size || actual.sha256 !== digest) {
    throw new Error(`GET readback SHA-256 mismatch for ${chapter.voice}/${chapter.book}/${chapter.chapter}.${extension}`);
  }
}

async function readJsonResponse(response, operation) {
  const text = await response.text();
  try {
    return JSON.parse(text);
  } catch {
    throw new Error(`${operation} returned a non-JSON response (HTTP ${response.status}).`);
  }
}

async function hashResponse(response, maxBytes) {
  const hash = createHash("sha256");
  let total = 0;
  const nodeStream = Readable.fromWeb(response.body);
  for await (const chunk of nodeStream) {
    total += chunk.byteLength;
    if (total > maxBytes) throw new Error("Readback exceeded the configured size limit.");
    hash.update(chunk);
  }
  return { bytes: total, sha256: hash.digest("hex") };
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
