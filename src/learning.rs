//! Source-bound illustrated studies. Images and editorial guidance are never Scripture.
use super::{
    esc, html_response, parse_ref, ref_label, reference_url, route_path, verses, Ref, RouteResponse,
};

struct Stop {
    id: &'static str,
    name: &'static str,
    position: (f32, f32),
    refs: &'static [&'static str],
    note: &'static str,
    question: &'static str,
    answer: &'static str,
}
struct Scene {
    slug: &'static str,
    art: &'static str,
    title: &'static str,
    subtitle: &'static str,
    stops: &'static [Stop],
}
const ARK_STOPS: [Stop; 4] = [
    Stop { id: "construction", name: "The three decks", position: (49.,43.), refs: &["Genesis 6:14-16"], note: "Genesis describes rooms, pitch inside and outside, and three stories. The cutaway makes those levels visible; the detailed floor plan is an artistic reconstruction, not something specified by the passage.", question: "How many levels does the passage name?", answer: "Genesis 6:16 names lower, second, and third stories: three levels. The rooms shown here help visualize that description; their exact arrangement is not given." },
    Stop { id: "door", name: "The door in the side", position: (77.,58.), refs: &["Genesis 6:16", "Genesis 7:16"], note: "The instruction places the door in the side of the ark. Genesis 7:16 then describes entry and the Lord shutting Noah in. The doorway’s shape and joinery in this illustration are illustrative.", question: "Where is the door placed?", answer: "Genesis 6:16 places it in the side. The text does not provide the hinge, latch, or framing details, so this image should not be treated as a construction drawing." },
    Stop { id: "window", name: "The window", position: (77.,35.), refs: &["Genesis 6:16", "Genesis 8:6-9"], note: "A window is included in the instructions. Later Noah opens the window and sends out a raven, then a dove. This close-up gives the opening a visible wooden frame; its exact appearance is not established by the text.", question: "What does Noah do after opening the window?", answer: "Genesis 8:7–8 describes Noah sending out a raven and then a dove. Read the passage to distinguish those events from the reconstructed window details." },
    Stop { id: "food", name: "Food for the journey", position: (27.,54.), refs: &["Genesis 6:19-22"], note: "Noah is instructed to gather food for his household and the living creatures. Baskets, sacks, and grain make that provision visible here; the passage does not specify these particular containers or foods.", question: "Who is the gathered food for?", answer: "Genesis 6:21 says the food is for Noah and for the living creatures. The containers in the artwork are an illustration of storage, not a list supplied by Scripture." },
];
const SOWER_STOPS: [Stop; 4] = [
    Stop { id: "path", name: "Seed by the wayside", position: (26.,64.), refs: &["Matthew 13:3-4", "Matthew 13:18-19"], note: "In the parable, birds take the seed that falls beside the path. Matthew 13:19 gives Jesus’ explanation of this part. The field is a visual aid to reading the parable, not a map of a recorded location.", question: "Where is the explanation of the wayside seed?", answer: "Matthew 13:19 explains the wayside seed in terms of hearing the word of the kingdom without understanding it. Read the Scripture panel for the exact wording." },
    Stop { id: "rocky", name: "Shallow, stony ground", position: (27.,28.), refs: &["Matthew 13:5-6", "Matthew 13:20-21"], note: "The seed springs up quickly in shallow soil, but withers without deep roots. Matthew 13:20–21 supplies the explanation. The cut soil and visible stones help show the contrast between early growth and lasting roots.", question: "Why does the young growth wither?", answer: "Matthew 13:5–6 says there is little earth and no depth of root. The explanation in verses 20–21 connects this part of the parable with a response that does not endure." },
    Stop { id: "thorns", name: "Among the thorns", position: (71.,24.), refs: &["Matthew 13:7", "Matthew 13:22"], note: "Thorns grow alongside the seed and choke it. Matthew 13:22 names the cares of this world and the deceitfulness of riches in the explanation. The brambles in the illustration make that crowding visible.", question: "What does the explanation compare the thorns with?", answer: "Matthew 13:22 names the care of this world and the deceitfulness of riches. Keep the image, this editorial summary, and the exact Scripture wording distinct." },
    Stop { id: "good-soil", name: "The good ground", position: (72.,60.), refs: &["Matthew 13:8-9", "Matthew 13:23"], note: "Seed in good ground bears fruit in differing amounts. Matthew 13:23 connects the good ground with hearing and understanding the word. The healthy grain illustrates fruitfulness without assigning a measured harvest to this scene.", question: "Does all the fruitful seed yield the same amount?", answer: "No. Matthew 13:8 and 13:23 name a hundredfold, sixtyfold, and thirtyfold. The pictured grain is illustrative, not a measurement of those yields." },
];
const SCENES: [Scene; 2] = [
    Scene {
        slug: "noahs-ark",
        art: "ark",
        title: "Inside Noah’s Ark",
        subtitle: "Genesis 6–8",
        stops: &ARK_STOPS,
    },
    Scene {
        slug: "sower",
        art: "sower",
        title: "The Sower’s Field",
        subtitle: "Matthew 13",
        stops: &SOWER_STOPS,
    },
];
fn scene_for(slug: &str) -> Option<&'static Scene> {
    SCENES.iter().find(|scene| scene.slug == slug)
}
fn scene_path(scene: &Scene, stop: &Stop, view: &str, client: bool) -> String {
    route_path(
        &format!("/explore/{}?part={}&view={view}", scene.slug, stop.id),
        client,
    )
}
struct StudyPreview {
    slug: &'static str,
    name: &'static str,
    title: &'static str,
    reference: &'static str,
    description: &'static str,
    image: &'static str,
    details: usize,
}
const STUDIES: [StudyPreview; 3] = [
    StudyPreview {
        slug: "tabernacle",
        name: "The Tabernacle",
        title: "Inside the Tabernacle",
        reference: "Exodus 25–27",
        description: "Explore the sanctuary and its furnishings alongside Exodus 25–27.",
        image: "/art/tabernacle.webp",
        details: 8,
    },
    StudyPreview {
        slug: "noahs-ark",
        name: "Noah’s Ark",
        title: "Inside Noah’s Ark",
        reference: "Genesis 6",
        description: "Look inside the ark and explore its construction alongside Genesis 6–8.",
        image: "/art/learn/ark/scene.webp",
        details: 4,
    },
    StudyPreview {
        slug: "sower",
        name: "The Sower",
        title: "The Sower’s Field",
        reference: "Matthew 13",
        description: "Follow the seed through four kinds of ground alongside Matthew 13.",
        image: "/art/learn/sower/scene.webp",
        details: 4,
    },
];
pub(super) fn hub(selected: Option<&str>, client: bool) -> RouteResponse {
    let study = STUDIES
        .iter()
        .find(|study| Some(study.slug) == selected)
        .unwrap_or(&STUDIES[0]);
    let arrow =
        include_str!("../web/icons/arrow-right.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let book =
        include_str!("../web/icons/book-open.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let mut body = String::from(
        r#"<section class="learning-page" data-learning-hub><div class="learning-browser"><div class="learning-index"><header class="learning-heading"><p class="eyebrow">Illustrated studies</p><h1>Illustrated studies</h1><p>Choose a scene. Read its passage. Look closer.</p></header><nav class="learning-scenes" aria-label="Choose an illustrated study">"#,
    );
    for item in &STUDIES {
        let href = route_path(&format!("/explore?scene={}", item.slug), client);
        let current = if item.slug == study.slug {
            " aria-current=\"true\""
        } else {
            ""
        };
        body.push_str(&format!(
            r#"<a class="learning-scene-choice" data-study-choice="{}" href="{}"{current}><span class="learning-thumbnail"><img src="{}" width="1536" height="1024" alt=""></span><span class="learning-choice-copy"><span class="learning-choice-name">{}</span><span class="learning-choice-source">{} <span aria-hidden="true">·</span> {} details</span></span>{arrow}</a>"#,
            item.slug, esc(&href), item.image, item.name, item.reference, item.details,
        ));
    }
    body.push_str(&format!(
        r#"</nav></div><article class="learning-feature" data-featured-study="{}" aria-labelledby="featured-study-title"><figure><img class="learning-feature-art" src="{}" width="1536" height="1024" alt="Illustrated reconstruction: {}" fetchpriority="high"><figcaption>An illustrated reconstruction</figcaption></figure><div class="learning-feature-copy"><h2 id="featured-study-title">{}</h2><p>{}</p><div class="learning-feature-actions"><a class="learning-enter" href="{}">{book}<span>Enter study</span>{arrow}</a><aside class="learning-integrity" aria-label="About the illustrations"><a href="{}">Illustrations and explanations accompany the KJV text.</a></aside></div></div></article></div></section>"#,
        study.slug,
        study.image,
        study.name,
        study.title,
        study.description,
        esc(&route_path(&format!("/explore/{}", study.slug), client)),
        esc(&route_path("/about", client)),
    ));
    html_response(
        200,
        "Explore Scripture · The Bible",
        "",
        &body,
        client,
        None,
    )
}
pub(super) fn today_card(slot: usize, client: bool) -> String {
    let (title, description, path) = match slot % 3 {
        0 => (
            "Inside Noah’s Ark",
            "Four details to explore · Genesis 6–8",
            "/explore/noahs-ark",
        ),
        1 => (
            "The Sower’s Field",
            "Four kinds of ground · Matthew 13",
            "/explore/sower",
        ),
        _ => (
            "Inside the Tabernacle",
            "Eight details to explore · Exodus 25–27",
            "/explore/tabernacle",
        ),
    };
    format!(
        r#"<section class="today-explore"><div><p class="eyebrow">An illustrated study</p><a href="{}">{title} →</a><p>{description}</p><a class="all-studies" href="{}">Browse all illustrated studies →</a></div></section>"#,
        esc(&route_path(path, client)),
        esc(&route_path("/explore/studies", client))
    )
}
fn passage(stop: &Stop, client: bool) -> String {
    let mut html = String::new();
    for source in stop.refs {
        let reference = parse_ref(source).expect("reviewed study references parse");
        let label = ref_label(&reference);
        html.push_str(&format!(r#"<div class="passage-excerpt"><h3>{}</h3><div class="scripture" data-reference="{}" aria-label="{} · King James Version">"#,esc(&label),esc(&label),esc(&label)));
        for (number, text) in verses(&reference) {
            html.push_str(&format!(r#"<span class="v" data-reference="{}" data-verse="{number}"><sup>{number}</sup> {}</span>"#,esc(&label),esc(text)));
        }
        html.push_str("</div></div>");
    }
    let reference = parse_ref(stop.refs[0]).unwrap();
    let chapter = Ref {
        v1: None,
        v2: None,
        ..reference
    };
    html.push_str(&format!(
        r#"<p class="chapter-link"><a href="{}">Read the full chapter →</a></p>"#,
        esc(&reference_url(&chapter, client))
    ));
    html
}
pub(super) fn page(
    slug: &str,
    selected: Option<&str>,
    view: Option<&str>,
    client: bool,
) -> RouteResponse {
    let Some(scene) = scene_for(slug) else {
        return missing(client);
    };
    let selected = selected.unwrap_or(scene.stops[0].id);
    let Some(index) = scene.stops.iter().position(|stop| stop.id == selected) else {
        return missing(client);
    };
    if view.is_some_and(|v| v != "scripture" && v != "object") {
        return missing(client);
    }
    let stop = &scene.stops[index];
    let object = view == Some("object");
    let scripture_href = scene_path(scene, stop, "scripture", client);
    let object_href = scene_path(scene, stop, "object", client);
    let mut html = format!(
        r#"<section class="explore-page illustrated-explore learning-study" data-learning-scene="{}"><nav class="explore-breadcrumb" aria-label="Breadcrumb"><a href="{}">Illustrated studies</a><span aria-hidden="true"> / </span><span aria-current="page">{}</span></nav><div class="explore-heading"><p class="eyebrow">An illustrated study · {}</p><h1>{}</h1><p>Read the passage, or choose a hotspot to look more closely.</p></div><div class="explore-layout"><section class="scene-column" aria-label="Interactive illustrated scene"><div class="illustration"><img class="scene-art" src="/art/learn/{}/scene.webp" width="1536" height="1024" alt="Illustrated reconstruction: {}">"#,
        scene.slug,
        esc(&route_path("/explore/studies", client)),
        esc(scene.title),
        esc(scene.subtitle),
        esc(scene.title),
        scene.art,
        esc(scene.title)
    );
    for item in scene.stops {
        html.push_str(&format!(r#"<a class="scene-hotspot" data-scene-part="{}" style="left:{}%;top:{}%" href="{}" aria-label="Explore {}"{}><span class="hotspot-dot"></span><span class="hotspot-label">{}</span></a>"#,item.id,item.position.0,item.position.1,esc(&scene_path(scene,item,"object",client)),esc(item.name),if item.id==selected{" aria-current=\"page\""}else{""},esc(item.name)));
    }
    html.push_str(&format!(r#"</div><p class="scene-caption">Illustrative reconstruction · Tap a marker or use the part selector.</p></section><article class="object-card study-panel" data-panel-view="{}" aria-label="Passage and illustrated detail"><nav class="panel-navigation" aria-label="Panel view"><a href="{}"{}>Scripture</a><a href="{}"{}>Illustration</a></nav><section class="panel-view scripture-panel" aria-label="Scripture" tabindex="0"{}><div class="panel-title"><p class="eyebrow">Scripture · King James Version</p><h2>{}</h2></div><label class="panel-part-picker">Choose a detail<select data-explore-part aria-label="Study detail">"#,if object{"object"}else{"scripture"},esc(&scripture_href),if object{""}else{" aria-current=\"page\""},esc(&object_href),if object{" aria-current=\"page\""}else{""},if object{" hidden"}else{""},esc(stop.name)));
    for item in scene.stops {
        html.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            esc(&scene_path(scene, item, "scripture", client)),
            if item.id == selected { " selected" } else { "" },
            esc(item.name)
        ));
    }
    html.push_str("</select></label>");
    html.push_str(&passage(stop, client));
    html.push_str(&format!(r#"</section><section class="panel-view object-panel" aria-label="Illustration and explanation" tabindex="0"{}><img class="object-preview" src="/art/learn/{}/{}.webp" width="1536" height="1024" alt="Dedicated illustration: {}" loading="lazy"><div class="object-description"><p class="eyebrow">Illustrated detail · Stop {} of {}</p><h2>{}</h2><p class="object-reference">{}</p><section class="context-note"><h3>Explanation</h3><p>{}</p></section><details class="study-question"><summary>{}</summary><p>{}</p></details><a class="open-passage" href="{}">Back to Scripture <span aria-hidden="true">←</span></a><small class="reconstruction-note">Artist’s reconstruction. Appearance and placement are illustrative.</small></div></section><nav class="study-steps" aria-label="Guided study"><span>Stop {} of {}</span>"#,if object{""}else{" hidden"},scene.art,stop.id,esc(stop.name),index+1,scene.stops.len(),esc(stop.name),esc(&stop.refs.join(" · ")),esc(stop.note),esc(stop.question),esc(stop.answer),esc(&scripture_href),index+1,scene.stops.len()));
    if index > 0 {
        html.push_str(&format!(
            r#"<a href="{}" rel="prev">← Previous detail</a>"#,
            esc(&scene_path(
                scene,
                &scene.stops[index - 1],
                if object { "object" } else { "scripture" },
                client
            ))
        ));
    }
    if index + 1 < scene.stops.len() {
        html.push_str(&format!(
            r#"<a href="{}" rel="next">Next detail →</a>"#,
            esc(&scene_path(
                scene,
                &scene.stops[index + 1],
                if object { "object" } else { "scripture" },
                client
            ))
        ));
    } else {
        html.push_str(&format!(
            r#"<a href="{}">All illustrated studies →</a>"#,
            esc(&route_path("/explore/studies", client))
        ));
    }
    html.push_str("</nav></article></div><div class=\"explore-audio\">");
    html.push_str(&super::tabernacle::scene_listen_controls(
        &stop.refs.join("; "),
    ));
    html.push_str("</div></section>");
    html_response(
        200,
        &format!("{} · {}", scene.title, stop.name),
        "",
        &html,
        client,
        None,
    )
}
fn missing(client: bool) -> RouteResponse {
    html_response(
        404,
        "Study detail not found",
        "",
        &format!(
            r#"<section class="recovery-page"><h1>Study detail not found</h1><p>Choose one of the illustrated studies to continue.</p><a href="{}">Browse illustrated studies →</a></section>"#,
            esc(&route_path("/explore/studies", client))
        ),
        client,
        None,
    )
}
pub(super) fn related(book: usize, chapter: usize, client: bool) -> String {
    let scene = match (book, chapter) {
        (0, 6..=8) => Some(("/explore/noahs-ark", "Inside Noah’s Ark")),
        (1, 25..=27) => Some(("/explore/tabernacle", "Inside the Tabernacle")),
        (39, 13) => Some(("/explore/sower", "The Sower’s Field")),
        _ => None,
    };
    scene.map_or(String::new(),|(path,name)|format!(r#"<aside class="related-study" aria-label="Related illustrated study"><div><p class="eyebrow">Explore this passage</p><a href="{}">{} →</a></div><p>A separate illustrated study with Scripture, close-ups, and explanations.</p></aside>"#,esc(&route_path(path,client)),name))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_hotspot_has_exact_corpus_sources_and_distinct_art() {
        for scene in &SCENES {
            for stop in scene.stops {
                let body = page(scene.slug, Some(stop.id), Some("object"), true);
                assert_eq!(body.status, 200);
                assert!(body
                    .body
                    .contains(&format!("/art/learn/{}/{}.webp", scene.art, stop.id)));
                for source in stop.refs {
                    let r = parse_ref(source).unwrap();
                    for (_, text) in verses(&r) {
                        assert!(body.body.contains(&esc(text)));
                    }
                }
                assert!(body.body.contains("<h3>Explanation</h3>"));
                assert!(body.body.contains("class=\"study-question\""));
                assert_eq!(body.body.matches("class=\"scene-hotspot\"").count(), 4);
            }
        }
    }
    #[test]
    fn study_routes_have_recovery_and_scripture_default() {
        for scene in &SCENES {
            let first = page(scene.slug, None, None, false);
            assert_eq!(first.status, 200);
            assert!(first.body.contains("data-panel-view=\"scripture\""));
            assert!(first.body.contains("Stop 1 of 4"));
        }
        assert_eq!(page("noahs-ark", Some("invalid"), None, true).status, 404);
        assert_eq!(page("sower", None, Some("invalid"), true).status, 404);
        assert_eq!(page("invalid", None, None, true).status, 404);
        for study in &STUDIES {
            let gallery = hub(Some(study.slug), true);
            assert_eq!(gallery.body.matches("data-study-choice=").count(), 3);
            assert!(gallery
                .body
                .contains(&format!("data-featured-study=\"{}\"", study.slug)));
            assert!(gallery
                .body
                .contains(&format!("href=\"#/explore/{}\"", study.slug)));
        }
        assert!(hub(Some("unknown"), false)
            .body
            .contains("data-featured-study=\"tabernacle\""));
    }
    #[test]
    fn related_studies_are_separate_from_scripture() {
        assert!(related(0, 6, true).contains("#/explore/noahs-ark"));
        assert!(related(1, 25, true).contains("#/explore/tabernacle"));
        assert!(related(39, 13, true).contains("#/explore/sower"));
        assert!(related(42, 3, true).is_empty());
    }
}
