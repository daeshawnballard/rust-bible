//! Context belongs to the passage. Editorial scenery never enters Scripture/audio.
use super::{bible, esc, json_response, learning, route_path, slug, RouteResponse, NAMES};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) fn controls(book: usize, chapter: usize) -> String {
    let study = learning::relevant_scene(book, chapter).is_some();
    let journey_state = if illustrated_journey(book, chapter) {
        " class=\"has-journey\" title=\"Illustrated Journey available in this chapter\""
    } else {
        " title=\"View this passage’s setting\""
    };
    let illustration_state = if study {
        " class=\"has-study\""
    } else {
        " hidden"
    };
    format!(
        r#"<div class="reader-modes" role="group" aria-label="Reading view"><button type="button" data-reader-mode="passage" aria-pressed="true">Passage</button><button type="button" data-reader-mode="journey" aria-pressed="false"{journey_state}>Journey</button><button type="button" data-reader-mode="illustration" aria-pressed="false"{illustration_state}>Illustrations</button></div><details class="reader-display"><summary>Text settings</summary><div><label>Text size<select data-reader-size aria-label="Reading text size"><option value="standard">Standard</option><option value="large">Large</option><option value="larger">Larger</option></select></label><label>Line spacing<select data-reader-spacing aria-label="Reading line spacing"><option value="comfortable">Comfortable</option><option value="spacious">Spacious</option></select></label></div></details>"#
    )
}

fn illustrated_journey(book: usize, chapter: usize) -> bool {
    (book == 0 && (2..=4).contains(&chapter))
        || super::passage_scenes::available(book, chapter)
        || super::journey_scenes::available(book, chapter)
        || super::journey_books::available(book, chapter)
}

pub(super) fn shell() -> &'static str {
    r#"<aside class="reader-context" data-reader-context hidden aria-label="Passage companion"><header class="reader-context-heading"><div><p class="eyebrow">Alongside the passage</p><h2 data-context-title>Journey</h2><p data-context-reference></p></div><button type="button" data-reader-mode="passage" aria-label="Close passage companion">×</button></header><div data-context-body></div><p class="reader-context-status" data-context-status role="status" aria-live="polite"></p></aside>"#
}

fn read_link(
    book: usize,
    chapter: usize,
    verse: usize,
    mode: &str,
    label: &str,
    client: bool,
) -> String {
    let href = route_path(
        &format!(
            "/book/{}/{chapter}?resume={verse}&context={mode}#v{verse}",
            slug(book)
        ),
        client,
    );
    format!(
        r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{chapter}" data-context-verse="{verse}">{}</a>"#,
        esc(&href),
        slug(book),
        esc(label)
    )
}

