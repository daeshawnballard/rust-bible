//! Scripture-led Tabernacle scene and exploration pages.
//!
//! Rust renders the illustrated scene hotspots, explanation panel, and separate
//! Scripture view in one fixed panel for both Axum and the WebAssembly reader.

use super::{
    esc, parse_ref, ref_label, reference_url, route_path, slug, verses, Ref, RouteResponse,
};

const PARTS: [Part; 8] = [
    Part {
        id: "courtyard",
        name: "Courtyard",
        image_alt: "Illustration of the Tabernacle courtyard, linen boundary, and east gate",
        aria: "Explore the courtyard",
        references: &["Exodus 27:13-18"],
        note: "Exodus specifies the court's linen hangings, east side, and embroidered gate curtain (Exodus 27:13–18).",
    },
    Part {
        id: "altar",
        name: "Altar",
        image_alt: "Illustration of the bronze-covered altar with projecting horns",
        aria: "Explore the altar",
        references: &["Exodus 27:1-2", "Exodus 40:6"],
        note: "The square shittim-wood altar had horns and a brass overlay, and stood before the tabernacle door (Exodus 27:1–2; 40:6).",
    },
    Part {
        id: "basin",
        name: "Laver",
        image_alt: "Illustration of the bronze laver and its foot",
        aria: "Explore the laver",
        references: &["Exodus 30:18-21"],
        note: "The brass laver and its foot were placed between the tabernacle and the altar for washing.",
    },
    Part {
        id: "tent",
        name: "Tabernacle tent",
        image_alt: "Illustration of the Tabernacle tent and its layered coverings",
        aria: "Explore the tabernacle tent",
        references: &["Exodus 26:1", "Exodus 26:7"],
        note: "The curtains used fine twined linen with blue, purple, and scarlet, with a goats’ hair covering also specified (Exodus 26:1, 7).",
    },
    Part {
        id: "table",
        name: "Table of shewbread",
        image_alt: "Illustration of the gold-covered table holding the shewbread",
        aria: "Explore the table of shewbread",
        references: &["Exodus 25:23-24", "Exodus 25:30", "Exodus 26:35"],
        note: "The gold-overlaid shittim table held shewbread and stood on the north side of the tabernacle (Exodus 25:23–24, 30; 26:35).",
    },
    Part {
        id: "lamp",
        name: "Candlestick",
        image_alt: "Illustration of the golden seven-branched candlestick and its lamps",
        aria: "Explore the candlestick",
        references: &["Exodus 25:31-32", "Exodus 25:37", "Exodus 26:35"],
        note: "The pure-gold candlestick had six branches and seven lamps, and stood opposite the table on the south side (Exodus 25:31–32, 37; 26:35).",
    },
    Part {
        id: "veil",
        name: "Veil",
        image_alt: "Illustration of the embroidered veil between the holy place and the most holy",
        aria: "Explore the veil",
        references: &["Exodus 26:31-33"],
        note: "The veil used blue, purple, scarlet, and fine twined linen, and divided the holy place from the most holy.",
    },
    Part {
        id: "ark",
        name: "Ark of the testimony",
        image_alt: "Illustration of the gold-covered Ark of the testimony and mercy seat",
        aria: "Explore the ark of the testimony",
        references: &[
            "Exodus 25:10-11",
            "Exodus 25:16",
            "Exodus 25:21-22",
            "Exodus 26:33",
        ],
        note: "The gold-overlaid shittim ark held the testimony and was placed within the veil (Exodus 25:10–11, 16, 21–22; 26:33).",
    },
];

struct Part {
    id: &'static str,
    name: &'static str,
    image_alt: &'static str,
    aria: &'static str,
    references: &'static [&'static str],
    note: &'static str,
}

fn part_for(id: &str) -> Option<&'static Part> {
    PARTS.iter().find(|part| part.id == id)
}

fn reader_context_target(reference: &Ref, part: &str, client: bool) -> String {
    let verse = reference.v1.unwrap_or(1);
    let chapter = reference.ch;
    route_path(
        &format!(
            "/book/{}/{chapter}?resume={verse}&context=illustration&scene=tabernacle&part={part}#v{verse}",
            slug(reference.book)
        ),
        client,
    )
}

