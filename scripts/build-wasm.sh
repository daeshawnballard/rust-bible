#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! rustup target list --installed | rg -qx 'wasm32-unknown-unknown'; then
  echo "Missing Rust target wasm32-unknown-unknown. Install it with: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi

cargo build --release --target wasm32-unknown-unknown --lib
mkdir -p dist/art/objects dist/art/covers dist/art/today dist/art/learn/ark dist/art/learn/sower dist/fonts dist/art/journey
cp web/fonts/eb-garamond.ttf dist/fonts/
cp web/fonts/cormorant-garamond.ttf dist/fonts/
cp web/index.html web/app.js web/experience.js web/local-voice.js web/local-voice-worker.js web/hosted-voice.js web/library.js web/journey.js web/progress.js web/atlas.js web/terrain.js web/reader-context.js web/codex.js web/logo.png dist/
mkdir -p dist/icons
cp web/icons/* dist/icons/
mkdir -p dist/vendor/kokoro
cp -R web/vendor/kokoro/. dist/vendor/kokoro/
mkdir -p dist/vendor/maplibre
cp web/vendor/maplibre/* dist/vendor/maplibre/
cat web/style.css web/reader.css web/today.css web/learning.css web/about.css web/vendor/maplibre/maplibre-gl.css web/atlas.css web/polish-library.css web/polish.css web/continue.css web/reader-illustrations.css web/reader-context.css web/gallery-polish.css web/codex.css web/audio-dock.css web/immersive.css > dist/style.css
cp web/art/today/*.webp dist/art/today/
cp web/art/learn/ark/*.webp dist/art/learn/ark/
cp web/art/learn/sower/*.webp dist/art/learn/sower/
cp web/social-preview.jpg dist/
cp web/art/journey/*.webp dist/art/journey/
cp web/art/tabernacle.webp dist/art/
cp web/art/book-binding.webp web/art/gold-foil.webp dist/art/
cp web/art/objects/*.webp dist/art/objects/
cp web/art/covers/*.webp dist/art/covers/
cp target/wasm32-unknown-unknown/release/rust_bible.wasm dist/rust_bible.wasm
echo "Static WASM site built in $repo_root/dist"
