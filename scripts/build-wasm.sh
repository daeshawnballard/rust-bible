#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! rustup target list --installed | rg -qx 'wasm32-unknown-unknown'; then
  echo "Missing Rust target wasm32-unknown-unknown. Install it with: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi

cargo build --release --target wasm32-unknown-unknown --lib
mkdir -p dist/art/objects
cp web/index.html web/app.js web/experience.js web/style.css web/logo.png dist/
cp web/art/tabernacle.webp dist/art/
cp web/art/objects/*.webp dist/art/objects/
cp target/wasm32-unknown-unknown/release/rust_bible.wasm dist/rust_bible.wasm
echo "Static WASM site built in $repo_root/dist"