fn context_read_link(reference: &Ref, part: &str, client: bool) -> String {
    let verse = reference.v1.unwrap_or(1);
    let label = ref_label(reference);
    format!(
        r#"<a class="reader-illustration-read" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}" href="{}">Read {} →</a>"#,
        esc(&slug(reference.book)),
        reference.ch,
        verse,
        esc(&reader_context_target(reference, part, client)),
        esc(&label),
    )
}

pub(super) fn recommended_part(book: usize, chapter: usize, verse: usize) -> Option<&'static str> {
    for part in &PARTS {
        for source in part.references {
            if let Some(reference) = parse_ref(source) {
                if reference.book == book
                    && reference.ch == chapter
                    && verses(&reference)
                        .iter()
                        .any(|(number, _)| *number == verse)
                {
                    return Some(part.id);
                }
            }
        }
    }
    None
}

pub(super) fn legacy_target(part: Option<&str>, client: bool) -> Option<String> {
    let selected = part.unwrap_or("lamp");
    let part = part_for(selected)?;
    let reference = parse_ref(part.references[0])?;
    Some(reader_context_target(&reference, part.id, client))
}

pub(super) fn embedded(selected: Option<&str>, client: bool) -> Option<String> {
    let selected = selected.unwrap_or("lamp");
    let part = part_for(selected)?;
    let source = parse_ref(part.references[0])?;
    let positions = [
        ("courtyard", 16.0, 60.5),
        ("altar", 31.6, 52.7),
        ("basin", 41.7, 53.0),
        ("tent", 64.0, 11.5),
        ("table", 59.0, 41.0),
        ("lamp", 54.6, 34.0),
        ("veil", 68.0, 28.0),
        ("ark", 75.0, 23.5),
    ];
    let mut html = format!(
        r#"<section class="reader-illustration" data-context-scene="tabernacle" data-context-selected-part="{}" aria-label="Inside the Tabernacle illustration"><figure class="reader-illustration-scene"><img src="/art/tabernacle.webp" width="1536" height="1024" alt="Illustrated reconstruction of the Tabernacle courtyard, tent, and furnishings">"#,
        part.id,
    );
    for (id, x, y) in positions {
        let item = part_for(id)?;
        let pressed = if id == selected {
            " aria-pressed=\"true\""
        } else {
            " aria-pressed=\"false\""
        };
        html.push_str(&format!(
            r#"<button class="reader-illustration-hotspot" type="button" data-context-part="{id}" style="left:{x}%;top:{y}%" aria-label="Select {}"{pressed}><span class="reader-illustration-hotspot-dot" aria-hidden="true"></span><span class="reader-illustration-hotspot-label">{}</span></button>"#,
            esc(item.name),
            esc(item.name),
        ));
    }
    html.push_str(
        "<figcaption>Artist’s reconstruction · Scene details are illustrative.</figcaption></figure>",
    );
    let index = PARTS
        .iter()
        .position(|candidate| candidate.id == selected)?;
    let (question, answer) = review_question(selected);
    html.push_str(&format!(
        r#"<div class="reader-illustration-detail" data-context-detail><img class="reader-illustration-object" src="/art/objects/{}.webp" width="1536" height="1024" alt="{}" loading="lazy" decoding="async"><div class="reader-illustration-copy"><p class="reader-illustration-eyebrow">Illustrated detail · {} of {}</p><h3>{}</h3><p class="reader-illustration-reference">{}</p><section class="reader-illustration-explanation"><h4>Explanation</h4><p>{}</p></section><details class="reader-illustration-question"><summary>{}</summary><p>{}</p></details>{}<small class="reader-illustration-reconstruction-note">Artist’s reconstruction. Appearance and placement are illustrative.</small></div></div></section>"#,
        part.id,
        esc(part.image_alt),
        index + 1,
        PARTS.len(),
        esc(part.name),
        esc(&part.references.join(" · ")),
        esc(part.note),
        esc(question),
        esc(answer),
        context_read_link(&source, part.id, client),
    ));
    Some(html)
}

