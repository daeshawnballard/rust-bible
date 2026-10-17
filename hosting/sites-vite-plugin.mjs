import { access, cp, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

async function exists(filename) {
  try {
    await access(filename);
    return true;
  } catch (error) {
    if (error?.code === "ENOENT") return false;
    throw error;
  }
}

// Sites reads this metadata from the packaged output. Copy the current root
// declaration into the isolated build output without clearing any directory.
export function sites() {
  let projectRoot = process.cwd();
  let outputDir = process.env.HOSTED_BUILD_STAGE
    ? path.resolve(process.env.HOSTED_BUILD_STAGE)
    : null;
  let command = "serve";

  return {
    name: "sites",
    configResolved(config) {
      projectRoot = config.root;
      outputDir = process.env.HOSTED_BUILD_STAGE
        ? path.resolve(process.env.HOSTED_BUILD_STAGE)
        : config.build.outDir;
      command = config.command;
    },
    async closeBundle() {
      if (command !== "build" || !outputDir) return;

      const sourceManifest = path.resolve(projectRoot, ".openai", "hosting.json");
      const manifest = JSON.parse(await readFile(sourceManifest, "utf8"));
      if (!manifest.project_id || typeof manifest.project_id !== "string") {
        throw new Error(".openai/hosting.json must declare a Site project_id.");
      }

      const outputMetadata = path.resolve(outputDir, ".openai");
      await mkdir(outputMetadata, { recursive: true });
      await writeFile(
        path.join(outputMetadata, "hosting.json"),
        `${JSON.stringify(manifest, null, 2)}\n`,
        "utf8",
      );

      const drizzle = path.resolve(projectRoot, "drizzle");
      if (await exists(drizzle)) {
        await cp(drizzle, path.join(outputMetadata, "drizzle"), { recursive: true, force: true });
      }
    },
  };
}
