#!/usr/bin/env node

import { createHash, randomUUID } from "node:crypto";
import { createReadStream } from "node:fs";
import { lstat, mkdir, open, readFile, realpath, rename, stat } from "node:fs/promises";
import path from "node:path";
import {
  AUDIO_MAX_BYTES,
  CHAPTER_COUNTS,
  CUES_MAX_BYTES,
  VOICES,
  isSha256,
  r2Key,
  validateCueDocument,
} from "./audio-contract.mjs";

const EXPECTED_PAIR_COUNT = VOICES.length * CHAPTER_COUNTS.reduce((sum, count) => sum + count, 0);
const MAX_CONCURRENCY = 4;
const MAX_ATTEMPTS = 2;
const DEFAULT_INTERVAL_SECONDS = 15;
const REQUEST_TIMEOUT_MS = 20_000;
const RECEIPT_SCHEMA_VERSION = 1;

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

main().catch((error) => {
  console.error(`Verifier stopped: ${safeTopLevelError(error)}`);
  process.exitCode = 1;
});

async function main() {
  const contractPairCount = VOICES.length * CHAPTER_COUNTS.reduce((sum, count) => sum + count, 0);
  if (contractPairCount !== 2_378 || VOICES.join(",") !== "af_heart,am_michael") {
    throw new Error("audio-contract does not describe the expected 2,378 Heart/Michael pairs");
  }
  const siteOrigin = parseSiteOrigin(cli.origin);
  const sourceRoot = path.resolve(cli.sourceRoot);
  const receiptPath = await prepareReceiptPath(cli.receiptPath, sourceRoot);
  const rootInfo = await lstat(sourceRoot);
  if (!rootInfo.isDirectory() || rootInfo.isSymbolicLink()) {
    throw new Error("source-root must be a real directory, not a symlink");
  }
  const realSourceRoot = await realpath(sourceRoot);

  const runId = randomUUID();
  const lock = await acquireLock(`${receiptPath}.lock`, runId);
  const startedAt = new Date().toISOString();
  const progress = createReceipt({ runId, startedAt, siteOrigin, watch: cli.watch });
  let finishedNormally = false;
  let stopRequested = false;
  let lastPersistAt = 0;
  let persistTail = Promise.resolve();
  const setStopRequested = () => { stopRequested = true; };
  process.on("SIGINT", setStopRequested);
  process.on("SIGTERM", setStopRequested);

  const nextRequestNonce = () => randomUUID();
  const persist = async (force = false) => {
    const now = Date.now();
    if (!force && now - lastPersistAt < 5_000) return persistTail;
    lastPersistAt = now;
    progress.updatedAt = new Date().toISOString();
    persistTail = persistTail.then(() => writeReceipt(receiptPath, progress));
    await persistTail;
  };

  try {
    await persist(true);
    console.log(`RUNNING origin=${siteOrigin} expected_pairs=${EXPECTED_PAIR_COUNT} concurrency=${MAX_CONCURRENCY} mode=${cli.watch ? "watch" : "once"}`);
    let scanNumber = 0;
    while (!stopRequested) {
      scanNumber += 1;
      progress.scanNumber = scanNumber;
      progress.lastScanStartedAt = new Date().toISOString();
      const health = await checkHealth(siteOrigin, nextRequestNonce());
      progress.health = health;
      const workerEchoGate = await checkWorkerEchoGate(siteOrigin, nextRequestNonce());
      progress.workerEchoGate = workerEchoGate;

      let processed = 0;
      await runPool(expectedPairs(), MAX_CONCURRENCY, async (pair) => {
        if (stopRequested) return;
        const key = pairKey(pair);
        const current = progress.pairs[key];
        try {
          await inspectPair({
            pair,
            state: current,
            sourceRoot: realSourceRoot,
            siteOrigin,
            nextRequestNonce,
            scanNumber,
            workerEchoGate,
          });
        } catch (error) {
          recordFailure(current, {
            stage: "verifier.internal",
            code: safeErrorCode(error),
            errorName: safeErrorName(error),
            at: new Date().toISOString(),
          });
          current.status = "FAIL";
        }
        processed += 1;
        if (processed % 25 === 0) await persist();
      });

      updateSummary(progress);
      progress.lastScanCompletedAt = new Date().toISOString();
      await persist(true);
      console.log(
        `${progress.status === "COMPLETE" ? "COMPLETE" : "PROGRESS"} scan=${scanNumber} verified=${progress.summary.verifiedPairs}/${EXPECTED_PAIR_COUNT} source_ready=${progress.summary.sourceReadyPairs} pending_source=${progress.summary.pendingSourcePairs} pending_public=${progress.summary.pendingPublicPairs} unverified=${progress.summary.unverifiedPairs} failed=${progress.summary.failedPairs} echo_gate=${workerEchoGate.ready ? "PASS" : "UNVERIFIED"} health_http=${health.httpStatus ?? "UNKNOWN"} r2_binding=${health.r2BindingAvailable ?? "UNKNOWN"} reported_revision=${health.reportedRevision ?? "UNKNOWN"} release_identity=UNKNOWN`,
      );

      if (progress.status === "COMPLETE") {
        finishedNormally = true;
        break;
      }
      if (!cli.watch || stopRequested) {
        progress.status = stopRequested ? "INTERRUPTED" : "INCOMPLETE";
        progress.completedAt = new Date().toISOString();
        await persist(true);
        finishedNormally = true;
        break;
      }
      await delay(cli.intervalSeconds * 1_000, () => stopRequested);
    }
  if (stopRequested && progress.status !== "COMPLETE") {
    progress.status = "INTERRUPTED";
    progress.completedAt = new Date().toISOString();
    await persist(true);
    finishedNormally = true;
  }
    await persistTail;
  } finally {
    process.off("SIGINT", setStopRequested);
    process.off("SIGTERM", setStopRequested);
    await closeLock(lock, runId, finishedNormally ? progress.status : "ABORTED");
  }

  process.exitCode = progress.status === "COMPLETE" ? 0 : 1;
}

