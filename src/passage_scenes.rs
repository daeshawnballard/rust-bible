//! Passage-specific storyboards backed by the authored scene corpus.
//! Pins identify depicted subjects; text and reading remain in the Context panel.
use super::{esc, route_path, slug, NAMES};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct Catalog {
    schema_version: usize,
    scenes: Vec<Scene>,
}

#[derive(Debug, Deserialize)]
struct Scene {
    id: String,
    book: usize,
    title: String,
    coverage: String,
    art: Art,
    caption: String,
    ranges: Vec<VerseRange>,
    intro: Intro,
    hotspots: Vec<Hotspot>,
}

#[derive(Debug, Deserialize)]
struct Art {
    path: String,
    width: usize,
    height: usize,
    alt: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct VerseRange {
    chapter: usize,
    first: usize,
    last: usize,
}

#[derive(Debug, Deserialize)]
struct Intro {
    title: String,
    note: String,
}

#[derive(Debug, Deserialize)]
struct Hotspot {
    id: String,
    title: String,
    x: f64,
    y: f64,
    active: Vec<VerseRange>,
    anchor: VerseRange,
    sources: Vec<VerseRange>,
    note: String,
    detail: Option<Art>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let catalog: Catalog =
            serde_json::from_str(include_str!("../data/journey/passage-scenes.json"))
                .expect("valid passage scene catalog");
        assert_eq!(catalog.schema_version, 1);
        catalog
    })
}

fn scenes() -> &'static [Scene] {
    &catalog().scenes
}

fn contains(range: &VerseRange, chapter: usize, verse: usize) -> bool {
    range.chapter == chapter && (range.first..=range.last).contains(&verse)
}

fn in_ranges(ranges: &[VerseRange], chapter: usize, verse: usize) -> bool {
    ranges.iter().any(|range| contains(range, chapter, verse))
}

fn scene_for_chapter(book: usize, chapter: usize) -> Option<&'static Scene> {
    scenes()
        .iter()
        .find(|s| s.book == book && s.ranges.iter().any(|r| r.chapter == chapter))
}

fn scene_for_verse(book: usize, chapter: usize, verse: usize) -> Option<&'static Scene> {
    scenes()
        .iter()
        .find(|s| s.book == book && in_ranges(&s.ranges, chapter, verse))
}

fn active_hotspot(scene: &Scene, chapter: usize, verse: usize) -> Option<&Hotspot> {
    scene
        .hotspots
        .iter()
        .find(|h| in_ranges(&h.active, chapter, verse))
}

fn selected_hotspot<'a>(
    scene: &'a Scene,
    chapter: usize,
    verse: usize,
    subject: Option<&str>,
) -> Option<&'a Hotspot> {
    subject
        .and_then(|id| {
            scene
                .hotspots
                .iter()
                .find(|h| h.id == id && in_ranges(&h.active, chapter, verse))
        })
        .or_else(|| active_hotspot(scene, chapter, verse))
}

fn ref_label(book: usize, range: &VerseRange) -> String {
    let end = if range.first == range.last {
        String::new()
    } else {
        format!("–{}", range.last)
    };
    format!("{} {}:{}{}", NAMES[book], range.chapter, range.first, end)
}

fn href(book: usize, range: &VerseRange, client: bool) -> String {
    route_path(
        &format!(
            "/book/{}/{}?resume={}&context=journey#v{}",
            slug(book),
            range.chapter,
            range.first,
            range.first
        ),
        client,
    )
}

fn read_link(book: usize, range: &VerseRange, client: bool, label: &str) -> String {
    let href = route_path(
        &format!(
            "/book/{}/{}?resume={}&context=journey#v{}",
            slug(book),
            range.chapter,
            range.first,
            range.first
        ),
        client,
    );
    format!(
        r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}">{}</a>"#,
        esc(&href),
        esc(&slug(book)),
        range.chapter,
        range.first,
        esc(label)
    )
}

