# Passage companion

The chapter reader owns one canonical Scripture stream, one audio player, and one exact-verse bookmark. Today and Library lead into it; the book overview remains the chapter chooser. Context has no separate Scripture copy or audio queue.

## Interaction

- Passage closes the companion. Journey opens story or geographic context for the verse reached. Illustrations is offered only in chapters with one of the three existing studies.
- Scene / Detail switches the companion between its overview and selected object card. A study hotspot opens Detail and changes editorial content only. A labeled selector provides the same choices on narrow screens and with a keyboard.
- Read reference is an intentional navigation. A verse already present in the reading stream is selected without remounting the reader; active narration stops so the next Listen begins at the deliberately selected verse. A different chapter uses its canonical reader link. Browsing an object or switching context views does not stop narration.
- Follow passage releases a manually selected part. Rust then selects the first existing object whose cited source range contains the active verse. If no exact object source matches, the study retains a chapter-level default; its source citation remains visible.
- Journey listens to the same real reading/audio position events as the saved bookmark. Context-only view changes replace the current share URL and do not create a separate page history.
- Text size and spacing are browser-local preferences. Continue links consistently use the saved exact verse; explicit verse links take precedence during normal navigation. Reloading a continued chapter resumes the latest saved position.

## Coverage and editorial boundaries

Genesis 2–4 story stages follow 2:8, 3:23, 3:24, and 4:16. Genesis 2:1–7 is explicitly unassigned before the garden is introduced. The landscape and line are schematic illustrations, not archaeological or geographic evidence. Eden and Nod have no assigned coordinates. The move to Nod follows Cain rather than Adam and Eve. No persons or deity are depicted in the scenery.

Geographic camera-follow coverage remains Acts 16–17, Luke 2, and Revelation 1:9, using existing curated ranges and chapter journey stops. A named place is not automatically a setting. No all-book or all-world fallback is shown. The rest of Scripture remains available with an honest no-journey state.

Illustrated studies retain Noah’s Ark (Genesis 6–9), the Tabernacle (Exodus 25–31 and 35–40), and the Sower (Matthew 13). These are chapter-context aids, not a claim that every verse depicts the selected object. Explanations and reconstruction captions are editorial material. Scripture is still the embedded KJV text.

Original Genesis PNG: `assets/imagegen/source/journey/genesis.png`; production WebP: `web/art/journey/genesis.webp`; generation and processing provenance: `assets/imagegen/manifest.json`. Existing scene and object assets remain unchanged.

## Implementation

`src/reader_context.rs` validates positions and renders companion HTML/state through `/api/reader-context`. Static Sites use the same route through the in-page Rust WASM API. `web/reader-context.js` updates only the companion, consumes real `bible:position` events, and reuses the terrain renderer. Scripture and playback never appear inside the companion HTML. Native server assets and the static build include the same JS, CSS, and optimized artwork.

Legacy Explore and study routes redirect into the corresponding source chapter. The former standalone render helpers remain internal for source/render regression checks; they are not routed as public pages.