function parseArguments(args) {
  if (args.length === 0 || args.includes("--help") || args.includes("-h")) return { help: true };
  let watch = false;
  let receiptPath = null;
  let intervalSeconds = DEFAULT_INTERVAL_SECONDS;
  const positional = [];
  for (let index = 0; index < args.length; index += 1) {
    const value = args[index];
    if (value === "--watch") {
      watch = true;
    } else if (value === "--receipt") {
      receiptPath = args[index + 1] ?? null;
      if (!receiptPath || receiptPath.startsWith("--")) return { error: "--receipt needs an absolute private JSON path." };
      index += 1;
    } else if (value === "--interval-seconds") {
      const raw = args[index + 1];
      if (!raw || !/^\d+$/.test(raw)) return { error: "--interval-seconds needs an integer from 15 to 3600." };
      intervalSeconds = Number(raw);
      if (intervalSeconds < 15 || intervalSeconds > 3_600) return { error: "--interval-seconds must be from 15 to 3600." };
      index += 1;
    } else if (value.startsWith("--")) {
      return { error: "Unknown option." };
    } else {
      positional.push(value);
    }
  }

  if (positional.length !== 2) return { error: "Provide exactly <site-origin> and <source-root>." };
  if (!receiptPath) return { error: "--receipt <absolute-private-json-path> is required." };
  if (!path.isAbsolute(receiptPath)) return { error: "--receipt must be an absolute path." };
  if (!watch && intervalSeconds !== DEFAULT_INTERVAL_SECONDS) return { error: "--interval-seconds requires --watch." };
  return {
    watch,
    receiptPath,
    intervalSeconds,
    origin: positional[0],
    sourceRoot: positional[1],
  };
}

function printUsage() {
  console.error("Usage:");
  console.error("  node hosting/verify-narration.mjs <site-origin> <source-root> --receipt <absolute-private-json-path>");
  console.error("  node hosting/verify-narration.mjs --watch <site-origin> <source-root> --receipt <absolute-private-json-path> [--interval-seconds 15]");
  console.error("Read-only public verifier: sends unauthenticated HEAD and GET requests only; it never uploads, changes credentials, or writes to the narration source.");
  console.error("Remote origins require HTTPS. Plain HTTP is accepted only for localhost, 127.0.0.1, and ::1. The receipt must be new and outside source-root.");
}

function parseSiteOrigin(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("site-origin must be a valid HTTPS origin");
  }
  const localHosts = new Set(["localhost", "127.0.0.1", "[::1]"]);
  const secureRemote = url.protocol === "https:";
  const localHttp = url.protocol === "http:" && localHosts.has(url.hostname.toLowerCase());
  if ((!secureRemote && !localHttp) || url.username || url.password || url.search || url.hash || url.pathname !== "/") {
    throw new Error("site-origin must be an origin using HTTPS; HTTP is allowed only for localhost, 127.0.0.1, or ::1");
  }
  return url.origin;
}

