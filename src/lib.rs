//! Shared KJV Bible lookup, search, and HTML rendering for the native and WASM apps.
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::OnceLock};

static RAW: &str = include_str!("../data/kjv.json");
const CSS: &str = concat!(
    include_str!("../web/style.css"),
    include_str!("../web/reader.css"),
    include_str!("../web/today.css"),
    include_str!("../web/learning.css"),
    include_str!("../web/about.css"),
    include_str!("../web/vendor/maplibre/maplibre-gl.css"),
    include_str!("../web/atlas.css"),
    include_str!("../web/polish-library.css"),
    include_str!("../web/polish.css"),
    include_str!("../web/continue.css"),
    include_str!("../web/reader-illustrations.css"),
    include_str!("../web/reader-context.css"),
    include_str!("../web/gallery-polish.css"),
    include_str!("../web/codex.css"),
    include_str!("../web/audio-dock.css"),
    include_str!("../web/immersive.css"),
    include_str!("../web/ivory-library.css"),
    include_str!("../web/ivory.css")
);

mod about;
mod atlas;
mod codex;
mod journey_books;
mod journey_scenes;
mod learning;
mod library;
mod passage_scenes;
mod reader;
mod reader_context;
mod tabernacle;
mod today;

const NAMES: [&str; 66] = [
    "Genesis",
    "Exodus",
    "Leviticus",
    "Numbers",
    "Deuteronomy",
    "Joshua",
    "Judges",
    "Ruth",
    "1 Samuel",
    "2 Samuel",
    "1 Kings",
    "2 Kings",
    "1 Chronicles",
    "2 Chronicles",
    "Ezra",
    "Nehemiah",
    "Esther",
    "Job",
    "Psalms",
    "Proverbs",
    "Ecclesiastes",
    "Song of Solomon",
    "Isaiah",
    "Jeremiah",
    "Lamentations",
    "Ezekiel",
    "Daniel",
    "Hosea",
    "Joel",
    "Amos",
    "Obadiah",
    "Jonah",
    "Micah",
    "Nahum",
    "Habakkuk",
    "Zephaniah",
    "Haggai",
    "Zechariah",
    "Malachi",
    "Matthew",
    "Mark",
    "Luke",
    "John",
    "Acts",
    "Romans",
    "1 Corinthians",
    "2 Corinthians",
    "Galatians",
    "Ephesians",
    "Philippians",
    "Colossians",
    "1 Thessalonians",
    "2 Thessalonians",
    "1 Timothy",
    "2 Timothy",
    "Titus",
    "Philemon",
    "Hebrews",
    "James",
    "1 Peter",
    "2 Peter",
    "1 John",
    "2 John",
    "3 John",
    "Jude",
    "Revelation",
];

const ABBR: &[(&str, usize)] = &[
    ("gen", 0),
    ("ge", 0),
    ("gn", 0),
    ("ex", 1),
    ("exo", 1),
    ("exod", 1),
    ("lev", 2),
    ("lv", 2),
    ("num", 3),
    ("nm", 3),
    ("deut", 4),
    ("dt", 4),
    ("deu", 4),
    ("josh", 5),
    ("jos", 5),
    ("judg", 6),
    ("jdg", 6),
    ("rth", 7),
    ("ru", 7),
    ("1sam", 8),
    ("1sa", 8),
    ("2sam", 9),
    ("2sa", 9),
    ("1kgs", 10),
    ("1ki", 10),
    ("1kings", 10),
    ("2kgs", 11),
    ("2ki", 11),
    ("2kings", 11),
    ("1chr", 12),
    ("1ch", 12),
    ("1chron", 12),
    ("2chr", 13),
    ("2ch", 13),
    ("2chron", 13),
    ("ezr", 14),
    ("neh", 15),
    ("est", 16),
    ("esth", 16),
    ("jb", 17),
    ("ps", 18),
    ("psa", 18),
    ("psalm", 18),
    ("pss", 18),
    ("prov", 19),
    ("pr", 19),
    ("prv", 19),
    ("eccl", 20),
    ("ecc", 20),
    ("eccles", 20),
    ("qoh", 20),
    ("song", 21),
    ("sos", 21),
    ("songofsongs", 21),
    ("canticles", 21),
    ("isa", 22),
    ("is", 22),
    ("jer", 23),
    ("lam", 24),
    ("ezek", 25),
    ("eze", 25),
    ("dan", 26),
    ("dn", 26),
    ("hos", 27),
    ("joe", 28),
    ("jl", 28),
    ("am", 29),
    ("obad", 30),
    ("ob", 30),
    ("jon", 31),
    ("jnh", 31),
    ("mic", 32),
    ("nah", 33),
    ("hab", 34),
    ("zeph", 35),
    ("zep", 35),
    ("hag", 36),
    ("zech", 37),
    ("zec", 37),
    ("mal", 38),
    ("matt", 39),
    ("mt", 39),
    ("mat", 39),
    ("mk", 40),
    ("mrk", 40),
    ("mar", 40),
    ("lk", 41),
    ("luk", 41),
    ("jn", 42),
    ("jhn", 42),
    ("joh", 42),
    ("act", 43),
    ("rom", 44),
    ("rm", 44),
    ("1cor", 45),
    ("1co", 45),
    ("2cor", 46),
    ("2co", 46),
    ("gal", 47),
    ("eph", 48),
    ("phil", 49),
    ("php", 49),
    ("col", 50),
    ("1thess", 51),
    ("1th", 51),
    ("1thes", 51),
    ("2thess", 52),
    ("2th", 52),
    ("2thes", 52),
    ("1tim", 53),
    ("1ti", 53),
    ("2tim", 54),
    ("2ti", 54),
    ("tit", 55),
    ("philem", 56),
    ("phm", 56),
    ("phlm", 56),
    ("heb", 57),
    ("jas", 58),
    ("jm", 58),
    ("1pet", 59),
    ("1pe", 59),
    ("1pt", 59),
    ("2pet", 60),
    ("2pe", 60),
    ("2pt", 60),
    ("1jn", 61),
    ("1jo", 61),
    ("1joh", 61),
    ("2jn", 62),
    ("2jo", 62),
    ("3jn", 63),
    ("3jo", 63),
    ("jud", 64),
    ("jde", 64),
    ("rev", 65),
    ("re", 65),
    ("rv", 65),
    ("revelations", 65),
];

