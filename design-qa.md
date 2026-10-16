# Book material and mockup fidelity correction

Source truth: `assets/imagegen/reference/library-approved.png` and `assets/imagegen/reference/book-page-approved.png`.
Actual implementation: `docs/site-library.png` and `docs/site-book-page.png`, captured from the local Rust/WASM app at 1536 × 1024 CSS pixels, density 1. Both references are 1536 × 1024 pixels. State: default Library and Genesis overview, audio idle. Full-view and focused material comparisons are saved in `docs/binding-library-comparison.webp` and `docs/binding-book-comparison.webp`; both sides use equal scale.

## Findings corrected

- P1: the previous implementation substituted a flat CSS border/spine and generic glyph for the dimensional leather binding and gold filigree that defined the approved mockups. That earlier visual acceptance was incorrect. Both surfaces now use an individually generated, transparent production binding asset with rounded leather spine, raised gold bands, textured leather, physical highlights, and ornate corner tooling. Each painting and every title remain separate interface elements.
- P2: the first generated binding had too much opaque leather around its picture opening. A measured, targeted ImageGen edit expanded the alpha window to the inner gold rule. Final center and exterior alpha are 0; the actual browser shows the paintings underneath the ornate cutout, without a rectangular mat.
- P2: typography and header rhythm differed from the reference. Added locally hosted Cormorant Garamond, corrected the desktop header to 53px, aligned the Genesis book canvas to y=113px, refined cover titles, and added generated gold foil behind real buttons. The Library toolbar now sits above the visible desktop footer, matching the approved gallery hierarchy.
- P2: fitting titles into the real front-cover area exposed overflow in several medium/long names at narrow widths. Separate medium/long title sizes corrected those cases. A final measured Revelation adjustment reduced the remaining 151px title to fit its 149px opening. All affected medium names were rechecked; broader final-candidate browser checks follow deployment.

- P2: the first public run caught a wide short name (Numbers) in the large cover. Inspection also found an older scoped rule overriding its display font. Explicitly corrected the cover face and calibrated the longest word of all 66 names in the real loaded browser font. Rust now emits measured per-book advances and CSS caps their width to the insert. Fresh local checks of all 66 overviews at both 1536px and 320px passed (132 views, no clipping); this supersedes character-count-only fitting. Measurements and the font hash are in `assets/imagegen/binding-typography.json`.

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


## Library readability and restraint polish

Kept the dimensional binding, all 66 paintings, and approved gallery composition. Descriptions now use weight 500 with more line spacing; reading links use weight 600. Library-only CSS gently reduces the binding gold saturation and painting enhancement. The desktop floating tools bar is 64px tall at 1536px, down from 84px, with a softer border/shadow and smaller tool icon. Tablet padding is also reduced; mobile keeps its collapsible form and existing controls. Book overviews, reader, Tabernacle, and speech behavior are outside this presentation change.

Fresh local browser layout checks cover all 66 card texts at 1536, 1024, 768, 390, and 320 CSS pixels. No horizontal overflow or clipped titles/descriptions were observed. The updated Library screenshot above is a real local browser capture. Hosted acceptance is recorded separately after publication; these local checks do not establish physical-device or acoustic qualification.


## Selected Today and chapter reader — option 1

Authority: `assets/imagegen/reference/today-reader-approved.png`, the first displayed concept selected by the user. The paired board is 1983 × 793 pixels. Each panel was compared at 953 × 726 CSS pixels, density 1: Today crop (26,41)–(979,767), Reader crop (1006,41)–(1959,767). States: October 15, 2026 / Psalms 23:1–3 and Psalm 23 / idle playback. Current full browser captures are `docs/site-today.png` and `docs/site-reader.png` at 1536 × 1024. Equal-scale comparison revisions are retained in the private task evidence, rather than published as product assets.

Corrected P2 findings: inherited audio layout produced an oversized button; static and native CSS ordering differed; reader text width and controls were too wide at the reference viewport; button labels did not visibly reflect pause/resume. The final layout uses the same CSS order in both builds, scoped compact controls, real changing labels, and responsive reader width. Mobile selectors and scene hotspots have larger usable targets; the Scripture panel also offers a named part selector. Final public screenshot review caught a saved-reading row whose text squeezed between its actions on mobile. A two-row grid now gives the location the full text width and keeps Continue/Clear below it; fresh 320/390px checks show a readable two-line location and a 44px Clear target.

Required fidelity surfaces:

