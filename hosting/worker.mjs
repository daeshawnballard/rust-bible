import {
  AUDIO_MAX_BYTES,
  CUES_MAX_BYTES,
  isSha256,
  parseImportPath,
  parseNarrationPath,
  r2Key,
  validateCueDocument,
} from "./audio-contract.mjs";

const AUDIO_CACHE_CONTROL = "public, max-age=3600, must-revalidate";
const CUES_CACHE_CONTROL = "no-store";
const MAX_TOKEN_LENGTH = 512;
const encoder = new TextEncoder();

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (path === "/api/narration-health") {
      if (request.method !== "GET" && request.method !== "HEAD") {
        return methodNotAllowed("GET, HEAD");
      }
      const bucketAvailable = Boolean(
        env.BUCKET && typeof env.BUCKET.get === "function" && typeof env.BUCKET.put === "function",
      );
      const revision =
        typeof env.CF_VERSION_METADATA?.id === "string" ? env.CF_VERSION_METADATA.id : null;
      return jsonResponse(
        {
          service: "narration-storage",
          r2_binding_available: bucketAvailable,
          revision,
        },
        200,
        { "Cache-Control": "no-store" },
        request.method === "HEAD",
      );
    }

    if (path === "/audio/v1" || path.startsWith("/audio/v1/")) {
      const parsed = parseNarrationPath(path);
      if (!parsed) return jsonError("invalid_audio_path", 400, request.method === "HEAD");
      if (request.method !== "GET" && request.method !== "HEAD") {
        return methodNotAllowed("GET, HEAD");
      }
      if (!env.BUCKET || typeof env.BUCKET.get !== "function") {
        return jsonError("audio_storage_unavailable", 503, request.method === "HEAD");
      }
      const response = await serveNarration(request, env.BUCKET, parsed);
      // A fresh audit nonce proves that this read traversed the storage Worker.
      // Ordinary playback retains its existing cache policy.
      const verification = url.searchParams.get("verify");
      if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(verification ?? "")) {
        return response;
      }
      const headers = new Headers(response.headers);
      headers.set("X-Narration-Verification", verification);
      headers.set("Cache-Control", "no-store");
      return new Response(response.body, { status: response.status, headers });
    }

    if (path === "/_narration/import" || path.startsWith("/_narration/import/")) {
      const parsed = parseImportPath(path);
      if (!parsed) return jsonError("invalid_import_path", 400, request.method === "HEAD");
      if (request.method !== "POST") return methodNotAllowed("POST");
      if (!env.BUCKET || typeof env.BUCKET.put !== "function") {
        return jsonError("audio_storage_unavailable", 503);
      }
      return importNarration(request, env, parsed);
    }

    if (!env.ASSETS || typeof env.ASSETS.fetch !== "function") {
      return jsonError("static_assets_unavailable", 503, request.method === "HEAD");
    }
    return env.ASSETS.fetch(request);
  },
};

async function serveNarration(request, bucket, parsed) {
  const key = r2Key(parsed);
  const isCues = parsed.extension === "json";
  const headers = baseObjectHeaders(isCues);
  const requestedRange = isCues ? null : parseRange(request.headers.get("range"));
  let range = undefined;
  let rangeAllowed = false;
  let rangeMetadata = null;
  if (requestedRange) {
    rangeMetadata = await bucket.head(key);
    if (!rangeMetadata) return jsonError("narration_not_found", 404, request.method === "HEAD");
    if (requestedRange.invalid) {
      return rangeNotSatisfiable(rangeMetadata.size, request.method === "HEAD");
    }
    const boundedRange = boundRange(requestedRange.range, rangeMetadata.size);
    if (!boundedRange) return rangeNotSatisfiable(rangeMetadata.size, request.method === "HEAD");
    range = boundedRange;
    rangeAllowed = true;
    if (request.headers.has("if-range")) {
      rangeAllowed = ifRangeMatches(request.headers.get("if-range"), rangeMetadata);
      if (!rangeAllowed) range = undefined;
    }
  }

  const object = request.method === "HEAD"
    ? (rangeMetadata ?? await bucket.head(key))
    : await bucket.get(key, range ? { range } : undefined);
  if (!object) return jsonError("narration_not_found", 404, request.method === "HEAD");

  headers.set("ETag", object.httpEtag);
  headers.set("Last-Modified", object.uploaded.toUTCString());
  headers.set("X-Content-SHA256", object.customMetadata?.sha256 ?? "");
  if (isCues) {
    headers.set("Content-Type", "application/json; charset=utf-8");
    headers.set("Cache-Control", CUES_CACHE_CONTROL);
  } else {
    headers.set("Content-Type", "audio/mpeg");
    headers.set("Cache-Control", AUDIO_CACHE_CONTROL);
    headers.set("Accept-Ranges", "bytes");
  }

  const notModified = evaluatePreconditions(request.headers, object);
  if (notModified === "not-modified") {
    return new Response(null, { status: 304, headers });
  }
  if (notModified === "precondition-failed") {
    return new Response(null, { status: 412, headers });
  }

  let status = 200;
  const responseRange = object.range ?? (rangeAllowed ? range : null);
  if (!isCues && responseRange) {
    const { offset, length } = responseRange;
    headers.set("Content-Range", `bytes ${offset}-${offset + length - 1}/${object.size}`);
    headers.set("Content-Length", String(length));
    status = 206;
  } else {
    headers.set("Content-Length", String(object.size));
  }

  return new Response(
    request.method === "HEAD" ? null : object.body,
    { status, headers },
  );
}