#[derive(Deserialize)]
struct RawBook {
    chapters: Vec<Vec<String>>,
}
struct Bible {
    books: Vec<Vec<Vec<String>>>,
    keys: HashMap<String, usize>,
}

fn norm(s: &str) -> String {
    let s = s
        .to_lowercase()
        .replace("first ", "1")
        .replace("second ", "2")
        .replace("third ", "3");
    let s = s.replace("iii ", "3").replace("ii ", "2");
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn bible() -> &'static Bible {
    static B: OnceLock<Bible> = OnceLock::new();
    B.get_or_init(|| {
        let raw: Vec<RawBook> =
            serde_json::from_str(RAW.trim_start_matches('\u{feff}')).expect("bad json");
        let mut keys = HashMap::new();
        for (i, n) in NAMES.iter().enumerate() {
            keys.insert(norm(n), i);
        }
        for (a, i) in ABBR {
            keys.insert(a.to_string(), *i);
        }
        Bible {
            books: raw.into_iter().map(|b| b.chapters).collect(),
            keys,
        }
    })
}

fn find_book(s: &str) -> Option<usize> {
    let k = norm(s);
    if k.is_empty() {
        return None;
    }
    let b = bible();
    if let Some(i) = b.keys.get(&k) {
        return Some(*i);
    }
    // unique prefix match on full names
    let m: Vec<usize> = (0..66)
        .filter(|&i| norm(NAMES[i]).starts_with(&k))
        .collect();
    if m.len() == 1 {
        Some(m[0])
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
struct Ref {
    book: usize,
    ch: usize,
    v1: Option<usize>,
    v2: Option<usize>,
}

fn parse_ref(input: &str) -> Option<Ref> {
    let s = input.trim();
    // split book part from trailing chapter[:verse[-verse]]
    let idx = s
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i);
    let (bk, rest) = match idx {
        Some(i) => (&s[..i], s[i..].trim()),
        None => {
            // e.g. "John" alone or "jn3:16"
            let p = s
                .char_indices()
                .skip(1)
                .find(|(_, c)| c.is_ascii_digit())
                .map(|(i, _)| i);
            match p {
                Some(i) => (&s[..i], &s[i..]),
                None => (s, "1"),
            }
        }
    };
    let (bk, rest) = if rest.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        (bk, rest)
    } else {
        (s, "1")
    };
    let book = find_book(bk)?;
    let rest = rest.replace(['–', '—'], "-").replace('.', ":");
    let (ch, vs) = match rest.split_once(':') {
        Some((c, v)) => (c, Some(v)),
        None => (rest.as_str(), None),
    };
    let ch: usize = ch.trim().parse().ok()?;
    let chapters = &bible().books[book];
    if ch == 0 || ch > chapters.len() {
        return None;
    }
    let n = chapters[ch - 1].len();
    let (v1, v2) = match vs {
        None => (None, None),
        Some(v) => {
            let (a, b) = match v.split_once('-') {
                Some((a, b)) => (a, Some(b)),
                None => (v, None),
            };
            let a: usize = a.trim().parse().ok()?;
            let b: usize = match b {
                Some(b) => b.trim().parse().ok()?,
                None => a,
            };
            if a == 0 || a > n || b < a {
                return None;
            }
            (Some(a), Some(b.min(n)))
        }
    };
    Some(Ref { book, ch, v1, v2 })
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn slug(b: usize) -> String {
    NAMES[b].replace(' ', "-")
}
fn ref_label(r: &Ref) -> String {
    match (r.v1, r.v2) {
        (Some(a), Some(b)) if a != b => format!("{} {}:{}-{}", NAMES[r.book], r.ch, a, b),
        (Some(a), _) => format!("{} {}:{}", NAMES[r.book], r.ch, a),
        _ => format!("{} {}", NAMES[r.book], r.ch),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteResponse {
    pub status: u16,
    pub content_type: String,
    pub location: Option<String>,
    pub title: Option<String>,
    pub query: Option<String>,
    pub anchor: Option<String>,
    pub body: String,
}

fn query_params(raw: &str) -> HashMap<String, String> {
    raw.split('&')
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            let key = decode_component(key);
            let value = decode_component(value);
            Some((key, value))
        })
        .collect()
}

fn decode_component(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push(((hi << 4) | lo) as u8);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn split_route(route: &str) -> (&str, &str, Option<String>) {
    let route = route.strip_prefix('#').unwrap_or(route);
    let (path, raw_query) = route.split_once('?').unwrap_or((route, ""));
    let (query, fragment) = raw_query.split_once('#').unwrap_or((raw_query, ""));
    let path = if path.is_empty() { "/" } else { path };
    let anchor = if fragment.is_empty() {
        None
    } else {
        Some(decode_component(fragment))
    };
    (path, query, anchor)
}

fn route_path(path: &str, client: bool) -> String {
    if client {
        format!("#{path}")
    } else {
        path.to_string()
    }
}

fn page_body(q: &str, body: &str, client: bool) -> String {
    let home = esc(&route_path("/", client));
    let library = esc(&route_path("/library", client));
    let today_current = if body.contains("data-today ") {
        " aria-current=\"page\""
    } else {
        ""
    };
    let library_current = if !body.contains("data-atlas-page")
        && (body.contains("data-library")
            || body.contains("data-book-page")
            || body.contains("data-reader ")
            || body.contains("search-page"))
    {
        " aria-current=\"page\""
    } else {
        ""
    };
    let about = esc(&route_path("/about", client));
    let codex = esc(&route_path("/codex", client));
    let codex_current = if body.contains("data-codex-page") {
        " aria-current=\"page\""
    } else {
        ""
    };
    let brand_icon = include_str!("../web/icons/book-open.svg")
        .replace("<svg ", "<svg class=\"brand-mark\" aria-hidden=\"true\" ");
    let menu_icon =
        include_str!("../web/icons/dots-three.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let q = esc(q);
    let header = format!(
        r##"<a class="skip-link" href="#main-content">Skip to content</a><header class="library-header"><div class="wrap"><a class="brand" href="{home}" aria-label="The Bible — Today">{brand_icon}<span>The Bible</span></a><nav class="site-navigation" aria-label="Main navigation"><a href="{home}" data-today-link{today_current}>Today</a><a href="{library}" data-library-return{library_current}>Library</a><a href="{codex}"{codex_current}>Codex</a></nav><a class="header-back" href="{library}" data-library-return>← Library</a><div class="site-tools"><details class="site-search"><summary aria-label="Search"><img src="/icons/magnifying-glass-thin.svg" alt=""></summary><form id="lookup" action="/go" method="get"><input name="q" value="{q}" placeholder="Find a book or passage…" aria-label="Lookup or search"><button>Go</button></form></details><details class="site-menu"><summary aria-label="Site menu">{menu_icon}</summary><nav aria-label="Site tools"><a href="{home}" data-today-link>Today</a><a href="{library}" data-library-return>Library</a><a href="{codex}">Bible Codex</a><a href="{about}">About &amp; help</a><a href="https://github.com/daeshawnballard/rust-bible" target="_blank" rel="noopener noreferrer">Open source on GitHub ↗</a></nav></details></div></div></header>"##
    );
    format!("{header}<main class=\"wrap\" id=\"main-content\" tabindex=\"-1\">{body}</main><footer class=\"wrap muted site-footer\"><span>King James Version · Public domain</span><span class=\"footer-links\"><a href=\"{about}\">About &amp; help</a><a href=\"https://github.com/daeshawnballard/rust-bible\" target=\"_blank\" rel=\"noopener noreferrer\">Open source on GitHub ↗</a></span></footer>")
}

fn document(title: &str, body: &str) -> String {
    let library_script = if body.contains("data-library") {
        "<script type=\"module\" src=\"/library.js\"></script>"
    } else {
        ""
    };
    format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>{CSS}</style><script type=\"module\" src=\"/experience.js\"></script><script type=\"module\" src=\"/journey.js\"></script><script type=\"module\" src=\"/reader-context.js\"></script><script type=\"module\" src=\"/codex.js\"></script>{library_script}</head><body>{body}</body></html>", esc(title))
}

fn html_response(
    status: u16,
    title: &str,
    q: &str,
    content: &str,
    client: bool,
    anchor: Option<String>,
) -> RouteResponse {
    let body = page_body(q, content, client);
    RouteResponse {
        status,
        content_type: "text/html; charset=utf-8".into(),
        location: None,
        title: Some(title.to_string()),
        query: Some(q.to_string()),
        anchor,
        body: if client { body } else { document(title, &body) },
    }
}

fn redirect(location: String) -> RouteResponse {
    RouteResponse {
        status: 302,
        content_type: "text/plain; charset=utf-8".into(),
        location: Some(location),
        title: None,
        query: None,
        anchor: None,
        body: String::new(),
    }
}

fn json_response(status: u16, body: String) -> RouteResponse {
    RouteResponse {
        status,
        content_type: "application/json; charset=utf-8".into(),
        location: None,
        title: None,
        query: None,
        anchor: None,
        body,
    }
}

fn book_content(b: usize, client: bool) -> String {
    library::render_book(b, client)
}

fn chapter_content(
    b: usize,
    ch: usize,
    highlighted: Option<(usize, usize)>,
    client: bool,
) -> String {
    reader::chapter_content(b, ch, highlighted, client)
}

fn not_found(what: &str, client: bool, anchor: Option<String>) -> RouteResponse {
    html_response(
        404,
        "Not found",
        what,
        &format!("<section class=\"recovery-page\"><h1>Passage not found</h1><p>Couldn't find “{}”. Check the book, chapter, and verse, or choose a book from the Library.</p><a href=\"{}\" data-library-return>Browse the Library</a><a href=\"{}\">Return to Today</a></section>", esc(what), esc(&route_path("/library", client)), esc(&route_path("/", client))),
        client,
        anchor,
    )
}

fn parse_highlight(raw: Option<&String>, verse_count: usize) -> Option<(usize, usize)> {
    let raw = raw?;
    let (a, b) = raw.split_once('-').unwrap_or((raw.as_str(), raw.as_str()));
    let a = a.parse::<usize>().ok()?;
    let b = b.parse::<usize>().ok()?;
    if a == 0 || a > verse_count || b < a {
        return None;
    }
    Some((a, b.min(verse_count)))
}

fn reference_url(reference: &Ref, client: bool) -> String {
    let path = format!("/book/{}/{}", slug(reference.book), reference.ch);
    let mut url = route_path(&path, client);
    if let (Some(a), Some(b)) = (reference.v1, reference.v2) {
        url.push_str(&format!("?v={a}-{b}#v{a}"));
    }
    url
}

fn verses(reference: &Ref) -> Vec<(usize, &'static str)> {
    let chapter = &bible().books[reference.book][reference.ch - 1];
    let (a, b) = (
        reference.v1.unwrap_or(1),
        reference.v2.unwrap_or(chapter.len()),
    );
    (a..=b).map(|n| (n, chapter[n - 1].as_str())).collect()
}

fn passage_content(reference: &Ref, url: &str, client: bool) -> String {
    reader::passage_content(reference, url, client)
}

const MAX_RESULTS: usize = 200;

// Optimal-string-alignment distance: insertions, omissions, substitutions,
// and adjacent transpositions each cost one edit.
fn book_edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut rows = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b.len() {
        rows[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            rows[i][j] = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]));
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                rows[i][j] = rows[i][j].min(rows[i - 2][j - 2] + 1);
            }
        }
    }
    rows[a.len()][b.len()]
}

