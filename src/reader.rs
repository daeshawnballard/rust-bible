//! Chapter and passage reader markup, kept separate from route dispatch.

use super::{bible, esc, ref_label, route_path, slug, verses, Ref, NAMES};

fn chapter_title(book: usize, chapter: usize) -> String {
    if book == 18 {
        format!("Psalm {chapter}")
    } else {
        format!("{} {chapter}", NAMES[book])
    }
}

fn book_selector(book: usize, chapter: usize, client: bool) -> String {
    let mut html = String::from(
        "<label class=\"reader-select-label\">Book<select data-reader-book aria-label=\"Book\">",
    );
    for (index, name) in NAMES.iter().enumerate() {
        let selected = if index == book { " selected" } else { "" };
        let value = route_path(&format!("/book/{}/1", slug(index)), client);
        html.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            esc(&value),
            selected,
            esc(name)
        ));
    }
    html.push_str("</select></label>");

    let mut chapters = String::from(
        "<label class=\"reader-select-label\">Chapter<select data-reader-chapter aria-label=\"Chapter\">",
    );
    for number in 1..=bible().books[book].len() {
        let selected = if number == chapter { " selected" } else { "" };
        let value = route_path(&format!("/book/{}/{number}", slug(book)), client);
        chapters.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            esc(&value),
            selected,
            format!("Chapter {number}")
        ));
    }
    chapters.push_str("</select></label>");
    html.push_str(&chapters);
    html
}

pub(super) fn listen_controls(reference: &str, scope: &str) -> String {
    format!(
        r#"<div class="scripture-player reader-audio" data-read-aloud data-audio-compact data-audio-scope="{scope}" data-reference="{reference}">
  <button class="player-toggle reader-audio-play" type="button" data-audio-action="play" aria-label="Listen to this {scope}">
    <svg viewBox="0 0 24 24" aria-hidden="true"><path class="player-play-icon" d="M8 4 21 12 8 20Z"/><path class="player-pause-icon" d="M7 5h4v14H7zm7 0h4v14h-4z"/></svg><span class="player-title">Listen to this {scope}</span>
  </button>
  <div class="player-copy"><span data-audio-status role="status" aria-live="polite">Free natural voice · First download about 100 MB.</span></div>
  <details class="reader-audio-options"><summary aria-label="Playback options">•••</summary><div class="reader-audio-settings">
    <label>Speed<select data-audio-rate aria-label="Speech speed"><option value="0.75">0.75×</option><option value="1" selected>1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label>
    <label>Voice<select data-audio-voice aria-label="Voice"><option value="kokoro:af_heart">Heart · Natural American</option></select></label>
    <button type="button" data-audio-action="stop">Stop reading</button>
    <small data-voice-help>Free, synthetic Kokoro voice. First listen downloads about 100 MB, then generates audio on this device. Scripture only. Device voices are also available.</small>
  </div></details>
</div>"#,
        reference = esc(reference),
        scope = esc(scope)
    )
}

fn breadcrumb(book: usize, chapter: usize, client: bool, passage: Option<&str>) -> String {
    let library = route_path("/library", client);
    let book_path = route_path(&format!("/book/{}", slug(book)), client);
    let chapter_path = route_path(&format!("/book/{}/{chapter}", slug(book)), client);
    let mut html = format!(
        "<nav class=\"reader-breadcrumb\" aria-label=\"Breadcrumb\"><ol><li><a href=\"{}\">Library</a></li><li><a href=\"{}\">{}</a></li>",
        esc(&library),
        esc(&book_path),
        esc(NAMES[book])
    );
    if let Some(label) = passage {
        html.push_str(&format!(
            "<li><a href=\"{}\">{chapter}</a></li><li aria-current=\"page\">{}</li>",
            esc(&chapter_path),
            esc(label)
        ));
    } else {
        html.push_str(&format!("<li aria-current=\"page\">{chapter}</li>"));
    }
    html.push_str("</ol></nav>");
    html
}

fn reader_controls(reference: &str) -> String {
    listen_controls(reference, "reading")
        .replacen("data-read-aloud", "data-read-aloud data-reader-audio", 1)
        .replace("Listen to this reading", "Listen from here")
        .replace("</div></details>", "<small data-progress-notice>Verse saved on this device. Clear it on Today.</small></div></details>")
        .replace("<small data-voice-help>", "<label class=\"reader-follow\"><input type=\"checkbox\" data-audio-follow checked> Follow reading</label><small data-voice-help>Continues into following chapters. ")
}

pub(super) fn map_audio(reference: &str) -> String {
    reader_controls(reference)
}

