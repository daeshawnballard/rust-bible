import { createHash, randomUUID } from "node:crypto";
import { writeFile } from "node:fs/promises";

const [baseInput, expectedCommit, receiptPath] = process.argv.slice(2);
const base = new URL(baseInput);
if (base.protocol !== "https:" || base.search || base.hash || !base.pathname.endsWith("/")
  || !/^[a-f0-9]{40}$/.test(expectedCommit ?? "") || !receiptPath?.startsWith("/")) {
  throw new Error("Usage: node hosting/verify-public-assets.mjs <https-base-ending-in-/> <source-sha> <absolute-private-receipt.json>");
}
const receipt = { started_at: new Date().toISOString(), base: base.href, expected_commit: expectedCommit, status: "RUNNING", assets: [] };
const headers = { "Cache-Control": "no-store, no-cache" };
const manifestUrl = new URL(`public-assets.json?verify=${randomUUID()}`, base);
const manifestResponse = await fetch(manifestUrl, { cache: "no-store", headers });
if (!manifestResponse.ok) throw new Error(`Manifest HTTP ${manifestResponse.status}`);
const manifest = await manifestResponse.json();
if (manifest.source_commit !== expectedCommit) throw new Error("Manifest source does not match saved Site version.");
const entries = Object.entries(manifest.files);
let cursor = 0;
await Promise.all(Array.from({ length: 4 }, async () => {
  while (cursor < entries.length) {
    const [assetPath, expected] = entries[cursor++];
    const result = { path: assetPath, sha256: expected.sha256, expected_bytes: expected.bytes, status: "FAIL" };
    try {
      const nonce = randomUUID();
      const url = new URL(assetPath.slice(1), base);
      url.searchParams.set("verify", nonce);
      const response = await fetch(url, { cache: "no-store", headers });
      result.http_status = response.status;
      result.storage = response.headers.get("X-Asset-Storage");
      result.verification = response.headers.get("X-Asset-Verification");
      const bytes = new Uint8Array(await response.arrayBuffer());
      result.bytes = bytes.byteLength;
      result.actual_sha256 = createHash("sha256").update(bytes).digest("hex");
      if (response.status !== 200 || result.verification !== nonce || result.bytes !== expected.bytes
        || result.actual_sha256 !== expected.sha256 || response.headers.get("X-Content-SHA256") !== expected.sha256) {
        throw new Error("Live asset status, Worker echo, size, or digest mismatch.");
      }
      const headNonce = randomUUID();
      url.searchParams.set("verify", headNonce);
      const head = await fetch(url, { method: "HEAD", cache: "no-store", headers });
      const headLength = head.headers.get("Content-Length");
      result.head = { status: head.status, storage: head.headers.get("X-Asset-Storage"),
        content_length: headLength, encoding: head.headers.get("Content-Encoding") };
      result.stored_readback = head.status === 200 && head.headers.get("X-Asset-Storage") === "r2"
        && head.headers.get("X-Asset-Verification") === headNonce
        && head.headers.get("X-Content-SHA256") === expected.sha256
        && (headLength === null || Number(headLength) === expected.bytes);
      if (!result.stored_readback) throw new Error("Asset was not read back from Site storage.");
      // The edge removes Content-Length when compressing text (including SVG).
      // Strengthen that case to a complete second R2 GET instead of inferring size.
      if (headLength === null) {
        const readNonce = randomUUID();
        url.searchParams.set("verify", readNonce);
        const read = await fetch(url, { cache: "no-store", headers });
        const storedBytes = new Uint8Array(await read.arrayBuffer());
        result.stored_readback = read.status === 200 && read.headers.get("X-Asset-Storage") === "r2"
          && read.headers.get("X-Asset-Verification") === readNonce
          && storedBytes.byteLength === expected.bytes
          && createHash("sha256").update(storedBytes).digest("hex") === expected.sha256;
        if (!result.stored_readback) throw new Error("Complete compressed asset readback mismatch.");
      }
      result.status = "PASS";
    } catch (error) { result.error = error.message; }
    receipt.assets.push(result);
    if (receipt.assets.length % 25 === 0) {
      console.log(`${receipt.assets.length}/${entries.length} asset reads completed`);
      await writeFile(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, { mode: 0o600 });
    }
  }
}));
receipt.completed_at = new Date().toISOString();
receipt.assets.sort((a, b) => a.path.localeCompare(b.path));
receipt.status = receipt.assets.every((entry) => entry.status === "PASS") ? "PASS" : "FAIL";
await writeFile(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, { mode: 0o600 });
console.log(`${receipt.status}: ${receipt.assets.filter((entry) => entry.status === "PASS").length}/${entries.length} assets have fresh matching bytes and a Site-storage readback.`);
if (receipt.status !== "PASS") process.exitCode = 1;