- Typography: locally hosted Cormorant serif headings/Scripture, consistent small date labels and shared header. Reader type and column width track the selected quiet composition.
- Spacing: Today retains text left, painting right, paired reading controls, and compact supporting rows. Reader preserves breadcrumbs, centered text, compact selectors/player, and previous/next navigation. Mobile stacks these elements without horizontal overflow.
- Color: shared charcoal, parchment, muted gold rules and controls; existing book bindings remain unchanged.
- Imagery: a dedicated detailed harp, olive tree, and valley painting was generated from the selected authority and existing Psalms art. It is a separate production asset, not the page mockup. Source, prompts, and export provenance are saved in the repository.
- Copy and controls: actual KJV verses, accurate chapter counts, working navigation, and contextual speech are real interface elements. Illustrations, book descriptions, and Tabernacle explanations remain separate from Scripture and speech payloads.

Remaining differences: the new painting differs in its exact details from the generated concept. Today adds an honest rotation disclosure, a stable dated link, and Clear for saved reading; these can place the footer below a short reference-height viewport. The named Tabernacle row uses the shared book icon. Fine texture and letter rendering differ from ImageGen. No pixel-identical claim is made. Local 320/390px checks and all 66 cover-to-last-chapter journeys passed; final hosted acceptance is a separate release-bound run. Physical phones and independent acoustic measurement are not established by these captures.

final result: passed

## Reading-page footer placement correction

A fresh hosted John 3:16 passage at 1561 × 1200 placed the footer bottom at 487.53px, leaving 712.47px of viewport below it. Today also relied on a fixed viewport subtraction. Today and reader now use a column layout with a dynamic viewport minimum: short pages place the full-width footer at the bottom, and long chapters expand naturally. The Library and Tabernacle layouts are outside this scoped rule.

Fresh local Rust/WASM browser checks passed for the focused passage, John 3 (36 verses), Today, and Psalm 117 (2 verses), including 320px and 390px mobile widths. Footer bounds reach the document/viewport bottom without overlapping the reading area or horizontal overflow. Hosted verification is recorded separately for the final published source; these checks do not establish physical-phone behavior.

## Short readings continue into Scripture

Moving the footer left unused space above it; the user's correction requires more passage content. The Rust reader now continues short chapters and focused verse ranges into sequential KJV text with explicit chapter headings, separate verse anchors, and working chapter links. Psalm 23's six verses are followed immediately by Psalm 24's ten verses, visible within the same 1536 × 1024 viewport. Longer chapters keep their existing layout. The contextual audio selection remains the originally selected chapter/range, excluding the added Scripture sections.

Fresh local browser journeys cover Psalm 23 → Open Psalm 24, John 3:16 → following verses 17–36, Malachi 4 → Matthew 1 across the book boundary, and 320px mobile bounds. No duplicate verse IDs or horizontal overflow observed. The reader screenshot is an actual local Rust/WASM capture. Supporting Rust tests check all 1,189 chapters for unique anchors and bounded continuation, focused range boundaries, canonical book transitions, and the end of Revelation. Hosted acceptance is recorded separately on the final source.


## Continuous reader audio

Listen from here follows the Rust-rendered Scripture stream, including the following chapters already shown. At the end of the displayed verses, a validated Rust JSON route supplies the next chapter fragments; the initial render remains bounded. Chapter/verse status and the highlighted verse advance together. Follow reading scrolls only while enabled; active playback controls remain sticky. Today, book overview, and Tabernacle retain their existing audio scopes, and explanations stay outside the speech selection.

Supporting checks: 27 Rust tests, JavaScript syntax, WASM build, and diff checks passed. Local actual device speech progressed from Psalm 117 into Psalm 118 with the highlighted verse in view, and Pause, Follow reading, and Stop responded. Final hosted acceptance is recorded separately after deployment; physical phone hardware and independent acoustic quality are not claimed.

## Exact verse continuation

Reading and listening now share one local browser bookmark containing the canonical book, chapter, and verse. Settled user scrolling updates the reading position; automatic scrolling/focus and active or paused speech cannot replace the spoken verse. Today Continue reading and chapter reloads restore the verse without auto-play. Continuous playback records the actual later chapter, and book-overview Listen resumes its saved verse. Clear removes the bookmark; storage failure is disclosed under Playback options. No account or cross-device sync is introduced.

Supporting checks: 27 Rust tests, progress validation/migration tests, JavaScript syntax, and the production WASM build passed. Local browser journeys exercised actual scrolling, Today Continue, real device speech, a Psalm 23 to 24 transition, reload without auto-play, book-overview resume, and Clear followed by reload. An initial local navigation-focus overwrite was reproduced and corrected before publication. Final hosted flows are checked separately against the published asset identities; local checks do not establish phone-hardware or cross-browser acceptance.

## Illustrated learning expansion and finish pass