async function prepareReceiptPath(rawReceipt, sourceRoot) {
  if (!path.isAbsolute(rawReceipt)) throw new Error("receipt path must be absolute");
  const receiptPath = path.resolve(rawReceipt);
  if (isPathWithin(sourceRoot, receiptPath)) {
    throw new Error("private receipt must be outside source-root");
  }
  const sourceRealPath = await realpath(sourceRoot);
  const plannedParent = await resolvePlannedDirectory(path.dirname(receiptPath));
  const actualReceipt = path.join(plannedParent, path.basename(receiptPath));
  if (isPathWithin(sourceRealPath, actualReceipt)) {
    throw new Error("private receipt must resolve outside source-root");
  }
  await mkdir(path.dirname(receiptPath), { recursive: true, mode: 0o700 });
  const actualParent = await realpath(path.dirname(receiptPath));
  if (isPathWithin(sourceRealPath, path.join(actualParent, path.basename(receiptPath)))) {
    throw new Error("private receipt must resolve outside source-root");
  }
  for (const candidate of [receiptPath, `${receiptPath}.lock`]) {
    try {
      const info = await lstat(candidate);
      if (info.isSymbolicLink()) throw new Error("receipt and lock paths cannot be symlinks");
      if (candidate === receiptPath) throw new Error("receipt path already exists; choose a new private path for this run");
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
  return receiptPath;
}

async function resolvePlannedDirectory(directoryPath) {
  let candidate = path.resolve(directoryPath);
  const remaining = [];
  while (true) {
    try {
      const resolved = await realpath(candidate);
      return path.join(resolved, ...remaining.reverse());
    } catch (error) {
      if (error?.code !== "ENOENT" && error?.code !== "ENOTDIR") throw error;
      const parent = path.dirname(candidate);
      if (parent === candidate) throw error;
      remaining.push(path.basename(candidate));
      candidate = parent;
    }
  }
}

function createReceipt({ runId, startedAt, siteOrigin, watch }) {
  const pairs = {};
  for (const pair of expectedPairs()) {
    const key = pairKey(pair);
    pairs[key] = {
      status: "PENDING_SOURCE",
      sourceStatus: "PENDING_SOURCE",
      objectKeys: {
        audio: r2Key({ ...pair, extension: "mp3" }),
        cues: r2Key({ ...pair, extension: "json" }),
      },
      objects: { audio: emptyObjectState(), cues: emptyObjectState() },
      attempts: 0,
      failureAttempts: 0,
      attemptFailures: [],
      firstFailure: null,
      lastFailure: null,
      sourceFailure: null,
      sourceStatSignature: null,
      sourceSignature: null,
      audioSha256: null,
      cuesSha256: null,
      audioBytes: null,
      cuesBytes: null,
      verifiedAt: null,
    };
  }
  return {
    schemaVersion: RECEIPT_SCHEMA_VERSION,
    runId,
    mode: watch ? "watch" : "once",
    status: "RUNNING",
    startedAt,
    updatedAt: startedAt,
    completedAt: null,
    origin: siteOrigin,
    requestPolicy: {
      authentication: "none",
      sameOriginOnly: true,
      cacheMode: "no-store",
      cacheControl: "no-cache, no-store, max-age=0",
      audioVerificationQueryParameter: "verify",
      healthCacheBusterQueryParameter: "__narration_health_probe",
      echoResponseHeader: "X-Narration-Verification",
      exactEchoRequired: true,
      freshUuidPerPublicRequest: true,
      maximumPairConcurrency: MAX_CONCURRENCY,
    },
    expectedPairs: EXPECTED_PAIR_COUNT,
    scanNumber: 0,
    health: null,
    workerEchoGate: null,
    sourceCorpusSha256: null,
    sourceCorpusComplete: false,
    firstFailure: null,
    summary: emptySummary(),
    pairs,
  };
}

function emptySummary() {
  return {
    verifiedPairs: 0,
    sourceReadyPairs: 0,
    pendingSourcePairs: EXPECTED_PAIR_COUNT,
    pendingPublicPairs: 0,
    unverifiedPairs: 0,
    failedPairs: 0,
    notReadyPairs: 0,
  };
}

function emptyObjectState() {
  return {
    status: "NOT_READY",
    attempts: 0,
    headStatus: null,
    getStatus: null,
    bytes: null,
    sha256: null,
    headEtagSha256: null,
    getEtagSha256: null,
    headCacheStatus: null,
    getCacheStatus: null,
    headCacheEvidence: "UNKNOWN",
    getCacheEvidence: "UNKNOWN",
    headCacheStatusSha256: null,
    getCacheStatusSha256: null,
    headVerificationId: null,
    getVerificationId: null,
    headEchoMatched: null,
    getEchoMatched: null,
    cfRay: null,
    remoteSignature: null,
    firstFailure: null,
    lastFailure: null,
  };
}

function expectedPairs() {
  const pairs = [];
  for (const voice of VOICES) {
    for (let book = 1; book <= CHAPTER_COUNTS.length; book += 1) {
      for (let chapter = 1; chapter <= CHAPTER_COUNTS[book - 1]; chapter += 1) {
        pairs.push({ voice, book, chapter });
      }
    }
  }
  return pairs;
}

function pairKey({ voice, book, chapter }) {
  return `${voice}/${book}/${chapter}`;
}

async function inspectPair({ pair, state, sourceRoot, siteOrigin, nextRequestNonce, scanNumber, workerEchoGate }) {
  const previousStatus = state.status;
  const previousSourceSignature = state.sourceSignature;
  const local = await readLocalPair(sourceRoot, pair, state);
  if (local.status === "PENDING_SOURCE") {
    state.status = "PENDING_SOURCE";
    state.sourceStatus = "PENDING_SOURCE";
    state.sourceStatSignature = local.statSignature;
    state.sourceFailure = null;
    state.sourceSignature = null;
    state.audioSha256 = null;
    state.cuesSha256 = null;
    state.audioBytes = null;
    state.cuesBytes = null;
    state.verifiedAt = null;
    return;
  }
  if (local.status === "FAIL") {
    state.status = "FAIL";
    state.sourceStatus = "FAIL";
    state.sourceStatSignature = local.statSignature ?? state.sourceStatSignature;
    state.sourceSignature = local.sourceSignature ?? state.sourceSignature;
    state.sourceFailure = local.failure;
    recordFailure(state, local.failure);
    return;
  }

  state.sourceStatus = "PASS";
  state.sourceStatSignature = local.statSignature;
  state.sourceSignature = local.sourceSignature;
  state.audioSha256 = local.audioSha256;
  state.cuesSha256 = local.cuesSha256;
  state.audioBytes = local.audioBytes;
  state.cuesBytes = local.cuesBytes;
  state.sourceFailure = null;

  if (!workerEchoGate.ready) {
    if (state.status === "PASS" && state.verifiedAt) {
      state.lastVerifiedAt = state.verifiedAt;
      state.lastVerifiedRemoteSignature = state.remoteSignature;
    }
    state.status = "UNVERIFIED";
    state.verifiedAt = null;
    state.unverifiedAt = new Date().toISOString();
    state.unverifiedReason = workerEchoGate.reason;
    state.remoteSignature = null;
    state.headProbeSignature = null;
    state.objects = { audio: emptyObjectState(), cues: emptyObjectState() };
    for (const object of Object.values(state.objects)) {
      object.status = "UNVERIFIED";
      object.unverifiedReason = workerEchoGate.reason;
    }
    return;
  }

  if (previousStatus === "PASS" && previousSourceSignature === local.sourceSignature) {
    state.status = "PASS";
    return;
  }
  if (previousSourceSignature && previousSourceSignature !== local.sourceSignature) {
    state.reopenReason = "source_signature_changed";
    state.attempts = 0;
    state.failureAttempts = 0;
    state.retryable = false;
    state.objects = { audio: emptyObjectState(), cues: emptyObjectState() };
    state.status = "NOT_READY";
  }
  if (previousStatus === "FAIL" && previousSourceSignature === local.sourceSignature) {
    if (!(state.retryable && state.failureAttempts < MAX_ATTEMPTS)) {
      const previousHeadSignature = state.headProbeSignature;
      if (!previousHeadSignature) return;
      const observed = await probePairEtags({ pair, siteOrigin, nextRequestNonce });
      const newSignature = combinedProbeSignature(observed);
      state.lastRemoteProbe = {
        at: new Date().toISOString(),
        signature: newSignature,
        observations: observed.map((item) => ({
          status: item.status,
          verificationId: item.verificationId,
          echoMatched: item.echoMatched,
          etagSha256: item.etagSha256,
        })),
      };
      if (!newSignature || newSignature === previousHeadSignature) return;
      state.reopenReason = "remote_signature_changed";
      state.status = "NOT_READY";
      state.attempts = 0;
      state.failureAttempts = 0;
      state.retryable = false;
      state.objects = { audio: emptyObjectState(), cues: emptyObjectState() };
    }
  }
  if (state.status === "PASS" && previousSourceSignature === local.sourceSignature) return;
  state.lastAttemptSourceSignature = local.sourceSignature;
  state.lastAttemptScan = scanNumber;
  let result;
  do {
    state.attempts += 1;
    result = await verifyPairObjects({ pair, local, siteOrigin, nextRequestNonce });
    state.objects = result.objects;
    state.remoteSignature = result.remoteSignature;
    state.headProbeSignature = result.headProbeSignature;
    state.retryable = result.retryable;
    if (result.status !== "FAIL") break;
    state.failureAttempts += 1;
    if (result.failure) {
      state.attemptFailures ??= [];
      if (state.attemptFailures.length < MAX_ATTEMPTS) state.attemptFailures.push(result.failure);
      recordFailure(state, result.failure);
    }
    if (!result.retryable || state.failureAttempts >= MAX_ATTEMPTS) break;
    await delay(750, () => false);
  } while (state.failureAttempts < MAX_ATTEMPTS);

  if (result.status === "PASS") {
    state.status = "PASS";
    state.failureAttempts = 0;
    state.verifiedAt = new Date().toISOString();
    return;
  }
  if (result.status === "PENDING_PUBLIC") {
    state.status = "PENDING_PUBLIC";
    state.failureAttempts = 0;
    state.retryable = false;
    state.verifiedAt = null;
    state.lastFailure = result.failure ?? null;
    return;
  }
  if (result.status === "UNVERIFIED") {
    state.status = "UNVERIFIED";
    state.verifiedAt = null;
    state.unverifiedAt = new Date().toISOString();
    state.unverifiedReason = result.unverifiedReason ?? "verification_echo_missing";
    return;
  }

  state.status = "FAIL";
  state.verifiedAt = null;
  recordFailure(state, result.failure);
}

async function readLocalPair(sourceRoot, pair, previous) {
  const voicePath = path.join(sourceRoot, pair.voice);
  const bookPath = path.join(voicePath, String(pair.book));
  const audioPath = path.join(bookPath, `${pair.chapter}.mp3`);
  const cuesPath = path.join(bookPath, `${pair.chapter}.json`);
  try {
    const [voiceStat, bookStat] = await Promise.all([lstat(voicePath), lstat(bookPath)]);
    if (!voiceStat.isDirectory() || voiceStat.isSymbolicLink() || !bookStat.isDirectory() || bookStat.isSymbolicLink()) {
      return { status: "FAIL", failure: failureRecord("local.inspect", "source_directory_not_regular", null) };
    }
  } catch (error) {
    if (error?.code === "ENOENT" || error?.code === "ENOTDIR") {
      return { status: "PENDING_SOURCE", statSignature: await existingFileSignature(audioPath, cuesPath) };
    }
    return { status: "FAIL", failure: failureRecord("local.inspect", safeFsCode(error), null) };
  }
  let audioStat;
  let cuesStat;
  try {
    [audioStat, cuesStat] = await Promise.all([lstat(audioPath, { bigint: true }), lstat(cuesPath, { bigint: true })]);
  } catch (error) {
    if (error?.code === "ENOENT" || error?.code === "ENOTDIR") {
      const partialSignature = await existingFileSignature(audioPath, cuesPath);
      return { status: "PENDING_SOURCE", statSignature: partialSignature };
    }
    return { status: "FAIL", failure: failureRecord("local.inspect", safeFsCode(error), null) };
  }

  const statSignature = sourceStatSignature(audioStat, cuesStat);
  if (previous.sourceStatus === "FAIL" && previous.sourceStatSignature === statSignature && previous.sourceFailure) {
    return { status: "FAIL", statSignature, sourceSignature: previous.sourceSignature, failure: previous.sourceFailure };
  }
  if (!audioStat.isFile() || !cuesStat.isFile() || audioStat.isSymbolicLink() || cuesStat.isSymbolicLink()) {
    return {
      status: "FAIL",
      statSignature,
      failure: failureRecord("local.inspect", "source_not_regular_file", null),
    };
  }
  if (audioStat.size <= 0n || audioStat.size > BigInt(AUDIO_MAX_BYTES) || cuesStat.size <= 0n || cuesStat.size > BigInt(CUES_MAX_BYTES)) {
    return {
      status: "FAIL",
      statSignature,
      failure: failureRecord("local.inspect", "source_size_out_of_range", null),
    };
  }

  if (
    previous.sourceStatus === "PASS" &&
    previous.sourceStatSignature === statSignature &&
    previous.sourceSignature &&
    previous.audioSha256 &&
    previous.cuesSha256
  ) {
    return {
      status: "PASS",
      statSignature,
      sourceSignature: previous.sourceSignature,
      audioSha256: previous.audioSha256,
      cuesSha256: previous.cuesSha256,
      audioBytes: previous.audioBytes,
      cuesBytes: previous.cuesBytes,
      cueDocument: null,
      reusedContentHash: true,
    };
  }

  try {
    const [audioDigest, cuesBytes] = await Promise.all([
      hashFile(audioPath, AUDIO_MAX_BYTES),
      readBoundedFile(cuesPath, CUES_MAX_BYTES),
    ]);
    const audioAfter = await lstat(audioPath, { bigint: true });
    const cuesAfter = await lstat(cuesPath, { bigint: true });
    const afterSignature = sourceStatSignature(audioAfter, cuesAfter);
    if (afterSignature !== statSignature) {
      return { status: "PENDING_SOURCE", statSignature: afterSignature };
    }

    let cueDocument;
    try {
      cueDocument = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(cuesBytes));
    } catch {
      return {
        status: "FAIL",
        statSignature,
        failure: failureRecord("local.cues_parse", "invalid_cues_json", null),
      };
    }
    const cueError = validateCueDocument(cueDocument, {
      voice: pair.voice,
      book: pair.book,
      chapter: pair.chapter,
      audioSha256: audioDigest.sha256,
    });
    if (cueError) {
      return {
        status: "FAIL",
        statSignature,
        sourceSignature: sourceSignature(audioDigest.sha256, sha256(cuesBytes)),
        failure: failureRecord("local.cue_contract", cueError, null),
      };
    }
    const cuesSha256 = sha256(cuesBytes);
    return {
      status: "PASS",
      statSignature,
      sourceSignature: sourceSignature(audioDigest.sha256, cuesSha256),
      audioSha256: audioDigest.sha256,
      cuesSha256,
      audioBytes: audioDigest.bytes,
      cuesBytes: cuesBytes.byteLength,
      cueDocument,
      reusedContentHash: false,
    };
  } catch (error) {
    if (error?.code === "source_changed_during_read" || error?.code === "ENOENT" || error?.code === "ENOTDIR") {
      const changedSignature = await existingFileSignature(audioPath, cuesPath);
      return { status: "PENDING_SOURCE", statSignature: changedSignature };
    }
    return {
      status: "FAIL",
      statSignature,
      failure: failureRecord("local.read", safeErrorCode(error), null, safeErrorName(error)),
    };
  }
}

async function hashFile(filePath, maxBytes) {
  const hash = createHash("sha256");
  let bytes = 0;
  for await (const chunk of createReadStream(filePath)) {
    bytes += chunk.byteLength;
    if (bytes > maxBytes) throw Object.assign(new Error("source_too_large"), { code: "source_too_large" });
    hash.update(chunk);
  }
  if (bytes === 0) throw Object.assign(new Error("source_empty"), { code: "source_empty" });
  return { sha256: hash.digest("hex"), bytes };
}

async function readBoundedFile(filePath, maxBytes) {
  const info = await stat(filePath);
  if (!info.isFile() || info.size <= 0 || info.size > maxBytes) {
    throw Object.assign(new Error("source_size_out_of_range"), { code: "source_size_out_of_range" });
  }
  const bytes = await readFile(filePath);
  if (bytes.byteLength !== info.size || bytes.byteLength > maxBytes) {
    throw Object.assign(new Error("source_changed_during_read"), { code: "source_changed_during_read" });
  }
  return bytes;
}

async function existingFileSignature(audioPath, cuesPath) {
  const signatures = await Promise.all([audioPath, cuesPath].map(async (filePath) => {
    try {
      return statFingerprint(await lstat(filePath, { bigint: true }));
    } catch (error) {
      return error?.code === "ENOENT" || error?.code === "ENOTDIR" ? "missing" : safeFsCode(error);
    }
  }));
  return sha256(Buffer.from(signatures.join("\0"), "utf8"));
}

function sourceStatSignature(audioStat, cuesStat) {
  return sha256(Buffer.from(`${statFingerprint(audioStat)}\0${statFingerprint(cuesStat)}`, "utf8"));
}

function statFingerprint(info) {
  return [info.dev, info.ino, info.mode, info.size, info.mtimeNs, info.ctimeNs].map(String).join(":");
}

function sourceSignature(audioSha256, cuesSha256) {
  return sha256(Buffer.from(`${audioSha256}\0${cuesSha256}`, "utf8"));
}

async function verifyPairObjects({ pair, local, siteOrigin, nextRequestNonce }) {
  const objectStates = {
    audio: emptyObjectState(),
    cues: emptyObjectState(),
  };
  const checks = [
    { name: "audio", extension: "mp3", expectedSha256: local.audioSha256, expectedBytes: local.audioBytes, maxBytes: AUDIO_MAX_BYTES, expectedType: "audio/mpeg" },
    { name: "cues", extension: "json", expectedSha256: local.cuesSha256, expectedBytes: local.cuesBytes, maxBytes: CUES_MAX_BYTES, expectedType: "application/json" },
  ];
  const errors = [];
  let pendingPublic = false;
  let unverified = false;

  for (const check of checks) {
    const result = await verifyOneObject({ pair, check, siteOrigin, nextRequestNonce, local });
    objectStates[check.name] = result.state;
    if (result.pending) pendingPublic = true;
    if (result.unverified) {
      unverified = true;
      break;
    }
    if (result.failure) errors.push(result.failure);
  }

  const remoteSignature = combinedRemoteSignature({ objects: objectStates });
  const headProbeSignature = combinedProbeSignature([
    { remoteSignature: objectStates.audio.headSignature ?? "" },
    { remoteSignature: objectStates.cues.headSignature ?? "" },
  ]);
  const retryable = Object.values(objectStates).some((object) => object.transient === true);
  if (errors.length > 0) return { status: "FAIL", objects: objectStates, remoteSignature, headProbeSignature, retryable, failure: errors[0] };
  if (unverified) return { status: "UNVERIFIED", objects: objectStates, remoteSignature, headProbeSignature, retryable: false, failure: null };
  if (pendingPublic) return { status: "PENDING_PUBLIC", objects: objectStates, remoteSignature, headProbeSignature, retryable: false, failure: null };
  return { status: "PASS", objects: objectStates, remoteSignature, headProbeSignature, retryable: false, failure: null };
}

async function verifyOneObject({ pair, check, siteOrigin, nextRequestNonce, local }) {
  const pathname = `/audio/v1/${pair.voice}/${pair.book}/${pair.chapter}.${check.extension}`;
  const state = emptyObjectState();
  state.key = r2Key({ ...pair, extension: check.extension });
  state.attempts = 1;
  let headRequest;
  try {
    headRequest = await requestPublic(siteOrigin, pathname, "HEAD", nextRequestNonce());
  } catch (error) {
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.head`, "network_error", null, safeErrorName(error));
    state.firstFailure = state.lastFailure;
    state.remoteSignature = sha256(Buffer.from(`HEAD:network:${safeErrorName(error)}`, "utf8"));
    state.headSignature = state.remoteSignature;
    state.transient = isTransientError(error);
    return { state, failure: state.lastFailure, pending: false };
  }
  const headResponse = headRequest.response;
  state.headStatus = headResponse.status;
  state.headVerificationId = headRequest.verificationId;
  state.headEchoMatched = headRequest.echoMatched;
  state.headEtagSha256 = etagDigest(headResponse.headers.get("etag"));
  state.headCacheStatus = safeCacheStatus(headResponse.headers.get("cf-cache-status"));
  state.headCacheEvidence = cacheEvidence(headResponse.headers);
  state.headCacheStatusSha256 = cacheStatusDigest(headResponse.headers);
  state.cfRay = safeCorrelationHeader(headResponse.headers.get("cf-ray"));
  state.headSignature = signatureFromObservation("HEAD", headResponse.status, headResponse.headers);
  if (!headRequest.echoMatched) {
    await headResponse.body?.cancel();
    state.status = "UNVERIFIED";
    state.unverifiedReason = "verification_echo_missing_or_mismatched";
    return { state, failure: null, pending: false, unverified: true };
  }
  let headMetadataFailure = validateHeaders(headResponse.headers, check, "head");
  if (headResponse.status === 404) {
    state.status = "PENDING_PUBLIC";
    state.lastFailure = failureRecord(`remote.${check.name}.head`, "object_not_found", 404);
    state.remoteSignature = state.headSignature;
    return { state, pending: true, failure: null };
  }
  if (headResponse.status !== 200) {
    const code = `http_${headResponse.status}`;
    const isTransient = isTransientStatus(headResponse.status);
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.head`, code, headResponse.status);
    state.firstFailure = state.lastFailure;
    state.remoteSignature = state.headSignature;
    state.transient = isTransient;
    return { state, failure: state.lastFailure, pending: false };
  }

  let getRequest;
  try {
    getRequest = await requestPublic(siteOrigin, pathname, "GET", nextRequestNonce());
  } catch (error) {
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.get`, "network_error", null, safeErrorName(error));
    state.firstFailure = state.lastFailure;
    state.remoteSignature = state.headSignature;
    state.transient = isTransientError(error);
    return { state, failure: state.lastFailure, pending: false };
  }
  const getResponse = getRequest.response;
  state.getStatus = getResponse.status;
  state.getVerificationId = getRequest.verificationId;
  state.getEchoMatched = getRequest.echoMatched;
  state.getEtagSha256 = etagDigest(getResponse.headers.get("etag"));
  state.getCacheStatus = safeCacheStatus(getResponse.headers.get("cf-cache-status"));
  state.getCacheEvidence = cacheEvidence(getResponse.headers);
  state.getCacheStatusSha256 = cacheStatusDigest(getResponse.headers);
  state.cfRay ??= safeCorrelationHeader(getResponse.headers.get("cf-ray"));
  if (!getRequest.echoMatched) {
    await getResponse.body?.cancel();
    state.status = "UNVERIFIED";
    state.unverifiedReason = "verification_echo_missing_or_mismatched";
    return { state, failure: null, pending: false, unverified: true };
  }
  const getMetadataFailure = validateHeaders(getResponse.headers, check, "get");
  if (getResponse.status === 404) {
    await getResponse.body?.cancel();
    state.status = "PENDING_PUBLIC";
    state.lastFailure = failureRecord(`remote.${check.name}.get`, "object_not_found", 404);
    state.remoteSignature = combinedObservationSignature(headResponse, getResponse);
    return { state, pending: true, failure: null };
  }
  if (getResponse.status !== 200) {
    await getResponse.body?.cancel();
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.get`, `http_${getResponse.status}`, getResponse.status);
    state.firstFailure = state.lastFailure;
    state.remoteSignature = combinedObservationSignature(headResponse, getResponse);
    state.transient = isTransientStatus(getResponse.status);
    return { state, failure: state.lastFailure, pending: false };
  }

  let bodyResult;
  try {
    bodyResult = await hashResponseBody(getResponse, check.maxBytes, check.name === "cues");
  } catch (error) {
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.get_body`, safeErrorCode(error), null, safeErrorName(error));
    state.firstFailure = state.lastFailure;
    state.remoteSignature = combinedObservationSignature(headResponse, getResponse);
    state.transient = isTransientError(error);
    return { state, failure: state.lastFailure, pending: false };
  }
  state.bytes = bodyResult.bytes;
  state.sha256 = bodyResult.sha256;
  state.remoteSignature = combinedObservationSignature(headResponse, getResponse, bodyResult.sha256, bodyResult.bytes);

  if (headMetadataFailure || getMetadataFailure) {
    state.status = "FAIL";
    state.lastFailure = headMetadataFailure ?? getMetadataFailure;
  } else if (
    bodyResult.bytes !== check.expectedBytes ||
    bodyResult.sha256 !== check.expectedSha256 ||
    (bodyResult.contentLength !== null && bodyResult.contentLength !== bodyResult.bytes)
  ) {
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.get_body`, bodyResult.bytes !== check.expectedBytes ? "byte_count_mismatch" : "sha256_mismatch", 200);
  } else if (
    headResponse.headers.get("etag") &&
    getResponse.headers.get("etag") &&
    headResponse.headers.get("etag") !== getResponse.headers.get("etag")
  ) {
    state.status = "FAIL";
    state.lastFailure = failureRecord(`remote.${check.name}.etag`, "etag_changed_during_read", 200);
  } else if (check.name === "cues") {
    let remoteCueDocument;
    try {
      remoteCueDocument = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bodyResult.buffer));
    } catch {
      state.status = "FAIL";
      state.lastFailure = failureRecord("remote.cues.json_parse", "invalid_cues_json", 200);
    }
    if (!state.lastFailure) {
      const cueError = validateCueDocument(remoteCueDocument, {
        voice: pair.voice,
        book: pair.book,
        chapter: pair.chapter,
        audioSha256: local.audioSha256,
      });
      if (cueError) {
        state.status = "FAIL";
        state.lastFailure = failureRecord("remote.cues.cue_contract", cueError, 200);
      } else {
        state.status = "PASS";
      }
    }
  } else {
    state.status = "PASS";
  }

  if (state.status === "FAIL") state.firstFailure = state.lastFailure;
  return {
    state,
    failure: state.lastFailure,
    pending: false,
    unverified: state.status === "UNVERIFIED",
  };
}

