# Ivory and forest design QA

final result: passed

Scope: local browser visual and interaction comparison for the selected ivory/forest redesign. This is not public deployment acceptance or physical-device qualification.

## Authority and comparison

The selected authority is `assets/imagegen/reference/ivory-forest-approved.png`, with palette and constraints in its adjacent JSON. The yellow-paper alternative was not adopted. Corresponding reference panels and all four implementation captures were assembled into one comparison board and inspected together, followed by full-resolution Codex, Today, Library, Journey, and phone captures. Sanitized implementation captures are in `docs/screenshots/ivory-forest/`. Desktop viewport: 1440 × 960. Responsive checks: 320, 390, and 768 CSS pixels wide, 960 high.

## Findings corrected

- P2: legacy dark-theme variables overrode the ivory shell. Final shared tokens now win on native and WASM bodies.
- P2: the daily source label and Continue arrow retained low-contrast gold. Both now use dark ink/forest.
- P2: portrait voice and speed selectors occupied the same grid cell. Explicit dock placement now keeps both in separate 44px targets; desktop retains one row and portrait uses two.
- P2: the non-reader phone header wrapped into two rows. Condensed brand and exact legacy-selector overrides keep a single navigation rail.
- P2: phone Library filters squeezed Continue reading. Filters and Continue now each have a complete row.
- P2: Codex could retain stale note content during a failed or pending selection. Loading/error feedback replaces the old note, with a guarded response and retry. Same-route history remounts retain index scroll.

## Five fidelity surfaces

- Typography: bundled licensed Inter for navigation and controls; existing Garamond for Scripture and note titles. No generated text is used as interface copy.
- Spacing/layout: search first beneath navigation; five flat covers at the default desktop size; Codex index plus selected note; fullscreen Journey with ivory passage window; compact waveform dock.
- Colors: ivory `#F7F5EE`, ink `#172D27`, forest `#29664D`, pale surfaces `#ECEEE4`, selection `#E2EDE4`, borders `#D9DED4`. Light native form controls, visible focus treatment, forest active states.
- Imagery: all 66 existing cover paintings remain individual assets, now without binding overlays. Existing Journey scenes and dedicated object artwork are preserved. A new, separately generated Psalm 23 shepherd landscape matches the selected direction; its source and provenance are retained.
- Content: embedded KJV wording and reference ranges remain intact. The actual daily reading retains all three selected Psalm verses, whereas the concept shows one. Editorial notes remain explicitly labeled, with deliberate source links and location uncertainty.

## Observed local outcomes

The final responsive geometry pass had no horizontal overflow or voice/rate overlap across Today, Library, Codex, and Journey at 320/390/768. Real browser input covered Codex selection and browser Back, fuzzy `neimai` lookup with the Nehemiah suggestion, Testament and Gospel filtering, sort/cover-size changes, and empty Codex search results. Journey Scripture/Context selection and the current verse were inspected. Supporting Rust and JavaScript checks are recorded separately; neither this file nor local screenshots establish deployed behavior.

No open P0/P1/P2 visual finding remains in the inspected comparison. Retained differences: established paintings and scene compositions are reused instead of copying the reference's scenery; existing daily verse ranges, date, contextual links, optional Codex reader tab, and uncertainty labels remain. The idle waveform is blank until real narration supplies audio data. Physical devices, other engines, and screen-reader operation were not run in this design pass. Native fetch failure feedback was source-reviewed rather than forced through a simulated response.
