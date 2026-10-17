//! Passage-linked geography. Coordinates and references are editorial data;
//! Scripture is always rendered by the shared KJV reader.
use super::{
    bible, esc, find_book, html_response, json_response, reader, route_path, slug, RouteResponse,
    NAMES,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Deserialize)]
struct Place {
    name: String,
    lat: f64,
    lon: f64,
    confidence: String,
    url: String,
}
#[derive(Deserialize)]
struct Mention {
    id: String,
    verses: Vec<usize>,
}
#[derive(Deserialize)]
struct Focus {
    book: usize,
    chapter: usize,
    start: usize,
    end: usize,
    place: String,
    note: String,
    source: String,
}
#[derive(Deserialize)]
struct Stop {
    place: String,
    verse: usize,
    label: String,
}
#[derive(Deserialize)]
struct Journey {
    book: usize,
    chapter: usize,
    title: String,
    note: String,
    stops: Vec<Stop>,
}
#[derive(Deserialize)]
struct Index {
    places: HashMap<String, Place>,
    chapters: HashMap<String, Vec<Mention>>,
    focus: Vec<Focus>,
    journeys: Vec<Journey>,
}

fn index() -> &'static Index {
    static DATA: OnceLock<Index> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/atlas/index.json"))
            .expect("validated atlas index")
    })
}

fn position(q: &HashMap<String, String>) -> Option<(usize, usize, usize)> {
    let book = match q.get("book") {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|b| *b < 66)
            .or_else(|| find_book(value))?,
        None => 43,
    };
    let chapter = q
        .get("chapter")
        .map_or(Some(if book == 43 { 16 } else { 1 }), |v| {
            v.parse::<usize>().ok()
        })?;
    let verses = bible().books.get(book)?.get(chapter.checked_sub(1)?)?;
    let verse = q.get("verse").map_or(
        Some(if book == 43 && chapter == 16 { 11 } else { 1 }),
        |v| v.parse::<usize>().ok(),
    )?;
    (verse > 0 && verse <= verses.len()).then_some((book, chapter, verse))
}

fn point(id: &str, p: &Place, verses: &[usize]) -> Value {
    json!({"id":id,"name":p.name,"lat":p.lat,"lon":p.lon,"confidence":p.confidence,"url":p.url,"verses":verses})
}

