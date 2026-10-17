# Connected reader design QA

final result: passed

This is a browser-rendered **local visual comparison**, not public deployment acceptance. The source of truth is the four selected images in `assets/imagegen/reference/`: `immersive-journey-approved.png`, `reading-focus-approved.png` (corrected KJV wording), `library-polish-approved.png`, and `today-polish-approved.png`.

## Evidence and state

Implementation captures: `docs/screenshots/connected-reader/{journey,reader,library,today}-local.jpg`. Desktop CSS viewport 1536 × 1024; source images 1536 × 1024. Journey capture is 1536 × 1024; the other captures are 1521 × 1014 (same 3:2 aspect ratio, normalized to 1536 × 1024 in comparison composites). No phone frame. Captures use the same charcoal/gold theme, Genesis 2 selected verse 2 for the reading column, Genesis Journey, all-books Library and October 17 edition-2 Today (Exodus 25:23–26). Journey reference selects verse 10 while the comparison capture selects verse 2; this intentional content-state difference is not a layout mismatch. Audio is idle in captures; the reference's running waveform is a different state. The implementation renders a waveform from real generated PCM after Play, rather than decorative sample data.

Full-view side-by-side comparisons were composed and inspected for all four targets. Reading focus and Journey were also inspected at full viewport resolution for passage text, title, tabs, selected verse, scene pins and dock controls. Today and Library full-resolution views made the type and covers readable; no additional crop was needed. Comparison images are private review artifacts; only sanitized implementation screenshots are in this repository.

## Comparison history

- P2: Narrow header inherited a two-column grid and wrapped search/menu over the scene. Changed shared header to a single flex rail, retained 44px search/menu targets, and condensed the non-reader brand at 320px. Post-fix browser measurements: no horizontal overflow at 320 or 390px; both tools stay on the top rail.
- P2: Mobile Scripture sheet left too little text visible, with detached text settings. Increased the sheet to 340px at an 844px height, matched scene clearance, and moved text settings into the book/chapter menu. Post-fix screenshot shows scene, scrolling Scripture and audio separately accessible.
- P2: Today artwork had a framed letterbox and appeared too small/dim. Removed frame/shadow, preserved source artwork aspect ratio, softened its edge, increased desktop scale/brightness and clipped only page overflow. Revised Today capture shows the large detailed object; document scroll width equals client width.
- P2: Library header introduced an unnecessary extra row. Restored Library-first heading, placed metadata/Codex link on one row, and tightened spacing before covers. Revised gallery capture retains five columns and the floating tools.
- P2: SVG play/pause paths were both visible. Fixed hidden attributes and observed a single correct action icon.

- P1 interaction correction from public version 40: the reader root also carries `data-window-tab`, so a broad delegated selector swallowed ordinary clicks. Restricted tab handling to buttons. A new local real click now expands the title menu (44px chapter selector) and selecting Chapter 13 loads Nehemiah 13. The correction requires fresh public acceptance on the subsequent release.

- P1 playback correction from public version 41: Play used the initial idle preview after an explicit source selection. It now resolves the visible reader at click time. Local source selection to Genesis 2:10 produced the canonical v10 return link and the queue advanced through that chapter. Fresh release acceptance remains separate.

- P1 uncovered-range transition: live narration from Mark 5:43 reached Mark 6 but retained an empty Journey canvas. Empty context now expands into Read automatically, preserves the verse/queue, and hides the unavailable Journey action. Local direct Mark 6 Journey entry renders the full reading column. Keyboard focus also recenters object buttons as well as place links.

- P2 scene reachability: native image dragging interrupted pan after about 50px, leaving some object pins behind the overlay. Disabled native image drag and separated pointer focus from keyboard recentering. Actual preview drag moved the world -400px; the revealed Ark door selected `door.webp`. Removed duplicate in-figure captions and close playback options after Stop.

## Five fidelity surfaces

- Typography: bundled Cormorant Garamond display and EB Garamond Scripture; serif hierarchy, readable selected verses, compact sans-serif operational status. Exact generated-image glyph shapes are not treated as a font file. No visible clipped titles on inspected widths.
- Spacing/layout: full landscape, floating right window, central expanded reading column, persistent bottom dock; Library tools reserve audio clearance. Mobile becomes a scene plus inset sheet, not a compressed desktop panel.
- Colors/tokens: charcoal #121313, warm ivory #eee6d4, muted gold #e4c482; active verse uses a gold rule and translucent wash. Focus and selected states retain contrast.
- Imagery: dedicated full-frame generated Genesis landscape, existing individual book covers and object artwork. All labels, Scripture, controls and links are live interface elements. Original source assets and generation provenance remain in the repo.
- Copy/content: embedded KJV is unchanged, including corrected Genesis 2:2. Context and Codex are labeled editorial material, with explicit source links and uncertainty for illustrated locations. No claims that illustrations are geographic evidence or that Codex coverage is exhaustive.

## Interaction and runtime boundary

Local real-browser checks covered Read/Journey selection preserving verse, hotspot inspection versus explicit Read, inline Codex, mobile resizing, and generated Kokoro audio continuing into the next chapter. Source checks: 63 Rust tests plus progress and JavaScript syntax checks. These support implementation; they do not substitute for fresh public-site end-to-end acceptance. Public release results are recorded separately by exact source/deployment identity.

No open P0/P1/P2 visual finding remains in this comparison. P3 differences retained: existing cover illustrations vary from mockup subjects, Today retains a contextual-study link below its main content, and the Codex adds a third window tab. The reader's source artwork is a dedicated generated variant, not a pixel-for-pixel copy of the concept landscape. Whole-Bible illustrated coverage and physical-device qualification are not claimed.
