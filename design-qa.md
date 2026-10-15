# Book material and mockup fidelity correction

Source truth: `assets/imagegen/reference/library-approved.png` and `assets/imagegen/reference/book-page-approved.png`.
Actual implementation: `docs/site-library.png` and `docs/site-book-page.png`, captured from the local Rust/WASM app at 1536 × 1024 CSS pixels, density 1. Both references are 1536 × 1024 pixels. State: default Library and Genesis overview, audio idle. Full-view and focused material comparisons are saved in `docs/binding-library-comparison.webp` and `docs/binding-book-comparison.webp`; both sides use equal scale.

## Findings corrected

- P1: the previous implementation substituted a flat CSS border/spine and generic glyph for the dimensional leather binding and gold filigree that defined the approved mockups. That earlier visual acceptance was incorrect. Both surfaces now use an individually generated, transparent production binding asset with rounded leather spine, raised gold bands, textured leather, physical highlights, and ornate corner tooling. Each painting and every title remain separate interface elements.
- P2: the first generated binding had too much opaque leather around its picture opening. A measured, targeted ImageGen edit expanded the alpha window to the inner gold rule. Final center and exterior alpha are 0; the actual browser shows the paintings underneath the ornate cutout, without a rectangular mat.
- P2: typography and header rhythm differed from the reference. Added locally hosted Cormorant Garamond, corrected the desktop header to 53px, aligned the Genesis book canvas to y=113px, refined cover titles, and added generated gold foil behind real buttons. The Library toolbar now sits above the visible desktop footer, matching the approved gallery hierarchy.
- P2: fitting titles into the real front-cover area exposed overflow in several medium/long names at narrow widths. Separate medium/long title sizes corrected those cases. A final measured Revelation adjustment reduced the remaining 151px title to fit its 149px opening. All affected medium names were rechecked; broader final-candidate browser checks follow deployment.

## Required fidelity surfaces

- Fonts/typography: Cormorant Garamond is used on the book surfaces, with real small-cap titles, serif descriptions, metadata, controls, and numbers. Medium/long names scale to the framed area. Taglines retain a readable minimum size on mobile.
- Spacing/layout: the approved gallery and two-column book-page structure are retained. Frame and painting use percentage coordinates so they stay aligned while resizing. The binding asset maintains its 3:4 aspect ratio. Mobile layouts stack the volume and controls; the chapter grid keeps its existing touch targets.
- Color/material: charcoal, parchment, and antique gold remain the palette. Lighting and texture come from generated raster leather/gold assets, with restrained image lighting and ordinary UI shadows.
- Image fidelity: the flat CSS spine, border, and text-glyph ornaments are replaced by actual artwork. The accepted binding is 1086 × 1448 with native alpha, exported to lossless WebP. Existing 66 paintings remain distinct. The gold texture is a separate generated material; no page mockup is used as the interface.
- Copy/content: book names, chapter counts, links, and Scripture remain Rust-rendered. The first five cover taglines use the approved gallery's wording. Editorial descriptions remain separate from the KJV audio payload.

## Comparison history and remaining differences

The combined comparisons show the actual material correction and the subsequent layout/type fixes. The references and actual browser captures were inspected together, including a larger book crop and the first two Library covers.

The existing individual paintings differ from the exact generated mockup scenes. The reusable binding also has a different fine tooling pattern and uses ornamented spine panels rather than an individual spine title. Those differences are visible in the comparison; this is not a pixel-identical reproduction. The physical binding, material texture, dimensional lighting, and ornate gold framing are implemented rather than deferred as minor polish.

## Evidence boundaries

Supporting checks: 20 Rust tests, native compilation, WASM packaging, and formatting. Local real browser checks covered all 66 cover/overview titles, 320/390/768px gallery bounds, representative Read flows, and all affected medium-title corrections. Hosted controls, asset identity, audio, and navigation are accepted separately on the final pushed candidate. Physical device and independent acoustic checks are outside this browser run.

final result: passed