fn passage_blocks(part: &Part, client: bool) -> String {
    let mut output = String::new();
    for text_reference in part.references {
        let reference = parse_ref(text_reference).expect("Tabernacle source reference is valid");
        let canonical = ref_label(&reference);
        output.push_str(&format!(
            "<div class=\"passage-excerpt\"><h4>{}</h4><div class=\"scripture\" data-reference=\"{}\" aria-label=\"{} · King James Version\">",
            esc(&canonical),
            esc(&canonical),
            esc(&canonical)
        ));
        for (verse, text) in verses(&reference) {
            output.push_str(&format!(
                "<span class=\"v\" data-reference=\"{}\" data-verse=\"{}\"><sup>{}</sup> {}</span>",
                esc(&canonical),
                verse,
                verse,
                esc(text)
            ));
        }
        output.push_str("</div></div>");
    }
    let chapter = parse_ref(part.references[0]).expect("Tabernacle source reference is valid");
    let chapter_link = reference_url(
        &Ref {
            v1: None,
            v2: None,
            ..chapter
        },
        client,
    );
    output.push_str(&format!(
        "<p class=\"chapter-link\"><a href=\"{}\">Read {} {} →</a></p>",
        esc(&chapter_link),
        esc(super::NAMES[chapter.book]),
        chapter.ch
    ));
    output
}

pub(super) fn scene_listen_controls(reference: &str) -> String {
    // Decorative waveform silhouette, not sampled audio or a seek timeline.
    let mut wave = String::new();
    for i in 0..160 {
        let x = i as f64 / 159.0;
        let envelope = (std::f64::consts::PI * x).sin().powf(1.2);
        let texture = ((i * 37 + 11) % 29) as f64 / 29.0;
        let height = 2.0 + envelope * (9.0 + texture * 38.0);
        wave.push_str(&format!(
            "<line x1=\"{}\" x2=\"{}\" y1=\"{:.1}\" y2=\"{:.1}\"/>",
            i * 5,
            i * 5,
            32.0 - height / 2.0,
            32.0 + height / 2.0
        ));
    }
    format!(
        r#"<div class="scripture-player" data-read-aloud data-audio-compact data-reference="{}"><button class="player-toggle" type="button" data-audio-action="play" aria-label="Listen to Scripture"><svg viewBox="0 0 24 24" aria-hidden="true"><path class="player-play-icon" d="M8 4 21 12 8 20Z"/><path class="player-pause-icon" d="M7 5h4v14H7zm7 0h4v14h-4z"/></svg></button><div class="player-copy"><span class="player-title">Listen to Scripture</span><span data-audio-status role="status" aria-live="polite">Free natural voice · First download about 100 MB.</span></div><div class="player-wave" aria-hidden="true"><svg viewBox="0 0 795 64" preserveAspectRatio="none">{wave}</svg></div><label class="player-speed"><select data-audio-rate aria-label="Speech speed"><option value="0.75">0.75×</option><option value="1" selected>1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label><details class="player-options"><summary aria-label="Playback options">⋯</summary><div class="player-settings"><label>Voice <select data-audio-voice aria-label="Voice"><option value="kokoro:af_heart">Heart · Natural American</option></select></label><button type="button" data-audio-action="stop">Stop reading</button><small>Free, synthetic Kokoro voice. First listen downloads about 100 MB, then runs on this device. Scripture only. Device voices are also available.</small></div></details></div>"#,
        esc(reference)
    )
}

fn illustration(selected: &str, client: bool) -> String {
    let positions = [
        ("courtyard", 16.0, 60.5),
        ("altar", 31.6, 52.7),
        ("basin", 41.7, 53.0),
        ("tent", 64.0, 11.5),
        ("table", 59.0, 41.0),
        ("lamp", 54.6, 34.0),
        ("veil", 68.0, 28.0),
        ("ark", 75.0, 23.5),
    ];
    let mut scene = String::from("<div class=\"illustration\"><img class=\"scene-art\" src=\"/art/tabernacle.webp\" width=\"1536\" height=\"1024\" alt=\"Illustrated cutaway of the Tabernacle courtyard, tent, and furnishings\">");
    for (id, x, y) in positions {
        let part = part_for(id).unwrap();
        let current = if id == selected {
            " aria-current=\"page\""
        } else {
            ""
        };
        scene.push_str(&format!("<a class=\"scene-hotspot\" data-scene-part=\"{id}\" style=\"left:{x}%;top:{y}%\" href=\"{}\" aria-label=\"{}\"{current}><span class=\"hotspot-dot\"></span><span class=\"hotspot-label\">{}</span></a>",esc(&route_path(&format!("/explore/tabernacle?part={id}&view=object"),client)),esc(part.aria),esc(part.name)));
    }
    scene.push_str("</div>");
    scene
}

