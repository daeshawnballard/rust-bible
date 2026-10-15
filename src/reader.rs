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
  <div class="player-copy"><span data-audio-status role="status" aria-live="polite">Device voice · Scripture only.</span></div>
  <details class="reader-audio-options"><summary aria-label="Playback options">•••</summary><div class="reader-audio-settings">
    <label>Speed<select data-audio-rate aria-label="Speech speed"><option value="0.75">0.75×</option><option value="1" selected>1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label>
    <label>Voice<select data-audio-voice aria-label="Voice"><option value="">Device default</option></select></label>
    <button type="button" data-audio-action="stop">Stop reading</button>
    <small>Device voice · Reads the KJV text only.</small>
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
    html.push_str(&listen_controls(&reference, "chapter"));
    html.push_str("</div><div class=\"scripture reader-scripture\" role=\"group\" aria-label=\"Scripture text\">");
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
    html.push_str(&listen_controls(&label, "passage"));
    html.push_str("</div><div class=\"scripture reader-scripture\" role=\"group\" aria-label=\"Scripture text\">");
    for (number, text) in verses(reference) {
        html.push_str(&format!(
            "<p class=\"v\" id=\"v{number}\" data-reference=\"{}\" data-verse=\"{number}\"><sup>{number}</sup><span class=\"verse-text\">{}</span></p>",
            esc(&label),
            esc(text)
        ));
    }
    html.push_str("</div>");
    html.push_str(&format!(
        "<p class=\"reader-full-chapter\"><a href=\"{}\">Read full chapter →</a></p>",
        esc(url)
    ));
    html.push_str("</section>");
    html
}