Explore now has a three-scene gallery. The preserved Tabernacle has eight hotspots and dedicated object images; Noah’s Ark and the Sower’s Field add four each, with two overview paintings and eight individually generated detail illustrations. All sixteen stops have passage views, separate editorial explanations, review questions, and previous/next navigation. On narrow screens a selected hotspot brings the shared detail panel into view. Related links connect Genesis 6–8, Exodus 25–27, and Matthew 13 readers with their studies.

Today defaults to a curated 42-day Edition 2, with varied existing and new artwork. Edition 1’s original 14-day date mapping and share links remain available. About/help explains the text, images, device speech, and local progress. The footer links there and to the open-source repository. Sharing metadata includes a full-frame JPEG derivative of the existing Tabernacle painting; third-party preview-cache refresh is not implied.

Supporting checks: 33 Rust tests, progress validation tests, JavaScript syntax, the production WASM build, and diff checks passed. Local actual browser checks exercised scene selection, a dedicated close-up, source-bound questions, Scripture return, narrow-screen part selection, Today Edition 2, chapter-related study links, keyboard skip-to-content, keyboard hotspot selection, and About FAQs. The new artwork and unchanged source PNGs are recorded in the image manifest and learning provenance. Current public acceptance is performed separately after publication through Sites and the actual hosted UI; neither source tests nor a deployment receipt replaces that execution.

## Explore gallery consistency

The existing Today, Library and chapter reader are the visual authority for the Explore landing page. The gallery now uses Bible Display for descriptions, the reader's compact heading scale, thin gold artwork frames, unboxed copy and Library-style text links. Its instructional note is compact; the original scenes, illustrations, routes, Scripture panels and audio behavior are retained. Header styling and other page designs are unchanged.

Supporting checks: the three existing learning tests, formatting and the WASM build pass. Local browser checks at 1280px and 320px confirm three gallery links, serif copy, a full-width footer, no horizontal overflow, and a keyboard journey through a study hotspot and back to Scripture. These local checks are separate from hosted acceptance after publication.


## Selected scene-led Explore browser

Source visual truth: `assets/imagegen/reference/explore-scene-led-approved.png`, the first displayed concept selected on October 15, 2026. Implementation: `docs/site-explore.png`, a real local Rust/WASM browser capture. Both are 1536 × 1024 pixels at a 1536 × 1024 CSS viewport, density 1; no scaling was needed. State: Explore, Tabernacle selected, idle audio. Equal-scale full-view comparison: `docs/explore-scene-led-comparison.webp`; focused heading/index comparison: `docs/explore-scene-led-index-comparison.webp`. Both sides were inspected together.

Corrected findings and comparison history:

- P2, artwork integration: the first render isolated the image blending and exposed a hard rectangle. Removing that isolation allows the existing raster to blend into the shared charcoal. The final comparison was captured after the fix.
- P2, rhythm and controls: the initial index began 10px too low, the study action was narrower than the reference, and thumbnails showed too much surrounding ground. Reduced the heading gap, matched the desktop action width, and gave the three existing paintings deliberate thumbnail crops. Refined artwork contrast and color without replacing the source paintings.
- P2, intermediate width: an actual 901px render measured a 318px heading in a 286px column. A bounded responsive font size now measures 286px for both content and column at that width. The complete page fits at 320, 390, 768, 901, 1024 and 1536px.
- Capture limitation, not a product result: raw CDP viewport captures on the hidden Mac browser surface briefly returned a cropped native-density view. The final screenshots use the supported full-page browser capture and verified 1536 × 1024 dimensions. An initial Back-to-Scripture test used the decorative arrow in its accessible name; the corrected actual accessible name reached the passage. The first locator failure is retained in private test evidence.

Required fidelity surfaces:

- Typography: existing locally hosted Bible Display/Cormorant serif, compact spaced eyebrow, warm gold heading, serif source labels and descriptions. The existing header is preserved.
- Spacing/layout: compact three-scene index at left, one large scene at right, editorial title and Enter study below it, small separated source note, and the reference footer position. At narrow widths all three choices remain visible above the full-width scene and action.
- Colors/tokens: shared charcoal, ivory and aged gold; restrained rules and active-selection indicator, with no generic gallery cards.
- Images/icons: the three original scene paintings remain separate production assets. Their camera, object geometry and lighting differ from the ImageGen concept's reinterpretation; retaining the established study illustrations is intentional. Thumbnail crops and a modest CSS color treatment bring them closer while preserving the original files. Existing licensed Phosphor book and new Phosphor arrow icons are used. The page mockup is never a page background.
- Copy/content: heading, instructions, scene labels, source references, counts and action are live Rust-rendered content. Selection changes the URL, artwork, introduction and study destination. KJV Scripture, explanatory notes, dedicated details and device audio remain in the existing study routes.