pub(super) fn map_panel(book: usize, chapter: usize, verse: usize, client: bool) -> String {
    let reference = format!("{} {chapter}", NAMES[book]);
    let mut html = format!("<section data-reader data-book-index=\"{book}\" data-book-name=\"{}\" data-chapter=\"{chapter}\" data-reader-route=\"/book/{}/{chapter}\" aria-label=\"Bible passage reader\"><div class=\"scripture reader-scripture\" data-reader-primary-scripture>", esc(NAMES[book]), slug(book));
    for (i, text) in bible().books[book][chapter - 1].iter().enumerate() {
        let number = i + 1;
        html.push_str(&format!("<p class=\"v{}\" id=\"v{number}\" data-reference=\"{}\" data-verse=\"{number}\" tabindex=\"0\" aria-label=\"Select verse {number}\"><sup>{number}</sup><span class=\"verse-text\">{}</span></p>",if number==verse {" is-map-current"} else {""},esc(&reference),esc(text)));
    }
    let count = bible().books[book][chapter - 1].len();
    html.push_str("</div>");
    html.push_str(&continuation(book, chapter, count + 1, count, client));
    html.push_str("</section>");
    html
}

fn reader_open(book: usize, chapter: usize, client: bool) -> String {
    let route = format!("/book/{}/{chapter}", slug(book));
    let mut html = format!(
        "<section class=\"reader-page\" data-reader data-book-index=\"{book}\" data-book-name=\"{}\" data-chapter=\"{chapter}\" data-reader-route=\"{}\" aria-label=\"Bible chapter reader\">{}",
        esc(NAMES[book]),
        esc(&route),
        breadcrumb(book, chapter, client, None)
    );
    html.push_str(&format!(
        "<div class=\"reader-heading\"><h1>{}</h1><p>King James Version</p></div>",
        esc(&chapter_title(book, chapter))
    ));
    html.push_str("<div class=\"reader-toolbar\"><div class=\"reader-selectors\">");
    html.push_str(&book_selector(book, chapter, client));
    html.push_str("</div>");
    html.push_str(&super::atlas::reader_link(book, chapter, client));
    html
}

fn pagination(book: usize, chapter: usize, client: bool) -> String {
    let chapters = &bible().books[book];
    let previous = if chapter > 1 {
        Some((book, chapter - 1))
    } else if book > 0 {
        Some((book - 1, bible().books[book - 1].len()))
    } else {
        None
    };
    let next = if chapter < chapters.len() {
        Some((book, chapter + 1))
    } else if book + 1 < NAMES.len() {
        Some((book + 1, 1))
    } else {
        None
    };

    let mut html =
        String::from("<nav class=\"reader-pagination\" aria-label=\"Chapter navigation\">");
    if let Some((book, chapter)) = previous {
        let href = route_path(&format!("/book/{}/{chapter}", slug(book)), client);
        html.push_str(&format!(
            "<a rel=\"prev\" href=\"{}\">← {}</a>",
            esc(&href),
            esc(&chapter_title(book, chapter))
        ));
    } else {
        html.push_str("<span aria-hidden=\"true\"></span>");
    }

    let chapters_path = route_path(&format!("/book/{}", slug(book)), client);
    html.push_str(&format!(
        "<a class=\"reader-all-chapters\" href=\"{}\">All chapters</a>",
        esc(&chapters_path)
    ));

    if let Some((book, chapter)) = next {
        let href = route_path(&format!("/book/{}/{chapter}", slug(book)), client);
        html.push_str(&format!(
            "<a rel=\"next\" href=\"{}\">{} →</a>",
            esc(&href),
            esc(&chapter_title(book, chapter))
        ));
    } else {
        html.push_str("<span aria-hidden=\"true\"></span>");
    }
    html.push_str("</nav>");
    html
}

fn following_chapter(book: usize, chapter: usize) -> Option<(usize, usize)> {
    if chapter < bible().books[book].len() {
        Some((book, chapter + 1))
    } else if book + 1 < NAMES.len() {
        Some((book + 1, 1))
    } else {
        None
    }
}