async function importNarration(request, env, parsed) {
  const suppliedToken = readBearerToken(request.headers.get("authorization"));
  if (!constantTimeTokenMatch(env.NARRATION_UPLOAD_TOKEN, suppliedToken)) {
    return jsonError("unauthorized", 401, false, { "WWW-Authenticate": "Bearer" });
  }

  const expectedType = parsed.extension === "mp3" ? "audio/mpeg" : "application/json";
  if (mediaType(request.headers.get("content-type")) !== expectedType) {
    return jsonError("unsupported_media_type", 415);
  }

  const maxBytes = parsed.extension === "mp3" ? AUDIO_MAX_BYTES : CUES_MAX_BYTES;
  const rawLength = request.headers.get("content-length");
  if (rawLength !== null && (!/^\d+$/.test(rawLength) || Number(rawLength) > maxBytes)) {
    return jsonError("payload_too_large", 413);
  }
  const expectedSha = request.headers.get("x-content-sha256");
  if (!isSha256(expectedSha)) return jsonError("invalid_content_sha256", 400);

  let bytes;
  try {
    bytes = await readLimitedBody(request, maxBytes);
  } catch (error) {
    if (error?.code === "too_large") return jsonError("payload_too_large", 413);
    return jsonError("invalid_request_body", 400);
  }
  if (rawLength !== null && bytes.byteLength !== Number(rawLength)) {
    return jsonError("content_length_mismatch", 400);
  }

  const actualSha = await sha256Hex(bytes);
  if (actualSha !== expectedSha.toLowerCase()) return jsonError("content_sha256_mismatch", 422);

  if (parsed.extension === "mp3") {
    if (!looksLikeMp3(bytes)) return jsonError("invalid_mp3", 422);
  } else {
    let document;
    try {
      document = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    } catch {
      return jsonError("invalid_cues_json", 400);
    }
    const cueError = validateCueDocument(document, {
      voice: parsed.voice,
      book: parsed.book,
      chapter: parsed.chapter,
    });
    if (cueError) return jsonError(cueError, 422);
    const audioObject = await env.BUCKET.head(
      r2Key({ ...parsed, extension: "mp3" }),
    );
    if (!audioObject) return jsonError("audio_must_be_uploaded_first", 409);
    if (audioObject.customMetadata?.sha256 !== document.audio_sha256.toLowerCase()) {
      return jsonError("audio_sha256_mismatch", 422);
    }
  }

  const cacheControl = parsed.extension === "mp3" ? AUDIO_CACHE_CONTROL : CUES_CACHE_CONTROL;
  try {
    await env.BUCKET.put(r2Key(parsed), bytes, {
      httpMetadata: {
        contentType: expectedType,
        cacheControl,
      },
      customMetadata: {
        sha256: actualSha,
        schema: parsed.extension === "json" ? "narration-cues-v1" : "kokoro-mp3-v1",
      },
    });
  } catch {
    return jsonError("storage_write_failed", 503);
  }

  return jsonResponse({
    stored: true,
    key: r2Key(parsed),
    bytes: bytes.byteLength,
    sha256: actualSha,
  }, 201);
}

function baseObjectHeaders(isCues) {
  const headers = new Headers({
    "Cache-Control": isCues ? CUES_CACHE_CONTROL : AUDIO_CACHE_CONTROL,
    "X-Content-Type-Options": "nosniff",
  });
  if (!isCues) headers.set("Accept-Ranges", "bytes");
  return headers;
}

function parseRange(value) {
  if (value === null) return null;
  const match = value.match(/^bytes=(\d*)-(\d*)$/);
  if (!match || (!match[1] && !match[2])) return { invalid: true };
  if (match[1]) {
    const offset = Number(match[1]);
    const end = match[2] ? Number(match[2]) : undefined;
    if (!Number.isSafeInteger(offset) || (end !== undefined && (!Number.isSafeInteger(end) || end < offset))) {
      return { invalid: true };
    }
    return {
      range: end === undefined ? { offset } : { offset, length: end - offset + 1 },
    };
  }
  const suffix = Number(match[2]);
  if (!Number.isSafeInteger(suffix) || suffix <= 0) return { invalid: true };
  return { range: { suffix } };
}