fn detail_view(id: &str) -> String {
    let part = part_for(id).expect("selected Tabernacle part is known");
    format!(
        "<img class=\"object-preview\" loading=\"lazy\" decoding=\"async\" src=\"/art/objects/{id}.webp\" width=\"1536\" height=\"1024\" alt=\"{}\">",
        esc(part.image_alt)
    )
}

fn explore_body(selected: &str, object_view: bool, client: bool) -> String {
    let part = part_for(selected).expect("selected Tabernacle part is known");
    let scripture_link = route_path(
        &format!("/explore/tabernacle?part={selected}&view=scripture"),
        client,
    );
    let object_link = route_path(
        &format!("/explore/tabernacle?part={selected}&view=object"),
        client,
    );
    let current = " aria-current=\"page\"";
    let hidden = " hidden";
    let mut body = String::from("<section id=\"explore\" class=\"explore-page illustrated-explore\"><div class=\"explore-heading\"><p class=\"eyebrow\">An illustrated reconstruction · Exodus</p><h1>Inside the Tabernacle</h1><p>Read the passage, or tap a hotspot to explore an object.</p></div><div class=\"explore-layout\"><section class=\"scene-column\" aria-label=\"Interactive Tabernacle scene\">");
    let breadcrumb = format!("<nav class=\"explore-breadcrumb\" aria-label=\"Breadcrumb\"><a href=\"{}\">Explore</a> / Tabernacle</nav>", esc(&route_path("/explore", client)));
    body = body.replacen(
        "<div class=\"explore-heading\">",
        &format!("{breadcrumb}<div class=\"explore-heading\">"),
        1,
    );
    body.push_str(&illustration(selected, client));
    body.push_str(&format!("</section><article class=\"object-card study-panel\" data-panel-view=\"{}\" aria-label=\"Passage and object details\"><nav class=\"panel-navigation\" aria-label=\"Panel view\"><a href=\"{}\"{}>Scripture</a><a href=\"{}\"{}>Object</a></nav><section class=\"panel-view scripture-panel\" aria-label=\"Scripture\" tabindex=\"0\"{}><div class=\"panel-title\"><p class=\"eyebrow\">Scripture · King James Version</p><h2>{}</h2></div>",if object_view {"object"} else {"scripture"},esc(&scripture_link),if object_view {""} else {current},esc(&object_link),if object_view {current} else {""},if object_view {hidden} else {""},esc(part.name)));
    let mut parts = String::from("<label class=\"panel-part-picker\">Choose a part<select data-explore-part aria-label=\"Tabernacle part\">");
    for item in PARTS {
        let href = route_path(
            &format!("/explore/tabernacle?part={}&view=scripture", item.id),
            client,
        );
        parts.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            esc(&href),
            if item.id == selected { " selected" } else { "" },
            esc(item.name)
        ));
    }
    parts.push_str("</select></label>");
    body.push_str(&parts);
    body.push_str(&passage_blocks(part, client));
    body.push_str(&format!("</section><section class=\"panel-view object-panel\" aria-label=\"Object and explanation\" tabindex=\"0\"{}>",if object_view {""} else {hidden}));
    body.push_str(&detail_view(selected));
    let (question, answer) = review_question(selected);
    body.push_str(&format!("<div class=\"object-description\"><p class=\"eyebrow\">Selected part</p><h2>{}</h2><p class=\"object-reference\">{}</p><section class=\"context-note\"><h3>Explanation</h3><p>{}</p></section><details class=\"study-question\"><summary>{}</summary><p>{}</p></details><a class=\"open-passage\" href=\"{}\">Back to Scripture <span aria-hidden=\"true\">←</span></a><small class=\"reconstruction-note\">Artist’s reconstruction. Appearance and placement are illustrative.</small></div></section>",esc(part.name),esc(part.references[0]),esc(part.note),esc(question),esc(answer),esc(&scripture_link)));
    let index = PARTS.iter().position(|item| item.id == selected).unwrap();
    body.push_str(&format!(
        "<nav class=\"study-steps\" aria-label=\"Guided study\"><span>Stop {} of 8</span>",
        index + 1
    ));
    let view = if object_view { "object" } else { "scripture" };
    if index > 0 {
        body.push_str(&format!(
            "<a href=\"{}\" rel=\"prev\">← Previous detail</a>",
            esc(&route_path(
                &format!(
                    "/explore/tabernacle?part={}&view={view}",
                    PARTS[index - 1].id
                ),
                client
            ))
        ));
    }
    if index + 1 < PARTS.len() {
        body.push_str(&format!(
            "<a href=\"{}\" rel=\"next\">Next detail →</a>",
            esc(&route_path(
                &format!(
                    "/explore/tabernacle?part={}&view={view}",
                    PARTS[index + 1].id
                ),
                client
            ))
        ));
    } else {
        body.push_str(&format!(
            "<a href=\"{}\">All illustrated studies →</a>",
            esc(&route_path("/explore", client))
        ));
    }
    body.push_str("</nav></article></div><div class=\"explore-audio\">");
    body.push_str(&scene_listen_controls(&part.references.join("; ")));
    body.push_str("</div></section>");
    body
}