async function requestPublic(origin, pathname, method, nonce) {
  const url = new URL(pathname, origin);
  const verificationId = nonce ?? randomUUID();
  url.searchParams.set("verify", verificationId);
  const response = await fetch(url, {
    method,
    cache: "no-store",
    credentials: "omit",
    redirect: "error",
    signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    headers: {
      "Cache-Control": "no-cache, no-store, max-age=0",
      Pragma: "no-cache",
      Accept: method === "HEAD" ? "*/*" : "application/json, audio/mpeg, */*;q=0.1",
    },
  });
  if (new URL(response.url).origin !== origin || new URL(response.url).pathname !== pathname) {
    await response.body?.cancel();
    throw Object.assign(new Error("unexpected_response_origin"), { code: "unexpected_response_origin" });
  }
  const echo = response.headers.get("x-narration-verification");
  return {
    response,
    verificationId,
    echoMatched: echo === verificationId,
    echoedVerificationId: echo === verificationId ? verificationId : null,
  };
}

async function checkWorkerEchoGate(origin, verificationId) {
  const checkedAt = new Date().toISOString();
  const pathname = "/audio/v1/af_heart/1/1.mp3";
  try {
    const request = await requestPublic(origin, pathname, "HEAD", verificationId);
    const response = request.response;
    const status = response.status;
    const echoMatched = request.echoMatched;
    const cacheEvidenceStatus = cacheEvidence(response.headers);
    const value = {
      checkedAt,
      status,
      verificationId: request.verificationId,
      echoMatched,
      echoedVerificationId: request.echoedVerificationId,
      cacheStatus: safeCacheStatus(response.headers.get("cf-cache-status")),
      cacheEvidence: cacheEvidenceStatus,
      cfRay: safeCorrelationHeader(response.headers.get("cf-ray")),
      ready: echoMatched && (status === 200 || status === 404),
      reason: null,
    };
    if (!echoMatched) value.reason = "verification_echo_missing_or_mismatched";
    else if (!value.ready) value.reason = `worker_probe_http_${status}`;
    await response.body?.cancel();
    return value;
  } catch (error) {
    return {
      checkedAt,
      status: null,
      verificationId,
      echoMatched: false,
      echoedVerificationId: null,
      cacheStatus: null,
      cacheEvidence: "UNKNOWN",
      cfRay: null,
      ready: false,
      reason: safeErrorCode(error),
      errorName: safeErrorName(error),
    };
  }
}

