//! Book-section context for every chapter, below the curated verse-level scenes.
//! Chapter-stage links live in Context; broad book artwork has no image hotspots.
use super::{bible, esc, route_path, slug, NAMES};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Catalog {
    schema_version: usize,
    books: Vec<Book>,
}
#[derive(Deserialize)]
struct Book {
    book_index: usize,
    name: String,
    kind: String,
    sections: Vec<Section>,
}
#[derive(Deserialize)]
struct Section {
    id: String,
    first_chapter: usize,
    last_chapter: usize,
    anchor_chapter: usize,
    anchor_verse: usize,
    label: String,
    world: String,
    note: String,
    basis: String,
}

pub(super) const WORLDS: &[&str] = &[
    "origins-patriarchs",
    "egypt",
    "sinai-wilderness",
    "canaan-highlands",
    "jerusalem-temple-return",
    "babylon-exile",
    "persian-court",
    "wisdom-poetry",
    "prophetic-symbolic",
    "galilee-judea",
    "early-church-house",
    "mediterranean-cities",
    "apocalyptic-vision",
];

fn catalog() -> &'static Vec<Book> {
    static CATALOG: OnceLock<Vec<Book>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut ot: Catalog = serde_json::from_str(include_str!("../data/journey/ot.json"))
            .expect("valid Old Testament Journey catalog");
        let nt: Catalog = serde_json::from_str(include_str!("../data/journey/nt.json"))
            .expect("valid New Testament Journey catalog");
        assert_eq!(ot.schema_version, 1);
        assert_eq!(nt.schema_version, 1);
        ot.books.extend(nt.books);
        ot.books.sort_by_key(|book| book.book_index);
        ot.books
    })
}

fn selected(book: usize, chapter: usize) -> Option<(&'static Book, &'static Section)> {
    let profile = catalog()
        .get(book)
        .filter(|entry| entry.book_index == book)?;
    profile
        .sections
        .iter()
        .find(|section| (section.first_chapter..=section.last_chapter).contains(&chapter))
        .map(|section| (profile, section))
}

pub(super) fn available(book: usize, chapter: usize) -> bool {
    selected(book, chapter).is_some()
}

fn link(book: usize, section: &Section, client: bool) -> String {
    route_path(
        &format!(
            "/book/{}/{}?resume={}&context=journey#v{}",
            slug(book),
            section.anchor_chapter,
            section.anchor_verse,
            section.anchor_verse
        ),
        client,
    )
}

