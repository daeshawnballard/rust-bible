//! About and help page for the shared native and WebAssembly site.

use super::{esc, html_response, route_path, RouteResponse};

pub(super) fn page(client: bool) -> RouteResponse {
    let today = esc(&route_path("/today", client));
    let library = esc(&route_path("/library", client));
    let passage = esc(&route_path("/book/John/3?resume=16#v16", client));
    let codex = esc(&route_path("/codex", client));
    let journey = esc(&route_path(
        "/book/Acts/16?resume=12&context=journey#v12",
        client,
    ));
    let ark = esc(&route_path(
        "/book/Genesis/6?resume=14&context=illustration&scene=noahs-ark&part=construction#v14",
        client,
    ));
    let sower = esc(&route_path(
        "/book/Matthew/13?resume=3&context=illustration&scene=sower&part=path#v3",
        client,
    ));
    let tabernacle = esc(&route_path(
        "/book/Exodus/25?resume=31&context=illustration&scene=tabernacle&part=lamp#v31",
        client,
    ));
    let content = format!(
        r#"<section class="about-page" aria-labelledby="about-title">
  <header class="about-intro">
    <p class="about-eyebrow">The Bible · King James Version</p>
    <h1 id="about-title">About this Bible</h1>
    <p class="about-lede">Read the KJV, listen on your device, and open curated context beside the passage.</p>
    <p class="about-summary">The reader includes all 66 books of the King James Version (KJV). Scripture stays separate from editorial notes, explanations, and artwork.</p>
    <nav class="about-quick-links" aria-label="Start reading">
      <a class="about-primary" href="{today}">Read today’s passage <span aria-hidden="true">→</span></a>
      <a href="{library}">Browse the Library</a>
      <a href="{passage}">Open a passage</a>
      <a href="{codex}">Open the Bible Codex</a>
    </nav>
  </header>

  <section class="about-section" aria-labelledby="getting-started-title">
    <p class="about-eyebrow">Getting started</p>
    <h2 id="getting-started-title">Four ways to begin</h2>
    <div class="about-cards">
      <article class="about-card">
        <h3><a href="{today}">Today</a></h3>
        <p>Read the day’s short passage, open it in Passage, or listen to it.</p>
      </article>
      <article class="about-card">
        <h3><a href="{library}">Library</a></h3>
        <p>Browse all 66 illustrated books. A cover opens Passage directly and returns to your saved verse when it belongs to that book.</p>
      </article>
      <article class="about-card">
        <h3><a href="{passage}">Passage</a></h3>
        <p>Read continuous Scripture. Use the title menu for Book, Chapter, and text settings; choose Read to expand the reading column or Journey for the full-screen view.</p>
      </article>
      <article class="about-card">
        <h3><a href="{codex}">Bible Codex</a></h3>
        <p>Browse one book-organized guide with 360 selected People, Places, Miracles, and Study notes across all 66 books.</p>
      </article>
    </div>
  </section>

  <section class="about-section" aria-labelledby="reading-help-title">
    <p class="about-eyebrow">Reading and listening</p>
    <h2 id="reading-help-title">Find a passage and follow along</h2>
    <div class="about-reading-grid">
      <div>
        <h3>Find a book or reference</h3>
        <p>Use search in the header. Try <em>John 3:16</em>, <em>1 Cor 13:4-7</em>, <em>ps 23</em>, or a book name. If a book name has a typo, choose its suggested correction; for example, <em>neimai</em> suggests Nehemiah.</p>
        <p>Today and Library both offer <strong>Continue reading</strong> at your saved exact verse. A matching Library cover also resumes there. Progress is stored in this browser on this device, with no account or cloud sync. <strong>Clear</strong> removes the saved location.</p>
      </div>
      <div>
        <h3>Listen and follow the verse</h3>
        <p>Tap or click <strong>Listen</strong> to start; audio does not autoplay. The default Kokoro voice generates speech locally on your device. Its first use downloads about 100 MB of public model files. Device voices are also available. No account, API key, or paid speech service is used.</p>
        <p><strong>Follow reading</strong> scrolls the spoken verse into view. The bottom dock names the narrated passage and offers <strong>Return to passage</strong>. It stays available as you move among Today, Library, and Codex. Choosing Listen for another reference switches narration. Choosing a Read source link in Context or Codex stops narration and opens that reference.</p>
      </div>
    </div>
  </section>

  <section class="about-section" aria-labelledby="artwork-title">
    <p class="about-eyebrow">Text, notes, and illustrations</p>
    <h2 id="artwork-title">What is Scripture, and what is interpretation?</h2>
    <p>The KJV passages come from the embedded text corpus. Editorial notes, scene explanations, and artwork are separate reading aids; they are not part of Scripture.</p>
    <p>Illustrations are artistic interpretations or reconstructions, not archaeological evidence. Their hotspots can be inspected without moving the selected verse. Choose an explicit <strong>Read</strong> source link to jump to that Scripture; the jump stops narration.</p>
    <p class="about-source-links"><a href="https://github.com/daeshawnballard/rust-bible" target="_blank" rel="noopener noreferrer">Project source ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/tabernacle-sources.md" target="_blank" rel="noopener noreferrer">Tabernacle source notes ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/learning-sources.md" target="_blank" rel="noopener noreferrer">Illustration sources ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/library-artwork.md" target="_blank" rel="noopener noreferrer">Library artwork notes ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/assets/imagegen/manifest.json" target="_blank" rel="noopener noreferrer">Artwork provenance ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/thiagobodruk/bible" target="_blank" rel="noopener noreferrer">KJV corpus source ↗</a></p>
  </section>

  <section class="about-section about-access" aria-labelledby="access-title">
    <p class="about-eyebrow">Controls</p>
    <h2 id="access-title">Keyboard and display options</h2>
    <p>Links, buttons, chapter choices, and the questions below can be used with a keyboard. Hotspots can also be reached with a keyboard. Text and controls reflow for narrow screens; motion effects respect your device’s reduced-motion setting.</p>
  </section>

  <section class="about-section about-faq" aria-labelledby="faq-title">
    <p class="about-eyebrow">Help</p>
    <h2 id="faq-title">Common questions</h2>
    <details>
      <summary>Where is my reading progress saved?</summary>
      <p>The app saves your last exact verse in local browser storage on this device, when that storage is available. It is not connected to an account or synced to the cloud. Use <strong>Clear</strong> on Today to remove the saved location. Clearing site data or using private browsing can affect how long it remains. If storage is unavailable, reading and listening still work, but the location may not survive a reload.</p>
    </details>
    <details>
      <summary>How does Continue reading work?</summary>
      <p>The current verse is saved as you read; while audio is playing, the spoken verse takes priority. Continue reading on Today and Library returns to that exact verse. The matching Library cover opens Passage at the same location.</p>
    </details>
    <details>
      <summary>What is the difference between Read and Journey?</summary>
      <p>Read expands the central Scripture column. Journey opens the full-screen view with a window on the right, or a sheet on mobile. The window’s Scripture, Context, and Codex tabs reuse that same space while keeping the passage nearby.</p>
    </details>
    <details>
      <summary>Does inspecting a hotspot move my verse?</summary>
      <p>No. Inspecting a stop or illustrated detail leaves the selected verse in place. Its explicit Read source link moves to that passage and stops narration.</p>
    </details>
    <details>
      <summary>Does narration stop when I move around the app?</summary>
      <p>The single bottom audio dock stays available across Today, Library, and Codex navigation. It identifies the passage being narrated and has Return to passage. Browsing another book or chapter keeps the current narration. Choosing Listen for another reference switches it; choosing a Read source link in Context or Codex stops it and opens that reference. Kokoro playback shows waveform, elapsed time, and seeking for its current generated audio chunk. Device voices do not provide timing, so those controls are unavailable with them.</p>
    </details>
    <details>
      <summary>What does Journey cover?</summary>
      <p>Journey is available in all 66 books and all 1,189 chapters. There are 271 reading stages and thirteen contextual landscapes, alongside fourteen detailed story worlds. Book, literary, letter, prophetic, and vision context are labeled. Broad reading-stage markers are interface layout, not geographic coordinates or literal locations for every verse.</p>
      <p>Genesis 2–4 uses symbolic story scenery. Other illustrated journeys cover Noah and the flood (Genesis 7–9:17), the Exodus wilderness (Exodus 13:17–17), Ruth 1–2, Luke 2, and Mark 4–5. New worlds cover Joshua 3–4, 1 Samuel 17, Daniel 6, Jonah 1–4, Matthew 14, Luke 24, John 4, and Acts 27–28. Some verses within those chapters are deliberately unassigned. The active hotspot follows your reading or listening verse; the artwork compresses places and times. Curated geographic settings cover Acts 16–17 and Revelation 1:9. Precise stops only use their authored ranges and curated settings. Other chapters use broad book or literary context without assigning an exact location. Modern terrain needs a network connection and WebGL.</p>
      <p><a href="{journey}">Open the Acts 16 Journey example</a></p>
    </details>
    <details>
      <summary>Which illustrations are available?</summary>
      <p>The passage context includes Noah’s Ark across Genesis 6–9, the Sower in Matthew 13, and the Tabernacle in Exodus 25–31 and 35–40. Each scene opens at a linked verse and its selected part; its reconstruction and explanation remain separate from Scripture.</p>
      <p><a href="{ark}">Noah’s Ark</a> · <a href="{sower}">The Sower</a> · <a href="{tabernacle}">The Tabernacle</a></p>
    </details>
    <details>
      <summary>What is the Bible Codex?</summary>
      <p>It is one book-organized reference guide with 360 selected entries across all 66 books: People, Places, Miracles, and Study notes. Each entry points to a KJV verse. The selection is curated, not exhaustive.</p>
    </details>
    <details>
      <summary>Do I need an account or an API key?</summary>
      <p>No account or API key is needed to read or listen. Kokoro requires a supported browser, its initial model download, and enough device memory. If it cannot load, choose a device voice in Playback options. All narration is synthetic.</p>
    </details>
  </section>
</section>"#
    );
    html_response(
        200,
        "About and help — The Bible",
        "",
        &content,
        client,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::page;

    #[test]
    fn about_page_links_to_canonical_reader_contexts_and_lists_coverage() {
        let html = page(false).body;
        for expected in [
            "href=\"/book/Acts/16?resume=12&amp;context=journey#v12\"",
            "href=\"/book/Genesis/6?resume=14&amp;context=illustration&amp;scene=noahs-ark&amp;part=construction#v14\"",
            "href=\"/book/Matthew/13?resume=3&amp;context=illustration&amp;scene=sower&amp;part=path#v3\"",
            "href=\"/book/Exodus/25?resume=31&amp;context=illustration&amp;scene=tabernacle&amp;part=lamp#v31\"",
            "Genesis 2–4 uses symbolic story scenery. Other illustrated journeys cover Noah and the flood (Genesis 7–9:17), the Exodus wilderness (Exodus 13:17–17), Ruth 1–2, Luke 2, and Mark 4–5. New worlds cover Joshua 3–4, 1 Samuel 17, Daniel 6, Jonah 1–4, Matthew 14, Luke 24, John 4, and Acts 27–28. Some verses within those chapters are deliberately unassigned. The active hotspot follows your reading or listening verse; the artwork compresses places and times.",
            "Acts 16–17 and Revelation 1:9",
            "Genesis 6–9",
            "Exodus 25–31 and 35–40",
        ] {
            assert!(html.contains(expected), "missing about-page content: {expected}");
        }
        assert!(!html.contains(">Explore</a>"));
    }
}