function validateHeaders(headers, check, stage) {
  const rawLength = headers.get("content-length");
  if (rawLength !== null && (!/^\d+$/.test(rawLength) || Number(rawLength) !== check.expectedBytes)) {
    return failureRecord(`remote.${check.name}.${stage}_metadata`, "content_length_mismatch", null);
  }
  const contentType = headers.get("content-type");
  if (contentType && mediaType(contentType) !== check.expectedType) {
    return failureRecord(`remote.${check.name}.${stage}_metadata`, "content_type_mismatch", null);
  }
  const declaredSha = headers.get("x-content-sha256");
  if (declaredSha && (!isSha256(declaredSha) || declaredSha.toLowerCase() !== check.expectedSha256)) {
    return failureRecord(`remote.${check.name}.${stage}_metadata`, "declared_sha256_mismatch", null);
  }
  return null;
}

async function hashResponseBody(response, maxBytes, keepBuffer) {
  if (!response.body) throw Object.assign(new Error("empty_response_body"), { code: "empty_response_body" });
  const reader = response.body.getReader();
  const hash = createHash("sha256");
  const chunks = keepBuffer ? [] : null;
  const declaredLength = parseOptionalLength(response.headers.get("content-length"));
  let bytes = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      bytes += value.byteLength;
      if (bytes > maxBytes) {
        await reader.cancel();
        throw Object.assign(new Error("response_too_large"), { code: "response_too_large" });
      }
      hash.update(value);
      if (keepBuffer) chunks.push(Buffer.from(value));
    }
  } finally {
    reader.releaseLock();
  }
  const buffer = keepBuffer ? Buffer.concat(chunks, bytes) : null;
  return {
    sha256: hash.digest("hex"),
    bytes,
    contentLength: declaredLength,
    buffer,
  };
}

