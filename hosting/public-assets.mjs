const CACHE_CONTROL = "public, max-age=3600, must-revalidate";
const UPSTREAM = "https://raw.githubusercontent.com/daeshawnballard/rust-bible/";

// Only build-time, committed public files are addressable. Request parameters
// cannot choose an upstream host, revision, source path, or storage key.
export async function servePublicAsset(request, bucket, catalog, fetchAsset = fetch) {
  const url = new URL(request.url);
  const pathname = catalog.mount_path && url.pathname.startsWith(catalog.mount_path)
    ? `/${url.pathname.slice(catalog.mount_path.length)}` : url.pathname;
  const asset = Object.hasOwn(catalog.files, pathname) ? catalog.files[pathname] : null;
  if (!asset) return null;
  if (request.method !== "GET" && request.method !== "HEAD") {
    return new Response(null, { status: 405, headers: { Allow: "GET, HEAD" } });
  }
  const headers = new Headers({
    "Content-Type": asset.type,
    "Content-Length": String(asset.bytes),
    "Cache-Control": CACHE_CONTROL,
    "X-Content-Type-Options": "nosniff",
    "X-Content-SHA256": asset.sha256,
    ETag: `"${asset.sha256}"`,
  });
  const verification = url.searchParams.get("verify");
  if (/^[a-f0-9-]{36}$/i.test(verification ?? "")) {
    headers.set("X-Asset-Verification", verification);
    headers.set("Cache-Control", "no-store");
  }

  const key = `public-assets/v1/${asset.sha256}`;
  let object = null;
  try {
    object = bucket && await (request.method === "HEAD" && typeof bucket.head === "function"
      ? bucket.head(key) : bucket.get(key));
  } catch { /* A storage read failure can still use the verified public source. */ }
  if (object?.size === asset.bytes && object.customMetadata?.sha256 === asset.sha256) {
    if (request.method === "HEAD" && object.body) await object.body.cancel();
    headers.set("X-Asset-Storage", "r2");
    if (request.headers.get("if-none-match")?.split(/,\s*/).some((v) => v === headers.get("ETag") || v === "*")) {
      headers.delete("Content-Length");
      return new Response(null, { status: 304, headers });
    }
    return new Response(request.method === "HEAD" ? null : object.body, { headers });
  }
  if (object?.body) await object.body.cancel();

  try {
    const source = asset.source.split("/").map(encodeURIComponent).join("/");
    const upstream = await fetchAsset(`${UPSTREAM}${catalog.source_commit}/${source}`, {
      redirect: "error", signal: AbortSignal.timeout(20000),
    });
    if (!upstream.ok) throw new Error("asset_source_unavailable");
    const bytes = await boundedBytes(upstream, asset.bytes);
    const hash = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
      .map((value) => value.toString(16).padStart(2, "0")).join("");
    if (hash !== asset.sha256) throw new Error("asset_integrity_mismatch");
    let stored = false;
    if (bucket) {
      try {
        await bucket.put(key, bytes, {
          httpMetadata: { contentType: asset.type, cacheControl: CACHE_CONTROL },
          customMetadata: { sha256: hash, schema: "public-asset-v1" },
        });
        stored = true;
      } catch { /* Verified bytes remain usable if storage is temporarily unavailable. */ }
    }
    headers.set("X-Asset-Storage", stored ? "source-stored" : "verified-source");
    return new Response(request.method === "HEAD" ? null : bytes, { headers });
  } catch {
    return new Response(request.method === "HEAD" ? null : "Artwork or supporting asset is temporarily unavailable. Please retry.", {
      status: 502,
      headers: { "Content-Type": "text/plain; charset=utf-8", "Cache-Control": "no-store" },
    });
  }
}

async function boundedBytes(response, expectedBytes) {
  const reader = response.body?.getReader();
  if (!reader) throw new Error("asset_body_missing");
  const bytes = new Uint8Array(expectedBytes);
  let offset = 0;
  try {
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      if (offset + value.byteLength > expectedBytes) throw new Error("asset_size_mismatch");
      bytes.set(value, offset);
      offset += value.byteLength;
    }
    if (offset !== expectedBytes) throw new Error("asset_size_mismatch");
    return bytes;
  } catch (error) {
    await reader.cancel().catch(() => {});
    throw error;
  }
}