function boundRange(range, size) {
  if (!range || !Number.isSafeInteger(size) || size <= 0) return null;
  if ("suffix" in range) {
    const length = Math.min(range.suffix, size);
    return { offset: size - length, length };
  }
  if (!Number.isSafeInteger(range.offset) || range.offset < 0 || range.offset >= size) return null;
  const length = range.length === undefined ? size - range.offset : Math.min(range.length, size - range.offset);
  if (!Number.isSafeInteger(length) || length <= 0) return null;
  return { offset: range.offset, length };
}

function ifRangeMatches(value, object) {
  if (!value) return true;
  if (value.startsWith("\"")) return value === object.httpEtag;
  if (value.startsWith("W/")) return false;
  const date = Date.parse(value);
  return Number.isFinite(date) && object.uploaded.getTime() <= date;
}

function evaluatePreconditions(headers, object) {
  const etag = object.httpEtag;
  const ifMatch = headers.get("if-match");
  if (ifMatch !== null && !etagListMatches(ifMatch, etag, false)) {
    return "precondition-failed";
  }
  const ifUnmodifiedSince = headers.get("if-unmodified-since");
  if (ifMatch === null && ifUnmodifiedSince) {
    const date = Date.parse(ifUnmodifiedSince);
    if (Number.isFinite(date) && object.uploaded.getTime() > date + 999) {
      return "precondition-failed";
    }
  }

  const ifNoneMatch = headers.get("if-none-match");
  if (ifNoneMatch !== null && etagListMatches(ifNoneMatch, etag, true)) {
    return "not-modified";
  }
  const ifModifiedSince = headers.get("if-modified-since");
  if (ifNoneMatch === null && ifModifiedSince) {
    const date = Date.parse(ifModifiedSince);
    if (Number.isFinite(date) && object.uploaded.getTime() <= date + 999) {
      return "not-modified";
    }
  }
  return null;
}

function etagListMatches(header, etag, weakComparison) {
  return header.split(",").some((rawTag) => {
    const tag = rawTag.trim();
    if (tag === "*") return true;
    if (!tag) return false;
    if (weakComparison) return tag.replace(/^W\//, "") === etag;
    return !tag.startsWith("W/") && tag === etag;
  });
}

function readBearerToken(header) {
  if (typeof header !== "string" || !header.startsWith("Bearer ")) return "";
  const token = header.slice(7);
  return token.length <= MAX_TOKEN_LENGTH ? token : "";
}

function constantTimeTokenMatch(expected, supplied) {
  if (typeof expected !== "string" || expected.length === 0 || expected.length > MAX_TOKEN_LENGTH) {
    return false;
  }
  const expectedBytes = encoder.encode(expected);
  const suppliedBytes = encoder.encode(supplied);
  let difference = expectedBytes.length ^ suppliedBytes.length;
  const count = Math.max(expectedBytes.length, suppliedBytes.length);
  for (let index = 0; index < count; index += 1) {
    difference |= (expectedBytes[index] ?? 0) ^ (suppliedBytes[index] ?? 0);
  }
  return difference === 0;
}

function mediaType(value) {
  return (value ?? "").split(";", 1)[0].trim().toLowerCase();
}

async function readLimitedBody(request, maxBytes) {
  if (!request.body) return new Uint8Array();
  const reader = request.body.getReader();
  const chunks = [];
  let total = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > maxBytes) {
        await reader.cancel();
        const error = new Error("body too large");
        error.code = "too_large";
        throw error;
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  const result = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return result;
}

async function sha256Hex(bytes) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function looksLikeMp3(bytes) {
  if (bytes.length >= 3 && bytes[0] === 0x49 && bytes[1] === 0x44 && bytes[2] === 0x33) {
    return true;
  }
  for (let index = 0; index + 1 < Math.min(bytes.length, 4096); index += 1) {
    if (bytes[index] === 0xff && (bytes[index + 1] & 0xe0) === 0xe0) return true;
  }
  return false;
}

function jsonError(code, status, head = false, extraHeaders = {}) {
  return jsonResponse({ error: code }, status, extraHeaders, head);
}

function rangeNotSatisfiable(size, head) {
  return jsonError("range_not_satisfiable", 416, head, {
    "Content-Range": `bytes */${size}`,
    "Accept-Ranges": "bytes",
  });
}

function jsonResponse(value, status = 200, extraHeaders = {}, head = false) {
  const headers = new Headers({
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
    "X-Content-Type-Options": "nosniff",
    ...extraHeaders,
  });
  const body = JSON.stringify(value);
  headers.set("Content-Length", String(encoder.encode(body).byteLength));
  return new Response(head ? null : body, { status, headers });
}

function methodNotAllowed(allow) {
  return jsonError("method_not_allowed", 405, false, { Allow: allow });
}