async function checkHealth(origin, nonce) {
  const checkedAt = new Date().toISOString();
  const url = new URL("/api/narration-health", origin);
  url.searchParams.set("__narration_health_probe", nonce);
  try {
    const response = await fetch(url, {
      method: "GET",
      cache: "no-store",
      credentials: "omit",
      redirect: "error",
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      headers: { "Cache-Control": "no-cache, no-store, max-age=0", Pragma: "no-cache", Accept: "application/json" },
    });
    if (new URL(response.url).origin !== origin || new URL(response.url).pathname !== "/api/narration-health") {
      await response.body?.cancel();
      return { checkedAt, httpStatus: response.status, service: null, r2BindingAvailable: null, reportedRevision: null, error: "unexpected_response_origin" };
    }
    if (response.status !== 200) {
      await response.body?.cancel();
      return { checkedAt, httpStatus: response.status, service: null, r2BindingAvailable: null, reportedRevision: null, error: `http_${response.status}` };
    }
    const body = await readResponseAtMost(response, 64 * 1024);
    let value;
    try {
      value = JSON.parse(body.toString("utf8"));
    } catch {
      return { checkedAt, httpStatus: 200, service: null, r2BindingAvailable: null, reportedRevision: null, error: "invalid_health_json" };
    }
    const revision = typeof value?.revision === "string" && value.revision.length <= 128 && /^[A-Za-z0-9._:-]+$/.test(value.revision)
      ? value.revision
      : null;
    return {
      checkedAt,
      httpStatus: 200,
      service: value?.service === "narration-storage" ? value.service : null,
      r2BindingAvailable: typeof value?.r2_binding_available === "boolean" ? value.r2_binding_available : null,
      reportedRevision: revision,
      releaseIdentity: "UNKNOWN",
      error: null,
    };
  } catch (error) {
    return {
      checkedAt,
      httpStatus: null,
      service: null,
      r2BindingAvailable: null,
      reportedRevision: null,
      releaseIdentity: "UNKNOWN",
      error: safeErrorCode(error),
      errorName: safeErrorName(error),
    };
  }
}

