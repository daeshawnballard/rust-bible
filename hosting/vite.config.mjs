import path from "node:path";
import { fileURLToPath } from "node:url";
import { cloudflare } from "@cloudflare/vite-plugin";
import { defineConfig } from "vite";
import { sites } from "./sites-vite-plugin.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const stage = process.env.HOSTED_BUILD_STAGE;
const publicDir = process.env.HOSTED_PUBLIC_DIR;

if (!stage || !publicDir) {
  throw new Error("Use scripts/build-hosted.mjs to set the isolated Worker build paths.");
}

export default defineConfig({
  root: repoRoot,
  publicDir,
  plugins: [
    sites(),
    cloudflare({
      configPath: path.resolve(repoRoot, "hosting", "wrangler.jsonc"),
      viteEnvironment: { name: "server" },
      inspectorPort: false,
      persistState: { path: path.resolve(repoRoot, ".wrangler", "narration-local") },
    }),
  ],
  build: {
    outDir: stage,
    emptyOutDir: false,
    copyPublicDir: true,
    sourcemap: false,
  },
});
