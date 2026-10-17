#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { constants } from "node:fs";
import { access, copyFile, lstat, mkdir, mkdtemp, readFile, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distRoot = path.join(repoRoot, "dist");
const ignoredPublicRoots = new Set([".openai", "client", "server"]);

run("bash", ["scripts/build-wasm.sh"]);
await assertStaticInput(distRoot);

const stageRoot = await mkdtemp(path.join(os.tmpdir(), "rust-bible-sites-build-"));
const publicDir = path.join(stageRoot, "public");
const outputDir = path.join(stageRoot, "output");
await mkdir(publicDir, { recursive: true });
await mkdir(outputDir, { recursive: true });
await copyPublicTree(distRoot, publicDir, true);
const staticInventory = await inventory(publicDir);

const viteBin = path.join(repoRoot, "node_modules", "vite", "bin", "vite.js");
await access(viteBin, constants.R_OK);
run(process.execPath, [viteBin, "build", "--config", "hosting/vite.config.mjs"], {
  HOSTED_PUBLIC_DIR: publicDir,
  HOSTED_BUILD_STAGE: outputDir,
  CLOUDFLARE_VITE_WRANGLER_CONFIG_PATH: path.join(repoRoot, "hosting", "wrangler.jsonc"),
  CLOUDFLARE_CF_FETCH_ENABLED: "false",
  WRANGLER_SEND_METRICS: "false",
  WRANGLER_WRITE_LOGS: "false",
});

const clientOutput = path.join(outputDir, "client");
const clientInventory = await inventory(clientOutput);
for (const [relative, expectedHash] of staticInventory) {
  if (clientInventory.get(relative) !== expectedHash) {
    throw new Error(`Cloudflare client output did not preserve ${relative} byte-for-byte.`);
  }
}

const packagedManifest = path.join(outputDir, ".openai", "hosting.json");
const workerConfigFile = await locateWorkerConfig(outputDir);
const workerConfig = JSON.parse(await readFile(workerConfigFile, "utf8"));
const configuredMain = path.resolve(path.dirname(workerConfigFile), workerConfig.main);
await access(configuredMain, constants.R_OK);

const finalServer = path.join(distRoot, "server");
const finalClient = path.join(distRoot, "client");
await mkdir(finalServer, { recursive: true });
await mkdir(finalClient, { recursive: true });
await copyPublicTree(clientOutput, finalClient, false);
await copyWorkerBundle(outputDir, configuredMain, finalServer);

workerConfig.main = "./index.js";
workerConfig.assets = {
  ...(workerConfig.assets ?? {}),
  directory: "../client",
  binding: "ASSETS",
  run_worker_first: true,
};
workerConfig.$schema = "../../node_modules/wrangler/config-schema.json";
await writeFile(
  path.join(finalServer, "wrangler.json"),
  `${JSON.stringify(workerConfig, null, 2)}\n`,
  "utf8",
);
await mkdir(path.join(distRoot, ".openai"), { recursive: true });
await copyFile(packagedManifest, path.join(distRoot, ".openai", "hosting.json"));

for (const [relative, expectedHash] of staticInventory) {
  const actual = await fileSha256(path.join(finalClient, relative));
  if (actual !== expectedHash) throw new Error(`Final dist/client changed static asset bytes: ${relative}`);
}
await access(path.join(finalServer, "index.js"), constants.R_OK);
console.log(`Hosted Site build copied ${staticInventory.size} existing static files byte-for-byte.`);
console.log(`Worker package: ${path.join(finalServer, "index.js")}`);
console.log(`Sites metadata: ${path.join(distRoot, ".openai", "hosting.json")}`);
console.log(`Private build staging retained for recovery: ${stageRoot}`);

function run(command, args, extraEnv = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    env: { ...process.env, ...extraEnv },
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited with status ${result.status ?? "unknown"}.`);
}

async function assertStaticInput(root) {
  for (const file of ["index.html", "app.js", "style.css", "rust_bible.wasm"]) {
    await access(path.join(root, file), constants.R_OK);
  }
}

async function copyPublicTree(source, destination, filterBuildOutputs) {
  await mkdir(destination, { recursive: true });
  for (const entry of await readdir(source, { withFileTypes: true })) {
    if (filterBuildOutputs && ignoredPublicRoots.has(entry.name)) continue;
    const from = path.join(source, entry.name);
    const to = path.join(destination, entry.name);
    const info = await lstat(from);
    if (info.isSymbolicLink()) throw new Error(`Refusing to copy symlink from static output: ${from}`);
    if (info.isDirectory()) await copyPublicTree(from, to, false);
    else if (info.isFile()) await copyFile(from, to);
  }
}

async function inventory(root) {
  const result = new Map();
  await walk(root, root, result);
  return result;
}

async function walk(root, current, result) {
  for (const entry of await readdir(current, { withFileTypes: true })) {
    const filename = path.join(current, entry.name);
    const info = await lstat(filename);
    if (info.isSymbolicLink()) throw new Error(`Refusing to inventory symlink: ${filename}`);
    if (info.isDirectory()) await walk(root, filename, result);
    else if (info.isFile()) result.set(path.relative(root, filename), await fileSha256(filename));
  }
}

async function fileSha256(filename) {
  const hash = createHash("sha256");
  const bytes = await readFile(filename);
  hash.update(bytes);
  return hash.digest("hex");
}

async function locateWorkerConfig(root) {
  const preferred = [
    path.join(root, "server", "wrangler.json"),
    path.join(root, "wrangler.json"),
  ];
  for (const filename of preferred) {
    try {
      await access(filename, constants.R_OK);
      return filename;
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
  const matches = [];
  await findNamedFiles(root, "wrangler.json", matches);
  if (matches.length !== 1) {
    throw new Error(`Expected one generated Worker wrangler.json, found ${matches.length}.`);
  }
  return matches[0];
}

async function findNamedFiles(root, name, output) {
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const filename = path.join(root, entry.name);
    if (entry.isDirectory()) await findNamedFiles(filename, name, output);
    else if (entry.isFile() && entry.name === name) output.push(filename);
  }
}

async function copyWorkerBundle(outputRoot, entryFile, destination) {
  const workerDir = path.dirname(entryFile);
  const copied = new Set();
  const pending = [entryFile];
  while (pending.length) {
    const filename = pending.pop();
    const normalized = path.resolve(filename);
    if (copied.has(normalized)) continue;
    copied.add(normalized);
    const relative = path.relative(workerDir, normalized);
    if (relative.startsWith(".." + path.sep) || relative === "..") {
      throw new Error(`Worker bundle import escapes its output directory: ${filename}`);
    }
    const destinationName = normalized === path.resolve(entryFile) ? "index.js" : relative;
    const destinationFile = path.join(destination, destinationName);
    await mkdir(path.dirname(destinationFile), { recursive: true });
    await copyFile(normalized, destinationFile);

    const source = await readFile(normalized, "utf8");
    const specifiers = [...source.matchAll(/(?:\bfrom\s*|\bimport\s*\()\s*["'](\.{1,2}\/[^"']+)["']/g)]
      .map((match) => match[1]);
    for (const specifier of specifiers) {
      const resolved = path.resolve(path.dirname(normalized), specifier);
      const candidates = path.extname(resolved)
        ? [resolved]
        : [resolved, `${resolved}.js`, path.join(resolved, "index.js")];
      for (const candidate of candidates) {
        try {
          await access(candidate, constants.R_OK);
          pending.push(candidate);
          break;
        } catch (error) {
          if (error?.code !== "ENOENT") throw error;
        }
      }
    }
  }
}
