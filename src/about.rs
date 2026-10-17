//! About and help page for the shared native and WebAssembly site.

use super::{esc, html_response, route_path, RouteResponse};

pub(super) fn page(client: bool) -> RouteResponse {
    let today = esc(&route_path("/today", client));
    let library = esc(&route_path("/library", client));
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
    <p class="about-lede">A quiet place to read Scripture, listen with your device, and follow Journey or Illustrations in the reader.</p>
    <p class="about-summary">This reader presents the 66 books of the King James Version (KJV). Scripture text is kept separate from editorial notes, explanations, and artwork.</p>
    <nav class="about-quick-links" aria-label="Start reading">
      <a class="about-primary" href="{today}">Read today’s passage <span aria-hidden="true">→</span></a>
      <a href="{library}">Browse the Library</a>
      <a href="{journey}">Open Journey</a>
      <a href="{ark}">Open Illustrations</a>
    </nav>
  </header>

  <section class="about-section" aria-labelledby="getting-started-title">
    <p class="about-eyebrow">Getting started</p>
    <h2 id="getting-started-title">Choose a way in</h2>
    <div class="about-cards">
      <article class="about-card">
        <h3><a href="{today}">Today</a></h3>
        <p>Read a short passage for the day. Open its full chapter or listen to that passage.</p>
      </article>
      <article class="about-card">
        <h3><a href="{library}">Library</a></h3>
        <p>Browse all 66 books. A cover opens the passage directly, resuming your saved verse when it is in that book. Choose chapters inside the reader.</p>
      </article>
      <article class="about-card">
        <h3><a href="{journey}">Journey</a></h3>
        <p>Follow a verse through its curated geographic context beside the canonical Scripture text. Unsupported passages keep their setting unassigned.</p>
      </article>
      <article class="about-card">
        <h3><a href="{ark}">Illustrations</a></h3>
        <p>Open a scene from its linked verse. The illustration and its notes stay alongside Scripture in the canonical reader.</p>
      </article>
    </div>
  </section>

  <section class="about-section" aria-labelledby="reading-help-title">
    <p class="about-eyebrow">Reading and listening</p>
    <h2 id="reading-help-title">Find a passage and follow along</h2>
    <div class="about-reading-grid">
      <div>
        <h3>Find a book or reference</h3>
        <p>Use the search field in the header. Try <em>John 3:16</em>, <em>1 Cor 13:4-7</em>, or a book name. If a book name has a typo, choose the suggested correction—for example, <em>neimai</em> suggests Nehemiah. Then choose a chapter or verse link.</p>
        <p>The usual path is <strong>Books → Passage</strong>. The Library contains the books. Inside the passage, use Book and Chapter to move around, or expand All chapters. Journey, illustrated hotspots, and the audio player stay in this reader.</p>
      </div>
      <div>
        <h3>Listen to the selection you opened</h3>
        <p>The default natural narrator uses the open-source Kokoro model on your device. The first listen downloads about 100 MB of model files; your browser can cache them for future visits. There is no paid speech API, account, or key. Audio is generated locally from the KJV text; Scripture is never sent to a narration service. Device voices are also available in Playback options. Tap or click a Listen control to start—audio does not autoplay. Choose a voice and speed in the playback controls. <strong>Follow reading</strong> scrolls the spoken verse into view and can be turned off.</p>
        <p>Today reads only its passage. The reader’s <strong>Listen from here</strong> continues through later Scripture. Switching Journey or browsing illustrated hotspots keeps the same audio queue. Reading an illustration’s source is an intentional verse jump and stops narration. Playback stops when you navigate to another page.</p>
      </div>
    </div>
  </section>

  <section class="about-section" aria-labelledby="artwork-title">
    <p class="about-eyebrow">Text, notes, and illustrations</p>
    <h2 id="artwork-title">What is Scripture, and what is interpretation?</h2>
    <p>The KJV passages are canonical Scripture from the embedded text corpus. Editorial notes and scene explanations are separate reading aids; they are not part of the Bible text or audio.</p>
    <p>Illustrations are artistic interpretations or reconstructions. They are not archaeological evidence or proof of what a historical object or place looked like.</p>
    <p class="about-source-links"><a href="https://github.com/daeshawnballard/rust-bible" target="_blank" rel="noopener noreferrer">Project source ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/tabernacle-sources.md" target="_blank" rel="noopener noreferrer">Tabernacle source notes ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/learning-sources.md" target="_blank" rel="noopener noreferrer">Illustration sources ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/docs/library-artwork.md" target="_blank" rel="noopener noreferrer">Library artwork notes ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/daeshawnballard/rust-bible/blob/main/assets/imagegen/manifest.json" target="_blank" rel="noopener noreferrer">Artwork provenance ↗</a> <span aria-hidden="true">·</span> <a href="https://github.com/thiagobodruk/bible" target="_blank" rel="noopener noreferrer">KJV corpus source ↗</a></p>
  </section>

  <section class="about-section about-access" aria-labelledby="access-title">
    <p class="about-eyebrow">Controls</p>
    <h2 id="access-title">Keyboard and display options</h2>
    <p>Links, buttons, chapter choices, and the questions below can be used with a keyboard. In illustrations, hotspots can be reached with a keyboard, and a labeled selector offers another way to choose a stop. Text and controls reflow for narrow screens; motion effects respect your device’s reduced-motion setting.</p>
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
      <p>As you read, the current verse is saved after scrolling settles. While audio is playing, the verse being spoken takes priority. Continue reading opens at that verse, including when it is in a later chapter. The matching Library cover resumes that verse directly in the reader, where Listen from here starts narration.</p>
    </details>
    <details>
      <summary>What does Journey show?</summary>
      <p>Journey follows the selected verse in the canonical reader. Genesis 2–4 uses symbolic story scenery. Curated geographic settings cover Acts 16–17, Luke 2, and Revelation 1:9. Only curated settings and chapter journey stops appear in this companion; named places elsewhere do not automatically establish a story setting. When no curated setting or journey applies, Journey leaves the location unassigned; it does not fall back to a world view or borrow places from another chapter in the book. Modern terrain needs a network connection and WebGL; Scripture and available place context remain readable if the map fails.</p>
    </details>
    <details>
      <summary>Which illustrations are available?</summary>
      <p>The reader’s Illustrations context includes Noah’s Ark across Genesis 6–9, the Sower in Matthew 13, and the Tabernacle in Exodus 25–31 and 35–40. Each scene opens at a linked verse with its selected part; its reconstruction and explanation remain separate from canonical Scripture.</p>
      <p><a href="{ark}">Noah’s Ark</a> · <a href="{sower}">The Sower</a> · <a href="{tabernacle}">The Tabernacle</a></p>
    </details>
    <details>
      <summary>Do I need an account or an API key?</summary>
      <p>No account or API key is needed to read or listen. Natural speech requires a supported browser, an initial model download and enough device memory. If it cannot load, choose Device default in Playback options. All narration is synthetic.</p>
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
            "Genesis 2–4 uses symbolic story scenery.",
            "Acts 16–17, Luke 2, and Revelation 1:9",
            "Genesis 6–9",
            "Exodus 25–31 and 35–40",
        ] {
            assert!(html.contains(expected), "missing about-page content: {expected}");
        }
        assert!(!html.contains(">Explore</a>"));
    }
}
