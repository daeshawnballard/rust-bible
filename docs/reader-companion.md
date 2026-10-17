# Reader companion

The canonical passage has two visual modes: a full reading column, or a full-screen illustrated Journey with a floating Scripture window (an expandable sheet on narrow screens). These share the same Scripture nodes, exact-verse bookmark and global audio queue. The Scripture / Context / Codex tabs replace the window content; editorial notes never enter the Scripture or narration stream.

Hotspots inspect context while the saved reading location remains unchanged. An explicit **Read** source link changes the passage. **Follow passage** returns the scene and Scripture window to the current read/listen position. The audio dock remains separate and accessible at the bottom; Library tools reserve clearance above it. The Codex is one reference companion with book, category and text filters; it has selected, source-linked entries for all 66 books rather than an exhaustive index.

## Interaction

- Library covers enter the chapter reader directly. Book and chapter selection stays in the passage-title menu, alongside text settings.
- Passages without an assigned illustration expand into Read automatically; narration continues unchanged.
- Read expands the existing Scripture window into the reading column. Journey contracts it beside the scene. These view changes preserve the selected verse and audio queue.
- Scripture, Context and Codex reuse the same window. Selecting a hotspot shows its dedicated illustration or place note in Context; Back to Scripture or Follow passage resumes the current position.
- Read reference deliberately selects its cited verse and stops narration. Crossing a chapter or book remounts the canonical reader at that exact verse.
- Reading scroll and narration update the same exact-verse bookmark. Narration continues through chapters and while browsing Today, Library or Codex. The dock names the actual playing passage and returns to its current verse.
- Text size, spacing, narrator and speed preferences are browser-local. Reload restores the saved verse; playback requires Play again.

## Coverage and editorial boundaries

Genesis 2–4 story stages follow 2:8, 2:10–14 (river), 3:23, 3:24, and 4:16. Genesis 2:1–7 is explicitly unassigned before the garden is introduced. The landscape and line are schematic illustrations, not archaeological or geographic evidence. Eden and Nod have no assigned coordinates. The move to Nod follows Cain rather than Adam and Eve. No persons or deity are depicted in the scenery.

Additional illustrated story coverage is documented in [Illustrated Journey scenes](illustrated-journeys.md): Genesis 7–9:17, Exodus 13:17–17:16, Ruth 1–2, Luke 2, and Mark 4–5. Ranges are explicit and recurring places reactivate their existing hotspot.

Luke 2 now uses its illustrated story world. Geographic camera-follow coverage remains Acts 16–17 and Revelation 1:9, using existing curated ranges and chapter journey stops. A named place is not automatically a setting. No all-book or all-world fallback is shown. The rest of Scripture remains available with an honest no-journey state.

Illustrated studies retain Noah’s Ark (Genesis 6–9), the Tabernacle (Exodus 25–31 and 35–40), and the Sower (Matthew 13). These are chapter-context aids, not a claim that every verse depicts the selected object. Explanations and reconstruction captions are editorial material. Scripture is still the embedded KJV text.

Original schematic Genesis PNG: `assets/imagegen/source/journey/genesis.png`; production WebP: `web/art/journey/genesis.webp`; generation and processing provenance: `assets/imagegen/manifest.json`. The immersive edition uses `genesis-immersive-v1.png` and `genesis-immersive-v1.webp`, with generation provenance in `assets/imagegen/journey/genesis-immersive-v1.json`. Existing object assets remain available.

## Implementation

`src/reader_context.rs` validates positions and renders companion HTML/state through `/api/reader-context`. Static Sites use the same route through the in-page Rust WASM API. `web/reader-context.js` moves the existing Scripture nodes into the shared window, consumes real `bible:position` events, and reuses the terrain renderer. API-provided Context/Codex HTML never contains Scripture or playback copies. Native server assets and the static build include the same JS, CSS, and optimized artwork.

Legacy Explore and study routes redirect into the corresponding source chapter. The former standalone render helpers remain internal for source/render regression checks; they are not routed as public pages.
