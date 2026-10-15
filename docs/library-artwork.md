# Library artwork

The approved Library design is preserved at `assets/imagegen/reference/library-approved.png`. It is a visual reference only; the application never displays that mockup as its page.

Each of the 66 books has a separately generated portrait illustration. Built-in ImageGen created the scenes with a shared historical painting brief, charcoal shadows, warm antique gold light, and a distinct subject for each book. The complete input brief set is in `assets/imagegen/covers/briefs.json`; each adjacent book JSON records its actual prompt, generation timestamp, original output filename, source PNG, production WebP, dimensions, and processing. The combined registry remains `assets/imagegen/manifest.json`, including the existing Tabernacle artwork.

Source PNGs are retained at their original resolution. Production covers are exported full-frame at 768 × 1152 pixels as WebP at quality 88. CSS supplies the book binding, frame, title, and tagline. No title, interface label, or control is part of the artwork. Covers use decorative empty alt text because the surrounding link has the real book name as its accessible label.

These scenes are artistic interpretations and thematic illustrations, not archaeological evidence or additions to Scripture. The short descriptions and organizing categories are editorial navigation aids. The reader and audio player use the separate embedded King James Version corpus. Existing Tabernacle illustrations and the distinction between Scripture and Explanation are preserved.

The gallery loads covers lazily. Its filtering and sorting run in the shared Rust module, while the browser control module submits routes and previews cover sizing. The native Axum server embeds the same production images; the static build copies only the production WebPs into `dist/art/covers/`.

## Book overview reference and interface assets

The approved Genesis book-page concept is preserved at `assets/imagegen/reference/book-page-approved.png` (built-in ImageGen output `exec-a48e6061-b070-4081-8da9-4890e016f3d1.png`). The overview reuses all 66 existing cover paintings and their recorded provenance. Book titles, editorial descriptions, chapter links, and playback controls remain real HTML rendered by Rust. The concept is a design reference, not a deployed page image.

Book-page typography uses EB Garamond from the Google Fonts repository (`ofl/ebgaramond/EBGaramond[wght].ttf`), licensed under the included SIL Open Font License. Book-open and headphones icons are from Phosphor Icons (`core/assets/regular/`), under the included MIT license. These assets are stored locally; the live page does not require a third-party font or icon service.

## Dimensional binding correction

The Library and all 66 book overviews now compose each existing painting with a reusable ImageGen binding overlay. `web/art/book-binding.webp` supplies the rounded leather spine, raised bands, gold tooling, physical highlights, and ornate corners. Its large central picture opening and exterior use real alpha; the paintings show through beneath it. The 1086 × 1448 original is in `assets/imagegen/source/binding/book-binding.png`; the initial wider-mat draft is retained beside it. Production WebP uses lossless encoding with no resizing.

`web/art/gold-foil.webp` supplies a separate generated material behind live reading and selected-chapter buttons. Its source PNG, both generation prompts, the binding edit prompt, output identifiers, geometry, and processing are recorded in [binding provenance](../assets/imagegen/binding.json) and the shared manifest. Cormorant Garamond is locally hosted from the Google Fonts repository (`ofl/cormorantgaramond/CormorantGaramond[wght].ttf`), with its included SIL Open Font License. Reference/actual browser comparisons are in [design QA](../design-qa.md).
