# Tabernacle exploration sources

The Scripture excerpts are rendered from the local King James Version corpus in
`data/kjv.json`. Rust parses each displayed reference through the reader's
canonical reference parser and obtains its verse text from the same `verses`
function used by the rest of the site. This document records the source basis
for the brief explanations; it is not an independent Scripture transcription.

The isometric reconstruction is schematic. The Bible passages below ground the
named features and descriptions; they do not specify the scene's exact visual
scale or the complete relative placement shown in the illustration.

| Scene part | Displayed Scripture | Explanation basis |
| --- | --- | --- |
| Courtyard | Exodus 27:13–18 | East side, gate curtain, and dimensions/material description of the court. |
| Altar | Exodus 27:1–2; Exodus 40:6 | Square altar, shittim wood, horns, brass overlay, and position before the tabernacle door. |
| Laver | Exodus 30:18–21 | Brass laver and foot, its placement, and washing. |
| Tabernacle tent | Exodus 26:1; Exodus 26:7 | Linen and colored curtains, plus the goats' hair covering. |
| Table of shewbread | Exodus 25:23–24; Exodus 25:30; Exodus 26:35 | Shittim-wood table, gold overlay, shewbread, and north-side placement. |
| Candlestick | Exodus 25:31–32; Exodus 25:37; Exodus 26:35 | Pure gold, six branches, seven lamps, and south-side placement opposite the table. |
| Veil | Exodus 26:31–33 | Materials, pillars, and division between the holy place and most holy. |
| Ark of the testimony | Exodus 25:10–11; Exodus 25:16; Exodus 25:21–22; Exodus 26:33 | Shittim wood, gold overlay, the testimony, the mercy seat, and placement within the veil. |

## Illustration conventions

The scene is an ImageGen illustration based on the approved visual concept. Rust renders eight positioned hotspot links over the artwork; object close-ups are viewports into the same image. Each hotspot has an accessible name and keyboard focus. Materials, geometry, placement, and decorative details in the image are illustrative. It does not establish archaeological or theological claims. Scripture stays in its own labelled panel view, separate from the object explanation and comes verbatim from the embedded KJV corpus.
