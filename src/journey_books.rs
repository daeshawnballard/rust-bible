//! Book-section context for every chapter, below the curated verse-level scenes.
//! Pin positions represent reading stages, never coordinates or surveyed routes.
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
    let (kind, caption) = match profile.kind.as_str() {
        "poetry" => (
            "Literary context",
            "Literary illustration · Reading stages, not a travel route",
        ),
        "epistle" => (
            "Letter context",
            "Illustrated letter context · Reading stages, not a travel route",
        ),
        "prophetic" => (
            "Prophetic context",
            "Illustrated prophetic context · Symbols and settings are interpretive",
        ),
        "vision" => (
            "Vision context",
            "Symbolic illustration · Vision imagery is not mapped geography",
        ),
        _ => (
            "Book context",
            "Illustrated book context · Reading stages, not a mapped route",
        ),
    };
    let mut html = format!(
        r#"<section class="story-journey book-journey" data-story-stage="{}"><p class="reader-context-kind">{} · {}</p><figure class="story-journey-art" data-hotspot-layout="reading-stages"><img src="/art/journey/{}.webp" width="1536" height="1024" alt="{}; an artistic contextual illustration, not a reconstruction of this exact verse">"#,
        esc(&current.id),
        kind,
        esc(&profile.name),
        esc(&current.world),
        esc(&current.label)
    );
    // Pins form a reading-stage sequence. They deliberately do not claim to mark
    // the real positions of people or objects in this broad contextual artwork.
    let count = profile.sections.len();
    for (index, section) in profile.sections.iter().enumerate() {
        let x = if count == 1 {
            30.0
        } else {
            10.0 + index as f64 * 54.0 / (count - 1) as f64
        };
        let y = if index % 2 == 0 { 57 } else { 72 };
        let active = if section.id == current.id {
            " aria-current=\"step\""
        } else {
            ""
        };
        html.push_str(&format!(
            r#"<a class="story-journey-pin" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}" style="left:{x:.2}%;top:{y}%" aria-label="Read {} {}:{}: {}"{active}><span aria-hidden="true">●</span><span>{}</span></a>"#,
            esc(&link(book, section, client)), esc(NAMES[book]), section.anchor_chapter,
            section.anchor_verse, esc(NAMES[book]), section.anchor_chapter,
            section.anchor_verse, esc(&section.label), esc(&section.label)
        ));
    }
    html.push_str(&format!(
        r#"</figure><p class="story-journey-caption">{caption}</p><div class="reader-context-copy"><p class="reader-context-kind">{kind} · Explanation</p><h3>{}</h3><p>{}</p><p class="book-journey-range">Reading {} {chapter}:{verse} · Section: {}</p><p>This scene gives context for a section of the book. It does not show the literal location of every verse. The markers connect reading stages; their positions are interface layout.</p><a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}">Read {} {}:{} →</a></div><details class="reader-context-sources"><summary>Reading stages &amp; sources</summary><p>Source: {}. The King James text is available in the Scripture tab. These notes and the artistic setting are editorial explanations.</p><nav aria-label="{} reading stages">"#,
        esc(&current.label), esc(&current.note), esc(NAMES[book]), esc(&current.basis),
        esc(&link(book, current, client)), esc(NAMES[book]), current.anchor_chapter,
        current.anchor_verse, esc(NAMES[book]), current.anchor_chapter,
        current.anchor_verse, esc(&current.basis), esc(NAMES[book])
    ));
    for section in &profile.sections {
        html.push_str(&format!(
            r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}">{} · {}</a>"#,
            esc(&link(book, section, client)), esc(NAMES[book]), section.anchor_chapter,
            section.anchor_verse, esc(&section.label), esc(&section.basis)
        ));
    }
    html.push_str("</nav></details></section>");
    Some(
        json!({"kind":"book-context","title":current.label,"html":html,
        "scene":format!("book-{}",book),"world":current.world,"active":current.id,
        "stage_kind":profile.kind,"hotspot_layout":"reading-stages",
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
                assert!(html.contains("data-hotspot-layout=\"reading-stages\""));
                assert!(html.contains("aria-current=\"step\""));
                assert!(
                    !html.contains("<svg")
                        && !html.contains("data-audio")
                        && !html.contains("class=\"scripture")
                );
            }
        }
    }
}