Local browser outcomes: Ark selection/reload retains its matching introduction and link; keyboard Sower selection enters the correct default Scripture panel; a good-ground hotspot shows its dedicated illustration and Back to Scripture returns the exact cited KJV. Responsive checks cover six widths, a 56px narrow action, visible choices and no horizontal page overflow. Hosted release verification follows publication separately. Supporting checks: all 33 Rust tests before final CSS refinements, the focused three learning tests after hub markup changes, formatting, production WASM build and diff checks.

No actionable P0/P1/P2 findings remain in the compared default state. Intentional differences: preserved production artwork instead of the generated preview's reinterpreted scene; narrow layouts are responsive adaptations because the selected reference is desktop only. Exhaustive all-66-book/all-16-hotspot flows and physical-device qualification are outside this scoped layout run.

final result: passed


## Passage-following geographic Explore map — 2026-10-16

Authority: `assets/imagegen/reference/reading-map-approved.png`, selected with “yeah this is cool” and “proceed”. Actual local Rust/WASM screenshots: `docs/site-map.png` (1536 × 1024 CSS pixels, Acts 16:12, idle audio) and `docs/site-map-mobile.png` (390 × 844). The original and desktop screenshot were inspected together at equal scale; `docs/map-design-comparison.webp` retains the comparison.

Corrected findings:

- P1: inherited reader grid squeezed verse text into its number column. Scoped block layout and absolute verse numbers fix it.
- P1: the initial 3D control did not enable elevation. Initialization now sets the real DEM and tilted camera. Browser requests returned fresh NASA and DEM responses.
- P2: narrow grid children expanded past the viewport. A minimum-zero column, bounded selectors and stacked panel fix 320/390px clipping. Map tools have 44px targets.
- P2: resizing/font reflow could change the saved verse and retain a desktop camera offset. Reflow preserves the selected verse and recenters only while following. Acts 16:12 remains selected through desktop/390/320 changes.
- P2: overlapping labels crowded the map. Other-place labels now appear on hover/focus; journey labels remain visible. Anchors put marker dots at their coordinates and Neapolis has a separate label offset.
- Capture limitation: hidden in-app-browser WebGL screenshots after resizing sometimes omitted the canvas despite valid loaded state. Final screenshots were recaptured with the browser visually presented and actual terrain visible. Blank captures are not passing proof.

Required fidelity surfaces: the existing locally hosted Bible Display serif, ivory/gold and charcoal remain. Terrain fills the workspace beside a floating, independently scrolling Scripture panel, route strip, tools and bottom audio. Titles, Scripture, controls, links and selectors are real UI. Geographic explanations and source confidence remain separate from Scripture and speech. Existing branding is retained.

Intentional differences: real modern NASA regional imagery and sourced coordinates replace generated geography, so coastline, point spacing, color and terrain detail differ. Approximately 500m imagery is not street-level detail. Dashed connections are schematic. Device speech uses real verse progress instead of an invented waveform/time. Functional selectors, search, attribution and footer remain. Camera-follow settings are curated only for Acts 16–17, Luke 2 and Revelation 1:9; elsewhere the UI honestly shows named places/book context or no location. README, About and atlas notes disclose these limits.

Direct local browser outcomes: uncached NASA/DEM HTTP 200 responses; Philippi focus; sourced pin details and Scripture return; actual device speech, pause/resume, Acts 16:40 → Acts 17 advancement with visible verse and changed map setting; navigation cancels speech and returns to its verse; settled manual scrolling updates progress; reload/contextual reader links retain it; pan disables follow and Return restores it; 2D/3D and map controls; 66 book choices, Psalm 23 book context and Philemon no-location states; keyboard tabs; 320/390/768 layouts. Controlled tile-provider blocking produced a degraded message with readable Scripture; removing it and clicking Retry restored actual terrain. This negative is controlled failure evidence, not a real provider outage. Library 66-book display, neimai → Nehemiah, Today continuation, and each retained study with a representative dedicated image were exercised. Initial QA selector errors included decorative arrows or abbreviated headings; corrected actual accessible names reached the expected screens. First attempts remain in the private execution record.

Supporting checks: all 36 Rust tests, native binary compilation, JavaScript syntax, WASM production build and whitespace checks. Hosted acceptance is separate after publication. Exhaustive geographic review/all 16 hotspot journeys, physical phones, cross-browser speech and unavailable-WebGL hardware are NOT RUN.

No actionable P0/P1/P2 layout findings remain in compared states. Generated scenic detail and broader curated setting coverage remain explicit differences.

final result: passed
