# Library artwork

The approved Library design is preserved at `assets/imagegen/reference/library-approved.png`. It is a visual reference only; the application never displays that mockup as its page.

Each of the 66 books has a separately generated portrait illustration. Built-in ImageGen created the scenes with a shared historical painting brief, charcoal shadows, warm antique gold light, and a distinct subject for each book. The complete input brief set is in `assets/imagegen/covers/briefs.json`; each adjacent book JSON records its actual prompt, generation timestamp, original output filename, source PNG, production WebP, dimensions, and processing. The combined registry remains `assets/imagegen/manifest.json`, including the existing Tabernacle artwork.

Source PNGs are retained at their original resolution. Production covers are exported full-frame at 768 × 1152 pixels as WebP at quality 88. CSS supplies the book binding, frame, title, and tagline. No title, interface label, or control is part of the artwork. Covers use decorative empty alt text because the surrounding link has the real book name as its accessible label.

These scenes are artistic interpretations and thematic illustrations, not archaeological evidence or additions to Scripture. The short descriptions and organizing categories are editorial navigation aids. The reader and audio player use the separate embedded King James Version corpus. Existing Tabernacle illustrations and the distinction between Scripture and Explanation are preserved.

The gallery loads covers lazily. Its filtering and sorting run in the shared Rust module, while the browser control module submits routes and previews cover sizing. The native Axum server embeds the same production images; the static build copies only the production WebPs into `dist/art/covers/`.