fn genesis(chapter: usize, verse: usize, client: bool) -> Value {
    let active = match (chapter, verse) {
        (2, 10..=14) => "river",
        (2, 8..) | (3, 1..=22) => "garden",
        (3, 23) | (4, 1..=15) => "outside",
        (3, 24) => "way",
        (4, 16..) => "nod",
        _ => "introduced",
    };
    let (title, note) = match active {
        "river" => ("The river from Eden", "Genesis 2:10 describes a river watering the garden and dividing into four heads. Its course and modern location are unknown."),
        "garden" => ("In the garden", "Genesis introduces the garden in Eden at 2:8. The scene places the reading within the story, not at a verified modern coordinate."),
        "outside" => ("Outside the garden", "Genesis 3:23 describes the man being sent out to till the ground. The text does not identify a surveyed route or the location of his new home."),
        "way" => ("The way to the tree of life", "Genesis 3:24 places the cherubims and flaming sword east of the garden. This illustration marks a story boundary; it does not establish a real-world route."),
        "nod" => ("Cain in the land of Nod", "Genesis 4:16 names the land of Nod, east of Eden, after Cain departs. Its geographic location is unknown; this stage follows Cain, not Adam and Eve."),
        _ => ("The garden is introduced at Genesis 2:8", "No location is assigned to these opening verses. The garden scene introduces the setting that appears later in the chapter."),
    };
    let stops = [
        ("garden", "Garden", 2, 8, 19, 35),
        ("river", "River", 2, 10, 28, 65),
        ("outside", "Outside the garden", 3, 23, 71, 50),
        ("way", "Guarded way", 3, 24, 53, 43),
        ("nod", "Land of Nod", 4, 16, 88, 54),
    ];
    let mut html = format!(
        r#"<section class="story-journey" data-story-stage="{active}"><p class="reader-context-kind">Illustrated story journey · Genesis 2–4</p><figure class="story-journey-art"><img src="/art/journey/genesis-immersive-v1.webp" width="1536" height="1024" alt="Symbolic landscape with a lush garden and rough ground beyond; this is not a geographic map"><svg viewBox="0 0 100 66.667" aria-hidden="true"><path d="M31 23 C44 25 50 32 69 38 S81 30 85 19"/></svg>"#
    );
    for (id, label, ch, v, x, y) in stops {
        let current = if id == active {
            " aria-current=\"step\""
        } else {
            ""
        };
        let href = route_path(
            &format!("/book/Genesis/{ch}?resume={v}&context=journey#v{v}"),
            client,
        );
        html.push_str(&format!(r#"<a class="story-journey-pin" href="{}" data-context-read data-context-book="Genesis" data-context-chapter="{ch}" data-context-verse="{v}" style="left:{x}%;top:{y}%" aria-label="Read Genesis {ch}:{v}: {label}"{current}><span aria-hidden="true">●</span><span>{label}</span></a>"#, esc(&href)));
    }
    html.push_str(&format!(r#"</figure><p class="story-journey-caption">Symbolic scenery and trail · Locations, distances, and route unknown</p><div class="reader-context-copy"><h3>{title}</h3><p class="reader-context-kind">Explanation</p><p>{note}</p></div><nav class="story-journey-stops" aria-label="Genesis story stages">"#));
    for (id, label, ch, v, _, _) in stops {
        html.push_str(&format!(
            "<span{}>{}</span>",
            if id == active {
                " class=\"is-current\""
            } else {
                ""
            },
            read_link(0, ch, v, "journey", &format!("{label} · {ch}:{v}"), client)
        ));
    }
    html.push_str("</nav><details class=\"reader-context-sources\"><summary>How to read this journey</summary><p>This is an editorial illustration of Genesis 2–4, not a reconstruction of Eden’s geography. The path connects reading stages only. Genesis 3:24 describes guardians east of the garden; it does not describe the precise route of expulsion. The land of Nod is not geographically identified.</p></details></section>");
    json!({"kind":"story","title":"The early Genesis journey","html":html,"active":active})
}

fn geography(book: usize, chapter: usize, verse: usize, client: bool) -> Value {
    let state = super::atlas::focused_state(book, chapter, verse);
    if state["available"] != true {
        return json!({"kind":"empty","title":"Journey","html":format!(r#"<div class="reader-context-empty"><span aria-hidden="true">⌁</span><h3>No journey assigned here</h3><p>This passage does not yet have a sourced, verse-level setting in this reader. Keep reading; we won’t substitute unrelated places.</p><p class="reader-context-kind">Journey coverage</p><p>Illustrated Genesis 2–4; geographic settings in Acts 16–17, Luke 2, and Revelation 1:9.</p>{}</div>"#, read_link(0,2,8,"journey","See the Genesis journey →",client))});
    }
    let mut html = format!(
        r#"<section class="reader-geo"><p class="reader-context-kind">Geographic context · Explanation</p><div class="reader-geo-map" data-context-map role="region" aria-label="Terrain around this passage’s setting"></div><div class="reader-geo-tools" aria-label="Journey map tools"><button type="button" data-context-map-action="follow">Follow passage</button><button type="button" data-context-map-action="overview">Route overview</button><button type="button" data-context-map-action="terrain" aria-pressed="true">3D</button><button type="button" data-context-map-action="retry" hidden>Retry map</button><button type="button" data-context-map-action="zoom-in" aria-label="Zoom map in">+</button><button type="button" data-context-map-action="zoom-out" aria-label="Zoom map out">−</button></div><p class="reader-geo-status" data-context-map-status role="status">Loading local terrain…</p><div class="reader-context-copy"><h3 data-context-place-title>{}</h3><p data-context-place-note>{}</p></div><nav class="reader-geo-stops" aria-label="Journey stops">"#,
        esc(state["setting"]["name"]
            .as_str()
            .unwrap_or(state["title"].as_str().unwrap_or("This chapter’s journey"))),
        esc(state["setting"]["note"]
            .as_str()
            .unwrap_or(state["message"].as_str().unwrap_or("")))
    );
    for stop in state["stops"].as_array().into_iter().flatten() {
        let v = stop["verse"].as_u64().unwrap_or(1) as usize;
        html.push_str(&read_link(
            book,
            chapter,
            v,
            "journey",
            &format!(
                "{} · {chapter}:{v}",
                stop["name"].as_str().unwrap_or("Place")
            ),
            client,
        ));
    }
    let source = state["setting"]["source"]
        .as_str()
        .unwrap_or("Curated chapter journey; see the linked KJV verses.");
    html.push_str(&format!(r#"</nav><details class="reader-context-sources"><summary>Sources &amp; uncertainty</summary><p>{}</p><p>{}</p><p>Modern terrain is not an ancient reconstruction. Dashed connections are schematic, not surveyed ancient routes. Place identifications may be approximate or disputed.</p><p>Coordinates: <a href="https://www.openbible.info/geo/" target="_blank" rel="noopener noreferrer">OpenBible.info</a>, CC BY 4.0; some underlying OSM data is ODbL. Imagery: NASA GIBS. Elevation: Mapzen / AWS Terrain Tiles.</p></details></section>"#,esc(source),esc(state["journey_note"].as_str().unwrap_or(""))));
    json!({"kind":"geography","title":"Journey","html":html,"map":state})
}

pub(super) fn api(q: &HashMap<String, String>, client: bool) -> RouteResponse {
    let position = q
        .get("book")
        .and_then(|b| b.parse::<usize>().ok())
        .zip(q.get("chapter").and_then(|c| c.parse::<usize>().ok()))
        .zip(q.get("verse").and_then(|v| v.parse::<usize>().ok()));
    let Some(((book, chapter), verse)) = position else {
        return json_response(
            400,
            json!({"error":"Choose a valid reader position."}).to_string(),
        );
    };
    if book >= NAMES.len()
        || chapter == 0
        || chapter > bible().books[book].len()
        || verse == 0
        || verse > bible().books[book][chapter - 1].len()
    {
        return json_response(
            400,
            json!({"error":"Choose a valid reader position."}).to_string(),
        );
    }
    let mode = q.get("mode").map(String::as_str).unwrap_or("journey");
    let relevant = learning::relevant_scene(book, chapter);
    let mut value = if mode == "availability" {
        json!({"relevant_scene":relevant,"journey_available":illustrated_journey(book, chapter),"geographic_available":super::atlas::focused_state(book,chapter,verse)["available"] == true})
    } else if mode == "illustration" {
        if let Some(scene) = relevant {
            let requested = q.get("part").map(String::as_str);
            let recommended = learning::recommended_part(scene, book, chapter, verse);
            let selected = requested.or(recommended);
            let Some(html) = learning::embedded(scene, selected, client) else {
                return json_response(
                    400,
                    json!({"error":"This illustration detail is unavailable."}).to_string(),
                );
            };
            json!({"kind":"illustration","title":"Illustrations","html":html,"scene":scene,"part":selected,"following":requested.is_none(),"source_match":recommended.is_some()})
        } else {
            json!({"kind":"empty","title":"Illustrations","html":"<div class=\"reader-context-empty\"><h3>No illustrated study in this chapter yet</h3><p>Illustrations appear alongside the passages they explain. Current studies cover Noah’s Ark (Genesis 6–9), the Tabernacle (Exodus 25–31 and 35–40), and the Sower (Matthew 13).</p><p>Keep reading in Passage, or use Journey to see any available setting.</p></div>"})
        }
    } else if mode == "journey" {
        if let Some(scene) = super::passage_scenes::render_selected(
            book,
            chapter,
            verse,
            q.get("subject").map(String::as_str),
            client,
        ) {
            scene
        } else if book == 0 && (2..=4).contains(&chapter) {
            genesis(chapter, verse, client)
        } else if let Some(scene) = super::journey_scenes::render(book, chapter, verse, client) {
            scene
        } else if super::atlas::focused_state(book, chapter, verse)["available"] == true {
            geography(book, chapter, verse, client)
        } else if let Some(context) = super::journey_books::render(book, chapter, verse, client) {
            context
        } else {
            geography(book, chapter, verse, client)
        }
    } else {
        return json_response(400, json!({"error":"Unknown reading view."}).to_string());
    };
    value["book"] = json!(book);
    value["chapter"] = json!(chapter);
    value["verse"] = json!(verse);
    value["reference"] = json!(format!("{} {chapter}:{verse}", NAMES[book]));
    value["relevant_scene"] = json!(relevant);
    value["journey_available"] = json!(illustrated_journey(book, chapter));
    json_response(200, value.to_string())
}

pub(super) fn legacy(q: &HashMap<String, String>, client: bool) -> String {
    if let Some(scene) = q.get("scene") {
        if let Some(target) =
            learning::legacy_target(scene, q.get("part").map(String::as_str), client)
        {
            return target;
        }
    }
    let book = q
        .get("book")
        .and_then(|b| b.parse::<usize>().ok())
        .filter(|b| *b < 66)
        .unwrap_or(0);
    let chapter = q
        .get("chapter")
        .and_then(|c| c.parse::<usize>().ok())
        .filter(|c| *c > 0 && *c <= bible().books[book].len())
        .unwrap_or(1);
    let verse = q
        .get("verse")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0 && *v <= bible().books[book][chapter - 1].len())
        .unwrap_or(1);
    route_path(
        &format!(
            "/book/{}/{chapter}?resume={verse}&context=journey#v{verse}",
            slug(book)
        ),
        client,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn narrative_stages_follow_the_text_without_assigning_coordinates() {
        for (ch, v, active) in [
            (2, 1, "introduced"),
            (2, 8, "garden"),
            (3, 22, "garden"),
            (3, 23, "outside"),
            (3, 24, "way"),
            (4, 15, "outside"),
            (4, 16, "nod"),
        ] {
            let state = genesis(ch, v, true);
            assert_eq!(state["active"], active);
            assert!(state.get("map").is_none());
            assert!(state["html"]
                .as_str()
                .unwrap()
                .contains("Locations, distances, and route unknown"));
        }
    }
    #[test]
    fn context_positions_and_availability_are_validated_by_rust() {
        for uri in [
            "/api/reader-context?book=66&chapter=1&verse=1",
            "/api/reader-context?book=0&chapter=0&verse=1",
            "/api/reader-context?book=0&chapter=3&verse=25",
            "/api/reader-context?book=0&chapter=3&verse=1&mode=unknown",
        ] {
            assert_eq!(super::super::route(uri, true).status, 400);
        }
        let response = super::super::route(
            "/api/reader-context?book=1&chapter=25&verse=31&mode=availability",
            true,
        );
        let state: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(state["relevant_scene"], "tabernacle");
        assert!(state.get("html").is_none());
        let response = super::super::route(
            "/api/reader-context?book=0&chapter=2&verse=8&mode=availability",
            true,
        );
        let state: Value = serde_json::from_str(&response.body).unwrap();
        assert!(state["relevant_scene"].is_null());
    }
    #[test]
    fn precise_genesis_subjects_precede_the_legacy_landscape() {
        for (chapter, verse, expected) in [
            (1, 9, "genesis-creation-land"),
            (3, 23, "genesis-east-of-eden"),
            (4, 3, "genesis-brothers-offerings"),
            (5, 22, "genesis-family-line"),
        ] {
            let response = super::super::route(
                &format!("/api/reader-context?book=0&chapter={chapter}&verse={verse}&mode=journey"),
                true,
            );
            let state: Value = serde_json::from_str(&response.body).unwrap();
            assert_eq!(state["id"], expected);
        }
        let response = super::super::route(
            "/api/reader-context?book=0&chapter=2&verse=8&mode=journey",
            true,
        );
        let state: Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(state["active"], "garden");
    }

    #[test]
    fn context_never_duplicates_scripture_or_audio() {
        for uri in [
            "/api/reader-context?book=0&chapter=3&verse=23&mode=journey",
            "/api/reader-context?book=1&chapter=25&verse=31&mode=illustration",
            "/api/reader-context?book=18&chapter=23&verse=1&mode=journey",
        ] {
            let response = super::super::route(uri, true);
            assert_eq!(response.status, 200);
            let value: Value = serde_json::from_str(&response.body).unwrap();
            let html = value["html"].as_str().unwrap();
            assert!(!html.contains("class=\"scripture"));
            assert!(!html.contains("data-read-aloud"));
            assert!(!html.contains("data-reader "));
        }
    }
    #[test]
    fn all_chapters_offer_context_and_exact_scenes_keep_priority() {
        for (book, chapters) in bible().books.iter().enumerate() {
            for (chapter, verses) in chapters.iter().enumerate() {
                for verse in [1, verses.len()] {
                    let uri = format!(
                        "/api/reader-context?book={book}&chapter={}&verse={verse}&mode=journey",
                        chapter + 1
                    );
                    let response = super::super::route(&uri, true);
                    assert_eq!(response.status, 200);
                    let state: Value = serde_json::from_str(&response.body).unwrap();
                    assert_eq!(state["journey_available"], true);
                    assert_ne!(state["kind"], "empty", "{uri}");
                    assert!(!state["html"].as_str().unwrap().contains("data-audio"));
                }
            }
        }
        for (book, chapter, verse, kind) in [
            (0, 2, 8, "story"),
            (42, 4, 6, "story"),
            (43, 16, 12, "geography"),
            (18, 23, 1, "story"),
            (18, 24, 1, "book-context"),
            (30, 1, 5, "book-context"),
            (44, 1, 1, "book-context"),
            (65, 22, 1, "story"),
            (65, 22, 6, "book-context"),
        ] {
            let response = super::super::route(
                &format!(
                    "/api/reader-context?book={book}&chapter={chapter}&verse={verse}&mode=journey"
                ),
                true,
            );
            let state: Value = serde_json::from_str(&response.body).unwrap();
            assert_eq!(state["kind"], kind);
        }
    }
}
