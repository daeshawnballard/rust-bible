# rust-bible — KJV Bible reader

![Screenshot](docs/screenshot.png)

A small KJV reader with one shared Rust implementation. Run it as the original Axum + Tokio server, or build it as Rust WebAssembly for static hosting. The static browser app uses Rust for Bible lookup, reference parsing, search, and page rendering; its small JavaScript file only loads the module, passes routes through the ABI, and updates the page.

## Native server

Requirements: Rust stable (1.75+; originally built with 1.85). The KJV JSON is embedded in the binary with `include_str!`, so no runtime data file is needed.

```sh
cargo run --release
```

Open <http://localhost:3000>. The server binds to `0.0.0.0:3000`.

## Static browser build

Install the WebAssembly target once:

```sh
rustup target add wasm32-unknown-unknown
```

Build the static site:

```sh
./scripts/build-wasm.sh
```

The command writes `dist/index.html`, `dist/app.js`, `dist/style.css`, and `dist/rust_bible.wasm`. Publish the contents of `dist/` on any static host. The page shows a loading state, reports WebAssembly or asset-loading errors, and includes a no-JavaScript message. It does not use `wasm-bindgen`, `wasm-pack`, or a server-side process.

For a local static preview after building:

```sh
python3 -m http.server 8000 --directory dist
```

Then open <http://localhost:8000>.

The native service keeps its JSON routes:

- `/api/verse?ref=John+3:16`
- `/api/search?q=faith`

A static host cannot run Axum or provide these as HTTP endpoints. The same JSON behavior is available inside the browser page through the Rust WASM adapter, for example `window.rustBibleApi("/api/verse?ref=John%203%3A16")`. It returns the parsed JSON object. This hosted API boundary is an in-page JavaScript call into Rust WebAssembly, not a network endpoint.

## Reader features

- All 66 books and chapter navigation, with previous and next chapter links across book boundaries.
- Reference lookup such as `John 3:16`, `John 3`, `1 Cor 13:4-7`, and `ps 23`; names are case-insensitive and common abbreviations are supported.
- Full-text search where every query word must match, with up to 200 displayed results and escaped highlighting.
- Verse ranges and highlighted chapter links.

## Data source

<https://raw.githubusercontent.com/thiagobodruk/bible/master/json/en_kjv.json> (KJV, public domain). The source file's book names are in Portuguese, so English names are supplied in code by book order.

## License

Code is released under the MIT License (see `LICENSE`). The King James Version text is in the public domain, sourced from [thiagobodruk/bible](https://github.com/thiagobodruk/bible).