fn range_links(book: usize, ranges: &[VerseRange], client: bool) -> String {
    ranges
        .iter()
        .map(|r| read_link(book, r, client, &format!("Read {}", ref_label(book, r))))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn detail_image(art: Option<&Art>) -> String {
    art.map(|a| format!(
        r#"<figure class="reader-context-detail-art"><img class="passage-subject-art" src="{}" width="{}" height="{}" alt="{}" loading="lazy"></figure>"#,
        esc(&a.path), a.width, a.height, esc(&a.alt)
    )).unwrap_or_default()
}

fn render_html(scene: &Scene, active: Option<&Hotspot>, client: bool) -> String {
    let book_slug = slug(scene.book);
    let active_id = active.map(|hotspot| hotspot.id.as_str());
    let stage = active_id.unwrap_or("intro");
    let diagram = if scene.art.path.ends_with(".svg") {
        " data-passage-diagram"
    } else {
        ""
    };
    let mut html = format!(
        r#"<section class="story-journey" data-story-scene="{}" data-story-stage="{}"><p class="reader-context-kind">Illustrated passage · {}</p><figure class="story-journey-art" data-passage-scene{}><img src="{}" width="{}" height="{}" alt="{}">"#,
        esc(&scene.id),
        esc(stage),
        esc(&scene.coverage),
        diagram,
        esc(&scene.art.path),
        scene.art.width,
        scene.art.height,
        esc(&scene.art.alt)
    );
    for hotspot in &scene.hotspots {
        let current = if Some(hotspot.id.as_str()) == active_id {
            " aria-current=\"step\""
        } else {
            ""
        };
        let reference = ref_label(scene.book, &hotspot.anchor);
        let label = format!("Inspect {} · {}", hotspot.title, reference);
        html.push_str(&format!(
            r#"<a class="story-journey-pin" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}" data-context-subject="{}" style="left:{}%;top:{}%" aria-label="{}"{}><span aria-hidden="true">●</span><span>{}</span></a>"#,
            esc(&href(scene.book, &hotspot.anchor, client)),
            esc(&book_slug),
            hotspot.anchor.chapter,
            hotspot.anchor.first,
            esc(&hotspot.id),
            hotspot.x,
            hotspot.y,
            esc(&label),
            current,
            esc(&hotspot.title)
        ));
    }
    html.push_str(&format!(
        "</figure><p class=\"story-journey-caption\">{}</p>",
        esc(&scene.caption)
    ));

    let (title, note, detail, sources) = if let Some(hotspot) = active {
        (
            hotspot.title.as_str(),
            hotspot.note.as_str(),
            hotspot.detail.as_ref(),
            hotspot.sources.as_slice(),
        )
    } else {
        (
            scene.intro.title.as_str(),
            scene.intro.note.as_str(),
            None,
            scene.ranges.as_slice(),
        )
    };
    html.push_str(
        "<div class=\"reader-context-copy passage-scene-copy\"><p class=\"reader-context-kind\">Explanation</p>",
    );
    html.push_str(&detail_image(detail));
    html.push_str(&format!("<h3>{}</h3><p>{}</p>", esc(title), esc(note)));
    html.push_str(&format!(
        "<p class=\"story-journey-sources\">Sources: {}</p>",
        range_links(scene.book, sources, client)
    ));
    if let Some(hotspot) = active {
        let label = format!("Read {} →", ref_label(scene.book, &hotspot.anchor));
        html.push_str(&read_link(scene.book, &hotspot.anchor, client, &label));
    }
    html.push_str("</div><details class=\"reader-context-sources\"><summary>Scene sources &amp; subjects</summary>");
    html.push_str(&format!(
        "<p>Scene coverage: {}</p><nav aria-label=\"{} subject references\">",
        range_links(scene.book, &scene.ranges, client),
        esc(NAMES[scene.book])
    ));
    for hotspot in &scene.hotspots {
        let label = format!(
            "Inspect {} · {}",
            hotspot.title,
            ref_label(scene.book, &hotspot.anchor)
        );
        html.push_str(
            &read_link(scene.book, &hotspot.anchor, client, &label).replacen(
                "data-context-read",
                &format!(
                    "data-context-read data-context-inspect data-context-subject=\"{}\"",
                    esc(&hotspot.id)
                ),
                1,
            ),
        );
    }
    html.push_str("</nav></details></section>");
    html
}

/// Render only when this exact verse belongs to an authored passage scene.
pub(super) fn render(book: usize, chapter: usize, verse: usize, client: bool) -> Option<Value> {
    render_selected(book, chapter, verse, None, client)
}

/// Inspect a named subject without changing the passage or relying on a unique verse anchor.
pub(super) fn render_selected(
    book: usize,
    chapter: usize,
    verse: usize,
    subject: Option<&str>,
    client: bool,
) -> Option<Value> {
    let scene = scene_for_verse(book, chapter, verse)?;
    let active = selected_hotspot(scene, chapter, verse, subject);
    Some(json!({
        "kind": "story",
        "id": scene.id,
        "scene": scene.id,
        "title": scene.title,
        "active": active.map(|hotspot| hotspot.id.as_str()),
        "html": render_html(scene, active, client)
    }))
}

/// Whether any authored scene range touches this chapter.
pub(super) fn available(book: usize, chapter: usize) -> bool {
    scene_for_chapter(book, chapter).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bible;
    use std::{collections::HashSet, path::Path};

    fn valid_range(book: usize, r: &VerseRange) {
        assert!(r.chapter > 0 && r.first > 0 && r.first <= r.last);
        assert!(r.chapter <= bible().books[book].len());
        assert!(r.last <= bible().books[book][r.chapter - 1].len());
    }

    fn valid_art(a: &Art) {
        assert!(
            a.path.starts_with("/art/journey/")
                && (a.path.ends_with(".webp") || a.path.ends_with(".svg"))
        );
        assert!(a.width > 0 && a.height > 0 && !a.alt.trim().is_empty());
        assert!(Path::new("web")
            .join(a.path.trim_start_matches('/'))
            .is_file());
    }

    #[test]
    fn corpus_contract_and_unassigned_verses_are_valid() {
        assert_eq!(catalog().schema_version, 1);
        let (mut scene_ids, mut hotspot_ids, mut covered) =
            (HashSet::new(), HashSet::new(), HashSet::new());
        let mut covered_books = HashSet::new();
        for scene in scenes() {
            covered_books.insert(scene.book);
            assert!(scene_ids.insert(scene.id.as_str()));
            assert!(scene.book < NAMES.len());
            assert!(!scene.title.trim().is_empty() && !scene.coverage.trim().is_empty());
            assert!(
                !scene.caption.trim().is_empty()
                    && !scene.intro.title.trim().is_empty()
                    && !scene.intro.note.trim().is_empty()
            );
            valid_art(&scene.art);
            for r in &scene.ranges {
                valid_range(scene.book, r);
                for v in r.first..=r.last {
                    assert!(covered.insert((scene.book, r.chapter, v)));
                }
            }
            for h in &scene.hotspots {
                assert!(hotspot_ids.insert(h.id.as_str()));
                assert!(!h.title.trim().is_empty() && !h.note.trim().is_empty());
                assert!((0.0..=100.0).contains(&h.x) && (0.0..=100.0).contains(&h.y));
                assert!(!h.active.is_empty() && !h.sources.is_empty());
                for r in &h.active {
                    valid_range(scene.book, r);
                    for v in r.first..=r.last {
                        assert!(in_ranges(&scene.ranges, r.chapter, v));
                    }
                }
                valid_range(scene.book, &h.anchor);
                assert!(h
                    .active
                    .iter()
                    .any(|r| contains(r, h.anchor.chapter, h.anchor.first)
                        && contains(r, h.anchor.chapter, h.anchor.last)));
                let inspected = render_selected(
                    scene.book,
                    h.anchor.chapter,
                    h.anchor.first,
                    Some(&h.id),
                    true,
                )
                .unwrap();
                assert_eq!(inspected["active"], h.id);
                assert!(inspected["html"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("data-context-subject=\"{}\"", esc(&h.id))));
                for r in &h.sources {
                    valid_range(scene.book, r);
                }
                if let Some(a) = &h.detail {
                    valid_art(a);
                }
            }
        }
        assert_eq!(covered_books, (0..NAMES.len()).collect());
        for (book, chapters) in bible().books.iter().enumerate() {
            for (i, verses) in chapters.iter().enumerate() {
                let chapter = i + 1;
                assert_eq!(
                    available(book, chapter),
                    covered.iter().any(|(b, c, _)| *b == book && *c == chapter)
                );
                for verse in 1..=verses.len() {
                    if !covered.contains(&(book, chapter, verse)) {
                        assert!(render(book, chapter, verse, true).is_none());
                    }
                }
            }
        }
    }

    #[test]
    fn exodus_signs_follow_announcements_and_reported_outcomes_separately() {
        let book = NAMES.iter().position(|n| *n == "Exodus").unwrap();
        for (chapter, verse, expected) in [
            (7, 9, None),
            (7, 10, Some("exodus-signs-rods-aaron-down")),
            (7, 11, Some("exodus-signs-rods-magicians-cast")),
            (7, 12, Some("exodus-signs-rods-swallow")),
            (7, 13, None),
            (7, 19, None),
            (7, 20, Some("exodus-signs-river-struck")),
            (7, 21, Some("exodus-signs-river-undrinkable")),
            (8, 2, None),
            (8, 3, Some("exodus-signs-frogs-trough")),
            (8, 4, None),
            (8, 5, None),
            (8, 6, Some("exodus-signs-frogs-land")),
            (8, 16, None),
            (8, 17, Some("exodus-signs-lice-dust")),
            (8, 18, Some("exodus-signs-lice-magicians-cannot")),
            (8, 19, Some("exodus-signs-lice-gods-finger")),
            (8, 20, None),
            (8, 21, None),
            (8, 22, Some("exodus-signs-flies-goshen")),
            (8, 23, Some("exodus-signs-flies-goshen")),
            (8, 24, Some("exodus-signs-flies-houses")),
        ] {
            let scene = scene_for_verse(book, chapter, verse).unwrap();
            assert_eq!(
                active_hotspot(scene, chapter, verse).map(|h| h.id.as_str()),
                expected,
                "wrong subject at Exodus {chapter}:{verse}"
            );
        }
        for (chapter, verse) in [(7, 7), (7, 14), (7, 18), (7, 22), (8, 7), (8, 15), (8, 25)] {
            assert!(scene_for_verse(book, chapter, verse).is_none());
        }
    }

    #[test]
    fn genesis_subjects_and_intros_match_the_authored_ranges() {
        let b = NAMES.iter().position(|n| *n == "Genesis").unwrap();
        for (ch, v, scene_id, expected) in [
            (18, 5, "abraham-sarah-visitors", Some("abraham-welcome")),
            (18, 6, "abraham-sarah-visitors", Some("abraham-meal")),
            (18, 8, "abraham-sarah-visitors", Some("abraham-meal")),
            (18, 9, "abraham-sarah-visitors", Some("sarah-promise")),
            (18, 15, "abraham-sarah-visitors", Some("sarah-promise")),
            (18, 1, "abraham-sarah-visitors", None),
            (38, 18, "judah-pledge-objects", Some("judah-pledge")),
            (38, 25, "judah-tamar-recognition", Some("tamar-evidence")),
            (
                38,
                26,
                "judah-tamar-recognition",
                Some("judah-acknowledges"),
            ),
        ] {
            let scene = scene_for_verse(b, ch, v).unwrap();
            let active = active_hotspot(scene, ch, v);
            assert_eq!(scene.id, scene_id);
            assert_eq!(active.map(|h| h.id.as_str()), expected);
            let html = render(b, ch, v, true).unwrap()["html"]
                .as_str()
                .unwrap()
                .to_owned();
            if let Some(h) = active {
                assert!(html.contains(&format!("style=\"left:{}%;top:{}%\"", h.x, h.y)));
                assert!(html.contains(&esc(&format!(
                    "Inspect {} · {}",
                    h.title,
                    ref_label(b, &h.anchor)
                ))));
                assert!(html.contains(&format!(
                    "data-context-chapter=\"{}\" data-context-verse=\"{}\"",
                    h.anchor.chapter, h.anchor.first
                )));
            }
        }
        for (chapter, verse) in [(18, 16), (38, 24)] {
            assert!(scene_for_verse(b, chapter, verse).is_none());
            assert!(render(b, chapter, verse, true).is_none());
        }
    }

    #[test]
    fn a_shared_verse_keeps_each_subject_independently_inspectable() {
        let range = VerseRange {
            chapter: 5,
            first: 8,
            last: 8,
        };
        let hotspot = |id: &str| Hotspot {
            id: id.into(),
            title: id.into(),
            x: 30.0,
            y: 40.0,
            active: vec![range],
            anchor: range,
            sources: vec![range],
            note: "A named image in the verse".into(),
            detail: None,
        };
        let scene = Scene {
            id: "shared-verse-test".into(),
            book: 51,
            title: "Armor imagery".into(),
            coverage: "1 Thessalonians 5:8".into(),
            art: Art {
                path: String::new(),
                width: 1536,
                height: 1024,
                alt: String::new(),
            },
            caption: String::new(),
            ranges: vec![range],
            intro: Intro {
                title: String::new(),
                note: String::new(),
            },
            hotspots: vec![hotspot("breastplate"), hotspot("helmet")],
        };
        assert_eq!(
            selected_hotspot(&scene, 5, 8, None).unwrap().id,
            "breastplate"
        );
        assert_eq!(
            selected_hotspot(&scene, 5, 8, Some("helmet")).unwrap().id,
            "helmet"
        );
        assert_eq!(
            selected_hotspot(&scene, 5, 8, Some("unknown")).unwrap().id,
            "breastplate"
        );
        assert!(selected_hotspot(&scene, 5, 9, Some("helmet")).is_none());
    }
}
