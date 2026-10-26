import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { servePublicAsset } from "../hosting/public-assets.mjs";
import { assetContentType } from "./hosted-asset-catalog.mjs";

assert.equal(assetContentType("vendor/maplibre/maplibre-gl.css"), "text/css; charset=utf-8");

const bytes = new TextEncoder().encode("committed public artwork");
const sha256 = createHash("sha256").update(bytes).digest("hex");
const commit = "a".repeat(40);
const catalog = { source_commit: commit, mount_path: "/projects/bibleinrust/", files: {
  "/art/journey/scene.webp": { bytes: bytes.length, sha256, type: "image/webp", source: "web/art/journey/scene.webp" },
} };
const request = (suffix = "", options) => new Request(`https://example.test/art/journey/scene.webp${suffix}`, options);
let fetches = 0;
let puts = 0;
let stored = null;
let gets = 0;
let heads = 0;
const fetchAsset = async (url, options) => {
  fetches++;
  assert.equal(url, `https://raw.githubusercontent.com/daeshawnballard/rust-bible/${commit}/web/art/journey/scene.webp`);
  assert.equal(options.redirect, "manual");
  return new Response(bytes);
};
const bucket = {
  get: async () => { gets++; return stored && ({ ...stored, body: new Response(bytes).body }); },
  head: async () => { heads++; return stored; },
  put: async (key, value, metadata) => {
    puts++;
    assert.equal(key, `public-assets/v1/${sha256}`);
    assert.deepEqual(value, bytes);
    stored = { size: value.byteLength, customMetadata: metadata.customMetadata };
  },
};

for (const path of ["/unknown.webp", "/art/%2Fsecret", "/api/narration-health", "/__proto__"]) {
  assert.equal(await servePublicAsset(new Request(`https://example.test${path}`), bucket, catalog, fetchAsset), null);
}
assert.equal(fetches, 0);
assert.equal(gets, 0);
assert.equal((await servePublicAsset(request("", { method: "POST" }), bucket, catalog, fetchAsset)).status, 405);
assert.equal(fetches, 0);

let response = await servePublicAsset(request(), bucket, catalog, fetchAsset);
assert.equal(response.status, 200);
assert.equal(response.headers.get("X-Asset-Storage"), "source-stored");
assert.equal(response.headers.get("X-Content-SHA256"), sha256);
assert.deepEqual(new Uint8Array(await response.arrayBuffer()), bytes);
assert.equal(puts, 1);

const nonce = "12345678-1234-1234-1234-123456789abc";
response = await servePublicAsset(request(`?verify=${nonce}`), bucket, catalog, fetchAsset);
assert.equal(response.headers.get("X-Asset-Storage"), "r2");
assert.equal(response.headers.get("Cache-Control"), "no-store");
assert.equal(response.headers.get("X-Asset-Verification"), nonce);
assert.deepEqual(new Uint8Array(await response.arrayBuffer()), bytes);
assert.equal(fetches, 1);
response = await servePublicAsset(new Request("https://example.test/projects/bibleinrust/art/journey/scene.webp"), bucket, catalog, fetchAsset);
assert.equal(response.status, 200);
assert.equal(response.headers.get("X-Asset-Storage"), "r2");
assert.deepEqual(new Uint8Array(await response.arrayBuffer()), bytes);
response = await servePublicAsset(request("", { method: "HEAD" }), bucket, catalog, fetchAsset);
assert.equal(heads, 1);
assert.equal(response.body, null);
assert.equal(response.headers.get("Content-Length"), String(bytes.length));
response = await servePublicAsset(request("", { headers: { "If-None-Match": `"${sha256}"` } }), bucket, catalog, fetchAsset);
assert.equal(response.status, 304);
assert.equal(response.body, null);
assert.equal(fetches, 1);

stored = null;
for (const wrong of [new Uint8Array(bytes.length), new Uint8Array(bytes.length + 1), new Uint8Array(bytes.length - 1)]) {
  response = await servePublicAsset(request(), bucket, catalog, async () => new Response(wrong));
  assert.equal(response.status, 502);
  assert.equal(puts, 1);
}
response = await servePublicAsset(request(), bucket, catalog, async () => new Response("missing", { status: 404 }));
assert.equal(response.status, 502);
response = await servePublicAsset(request(`?verify=${nonce}`), bucket, catalog, async () => new Response(null, { status: 302, headers: { Location: "https://unexpected.test/" } }));
assert.equal(response.status, 502);
assert.equal(response.headers.get("X-Asset-Error"), "upstream_status");
assert.equal(puts, 1);
response = await servePublicAsset(request("", { method: "HEAD" }), bucket, catalog, async () => new Response("missing", { status: 404 }));
assert.equal(response.status, 502);
assert.equal(response.body, null);
response = await servePublicAsset(request(), {
  get: async () => { throw new Error("unavailable"); },
  put: async () => { throw new Error("unavailable"); },
}, catalog, fetchAsset);
assert.equal(response.status, 200);
assert.equal(response.headers.get("X-Asset-Storage"), "verified-source");
assert.deepEqual(new Uint8Array(await response.arrayBuffer()), bytes);

console.log("PASS: asset allowlist, immutable source, byte/digest checks, R2 reuse, HEAD/ETag, audit nonce, missing/corrupt/oversized source, storage failure.");