fn review_question(id: &str) -> (&'static str, &'static str) {
    match id {
        "courtyard" => ("What material is named for the court hangings?", "Exodus 27:18 names fine twined linen. The pictured boundary helps visualize the court; its exact appearance remains an artistic reconstruction."),
        "altar" => ("What covers the wooden altar?", "Exodus 27:2 specifies a brass overlay. The close-up illustrates that material without claiming the exact ancient finish."),
        "basin" => ("What were Aaron and his sons to wash?", "Exodus 30:19–21 names their hands and feet. Read the passage for its instructions and setting."),
        "tent" => ("How many linen curtains are specified?", "Exodus 26:1 specifies ten curtains. Verse 7 separately names the goats’ hair covering."),
        "table" => ("What was to be set on the table?", "Exodus 25:30 names shewbread, to be before the Lord continually. The pictured loaves are an illustration of that instruction."),
        "lamp" => ("How many lamps are specified?", "Exodus 25:37 specifies seven lamps. The preceding verses describe the candlestick and its branches."),
        "veil" => ("What does the veil divide?", "Exodus 26:33 distinguishes the holy place from the most holy. The image makes that division visible without establishing an exact floor plan."),
        _ => ("What was to be put into the ark?", "Exodus 25:16 names the testimony. The Scripture panel preserves the exact wording and its surrounding references."),
    }
}

pub(super) fn explore_page(client: bool) -> RouteResponse {
    response("Inside the Tabernacle", "lamp", false, client)
}

pub(super) fn selected_page(
    selected: Option<&str>,
    view: Option<&str>,
    client: bool,
) -> RouteResponse {
    let selected = selected.unwrap_or("lamp");
    if part_for(selected).is_none() {
        let content = format!(
            "<section class=\"explore-page\"><h1>Tabernacle part not found</h1><p>Choose one of the named parts to continue.</p><p><a href=\"{}\">Explore the Tabernacle</a></p></section>",
            esc(&route_path("/explore", client))
        );
        let body = super::page_body("", &content, client);
        return RouteResponse {
            status: 404,
            content_type: "text/html; charset=utf-8".into(),
            location: None,
            title: Some("Tabernacle part not found".into()),
            query: None,
            anchor: None,
            body: if client {
                body
            } else {
                super::document("Tabernacle part not found", &body)
            },
        };
    }
    response(
        "Inside the Tabernacle",
        selected,
        view == Some("object"),
        client,
    )
}