async function readResponseAtMost(response, maxBytes) {
  if (!response.body) return Buffer.alloc(0);
  const reader = response.body.getReader();
  const chunks = [];
  let bytes = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      bytes += value.byteLength;
      if (bytes > maxBytes) {
        await reader.cancel();
        throw Object.assign(new Error("health_response_too_large"), { code: "health_response_too_large" });
      }
      chunks.push(Buffer.from(value));
    }
  } finally {
    reader.releaseLock();
  }
  return Buffer.concat(chunks, bytes);
}

async function probePairEtags({ pair, siteOrigin, nextRequestNonce }) {
  const observations = [];
  for (const extension of ["mp3", "json"]) {
    const pathname = `/audio/v1/${pair.voice}/${pair.book}/${pair.chapter}.${extension}`;
    try {
      const request = await requestPublic(siteOrigin, pathname, "HEAD", nextRequestNonce());
      const response = request.response;
      observations.push({
        status: response.status,
        verificationId: request.verificationId,
        echoMatched: request.echoMatched,
        etagSha256: etagDigest(response.headers.get("etag")),
        remoteSignature: request.echoMatched ? signatureFromObservation("HEAD", response.status, response.headers) : null,
      });
      await response.body?.cancel();
    } catch (error) {
      observations.push({ status: null, verificationId: null, echoMatched: false, etagSha256: null, remoteSignature: null });
    }
  }
  return observations;
}

function combinedProbeSignature(observations) {
  if (observations.some((item) => !item.remoteSignature)) return null;
  return sha256(Buffer.from(observations.map((item) => item.remoteSignature).join("\0"), "utf8"));
}

function combinedRemoteSignature(state) {
  const objects = state.objects ?? {};
  const values = [objects.audio?.remoteSignature, objects.cues?.remoteSignature].filter(Boolean);
  return values.length === 0 ? null : sha256(Buffer.from(values.join("\0"), "utf8"));
}

function combinedObservationSignature(head, get, bodySha256 = null, bodyBytes = null) {
  const value = [
    signatureFromObservation("HEAD", head.status, head.headers),
    signatureFromObservation("GET", get.status, get.headers),
    bodySha256 ?? "",
    bodyBytes ?? "",
  ].join("\0");
  return sha256(Buffer.from(value, "utf8"));
}

function signatureFromObservation(method, status, headers) {
  const metadata = [
    method,
    status,
    etagDigest(headers.get("etag")) ?? "",
    headers.get("content-length") ?? "",
    (headers.get("x-content-sha256") ?? "").toLowerCase(),
    mediaType(headers.get("content-type") ?? ""),
    safeCacheStatus(headers.get("cf-cache-status")) ?? "",
    cacheStatusDigest(headers) ?? "",
  ].join("\0");
  return sha256(Buffer.from(metadata, "utf8"));
}

function etagDigest(value) {
  return value ? sha256(Buffer.from(value, "utf8")) : null;
}

function cacheEvidence(headers) {
  const edge = safeCacheStatus(headers.get("cf-cache-status"));
  const standard = headers.get("cache-status") ?? "";
  if (edge === "HIT" || /\bhit\b/i.test(standard)) return "HIT";
  if (edge === "MISS" || /\bfwd\s*=\s*miss\b/i.test(standard)) return "MISS";
  if (edge === "BYPASS" || /\bfwd\s*=\s*bypass\b/i.test(standard)) return "BYPASS";
  if (["DYNAMIC", "EXPIRED", "REVALIDATED"].includes(edge)) return edge;
  return "UNKNOWN";
}