pub(super) fn render(book: usize, chapter: usize, verse: usize, client: bool) -> Option<Value> {
    let (profile, current) = selected(book, chapter)?;
    let kind = match profile.kind.as_str() {
        "poetry" => "Literary context",
        "epistle" => "Letter context",
        "prophetic" => "Prophetic context",
        "vision" => "Vision context",
        _ => "Book context",
    };
    let caption = "Literary illustration of broad book context · No named reading stage depicted";
    let mut html = format!(
        r#"<section class="story-journey book-journey" data-story-stage="{}"><p class="reader-context-kind">{} · {}</p><figure class="story-journey-art" data-hotspot-layout="none"><img src="/art/journey/{}.webp" width="1536" height="1024" alt="Literary context illustration for {}; no named reading stage is depicted">"#,
        esc(&current.id),
        kind,
        esc(&profile.name),
        esc(&current.world),
        esc(&profile.name)
    );
    html.push_str(&format!(
        r#"</figure><p class="story-journey-caption">{caption}</p><div class="reader-context-copy"><p class="reader-context-kind">{kind} · Explanation</p><h3>{}</h3><p>{}</p><p class="book-journey-range">Context for {} {chapter}:{verse} · Section: {}</p><p>The illustration gives broad literary context for the book; it does not depict the named reading stage or the literal setting of every verse.</p><a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}">Read {} {}:{} →</a></div><details class="reader-context-sources"><summary>Reading stages &amp; sources</summary><p>Source: {}. The King James text is available in the Scripture tab. These notes and the artistic setting are editorial explanations.</p><nav aria-label="{} reading stages">"#,
        esc(&current.label), esc(&current.note), esc(NAMES[book]), esc(&current.basis),
        esc(&link(book, current, client)), esc(NAMES[book]), current.anchor_chapter,
        current.anchor_verse, esc(NAMES[book]), current.anchor_chapter,
        current.anchor_verse, esc(&current.basis), esc(NAMES[book])
    ));
    for section in &profile.sections {
        let active = if section.id == current.id {
            " aria-current=\"step\""
        } else {
            ""
        };
        html.push_str(&format!(
            r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}"{active}>{} · {}</a>"#,
            esc(&link(book, section, client)), esc(NAMES[book]), section.anchor_chapter,
            section.anchor_verse, esc(&section.label), esc(&section.basis)
        ));
    }
    html.push_str("</nav></details></section>");
    Some(
        json!({"kind":"book-context","title":current.label,"html":html,
        "scene":format!("book-{}",book),"world":current.world,"active":current.id,
        "stage_kind":profile.kind,"hotspot_layout":"none",
        "source_range":{"first_chapter":current.first_chapter,"last_chapter":current.last_chapter},
        "basis":current.basis}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn all_sixty_six_books_cover_every_chapter_without_holes_or_overlap() {
        assert_eq!(catalog().len(), 66);
        let mut ids = HashSet::new();
        for (index, book) in catalog().iter().enumerate() {
            assert_eq!(book.book_index, index);
            assert_eq!(book.name, NAMES[index]);
            assert!(matches!(
                book.kind.as_str(),
                "narrative" | "poetry" | "prophetic" | "epistle" | "vision"
            ));
            let mut next = 1;
            for section in &book.sections {
                assert!(ids.insert(&section.id), "duplicate {}", section.id);
                assert_eq!(section.first_chapter, next, "gap/overlap in {}", book.name);
                assert!(section.last_chapter >= section.first_chapter);
                assert!(section.last_chapter <= bible().books[index].len());
                assert!((section.first_chapter..=section.last_chapter)
                    .contains(&section.anchor_chapter));
                assert!((1..=bible().books[index][section.anchor_chapter - 1].len())
                    .contains(&section.anchor_verse));
                assert!(WORLDS.contains(&section.world.as_str()));
                assert!(
                    !section.note.trim().is_empty()
                        && !section.basis.trim().is_empty()
                        && !section.label.trim().is_empty()
                );
                next = section.last_chapter + 1;
            }
            assert_eq!(
                next,
                bible().books[index].len() + 1,
                "incomplete {}",
                book.name
            );
            for (ch, verses) in bible().books[index].iter().enumerate() {
                for verse in 1..=verses.len() {
                    let (_, section) = selected(index, ch + 1).unwrap();
                    assert!((section.first_chapter..=section.last_chapter).contains(&(ch + 1)));
                    assert!(verse > 0);
                }
                let state = render(index, ch + 1, 1, true).unwrap();
                let html = state["html"].as_str().unwrap();
                assert_eq!(state["hotspot_layout"], "none");
                assert!(html.contains("data-hotspot-layout=\"none\""));
                assert!(!html.contains("story-journey-pin"));
                let (_, current) = selected(index, ch + 1).unwrap();
                assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
                let figure_end = html.find("</figure>").unwrap();
                let figure = &html[..figure_end];
                assert!(!figure.contains("<a "));
                assert!(!figure.contains(&current.label));
                let caption_start = html.find("<p class=\"story-journey-caption\">").unwrap();
                let caption_end = caption_start + html[caption_start..].find("</p>").unwrap();
                assert!(!html[caption_start..caption_end].contains(&current.label));
                let details_start = html
                    .find("<details class=\"reader-context-sources\">")
                    .unwrap();
                let details_end = details_start + html[details_start..].find("</details>").unwrap();
                let details = &html[details_start..details_end];
                assert!(details.contains("<summary>Reading stages &amp; sources</summary>"));
                let nav_start = details.find("<nav aria-label=\"").unwrap();
                let nav_end = nav_start + details[nav_start..].find("</nav>").unwrap();
                let nav = &details[nav_start..nav_end];
                assert!(nav.contains(&format!("aria-label=\"{} reading stages\"", NAMES[index])));
                for section in &book.sections {
                    let active = if section.id == current.id {
                        " aria-current=\"step\""
                    } else {
                        ""
                    };
                    let expected = format!(
                        r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}"{active}>{} · {}</a>"#,
                        esc(&link(index, section, true)),
                        esc(NAMES[index]),
                        section.anchor_chapter,
                        section.anchor_verse,
                        esc(&section.label),
                        esc(&section.basis)
                    );
                    assert!(
                        nav.contains(&expected),
                        "missing stage navigation for {}: {}",
                        book.name,
                        section.id
                    );
                }
                assert!(
                    !html.contains("<svg")
                        && !html.contains("data-audio")
                        && !html.contains("class=\"scripture")
                );
            }
        }
    }
}
