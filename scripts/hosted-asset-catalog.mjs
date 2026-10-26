import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";

const types = {
  ".webp": "image/webp", ".png": "image/png", ".jpg": "image/jpeg",
  ".svg": "image/svg+xml", ".ttf": "font/ttf", ".woff2": "font/woff2",
  ".js": "text/javascript; charset=utf-8", ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm", ".json": "application/json; charset=utf-8",
  ".bin": "application/octet-stream", ".txt": "text/plain; charset=utf-8",
  ".md": "text/plain; charset=utf-8",
};

export function isStoredAsset(relative) {
  return /^(art|fonts|vendor)\//.test(relative)
    || relative === "logo.png" || relative === "social-preview.jpg";
}

export function assetContentType(relative) {
  return types[path.extname(relative)] ?? "application/octet-stream";
}

// The source revision is pushed before saving a Site version. Never publish a
// catalog for a dirty/untracked asset: its immutable upstream would be absent.
export async function buildAssetCatalog(repoRoot, staticRoot) {
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" }).trim();
  if (!/^[a-f0-9]{40}$/.test(commit)) throw new Error("Expected a full source commit.");
  const files = {};
  async function walk(current) {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      if (entry.isSymbolicLink()) throw new Error(`Unexpected static symlink: ${entry.name}`);
      const absolute = path.join(current, entry.name);
      const relative = path.relative(staticRoot, absolute).split(path.sep).join("/");
      if (entry.isDirectory()) {
        if (!["client", "server", ".openai"].includes(relative)) await walk(absolute);
      } else if (entry.isFile() && isStoredAsset(relative)) {
        const bytes = await readFile(absolute);
        const sourcePath = `web/${relative}`;
        const committed = execFileSync("git", ["show", `${commit}:${sourcePath}`], {
          cwd: repoRoot, maxBuffer: 32 * 1024 * 1024,
        });
        if (!bytes.equals(committed)) throw new Error(`Commit the changed asset before building: ${sourcePath}`);
        files[`/${relative}`] = {
          sha256: createHash("sha256").update(bytes).digest("hex"),
          bytes: bytes.length,
          type: assetContentType(relative),
          source: sourcePath,
        };
      }
    }
  }
  await walk(staticRoot);
  return { schema: 1, source_commit: commit, mount_path: "/projects/bibleinrust/", files };
}