fn book_suggestions(query: &str, client: bool) -> String {
    // Keep a requested chapter/range attached to each suggestion, and validate
    // it through the same exact parser used by ordinary reference lookup.
    let split = query
        .rsplit_once(char::is_whitespace)
        .filter(|(_, tail)| tail.starts_with(|c: char| c.is_ascii_digit()));
    let (name, suffix) = split.unwrap_or_else(|| {
        query
            .char_indices()
            .skip(1)
            .find(|(_, c)| c.is_ascii_digit())
            .map(|(index, _)| (&query[..index], &query[index..]))
            .unwrap_or((query, ""))
    });
    let key = norm(name);
    if key.len() < 3 || key.len() > 32 || find_book(name).is_some() {
        return String::new();
    }
    let mut candidates = Vec::new();
    for (book, canonical) in NAMES.iter().enumerate() {
        let target = norm(canonical);
        let distance = book_edit_distance(&key, &target);
        let longest = key.chars().count().max(target.chars().count());
        let similarity = 1.0 - distance as f64 / longest as f64;
        let shared_start = key.chars().take(2).eq(target.chars().take(2));
        if !(distance <= 2 && similarity >= 0.65
            || key.len() >= 5 && shared_start && distance <= 4 && similarity >= 0.5)
        {
            continue;
        }
        let (label, url) = if suffix.is_empty() {
            (
                canonical.to_string(),
                route_path(&format!("/book/{}", slug(book)), client),
            )
        } else {
            let Some(reference) = parse_ref(&format!("{canonical} {suffix}")) else {
                continue;
            };
            // A corrected range should retain the focused passage view.
            (
                ref_label(&reference),
                route_path(&format!("/go?q={canonical}%20{suffix}"), client),
            )
        };
        candidates.push((similarity, book, label, url));
    }
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    if candidates.is_empty() {
        return String::new();
    }
    let mut html = String::from("<aside class=\"suggestions\" aria-label=\"Suggested books\"><h2>Did you mean…?</h2><div class=\"grid\">");
    for (_, _, label, url) in candidates.into_iter().take(3) {
        html.push_str(&format!("<a href=\"{}\">{}</a>", esc(&url), esc(&label)));
    }
    html.push_str("</div></aside>");
    html
}