function safeCacheStatus(value) {
  if (!value) return null;
  const normalized = value.trim().toUpperCase();
  return /^[A-Z_-]{1,32}$/.test(normalized) ? normalized : "UNKNOWN";
}

function cacheStatusDigest(headers) {
  const value = headers.get("cache-status");
  return value ? sha256(Buffer.from(value, "utf8")) : null;
}

function safeCorrelationHeader(value) {
  return typeof value === "string" && /^[A-Za-z0-9-]{1,128}$/.test(value) ? value : null;
}

function updateSummary(progress) {
  const summary = {
    verifiedPairs: 0,
    sourceReadyPairs: 0,
    pendingSourcePairs: 0,
    pendingPublicPairs: 0,
    unverifiedPairs: 0,
    failedPairs: 0,
    notReadyPairs: 0,
  };
  const corpus = createHash("sha256");
  let sourceComplete = true;
  let earliestFailure = null;
  for (const [key, state] of Object.entries(progress.pairs)) {
    if (state.status === "PASS") summary.verifiedPairs += 1;
    if (state.sourceStatus === "PASS") summary.sourceReadyPairs += 1;
    else sourceComplete = false;
    if (state.status === "PENDING_SOURCE") summary.pendingSourcePairs += 1;
    if (state.status === "PENDING_PUBLIC") summary.pendingPublicPairs += 1;
    if (state.status === "UNVERIFIED") summary.unverifiedPairs += 1;
    if (state.status === "FAIL") summary.failedPairs += 1;
    if (state.status === "NOT_READY") summary.notReadyPairs += 1;
    if (state.firstFailure && (!earliestFailure || state.firstFailure.at < earliestFailure.at)) {
      earliestFailure = { ...state.firstFailure, pair: key };
    }
    corpus.update(key).update("\0");
    corpus.update(state.sourceStatus ?? "PENDING_SOURCE").update("\0");
    corpus.update(state.audioSha256 ?? "-").update("\0");
    corpus.update(state.cuesSha256 ?? "-").update("\n");
  }
  progress.summary = summary;
  if (!progress.firstFailure && earliestFailure) progress.firstFailure = earliestFailure;
  progress.sourceCorpusSha256 = corpus.digest("hex");
  progress.sourceCorpusComplete = sourceComplete;
  progress.status = summary.verifiedPairs === EXPECTED_PAIR_COUNT ? "COMPLETE" : "RUNNING";
  if (progress.status === "COMPLETE") progress.completedAt = new Date().toISOString();
}

function recordFailure(state, failure) {
  if (!failure) return;
  if (!state.firstFailure) state.firstFailure = failure;
  state.lastFailure = failure;
}

function failureRecord(stage, code, status = null, errorName = null) {
  return {
    at: new Date().toISOString(),
    stage,
    code,
    httpStatus: Number.isInteger(status) ? status : null,
    errorName: errorName ?? null,
  };
}

function parseOptionalLength(value) {
  if (value === null) return null;
  if (!/^\d+$/.test(value)) return Number.NaN;
  const number = Number(value);
  return Number.isSafeInteger(number) ? number : Number.NaN;
}

function mediaType(value) {
  return value.split(";", 1)[0].trim().toLowerCase();
}

function isTransientStatus(status) {
  return status === 408 || status === 429 || status >= 500;
}

function isTransientError(error) {
  return error instanceof TypeError || error?.name === "TypeError" || error?.name === "AbortError" || error?.name === "TimeoutError";
}

async function runPool(items, concurrency, task) {
  let next = 0;
  const workerCount = Math.min(concurrency, items.length);
  await Promise.all(Array.from({ length: workerCount }, async () => {
    while (true) {
      const index = next++;
      if (index >= items.length) return;
      await task(items[index]);
    }
  }));
}

async function writeReceipt(receiptPath, progress) {
  const tempPath = `${receiptPath}.tmp-${progress.runId}-${randomUUID()}`;
  const serialized = `${JSON.stringify(progress)}\n`;
  const handle = await open(tempPath, "wx", 0o600);
  try {
    await handle.writeFile(serialized, "utf8");
    await handle.sync();
    await handle.chmod(0o600);
  } finally {
    await handle.close();
  }
  await rename(tempPath, receiptPath);
}

async function acquireLock(lockPath, runId) {
  const lockContents = `${JSON.stringify({ runId, pid: process.pid, startedAt: new Date().toISOString() })}\n`;
  let handle;
  try {
    handle = await open(lockPath, "wx", 0o600);
  } catch (error) {
    if (error?.code === "EEXIST") throw new Error("receipt lock already exists; choose a new private receipt path");
    throw error;
  }
  try {
    await handle.writeFile(lockContents, "utf8");
    await handle.sync();
    await handle.chmod(0o600);
  } finally {
    await handle.close();
  }
  return { path: lockPath };
}

async function closeLock(lock, runId, status) {
  const closedPath = `${lock.path}.closed-${runId}`;
  try {
    await rename(lock.path, closedPath);
    const handle = await open(closedPath, "r+");
    try {
      await handle.truncate(0);
      await handle.writeFile(`${JSON.stringify({ runId, pid: process.pid, status, closedAt: new Date().toISOString() })}\n`, "utf8");
      await handle.sync();
      await handle.chmod(0o600);
    } finally {
      await handle.close();
    }
  } catch {
    console.error("Could not rename the private verifier lock after stopping.");
  }
}

function isPathWithin(parent, candidate) {
  const relative = path.relative(path.resolve(parent), path.resolve(candidate));
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function safeFsCode(error) {
  return typeof error?.code === "string" && /^[A-Z0-9_]{1,48}$/.test(error.code) ? error.code : "filesystem_error";
}

function safeErrorCode(error) {
  const code = typeof error?.code === "string" ? error.code : "unknown_error";
  return /^[A-Za-z0-9_:-]{1,64}$/.test(code) ? code : "unknown_error";
}

function safeErrorName(error) {
  return typeof error?.name === "string" && /^[A-Za-z0-9_]{1,48}$/.test(error.name) ? error.name : "Error";
}

function safeTopLevelError(error) {
  if (typeof error?.message === "string" && /^[A-Za-z0-9 _-]{1,160}$/.test(error.message)) return error.message;
  return safeErrorCode(error);
}

function delay(milliseconds, shouldStop) {
  return new Promise((resolve) => {
    const started = Date.now();
    const tick = () => {
      if (shouldStop() || Date.now() - started >= milliseconds) return resolve();
      setTimeout(tick, Math.min(500, milliseconds));
    };
    tick();
  });
}