/// Keep short readings flowing into real, sequential Scripture. Each added
/// chapter has an explicit heading and its own route. Bound the initial page;
/// continuous playback requests the next Rust-rendered fragment when needed.
fn continuation(
    book: usize,
    chapter: usize,
    next_verse: usize,
    visible_verses: usize,
    client: bool,
) -> String {
    const MIN_READING_VERSES: usize = 16;
    let mut count = visible_verses;
    let mut cursor = if next_verse <= bible().books[book][chapter - 1].len() {
        Some((book, chapter, next_verse))
    } else {
        following_chapter(book, chapter).map(|(b, ch)| (b, ch, 1))
    };
    let mut html = String::new();
    let mut added = 0;
    while count < MIN_READING_VERSES && added < 3 {
        let Some((b, ch, start)) = cursor else { break };
        let text = &bible().books[b][ch - 1];
        let title = chapter_title(b, ch);
        let label = if start > 1 {
            format!("{title}:{start}–{}", text.len())
        } else {
            title.clone()
        };
        let href = route_path(&format!("/book/{}/{ch}", slug(b)), client);
        let heading_id = format!("continued-{b}-{ch}");
        html.push_str(&format!(
            "<section class=\"reader-continuation\" data-continuation-book=\"{b}\" data-continuation-chapter=\"{ch}\" data-book-index=\"{b}\" data-chapter=\"{ch}\" data-book-name=\"{}\" data-reader-route=\"/book/{}/{ch}\" aria-labelledby=\"{heading_id}\"><div class=\"reader-continuation-heading\"><h2 id=\"{heading_id}\">{}</h2><a href=\"{}\" aria-label=\"Open {}\">Open chapter ↗</a></div><div class=\"scripture reader-scripture\" role=\"group\" aria-label=\"{} Scripture\">",
            esc(NAMES[b]), esc(&slug(b)), esc(&label), esc(&href), esc(&title), esc(&label)
        ));
        for (index, verse) in text.iter().enumerate().skip(start - 1) {
            let number = index + 1;
            html.push_str(&format!(
                "<p class=\"v\" id=\"continued-{b}-{ch}-v{number}\" data-reference=\"{} {ch}\" data-verse=\"{number}\"><sup>{number}</sup><span class=\"verse-text\">{}</span></p>",
                esc(NAMES[b]), esc(verse)
            ));
        }
        html.push_str("</div></section>");
        count += text.len() - start + 1;
        added += 1;
        cursor = following_chapter(b, ch).map(|(b, ch)| (b, ch, 1));
    }
    if let Some((b, ch, start)) = cursor {
        let href = route_path(&format!("/book/{}/{ch}", slug(b)), client);
        html.push_str(&format!(
                "<p class=\"reader-continue-next\" data-reader-next=\"/api/reader-next?book={b}&amp;chapter={ch}&amp;verse={start}\"><a href=\"{}\">Continue with {} →</a></p>",
                esc(&href),
                esc(&chapter_title(b, ch))
            ));
    }
    html
}

pub(super) fn next_fragment(book: usize, chapter: usize, verse: usize, client: bool) -> String {
    continuation(book, chapter, verse, 0, client)
}

/// Render a complete chapter, keeping each verse addressable for highlighting,
/// speech playback, and direct links such as `#v3`.
pub(super) fn chapter_content(
    book: usize,
    chapter: usize,
    highlighted: Option<(usize, usize)>,
    client: bool,
) -> String {
    let mut html = reader_open(book, chapter, client);
    let reference = format!("{} {chapter}", NAMES[book]);
    html.push_str(&reader_controls(&reference));
    html.push_str("</div><div class=\"scripture reader-scripture\" data-reader-primary-scripture role=\"group\" aria-label=\"Scripture text\">");
    for (index, text) in bible().books[book][chapter - 1].iter().enumerate() {
        let number = index + 1;
        let class = if highlighted.is_some_and(|(start, end)| number >= start && number <= end) {
            "v hl"
        } else {
            "v"
        };
        html.push_str(&format!(
            "<p class=\"{class}\" id=\"v{number}\" data-reference=\"{}\" data-verse=\"{number}\"><sup>{number}</sup><span class=\"verse-text\">{}</span></p>",
            esc(&reference),
            esc(text)
        ));
    }
    html.push_str("</div>");
    html.push_str(&pagination(book, chapter, client));
    let verse_count = bible().books[book][chapter - 1].len();
    html.push_str(&continuation(
        book,
        chapter,
        verse_count + 1,
        verse_count,
        client,
    ));
    html.push_str(&super::learning::related(book, chapter, client));
    html.push_str("</section>");
    html
}