fn state(book: usize, chapter: usize, verse: usize) -> Value {
    let data = index();
    let mentions = data.chapters.get(&format!("{book}:{chapter}"));
    let mut places: Vec<Value> = mentions
        .into_iter()
        .flatten()
        .filter_map(|m| data.places.get(&m.id).map(|p| point(&m.id, p, &m.verses)))
        .collect();
    let has_chapter = !places.is_empty();
    places.sort_by(|a, b| {
        a["name"]
            .as_str()
            .cmp(&b["name"].as_str())
            .then(a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let focus = data
        .focus
        .iter()
        .filter(|f| f.book == book && f.chapter == chapter && verse >= f.start && verse <= f.end)
        .max_by_key(|f| f.start);
    let setting = focus.and_then(|f| {
        data.places.get(&f.place).map(|p| {
            if !places.iter().any(|p| p["id"] == f.place) {
                places.push(point(&f.place, p, &[]));
            }
            let mut item = point(&f.place, p, &[]);
            item["note"] = json!(f.note);
            item["source"] = json!(f.source);
            item
        })
    });
    let journey = data
        .journeys
        .iter()
        .find(|j| j.book == book && j.chapter == chapter);
    let stops: Vec<Value> = journey
        .into_iter()
        .flat_map(|j| &j.stops)
        .filter_map(|s| {
            data.places.get(&s.place).map(|p| {
                let mut item = point(&s.place, p, &[s.verse]);
                item["chapter"] = json!(chapter);
                item["verse"] = json!(s.verse);
                item["label"] = json!(s.label);
                item
            })
        })
        .collect();
    for stop in &stops {
        if !places.iter().any(|p| p["id"] == stop["id"]) {
            places.push(stop.clone());
        }
    }
    let scope = if has_chapter || !places.is_empty() {
        "chapter"
    } else {
        "none"
    };
    let message = if setting.is_some() {
        format!("Story setting · {} {chapter}:{verse}", NAMES[book])
    } else if has_chapter {
        format!(
            "Places named in {} {chapter} · setting not established for verse {verse}",
            NAMES[book]
        )
    } else if !stops.is_empty() {
        format!(
            "Curated journey stops are available in {} {chapter}; no setting is assigned to verse {verse}.",
            NAMES[book],
        )
    } else {
        format!("No identifiable geographic place is named in {} {chapter}. No location is assigned to this passage.", NAMES[book])
    };
    json!({"book":book,"book_name":NAMES[book],"chapter":chapter,"verse":verse,"scope":scope,
        "title":journey.map(|j|j.title.as_str()).unwrap_or("Follow the passage"),"places":places,"setting":setting,"stops":stops,
        "journey_title":journey.map(|j|j.title.as_str()),"journey_note":journey.map(|j|j.note.as_str()),"message":message,
        "reader_path":format!("/book/{}/{chapter}?resume={verse}",slug(book)),"verse_count":bible().books[book][chapter-1].len(),"chapter_count":bible().books[book].len()})
}

pub(super) fn focused_state(book: usize, chapter: usize, verse: usize) -> Value {
    let mut focused = state(book, chapter, verse);
    let setting = focused["setting"].clone();
    let stops = focused["stops"].as_array().cloned().unwrap_or_default();
    let available = !setting.is_null() || !stops.is_empty();
    let mut places = Vec::new();

    if setting.is_object() {
        places.push(setting.clone());
    }
    for stop in &stops {
        if !places.iter().any(|place: &Value| place["id"] == stop["id"]) {
            places.push(stop.clone());
        }
    }

    focused["places"] = json!(places);
    focused["scope"] = json!(if available { "chapter" } else { "none" });
    focused["available"] = json!(available);
    if !available {
        focused["message"] = json!(format!(
            "No curated geographic context is available for {} {chapter}:{verse}.",
            NAMES[book]
        ));
    } else if setting.is_null() && !stops.is_empty() {
        focused["message"] = json!(format!(
            "Curated journey stops are available in {} {chapter}; no setting is assigned to verse {verse}.",
            NAMES[book]
        ));
    }
    focused
}

pub(super) fn api(q: &HashMap<String, String>) -> RouteResponse {
    match position(q) {
        Some((b, c, v)) => json_response(200, state(b, c, v).to_string()),
        None => json_response(
            400,
            json!({"error":"Choose a valid book, chapter, and verse."}).to_string(),
        ),
    }
}

pub(super) fn reader_link(book: usize, chapter: usize, client: bool) -> String {
    let href = route_path(
        &format!("/explore?book={book}&chapter={chapter}&verse=1"),
        client,
    );
    format!("<a class=\"reader-map-link\" href=\"{}\" data-atlas-reader-link>Map this passage <span aria-hidden=\"true\">↗</span></a>",esc(&href))
}

pub(super) fn page(q: &HashMap<String, String>, client: bool) -> RouteResponse {
    let Some((book, chapter, verse)) = position(q) else {
        let recovery = esc(&route_path("/explore", client));
        return html_response(400,"Passage not found","",&format!("<section class=\"suggestions\"><h1>Choose a valid passage</h1><p>The map needs a valid book, chapter, and verse.</p><a href=\"{recovery}\">Open the map</a></section>"),client,None);
    };
    let initial = state(book, chapter, verse);
    let data = initial
        .to_string()
        .replace('<', "\\u003c")
        .replace('&', "\\u0026");
    let mut books = String::new();
    for (b, name) in NAMES.iter().enumerate() {
        books.push_str(&format!(
            "<option value=\"{b}\"{}>{}</option>",
            if b == book { " selected" } else { "" },
            esc(name)
        ));
    }
    let mut chapters = String::new();
    for ch in 1..=bible().books[book].len() {
        chapters.push_str(&format!(
            "<option value=\"{ch}\"{}>Chapter {ch}</option>",
            if ch == chapter { " selected" } else { "" }
        ));
    }
    let back = route_path(
        &format!("/book/{}/{chapter}?resume={verse}", slug(book)),
        client,
    );
    let library = route_path("/library", client);
    let studies = route_path("/explore/studies", client);
    let content = format!(
        r#"<section class="atlas-page" data-atlas-page data-atlas-client="{client}">
<script type="application/json" data-atlas-state>{data}</script>
<div class="atlas-toolbar"><nav aria-label="Breadcrumb"><a href="{library}">Library</a><span>/</span><label class="atlas-select-label">Book<select data-atlas-book aria-label="Map book">{books}</select></label><label class="atlas-select-label">Chapter<select data-atlas-chapter aria-label="Map chapter">{chapters}</select></label></nav><h1 data-atlas-title>{title}</h1><a class="atlas-back" data-atlas-back href="{back}">Back to reader <span aria-hidden="true">↗</span></a></div>
<div class="atlas-stage"><div class="atlas-map" data-atlas-map role="region" aria-label="Interactive geographic terrain map"></div>
<div class="atlas-follow-status" data-atlas-follow-status>Following {name} {chapter}:{verse}</div>
<div class="atlas-map-tools" aria-label="Map tools"><button type="button" data-atlas-action="north" aria-label="Reset map north">N</button><button type="button" data-atlas-action="zoom-in" aria-label="Zoom in">+</button><button type="button" data-atlas-action="zoom-out" aria-label="Zoom out">−</button><button type="button" data-atlas-action="terrain" aria-pressed="true" aria-label="Toggle 3D terrain">3D</button><button type="button" data-atlas-action="overview" aria-label="Show chapter overview">Fit</button></div>
<div class="atlas-map-message" data-atlas-map-message role="status">Loading geographic terrain…</div>
<button class="atlas-retry" type="button" data-atlas-action="retry" hidden>Retry map</button>
<aside class="atlas-panel" aria-label="Passage and place context"><div class="atlas-panel-heading"><h2 data-atlas-reference>{name} {chapter}</h2><span>King James Version</span></div><div class="atlas-tabs" role="tablist" aria-label="Map panel"><button type="button" role="tab" id="atlas-scripture-tab" aria-controls="atlas-scripture" aria-selected="true" data-atlas-tab="scripture">Scripture</button><button type="button" role="tab" id="atlas-context-tab" aria-controls="atlas-context" aria-selected="false" data-atlas-tab="context">Place context</button></div>
<div class="atlas-scripture-scroll" id="atlas-scripture" role="tabpanel" aria-labelledby="atlas-scripture-tab" tabindex="0" data-atlas-scroll>{reader}</div>
<section class="atlas-context" id="atlas-context" role="tabpanel" aria-labelledby="atlas-context-tab" tabindex="0" data-atlas-context hidden><p class="eyebrow">Geographic context · Explanation</p><h3 data-atlas-place-name>Places in this passage</h3><p data-atlas-explanation>{message}</p><div data-atlas-place-detail></div><h4>Places named in Scripture</h4><p class="atlas-place-help" data-atlas-place-help>These pins identify references to places; they do not establish the setting of every verse.</p><div class="atlas-place-list" data-atlas-place-list></div><details class="atlas-sources"><summary>Map sources &amp; limitations</summary><p>Modern terrain is not a reconstruction of the ancient landscape. Identifications may be approximate or disputed. Dashed lines connect named stops; they are not surveyed ancient routes.</p><p>Coordinates and KJV place references: <a href="https://www.openbible.info/geo/" target="_blank" rel="noopener noreferrer">OpenBible.info</a> (CC BY 4.0; some OpenStreetMap-derived data is ODbL). Imagery: NASA GIBS. Elevation: Mapzen / AWS Terrain Tiles.</p><a href="{studies}">Open illustrated studies →</a></details></section>
<div class="atlas-panel-bottom"><a href="{back}" data-atlas-back>Read full chapter →</a><span data-atlas-current>{name} {chapter}:{verse}</span></div></aside>
<div class="atlas-map-caption">Modern terrain · <span data-atlas-route-caption>Approximate locations</span></div></div>
<div class="atlas-journey-bar"><nav data-atlas-stops aria-label="Journey stops"></nav><div class="atlas-follow-actions"><button type="button" data-atlas-action="panel" aria-expanded="true" aria-controls="atlas-scripture">Passage</button><button type="button" data-atlas-action="follow" aria-pressed="true">Follow passage</button></div></div>
<div class="atlas-audio">{audio}<progress data-atlas-progress value="{verse}" max="{verse_count}" aria-label="Chapter verse progress"></progress></div></section>"#,
        name = esc(NAMES[book]),
        title = esc(initial["title"].as_str().unwrap_or("Follow the passage")),
        message = esc(initial["message"].as_str().unwrap_or("")),
        reader = reader::map_panel(book, chapter, verse, client),
        audio = reader::map_audio(&format!("{} {chapter}", NAMES[book])),
        verse_count = bible().books[book][chapter - 1].len()
    );
    html_response(
        200,
        &format!("Map · {} {chapter}", NAMES[book]),
        "",
        &content,
        client,
        Some(format!("v{verse}")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_and_kjv_references_are_valid() {
        for p in index().places.values() {
            assert!(
                p.lat.is_finite()
                    && p.lon.is_finite()
                    && p.lat.abs() <= 90.0
                    && p.lon.abs() <= 180.0
            );
        }
        for (key, mentions) in &index().chapters {
            let (b, c) = key.split_once(':').unwrap();
            let b = b.parse::<usize>().unwrap();
            let c = c.parse::<usize>().unwrap();
            let text = &bible().books[b][c - 1];
            for m in mentions {
                assert!(index().places.contains_key(&m.id));
                assert!(m.verses.iter().all(|v| *v > 0 && *v <= text.len()));
            }
        }
    }
    #[test]
    fn every_book_has_a_safe_map_response() {
        for b in 0..66 {
            let result = state(b, 1, 1);
            assert_eq!(result["book"], b);
            assert!(result["places"].is_array());
        }
        assert!(position(&HashMap::from([("book".into(), "900".into())])).is_none());
        assert!(position(&HashMap::from([
            ("book".into(), "Acts".into()),
            ("chapter".into(), "16".into()),
            ("verse".into(), "999".into())
        ]))
        .is_none());
    }
    #[test]
    fn curated_geography_does_not_invent_psalm_or_vision_locations() {
        assert_eq!(state(43, 16, 12)["setting"]["name"], "Philippi");
        assert!(state(18, 23, 1)["setting"].is_null());
        assert!(state(65, 21, 1)["setting"].is_null());
        assert_eq!(state(43, 16, 12)["stops"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn chapter_state_does_not_fall_back_to_other_book_chapters() {
        let psalm = state(18, 23, 1);
        assert_eq!(psalm["scope"], "none");
        assert!(psalm["places"].as_array().unwrap().is_empty());
        assert!(psalm["message"]
            .as_str()
            .unwrap()
            .contains("No identifiable geographic place"));

        let acts = state(43, 17, 1);
        assert_eq!(acts["scope"], "chapter");
        assert!(!acts["places"].as_array().unwrap().is_empty());
    }

    #[test]
    fn focused_state_keeps_only_curated_settings_and_current_chapter_stops() {
        let acts_16 = focused_state(43, 16, 12);
        assert_eq!(acts_16["available"], true);
        assert_eq!(acts_16["scope"], "chapter");
        assert_eq!(acts_16["setting"]["name"], "Philippi");
        assert_eq!(acts_16["stops"].as_array().unwrap().len(), 4);
        assert!(acts_16["places"].as_array().unwrap().iter().all(|place| {
            place["id"] == acts_16["setting"]["id"]
                || acts_16["stops"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|stop| stop["id"] == place["id"])
        }));
        assert!(acts_16.get("reader_path").is_some());
        assert!(acts_16.get("verse_count").is_some());
        assert!(acts_16.get("chapter_count").is_some());

        for (book, chapter, verse, name) in [
            (43, 17, 1, "Thessalonica"),
            (41, 2, 5, "Bethlehem"),
            (65, 1, 9, "Patmos"),
        ] {
            let result = focused_state(book, chapter, verse);
            assert_eq!(result["available"], true);
            assert_eq!(result["setting"]["name"], name);
        }

        for (book, chapter) in [(0, 2), (0, 3), (0, 4), (18, 23)] {
            let result = focused_state(book, chapter, 1);
            assert_eq!(result["available"], false);
            assert_eq!(result["scope"], "none");
            assert!(result["places"].as_array().unwrap().is_empty());
            assert!(result["message"]
                .as_str()
                .unwrap()
                .contains("No curated geographic context"));
        }
    }
}
