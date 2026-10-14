//! Scripture-led Tabernacle scene and exploration pages.
//!
//! Rust renders the illustrated scene hotspots, explanation panel, and separate
//! Scripture view in one fixed panel for both Axum and the WebAssembly reader.

use super::{esc, parse_ref, ref_label, reference_url, route_path, verses, Ref, RouteResponse};

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

pub(super) fn listen_controls(reference: &str) -> String {
    format!(
        "<div class=\"listen-controls\" data-read-aloud data-reference=\"{}\"><div class=\"listen-actions\"><button type=\"button\" data-audio-action=\"play\">Play</button><button type=\"button\" data-audio-action=\"pause\">Pause</button><button type=\"button\" data-audio-action=\"stop\">Stop</button></div><label>Speed <select data-audio-rate aria-label=\"Speech speed\"><option value=\"0.75\">0.75×</option><option value=\"1\" selected>1×</option><option value=\"1.25\">1.25×</option><option value=\"1.5\">1.5×</option></select></label><label>Voice <select data-audio-voice aria-label=\"Voice\"><option value=\"\">Default voice</option></select></label><span data-audio-status role=\"status\" aria-live=\"polite\">Ready to read Scripture.</span><small>Synthetic voice. Availability depends on your browser and device.</small></div>",
        esc(reference)
    )
}

fn scene_listen_controls(reference: &str) -> String {
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
        r#"<div class="scripture-player" data-read-aloud data-audio-compact data-reference="{}"><button class="player-toggle" type="button" data-audio-action="play" aria-label="Listen to Scripture"><svg viewBox="0 0 24 24" aria-hidden="true"><path class="player-play-icon" d="M8 4 21 12 8 20Z"/><path class="player-pause-icon" d="M7 5h4v14H7zm7 0h4v14h-4z"/></svg></button><div class="player-copy"><span class="player-title">Listen to Scripture</span><span data-audio-status role="status" aria-live="polite">Ready. Reads the KJV text only.</span></div><div class="player-wave" aria-hidden="true"><svg viewBox="0 0 795 64" preserveAspectRatio="none">{wave}</svg></div><label class="player-speed"><select data-audio-rate aria-label="Speech speed"><option value="0.75">0.75×</option><option value="1" selected>1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label><details class="player-options"><summary aria-label="Playback options">⋯</summary><div class="player-settings"><label>Voice <select data-audio-voice aria-label="Voice"><option value="">Device default</option></select></label><button type="button" data-audio-action="stop">Stop reading</button><small>Synthetic device voice. Reads Scripture only.</small></div></details></div>"#,
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
    body.push_str(&illustration(selected, client));
    body.push_str(&format!("</section><article class=\"object-card study-panel\" data-panel-view=\"{}\" aria-label=\"Passage and object details\"><nav class=\"panel-navigation\" aria-label=\"Panel view\"><a href=\"{}\"{}>Scripture</a><a href=\"{}\"{}>Object</a></nav><section class=\"panel-view scripture-panel\" aria-label=\"Scripture\" tabindex=\"0\"{}><div class=\"panel-title\"><p class=\"eyebrow\">Scripture · King James Version</p><h2>{}</h2></div>",if object_view {"object"} else {"scripture"},esc(&scripture_link),if object_view {""} else {current},esc(&object_link),if object_view {current} else {""},if object_view {hidden} else {""},esc(part.name)));
    body.push_str(&passage_blocks(part, client));
    body.push_str(&format!("</section><section class=\"panel-view object-panel\" aria-label=\"Object and explanation\" tabindex=\"0\"{}>",if object_view {""} else {hidden}));
    body.push_str(&detail_view(selected));
    body.push_str(&format!("<div class=\"object-description\"><p class=\"eyebrow\">Selected part</p><h2>{}</h2><p class=\"object-reference\">{}</p><section class=\"context-note\"><h3>Explanation</h3><p>{}</p></section><a class=\"open-passage\" href=\"{}\">Back to Scripture <span aria-hidden=\"true\">←</span></a><small class=\"reconstruction-note\">Artist’s reconstruction. Appearance and placement are illustrative.</small></div></section></article></div><div class=\"explore-audio\">",esc(part.name),esc(part.references[0]),esc(part.note),esc(&scripture_link)));
    body.push_str(&scene_listen_controls(&part.references.join("; ")));
    body.push_str("</div></section>");
    body
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
        let landing = route("/explore", false);
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
        let object = route("/explore/tabernacle?part=ark&view=object", false);
        assert!(object.body.contains("data-panel-view=\"object\""));
        assert!(object.body.contains("Back to Scripture"));
    }

    #[test]
    fn each_part_uses_canonical_kjv_excerpts_and_its_matching_route_state() {
        for part in &PARTS {
            let route_path = format!("/explore/tabernacle?part={}", part.id);
            let response = route(&route_path, false);
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