/// Render only the requested verses while preserving a link to the full chapter.
pub(super) fn passage_content(reference: &Ref, url: &str, client: bool) -> String {
    let book = reference.book;
    let chapter = reference.ch;
    let label = ref_label(reference);
    let mut html = format!(
        "<section class=\"reader-page reader-passage\" data-reader data-book-index=\"{book}\" data-book-name=\"{}\" data-chapter=\"{chapter}\" data-reader-route=\"{}\" aria-label=\"Bible passage reader\">{}",
        esc(NAMES[book]),
        esc(&format!("/book/{}/{chapter}", slug(book))),
        breadcrumb(book, chapter, client, Some(&label))
    );
    html.push_str(&format!(
        "<div class=\"reader-heading\"><h1>{}</h1><p>King James Version</p></div>",
        esc(&label)
    ));
    html.push_str("<div class=\"reader-toolbar\"><div class=\"reader-selectors\">");
    html.push_str(&book_selector(book, chapter, client));
    html.push_str("</div>");
    html.push_str(&super::atlas::reader_link(book, chapter, client));
    html.push_str(&reader_controls(&label));
    html.push_str("</div><div class=\"scripture reader-scripture\" data-reader-primary-scripture role=\"group\" aria-label=\"Scripture text\">");
    for (number, text) in verses(reference) {
        html.push_str(&format!(
            "<p class=\"v\" id=\"v{number}\" data-reference=\"{}\" data-verse=\"{number}\"><sup>{number}</sup><span class=\"verse-text\">{}</span></p>",
            esc(&format!("{} {chapter}", NAMES[book])),
            esc(text)
        ));
    }
    html.push_str("</div>");
    html.push_str(&format!(
        "<p class=\"reader-full-chapter\"><a href=\"{}\">Read full chapter →</a></p>",
        esc(url)
    ));
    let last_verse = reference
        .v2
        .or(reference.v1)
        .unwrap_or(bible().books[book][chapter - 1].len());
    let first_verse = reference.v1.unwrap_or(1);
    html.push_str(&continuation(
        book,
        chapter,
        last_verse + 1,
        last_verse - first_verse + 1,
        client,
    ));
    html.push_str(&super::learning::related(book, chapter, client));
    html.push_str("</section>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn short_psalm_continues_in_canonical_order_without_changing_primary_scripture() {
        let html = chapter_content(18, 23, None, true);
        let primary = html
            .split("data-reader-primary-scripture")
            .nth(1)
            .unwrap()
            .split("</div>")
            .next()
            .unwrap();
        assert_eq!(primary.matches("class=\"v\"").count(), 6);
        assert!(html.contains("<h2 id=\"continued-18-24\">Psalm 24</h2>"));
        assert!(html.contains(&esc(&bible().books[18][23][0])));
        assert!(html.contains("Continue with Psalm 25 →"));
        assert_eq!(html.matches("data-read-aloud ").count(), 1);
    }

    #[test]
    fn focused_range_continues_after_the_range_without_repeating_it() {
        let reference = Ref {
            book: 42,
            ch: 3,
            v1: Some(16),
            v2: Some(16),
        };
        let html = passage_content(&reference, "#/book/John/3?v=16-16#v16", true);
        assert!(html.contains("John 3:17–36</h2>"));
        assert!(html.contains("id=\"continued-42-3-v17\""));
        assert!(!html.contains("id=\"continued-42-3-v16\""));
        assert_eq!(html.matches("For God so loved the world").count(), 1);
    }

    #[test]
    fn continuation_crosses_book_boundaries_but_stops_at_the_end_of_scripture() {
        let malachi = chapter_content(38, 4, None, true);
        assert!(malachi.contains("<h2 id=\"continued-39-1\">Matthew 1</h2>"));
        assert!(malachi.contains("data-book-index=\"39\" data-chapter=\"1\" data-book-name=\"Matthew\" data-reader-route=\"/book/Matthew/1\""));
        assert!(malachi.contains("href=\"#/book/Matthew/1\" aria-label=\"Open Matthew 1\""));
        let final_chapter = chapter_content(65, 22, None, true);
        assert!(!final_chapter.contains("class=\"reader-continuation\""));
        assert!(!final_chapter.contains("rel=\"next\""));
    }

    #[test]
    fn all_chapters_keep_unique_verse_anchors_and_bounded_continuations() {
        for book in 0..NAMES.len() {
            for chapter in 1..=bible().books[book].len() {
                let html = chapter_content(book, chapter, None, true);
                let mut ids = HashSet::new();
                for tail in html.split(" id=\"").skip(1) {
                    let id = tail.split('"').next().unwrap();
                    assert!(
                        ids.insert(id),
                        "duplicate {id} in {} {chapter}",
                        NAMES[book]
                    );
                }
                assert!(html.matches("class=\"reader-continuation\"").count() <= 3);
            }
        }
    }

    #[test]
    fn continuous_audio_fragments_resume_exactly_and_end_without_wrapping() {
        let primary = chapter_content(18, 119, None, true);
        assert!(primary.contains("data-reader-audio"));
        assert!(primary.contains("book=18&amp;chapter=120&amp;verse=1"));
        let next = next_fragment(18, 120, 1, true);
        assert!(next.contains("continued-18-120-v1"));
        assert!(!next.contains("continued-18-119"));
        assert!(next.contains("Psalm 121</h2>"));
        let focused = next_fragment(0, 1, 17, true);
        assert!(focused.contains("continued-0-1-v17"));
        assert!(!focused.contains("continued-0-1-v16"));
        let last = next_fragment(65, 22, 21, true);
        assert!(last.contains("continued-65-22-v21"));
        assert!(!last.contains("data-reader-next"));
    }
}
