# Illustrated Journey scenes

Journey now covers all 66 books and all 1,189 chapters through 271 authored chapter-section stages, with thirteen contextual landscapes. The broad markers indicate reading stages, not surveyed places or a literal setting for every verse. Poetry, letters, prophecy, and visions are labeled as their own forms of context. The fourteen detailed story worlds below keep priority and their exact verse-level ranges.

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
| Jordan crossing | Joshua 3–4:24 | River crossing, stones in Jordan, Gilgal memorial |
| David and Goliath | 1 Samuel 17:1–53; 17:54–58 unassigned | Armies in the valley, brook, confrontation, Philistine retreat |
| Daniel | Daniel 6:1–24; 6:25–28 unassigned | Prayer, court, lions’ den, dawn at the den |
| Jonah | Jonah 1:1–4:11; 1:1–2 and 4:1–4 unassigned | Joppa, storm, dry land, Nineveh, outside the city |
| Bread and sea | Matthew 14:1–36; 14:1–12 and 14:23 unassigned | Solitary place, boat crossing, Gennesaret |
| Emmaus | Luke 24:1–53; 24:50–51 unassigned | Empty tomb, road, meal, return to Jerusalem |
| Samaria and Cana | John 4:1–54; 4:1–4 unassigned | Jacob’s well, Sychar and its people, Cana of Galilee |
| Paul’s voyage | Acts 27:1–28:31; 27:1–6 and 28:11–15 unassigned | Fair Havens, storm, Melita, Rome |

Ranges are explicit. Gaps shown as unassigned render with no active hotspot and link to their own source verses; the page does not claim a scene for those verses. A return to a previously visited setting reactivates its existing hotspot: for example Luke 2 returns to the temple and then Nazareth, Mark 5 returns to the shore, and Joshua 4 returns to the river crossing before the Gilgal memorial. Chapters outside these ranges retain existing curated geographic context or an honest no-journey state.

## Interaction

- Journey is marked when an illustrated scene exists for the chapter. Passage remains the default view.
- Scrolling through Scripture or listening updates the active hotspot and its separate explanation. Playback remains in the same player.
- Choosing a hotspot’s Read reference intentionally selects its source verse. This stops active narration; the next Listen starts at that selected source.
- **Context & journey stops** expands the editorial note, source ranges, and all stop links. Keeping this closed leaves the artwork visible as reading or audio advances.
- On smaller screens, the companion stacks in normal flow with the reading text. Its visible source links provide the same destinations as the image hotspots.
- The original Genesis scene, curated terrain journeys, Noah’s Ark/Tabernacle/Sower object studies, Library covers, and local reading preferences remain available.

## Assets and source

Each world uses its own illustration, with labels rendered as interface elements rather than painted into the artwork. Optimized scene WebPs live under `web/art/journey/`; original PNGs, exact prompts, generation identifiers, and manifest entries are tracked under `assets/imagegen/source/journey/`, `assets/imagegen/journey/`, and `assets/imagegen/manifest.json`.

`src/journey_scenes.rs` holds the new scene ranges, references, rendering, and corpus-bound checks. `src/reader_context.rs` selects the relevant illustration; `web/reader-context.js` consumes actual position events and updates only the companion.

## All-book context catalog

`data/journey/ot.json` and `data/journey/nt.json` partition every book into contiguous chapter sections. Rust validates all 66 canonical names, all chapter intervals, unique stage identifiers, and source anchors against the embedded KJV. Each stage has a factual editorial summary and source range. `src/journey_books.rs` renders the catalog below precise story and curated terrain context. It never contributes words to Scripture or narration.

The thirteen new original PNGs, production full-frame WebPs, exact prompts, generation IDs, dimensions, and SHA-256 hashes are retained under `assets/imagegen/source/journey/`, `web/art/journey/`, and `assets/imagegen/journey/`. No book cover or entire UI mockup is substituted for a Journey landscape.