fn response(title: &str, selected: &str, object_view: bool, client: bool) -> RouteResponse {
    let content = explore_body(selected, object_view, client);
    let body = super::page_body("", &content, client);
    RouteResponse {
        status: 200,
        content_type: "text/html; charset=utf-8".into(),
        location: None,
        title: Some(title.to_string()),
        query: None,
        anchor: None,
        body: if client {
            body
        } else {
            super::document(title, &body)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{part_for, PARTS};
    use crate::{parse_ref, route, verses};

    #[test]
    fn explore_routes_render_all_eight_named_parts_and_keyboard_links() {
        let landing = route("/explore/tabernacle", false);
        assert_eq!(landing.status, 302);
        assert_eq!(
            landing.location.as_deref(),
            super::legacy_target(None, false).as_deref()
        );
        let landing = super::selected_page(None, None, false);
        assert_eq!(landing.status, 200);
        for part in &PARTS {
            assert!(landing
                .body
                .contains(&format!("data-scene-part=\"{}\"", part.id)));
            assert!(landing
                .body
                .contains(&format!("aria-label=\"{}\"", part.aria)));
            assert!(landing.body.contains(&format!("?part={}", part.id)));
        }
        assert!(!landing.body.contains("class=\"part-picker\""));
        assert!(landing.body.contains("/art/tabernacle.webp"));
        assert_eq!(landing.body.matches("class=\"scene-hotspot\"").count(), 8);
        assert!(landing.body.contains("data-panel-view=\"scripture\""));
        assert!(!landing.body.contains("<dialog"));
        let object_redirect = route("/explore/tabernacle?part=ark&view=object", false);
        assert_eq!(object_redirect.status, 302);
        assert_eq!(
            object_redirect.location.as_deref(),
            super::legacy_target(Some("ark"), false).as_deref()
        );
        let object = super::selected_page(Some("ark"), Some("object"), false);
        assert!(object.body.contains("data-panel-view=\"object\""));
        assert!(object.body.contains("Back to Scripture"));
    }

    #[test]
    fn all_legacy_part_routes_redirect_to_reader_and_context_api_art() {
        for part in &PARTS {
            let route_path = format!("/explore/tabernacle?part={}", part.id);
            let redirect = route(&route_path, false);
            assert_eq!(redirect.status, 302, "{route_path}");
            let target = redirect
                .location
                .expect("legacy route has a canonical target");
            assert_eq!(
                target,
                super::legacy_target(Some(part.id), false).unwrap(),
                "{route_path}"
            );

            let reader = route(&target, false);
            assert_eq!(reader.status, 200, "{target}");
            let reference = parse_ref(part.references[0]).expect("part source parses");
            let expected_verse = reference.v1.expect("part source names its first verse");
            assert!(reader.body.contains(&format!(
                "data-reader-route=\"/book/Exodus/{}\"",
                reference.ch
            )));
            assert!(reader.body.contains(&format!("id=\"v{expected_verse}\"")));

            let api_path = format!(
                "/api/reader-context?book={}&chapter={}&verse={expected_verse}&mode=illustration&part={}",
                reference.book,
                reference.ch,
                part.id
            );
            let context = route(&api_path, true);
            assert_eq!(context.status, 200, "{api_path}");
            let value: serde_json::Value =
                serde_json::from_str(&context.body).expect("reader context returns JSON");
            assert_eq!(value["kind"], "illustration", "{api_path}");
            assert_eq!(value["scene"], "tabernacle", "{api_path}");
            assert_eq!(value["part"], part.id, "{api_path}");
            let html = value["html"].as_str().expect("context HTML is present");
            assert!(html.contains(&format!("data-context-selected-part=\"{}\"", part.id)));
            assert!(html.contains(&format!("src=\"/art/objects/{}.webp\"", part.id)));

            // Keep source text and full-page artwork assertions bound to the
            // legacy renderer helper while the public route redirects.
            let response = super::selected_page(Some(part.id), None, false);
            assert_eq!(response.status, 200, "{route_path}");
            assert!(response.body.contains("class=\"scripture\""));
            assert!(response.body.contains("class=\"context-note\""));
            assert!(response
                .body
                .contains(&format!("data-scene-part=\"{}\"", part.id)));
            let preview = response
                .body
                .split("class=\"object-preview\"")
                .nth(1)
                .unwrap()
                .split('>')
                .next()
                .unwrap();
            for attribute in [
                format!("src=\"/art/objects/{}.webp\"", part.id),
                format!("alt=\"{}\"", part.image_alt),
                "width=\"1536\"".into(),
                "height=\"1024\"".into(),
            ] {
                assert!(
                    preview.contains(&attribute),
                    "missing artwork attribute: {attribute}"
                );
            }
            for source in part.references {
                let reference = parse_ref(source).expect("part source parses");
                let canonical = crate::ref_label(&reference);
                assert!(response
                    .body
                    .contains(&format!("data-reference=\"{canonical}\"")));
                for (number, text) in verses(&reference) {
                    assert!(response.body.contains(&format!("data-verse=\"{number}\"")));
                    assert!(response.body.contains(&crate::esc(text)));
                }
            }
            assert_eq!(part_for(part.id).unwrap().id, part.id);
        }
    }

    #[test]
    fn embedded_art_uses_hotspot_controls_and_contains_no_reader_or_scripture_markup() {
        for part in &PARTS {
            let html = super::embedded(Some(part.id), false).unwrap();
            assert!(html.contains(&format!("data-context-selected-part=\"{}\"", part.id)));
            assert!(html.contains("data-context-part=\"lamp\""));
            assert!(html.contains("data-context-read"));
            assert!(html.contains("data-context-book=\"Exodus\""));
            assert!(html.contains(&format!("/art/objects/{}.webp", part.id)));
            assert!(html.contains("<h4>Explanation</h4>"));
            assert!(html.contains("Artist’s reconstruction"));
            for forbidden in [
                "data-reader",
                "class=\"scripture",
                "class=\"v\"",
                "data-read-aloud",
                "scripture-player",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "{} unexpectedly includes {forbidden}",
                    part.id
                );
            }
        }
        assert!(super::embedded(Some("unknown"), false).is_none());
    }

    #[test]
    fn every_legacy_target_points_to_a_valid_source_reference() {
        for part in &PARTS {
            let target = super::legacy_target(Some(part.id), false).unwrap();
            assert!(target.contains(&format!("scene=tabernacle&part={}", part.id)));
            assert_eq!(route(&target, false).status, 200, "{target}");
        }
        assert_eq!(
            super::legacy_target(None, true).as_deref(),
            Some("#/book/Exodus/25?resume=31&context=illustration&scene=tabernacle&part=lamp#v31")
        );
    }

    #[test]
    fn recommended_part_matches_exact_referenced_verses() {
        assert_eq!(super::recommended_part(1, 25, 31), Some("lamp"));
        assert_eq!(super::recommended_part(1, 25, 30), Some("table"));
        assert_eq!(super::recommended_part(1, 27, 2), Some("altar"));
        assert_eq!(super::recommended_part(1, 35, 1), None);
        assert_eq!(super::recommended_part(0, 25, 31), None);
    }

    #[test]
    fn invalid_part_is_a_real_not_found_response() {
        assert_eq!(route("/explore/tabernacle?part=made-up", false).status, 404);
    }

    #[test]
    fn chapter_and_passage_views_share_the_kjv_read_aloud_controls() {
        for (chapter_uri, passage_uri) in [
            ("/book/Exodus/25", "/go?q=Exodus%2025%3A31-32"),
            ("#/book/Exodus/25", "#/go?q=Exodus%2025%3A31-32"),
        ] {
            for uri in [chapter_uri, passage_uri] {
                let response = route(uri, uri.starts_with('#'));
                assert_eq!(response.status, 200);
                assert!(response.body.contains("data-read-aloud"));
                assert!(response.body.contains("data-audio-action=\"play\""));
                assert!(response.body.contains("data-audio-rate"));
                assert!(response.body.contains("data-audio-voice"));
                assert!(response.body.contains("data-audio-status"));
                assert!(response.body.contains("class=\"v\""));
            }
        }
    }
}
