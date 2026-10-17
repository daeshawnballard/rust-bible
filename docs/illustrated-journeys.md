# Illustrated Journey scenes

Journey is a companion inside the canonical chapter reader. Its current stop follows the same verse position used by manual reading, continuous audio, and the local bookmark. It never supplies another Scripture stream or audio queue.

The landscape is an artistic storyboard. Places and successive moments are compressed into one miniature; screen position, distance, paths, architecture, and scenery are not archaeological evidence. Hotspot names, paths, references, and active states are real interface elements over individual production images.

## Coverage

| Scene | Passages | Reading stages |
| --- | --- | --- |
| Eden | Genesis 2–4 | Garden, outside the garden, guarded way, Cain in Nod |
| Noah and the flood | Genesis 7–9:17 | Entering the ark, flood waters, mountains of Ararat, birds, dry ground and altar |
| Exodus wilderness | Exodus 13:17–17:16 | Departure, sea crossing, Marah, Elim, wilderness of Sin, Rephidim |
| Ruth’s return | Ruth 1–2 | Moab, the road, Bethlehem, Boaz’s field |
| Luke’s birth narrative | Luke 2 | Road to Bethlehem, manger, shepherds’ fields, temple, Nazareth |
| Galilee | Mark 4–5 | Teaching/return shore, crossing, opposite shore, Jairus’s house |

Ranges are explicit. A return to a previously visited setting reactivates its existing hotspot: for example Luke 2 returns to the temple and then Nazareth, while Mark 5 returns to the shore. Opening verses before a setting is assigned remain unassigned. Chapters outside these ranges retain existing curated geographic context or an honest no-journey state.

## Interaction

- Journey is marked when an illustrated scene exists for the chapter. Passage remains the default view.
- Scrolling through Scripture or listening updates the active hotspot and its separate explanation. Playback remains in the same player.
- Choosing a hotspot’s Read reference intentionally selects its source verse. This stops active narration; the next Listen starts at that selected source.
- On smaller screens, the companion stacks in normal flow with the reading text. Its visible source links provide the same destinations as the image hotspots.
- The original Genesis scene, curated terrain journeys, Noah’s Ark/Tabernacle/Sower object studies, Library covers, and local reading preferences remain available.

## Assets and source

Each of the five new scenes was generated separately with built-in ImageGen. Original PNGs are preserved under `assets/imagegen/source/journey/`, optimized WebPs under `web/art/journey/`, and exact prompts and generation identifiers under `assets/imagegen/journey/` and `assets/imagegen/manifest.json`. Labels are not painted into the artwork.

`src/journey_scenes.rs` holds the new scene ranges, references, rendering, and corpus-bound checks. `src/reader_context.rs` selects the relevant illustration; `web/reader-context.js` consumes actual position events and updates only the companion.