fn search(q: &str) -> (Vec<(usize, usize, usize, &'static str)>, usize) {
    let terms: Vec<String> = q
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .collect();
    let mut out = Vec::new();
    let mut total = 0;
    if terms.is_empty() {
        return (out, 0);
    }
    for (b, book) in bible().books.iter().enumerate() {
        for (c, chapter) in book.iter().enumerate() {
            for (v, text) in chapter.iter().enumerate() {
                let lower = text.to_lowercase();
                if terms.iter().all(|word| lower.contains(word.as_str())) {
                    total += 1;
                    if out.len() < MAX_RESULTS {
                        out.push((b, c + 1, v + 1, text.as_str()));
                    }
                }
            }
        }
    }
    (out, total)
}

fn highlight(text: &str, q: &str) -> String {
    let mut lower = String::new();
    let mut original_starts = Vec::new();
    let mut original_ends = Vec::new();
    for (start, character) in text.char_indices() {
        let end = start + character.len_utf8();
        let folded = character.to_lowercase().collect::<String>();
        lower.push_str(&folded);
        original_starts.extend(std::iter::repeat(start).take(folded.len()));
        original_ends.extend(std::iter::repeat(end).take(folded.len()));
    }
    let mut ranges: Vec<(usize, usize)> = q
        .to_lowercase()
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .flat_map(|word| {
            lower.match_indices(word).filter_map(|(start, found)| {
                let end = start + found.len();
                (start < original_starts.len() && end > 0 && end <= original_ends.len())
                    .then_some((original_starts[start], original_ends[end - 1]))
            })
        })
        .collect();
    ranges.sort_unstable();
    let mut output = String::new();
    let mut last = 0;
    for (start, end) in ranges {
        if start < last {
            continue;
        }
        output.push_str(&esc(&text[last..start]));
        output.push_str("<mark>");
        output.push_str(&esc(&text[start..end]));
        output.push_str("</mark>");
        last = end;
    }
    output.push_str(&esc(&text[last..]));
    output
}

fn search_content(q: &str, client: bool) -> String {
    let (results, total) = search(q);
    let mut s = format!(
        "<section class=\"search-page\"><header class=\"search-heading\"><p class=\"page-eyebrow\">Search the Scriptures</p><h1>“{}”</h1><p class=\"muted\">{} verse{} found{} · King James Version</p></header><div class=\"res\">",
        esc(q),
        total,
        if total == 1 { "" } else { "s" },
        if total > MAX_RESULTS {
            format!(" — showing first {MAX_RESULTS}")
        } else {
            String::new()
        }
    );
    s.push_str(&book_suggestions(q, client));
    for (b, ch, verse, text) in results {
        let target = route_path(
            &format!("/book/{}/{ch}?v={verse}-{verse}#v{verse}", slug(b)),
            client,
        );
        s.push_str(&format!("<article class=\"search-result\"><h2><a href=\"{target}\">{} {ch}:{verse}<span aria-hidden=\"true\"> ↗</span></a></h2><p class=\"scripture\">{}</p></article>", NAMES[b], highlight(text, q)));
    }
    s.push_str("</div></section>");
    s
}

fn audio_chapter(q: &HashMap<String, String>) -> RouteResponse {
    let book = q
        .get("book")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(66);
    let chapter = q
        .get("chapter")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    if book >= 66 || chapter == 0 || chapter > bible().books[book].len() {
        return json_response(400, json!({"error":"Choose a valid chapter."}).to_string());
    }
    let next = if chapter < bible().books[book].len() {
        Some(json!({"book":book,"chapter":chapter+1}))
    } else if book < 65 {
        Some(json!({"book":book+1,"chapter":1}))
    } else {
        None
    };
    json_response(200,json!({"book":NAMES[book],"book_index":book,"chapter":chapter,
        "route":format!("/book/{}/{chapter}",slug(book)),
        "verses":bible().books[book][chapter-1].iter().enumerate().map(|(i,t)|json!({"verse":i+1,"text":t})).collect::<Vec<_>>(),
        "next":next}).to_string())
}

fn api_verse(q: &HashMap<String, String>) -> RouteResponse {
    let input = q.get("ref").cloned().unwrap_or_default();
    match parse_ref(&input) {
        Some(reference) => json_response(200, json!({
            "reference": ref_label(&reference), "book": NAMES[reference.book], "book_index":reference.book, "chapter": reference.ch, "route":format!("/book/{}/{}",slug(reference.book),reference.ch),
            "verses": verses(&reference).iter().map(|(verse, text)| json!({"verse": verse, "text": text})).collect::<Vec<_>>(),
            "text": verses(&reference).iter().map(|(_, text)| *text).collect::<Vec<_>>().join(" "),
            "translation": "KJV"
        }).to_string()),
        None => json_response(400, json!({"error": format!("could not parse reference: {input}")}).to_string()),
    }
}

fn api_search(q: &HashMap<String, String>) -> RouteResponse {
    let query = q.get("q").cloned().unwrap_or_default();
    let (results, total) = search(&query);
    json_response(200, json!({"query": query, "total": total, "returned": results.len(), "limit": MAX_RESULTS,
        "results": results.iter().map(|(book, chapter, verse, text)| json!({
            "reference": format!("{} {chapter}:{verse}", NAMES[*book]), "book": NAMES[*book], "chapter": chapter, "verse": verse, "text": text
        })).collect::<Vec<_>>()
    }).to_string())
}

/// Render a URI (native path + query) or a browser hash route through shared Rust logic.
/// In client mode `body` is the app shell fragment for insertion into the static page.
pub fn route(uri: &str, client: bool) -> RouteResponse {
    let (raw_path, raw_query, anchor) = split_route(uri);
    let path = decode_component(raw_path);
    let params = query_params(raw_query);
    match path.as_str() {
        "/api/reader-next" => {
            let location = params
                .get("book")
                .and_then(|b| b.parse::<usize>().ok())
                .zip(
                    params
                        .get("chapter")
                        .and_then(|ch| ch.parse::<usize>().ok()),
                );
            let verse = params
                .get("verse")
                .map_or(Some(1), |v| v.parse::<usize>().ok())
                .unwrap_or(0);
            match location {
                Some((book, chapter))
                    if book < NAMES.len()
                        && chapter > 0
                        && chapter <= bible().books[book].len()
                        && verse > 0
                        && verse <= bible().books[book][chapter - 1].len() =>
                {
                    json_response(
                        200,
                        json!({"html": reader::next_fragment(book, chapter, verse, client)})
                            .to_string(),
                    )
                }
                _ => json_response(400, json!({"error": "Invalid reader chapter."}).to_string()),
            }
        }
        "/" | "/today" => today::page(&params, client),
        "/library" => html_response(
            200,
            "The Bible Library",
            "",
            &library::render(&params, client),
            client,
            anchor,
        ),
        "/go" => {
            let query = params.get("q").cloned().unwrap_or_default();
            let query = query.trim();
            if query.is_empty() {
                return redirect(route_path("/library", client));
            }
            if let Some(book) = find_book(query) {
                return redirect(route_path(&format!("/book/{}", slug(book)), client));
            }
            if let Some(reference) = parse_ref(query) {
                let target = reference_url(&reference, client);
                if reference.v1.is_some() {
                    let content = passage_content(&reference, &target, client);
                    html_response(200, &ref_label(&reference), query, &content, client, anchor)
                } else {
                    redirect(target)
                }
            } else {
                html_response(
                    200,
                    &format!("Search: {query}"),
                    query,
                    &search_content(query, client),
                    client,
                    anchor,
                )
            }
        }
        "/api/verse" => api_verse(&params),
        "/api/search" => api_search(&params),
        "/api/atlas" => atlas::api(&params),
        "/api/reader-context" => reader_context::api(&params, client),
        "/api/codex" => codex::api(&params, client),
        "/codex" => html_response(
            200,
            "Bible Codex",
            "",
            &codex::page(&params, client),
            client,
            anchor,
        ),
        "/api/audio-chapter" => audio_chapter(&params),
        "/about" => about::page(client),
        "/explore" | "/explore/studies" => redirect(reader_context::legacy(&params, client)),
        "/explore/noahs-ark" | "/explore/sower" | "/explore/tabernacle" => {
            match learning::legacy_target(
                path.trim_start_matches("/explore/"),
                params.get("part").map(String::as_str),
                client,
            ) {
                Some(target) => redirect(target),
                None => not_found(&path, client, anchor),
            }
        }
        _ if path.starts_with("/book/") => {
            let rest = &path[6..];
            if let Some((name, chapter)) = rest.split_once('/') {
                let book_name = decode_component(name).replace('-', " ");
                let Some(book) = find_book(&book_name) else {
                    return not_found(&book_name, client, anchor);
                };
                let chapter = match chapter.parse::<usize>() {
                    Ok(chapter) => chapter,
                    Err(_) => {
                        return not_found(&format!("{} {chapter}", NAMES[book]), client, anchor)
                    }
                };
                let chapters = &bible().books[book];
                if chapter == 0 || chapter > chapters.len() {
                    return not_found(&format!("{} {chapter}", NAMES[book]), client, anchor);
                }
                let highlight = parse_highlight(params.get("v"), chapters[chapter - 1].len());
                html_response(
                    200,
                    &format!("{} {chapter}", NAMES[book]),
                    "",
                    &chapter_content(book, chapter, highlight, client),
                    client,
                    anchor,
                )
            } else {
                let book_name = decode_component(rest).replace('-', " ");
                let Some(book) = find_book(&book_name) else {
                    return not_found(&book_name, client, anchor);
                };
                redirect(route_path(&format!("/book/{}/1", slug(book)), client))
            }
        }
        _ => not_found(raw_path, client, anchor),
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_abi {
    use super::route;
    use std::{cell::RefCell, mem, slice, str};

    thread_local! {
        static RESPONSE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    /// Reserve `length` bytes for the JavaScript caller to write a UTF-8 route into.
    /// The matching `dealloc` call must use the same pointer and length.
    #[no_mangle]
    pub extern "C" fn alloc(length: u32) -> u32 {
        if length == 0 {
            return 0;
        }
        let mut bytes = Vec::<u8>::with_capacity(length as usize);
        let pointer = bytes.as_mut_ptr() as usize as u32;
        mem::forget(bytes);
        pointer
    }

    /// Release a buffer previously returned by `alloc` with its original length.
    #[no_mangle]
    pub unsafe extern "C" fn dealloc(pointer: u32, length: u32) {
        if pointer != 0 && length != 0 {
            drop(Vec::from_raw_parts(
                pointer as usize as *mut u8,
                0,
                length as usize,
            ));
        }
    }

    /// Render one URI or hash route. The caller must pass a valid `alloc` buffer.
    /// The response remains available until the next `render` call.
    #[no_mangle]
    pub unsafe extern "C" fn render(pointer: u32, length: u32) {
        let response = if pointer == 0 && length != 0 {
            serde_json::to_vec(&serde_json::json!({
                "status": 400, "content_type": "application/json", "location": null,
                "title": null, "query": null, "anchor": null,
                "body": "route buffer pointer is null"
            }))
            .unwrap_or_default()
        } else if pointer == 0 {
            serde_json::to_vec(&route("", true)).unwrap_or_default()
        } else {
            let input = slice::from_raw_parts(pointer as usize as *const u8, length as usize);
            match str::from_utf8(input) {
                Ok(uri) => serde_json::to_vec(&route(uri, true)).unwrap_or_default(),
                Err(_) => serde_json::to_vec(&serde_json::json!({
                    "status": 400, "content_type": "application/json", "location": null,
                    "title": null, "query": null, "anchor": null,
                    "body": "route must be UTF-8"
                }))
                .unwrap_or_default(),
            }
        };
        RESPONSE.with(|current| *current.borrow_mut() = response);
    }

    /// Return the response's current byte pointer. Copy it before calling `render` again.
    #[no_mangle]
    pub extern "C" fn response_ptr() -> u32 {
        RESPONSE.with(|current| current.borrow().as_ptr() as usize as u32)
    }

    /// Return the response's current UTF-8 byte length.
    #[no_mangle]
    pub extern "C" fn response_len() -> u32 {
        RESPONSE.with(|current| current.borrow().len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::route;

    #[test]
    fn misspelled_books_offer_explicit_corrections() {
        let response = route("#/go?q=neimai", true);
        assert!(response.body.contains("Did you mean…?"));
        assert!(response
            .body
            .contains("href=\"#/book/Nehemiah\">Nehemiah</a>"));
        let transposed = route("/go?q=jhon", false);
        assert!(transposed.body.contains("href=\"/book/John\">John</a>"));
    }

    #[test]
    fn fuzzy_suggestions_keep_valid_reference_and_reject_invalid_chapter() {
        let response = route("#/go?q=neimai%202%3A3", true);
        assert!(response.body.contains("Nehemiah 2:3</a>"));
        assert!(response.body.contains("#/go?q=Nehemiah%202:3"));
        assert!(route("/go?q=neimai2%3A3", false)
            .body
            .contains("Nehemiah 2:3</a>"));
        assert!(!route("/go?q=neimai%20999", false)
            .body
            .contains("Did you mean…?"));
    }

    #[test]
    fn ordinary_keywords_and_nonsense_do_not_get_book_corrections() {
        for query in ["faith", "love", "hope", "zzzznoverse123"] {
            assert!(!route(&format!("/go?q={query}"), true)
                .body
                .contains("Did you mean…?"));
        }
    }

    #[test]
    fn parses_book_alias_and_verse_range() {
        let response = route("/api/verse?ref=1%20Cor%2013%3A4-7", false);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("1 Corinthians 13:4-7"));
        assert!(response.body.contains("\"verse\":4"));
        assert!(response.body.contains("\"verse\":7"));
    }

    #[test]
    fn parses_abbreviated_book_and_single_verse() {
        let response = route("/api/verse?ref=Jn+3%3A16", false);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("John 3:16"));
        assert!(response.body.contains("For God so loved the world"));
    }

    #[test]
    fn search_requires_all_terms_and_returns_highlighted_results() {
        let response = route("/go?q=God+so+loved", false);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("<h1>“God so loved”</h1>"));
        assert!(response.body.contains("<mark>God</mark>"));
        assert!(response.body.contains("John 3:16"));
    }

    #[test]
    fn invalid_reference_is_a_bad_request() {
        let response = route("/api/verse?ref=John+999%3A1", false);
        assert_eq!(response.status, 400);
        assert!(response.body.contains("could not parse reference"));
    }

    #[test]
    fn browser_routes_render_hash_links_and_verse_highlights() {
        let response = route("#/book/John/3?v=16-17#v16", true);
        assert_eq!(response.status, 200);
        assert_eq!(response.anchor.as_deref(), Some("v16"));
        assert!(response.body.contains("class=\"v hl\" id=\"v16\""));
        assert!(response.body.contains("<span>John</span>"));
        assert!(response.body.contains("class=\"reader-chapter-menu\""));
    }

    #[test]
    fn unknown_book_returns_not_found() {
        let response = route("/book/Not-A-Book", true);
        assert_eq!(response.status, 404);
        assert!(response.body.contains("Passage not found"));
    }

    #[test]
    fn all_66_legacy_book_links_enter_the_reader_without_an_overview() {
        for (book, name) in super::NAMES.iter().enumerate() {
            for client in [false, true] {
                let path = format!("/book/{}", super::slug(book));
                let response = route(&path, client);
                assert_eq!(response.status, 302, "{name}");
                assert_eq!(
                    response.location,
                    Some(super::route_path(&format!("{path}/1"), client))
                );
                let reader = route(&format!("{path}/1"), client);
                assert_eq!(reader.status, 200, "{name}");
                assert_eq!(
                    reader
                        .body
                        .matches("data-read-aloud data-reader-audio")
                        .count(),
                    1
                );
                assert!(reader.body.contains("data-reader-chapter"));
                assert!(!reader.body.contains("data-book-page"));
            }
        }
    }

    #[test]
    fn highlighting_handles_unicode_case_expansion_and_escapes_markup() {
        assert_eq!(super::highlight("İstanbul", "i"), "<mark>İ</mark>stanbul");
        assert_eq!(super::highlight("<Tag>", "tag"), "&lt;<mark>Tag</mark>&gt;");
    }
}

#[cfg(test)]
mod connected_reader_tests {
    use super::*;
    #[test]
    fn audio_api_uses_exact_corpus_and_handles_chapter_and_book_boundaries() {
        for (book, chapter) in [(0, 2), (18, 23), (38, 4), (65, 22)] {
            let response = route(
                &format!("/api/audio-chapter?book={book}&chapter={chapter}"),
                true,
            );
            assert_eq!(response.status, 200);
            let value: serde_json::Value = serde_json::from_str(&response.body).unwrap();
            assert_eq!(
                value["verses"][0]["text"],
                bible().books[book][chapter - 1][0]
            );
            assert_eq!(
                value["verses"].as_array().unwrap().len(),
                bible().books[book][chapter - 1].len()
            );
            if book == 65 {
                assert!(value["next"].is_null());
            }
        }
        let cross: serde_json::Value =
            serde_json::from_str(&route("/api/audio-chapter?book=38&chapter=4", true).body)
                .unwrap();
        assert_eq!(cross["next"]["book"], 39);
        assert_eq!(cross["next"]["chapter"], 1);
    }
    #[test]
    fn new_api_inputs_are_validated_and_codex_is_one_reference_surface() {
        for route_uri in [
            "/api/audio-chapter?book=66&chapter=1",
            "/api/audio-chapter?book=0&chapter=0",
            "/api/audio-chapter?book=0&chapter=51",
        ] {
            assert_eq!(route(route_uri, true).status, 400);
        }
        let codex = route("/codex", true);
        assert_eq!(codex.status, 200);
        assert!(codex.body.contains("data-codex-page"));
        assert!(!codex.body.contains("data-read-aloud"));
    }
}
