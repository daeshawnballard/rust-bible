//! Source-bound illustrated studies. Images and editorial guidance are never Scripture.
use super::{
    esc, html_response, parse_ref, ref_label, reference_url, route_path, slug, verses, Ref,
    RouteResponse,
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

fn reader_context_target(reference: &Ref, scene: &str, part: &str, client: bool) -> String {
    let verse = reference.v1.unwrap_or(1);
    let chapter = reference.ch;
    route_path(
        &format!(
            "/book/{}/{chapter}?resume={verse}&context=illustration&scene={scene}&part={part}#v{verse}",
            slug(reference.book)
        ),
        client,
    )
}

fn context_read_link(reference: &Ref, scene: &str, part: &str, client: bool) -> String {
    let book = slug(reference.book);
    let verse = reference.v1.unwrap_or(1);
    let label = ref_label(reference);
    format!(
        r#"<a class="reader-illustration-read" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}" href="{}">Read {} →</a>"#,
        esc(&book),
        reference.ch,
        verse,
        esc(&reader_context_target(reference, scene, part, client)),
        esc(&label),
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
    let (title, description, scene, part) = match slot % 3 {
        0 => (
            "Inside Noah’s Ark",
            "Four details to explore · Genesis 6–9",
            "noahs-ark",
            "construction",
        ),
        1 => (
            "The Sower’s Field",
            "Four kinds of ground · Matthew 13",
            "sower",
            "path",
        ),
        _ => (
            "Inside the Tabernacle",
            "Eight details to explore · Exodus 25–31, 35–40",
            "tabernacle",
            "lamp",
        ),
    };
    format!(
        r#"<section class="today-explore"><div><p class="eyebrow">An illustrated detail</p><a href="{}">{title} →</a><p>{description}</p></div></section>"#,
        esc(&legacy_target(scene, Some(part), client).unwrap())
    )
}

pub(super) fn relevant_scene(book: usize, chapter: usize) -> Option<&'static str> {
    match (book, chapter) {
        (0, 6..=9) => Some("noahs-ark"),
        (39, 13) => Some("sower"),
        (1, 25..=31 | 35..=40) => Some("tabernacle"),
        _ => None,
    }
}

pub(super) fn recommended_part(
    scene_name: &str,
    book: usize,
    chapter: usize,
    verse: usize,
) -> Option<&'static str> {
    if scene_name == "tabernacle" {
        return super::tabernacle::recommended_part(book, chapter, verse);
    }
    let scene = scene_for(scene_name)?;
    for stop in scene.stops {
        for source in stop.refs {
            if let Some(reference) = parse_ref(source) {
                if reference.book == book
                    && reference.ch == chapter
                    && verses(&reference)
                        .iter()
                        .any(|(number, _)| *number == verse)
                {
                    return Some(stop.id);
                }
            }
        }
    }
    None
}

pub(super) fn legacy_target(scene: &str, part: Option<&str>, client: bool) -> Option<String> {
    if scene == "tabernacle" {
        return super::tabernacle::legacy_target(part, client);
    }
    let scene = scene_for(scene)?;
    let selected = part.unwrap_or(match scene.slug {
        "noahs-ark" => "construction",
        "sower" => "path",
        _ => return None,
    });
    let stop = scene.stops.iter().find(|stop| stop.id == selected)?;
    let reference = parse_ref(stop.refs[0])?;
    Some(reader_context_target(
        &reference, scene.slug, stop.id, client,
    ))
}

pub(super) fn embedded(scene_name: &str, part: Option<&str>, client: bool) -> Option<String> {
    if scene_name == "tabernacle" {
        return super::tabernacle::embedded(part, client);
    }
    let scene = scene_for(scene_name)?;
    let selected = part.unwrap_or(match scene.slug {
        "noahs-ark" => "construction",
        "sower" => "path",
        _ => return None,
    });
    let selected_index = scene.stops.iter().position(|stop| stop.id == selected)?;
    let stop = &scene.stops[selected_index];
    let source = parse_ref(stop.refs[0])?;
    let mut html = format!(
        r#"<section class="reader-illustration" data-context-scene="{}" data-context-selected-part="{}" aria-label="{} illustration"><figure class="reader-illustration-scene"><img src="/art/learn/{}/scene.webp" width="1536" height="1024" alt="Illustrated reconstruction of {}">"#,
        scene.slug,
        stop.id,
        esc(scene.title),
        scene.art,
        esc(scene.title),
    );
    for item in scene.stops {
        let selected = if item.id == stop.id {
            " aria-pressed=\"true\""
        } else {
            " aria-pressed=\"false\""
        };
        html.push_str(&format!(
            r#"<button class="reader-illustration-hotspot" type="button" data-context-part="{}" style="left:{}%;top:{}%" aria-label="Select {}"{}><span class="reader-illustration-hotspot-dot" aria-hidden="true"></span><span class="reader-illustration-hotspot-label">{}</span></button>"#,
            item.id,
            item.position.0,
            item.position.1,
            esc(item.name),
            selected,
            esc(item.name),
        ));
    }
    html.push_str(
        "<figcaption>Artist’s reconstruction · Scene details are illustrative.</figcaption></figure>",
    );
    html.push_str(&format!(
        r#"<div class="reader-illustration-detail" data-context-detail><img class="reader-illustration-object" src="/art/learn/{}/{}.webp" width="1536" height="1024" alt="Dedicated illustration of {}" loading="lazy" decoding="async"><div class="reader-illustration-copy"><p class="reader-illustration-eyebrow">Illustrated detail · {} of {}</p><h3>{}</h3><p class="reader-illustration-reference">{}</p><section class="reader-illustration-explanation"><h4>Explanation</h4><p>{}</p></section><details class="reader-illustration-question"><summary>{}</summary><p>{}</p></details>{}<small class="reader-illustration-reconstruction-note">Artist’s reconstruction. Appearance and placement are illustrative.</small></div></div></section>"#,
        scene.art,
        stop.id,
        esc(stop.name),
        selected_index + 1,
        scene.stops.len(),
        esc(stop.name),
        esc(&stop.refs.join(" · ")),
        esc(stop.note),
        esc(stop.question),
        esc(stop.answer),
        context_read_link(&source, scene.slug, stop.id, client),
    ));
    Some(html)
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
    let Some(scene) = relevant_scene(book, chapter) else {
        return String::new();
    };
    let (part, title) = match scene {
        "noahs-ark" => (
            match chapter {
                7 => "door",
                8 => "window",
                _ => "construction",
            },
            "Inside Noah’s Ark",
        ),
        "sower" => ("path", "The Sower’s Field"),
        _ => (
            match chapter {
                26 => "tent",
                27 => "courtyard",
                _ => "lamp",
            },
            "Inside the Tabernacle",
        ),
    };
    let Some(target) = legacy_target(scene, Some(part), client) else {
        return String::new();
    };
    format!(
        r#"<aside class="related-study" aria-label="Related illustration"><div><p class="eyebrow">Illustration in this passage</p><a href="{}">{} →</a></div><p>Explore a scene detail beside the King James text.</p></aside>"#,
        esc(&target),
        title
    )
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
    fn related_illustrations_link_into_the_reader_context() {
        assert!(related(0, 6, true).contains(
            "#/book/Genesis/6?resume=14&amp;context=illustration&amp;scene=noahs-ark&amp;part=construction#v14"
        ));
        assert!(related(1, 25, true).contains(
            "#/book/Exodus/25?resume=31&amp;context=illustration&amp;scene=tabernacle&amp;part=lamp#v31"
        ));
        assert!(related(39, 13, true).contains(
            "#/book/Matthew/13?resume=3&amp;context=illustration&amp;scene=sower&amp;part=path#v3"
        ));
        assert!(related(42, 3, true).is_empty());
    }

    #[test]
    fn embedded_scene_contains_only_illustration_and_context_controls() {
        for scene in &SCENES {
            for stop in scene.stops {
                let html = embedded(scene.slug, Some(stop.id), false).unwrap();
                assert!(html.contains("class=\"reader-illustration-scene\""));
                assert!(html.contains(&format!("data-context-selected-part=\"{}\"", stop.id)));
                assert!(html.contains(&format!("data-context-part=\"{}\"", stop.id)));
                assert!(html.contains("data-context-read"));
                assert!(html.contains("data-context-book=\""));
                assert!(html.contains("class=\"reader-illustration-object\""));
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
                        "{}:{} unexpectedly includes {forbidden}",
                        scene.slug,
                        stop.id
                    );
                }
            }
        }
        assert!(embedded("noahs-ark", Some("unknown"), false).is_none());
        assert!(embedded("unknown", None, false).is_none());
    }

    #[test]
    fn legacy_context_targets_resolve_to_reader_chapters() {
        for scene in &SCENES {
            for part in scene.stops {
                let target = legacy_target(scene.slug, Some(part.id), false).unwrap();
                assert!(target.contains("context=illustration"));
                assert!(target.contains(&format!("scene={}&part={}", scene.slug, part.id)));
                assert!(target.contains("#v"));
                assert_eq!(crate::route(&target, false).status, 200, "{target}");
            }
        }
        let tabernacle_target = legacy_target("tabernacle", Some("lamp"), false).unwrap();
        assert_eq!(crate::route(&tabernacle_target, false).status, 200);
        assert!(legacy_target("noahs-ark", Some("unknown"), true).is_none());
        assert_eq!(
            legacy_target("sower", None, true).as_deref(),
            Some("#/book/Matthew/13?resume=3&context=illustration&scene=sower&part=path#v3")
        );
    }

    #[test]
    fn relevant_scene_ranges_follow_the_existing_study_sources() {
        for chapter in 6..=9 {
            assert_eq!(relevant_scene(0, chapter), Some("noahs-ark"));
        }
        assert_eq!(relevant_scene(39, 13), Some("sower"));
        for chapter in (25..=31).chain(35..=40) {
            assert_eq!(relevant_scene(1, chapter), Some("tabernacle"));
        }
        assert_eq!(relevant_scene(1, 32), None);
        assert_eq!(relevant_scene(39, 12), None);
    }

    #[test]
    fn recommended_part_requires_an_exact_source_verse() {
        assert_eq!(
            recommended_part("noahs-ark", 0, 6, 14),
            Some("construction")
        );
        assert_eq!(recommended_part("noahs-ark", 0, 7, 16), Some("door"));
        assert_eq!(recommended_part("noahs-ark", 0, 8, 7), Some("window"));
        assert_eq!(recommended_part("noahs-ark", 0, 9, 1), None);
        assert_eq!(recommended_part("sower", 39, 13, 3), Some("path"));
        assert_eq!(recommended_part("sower", 39, 13, 20), Some("rocky"));
        assert_eq!(recommended_part("sower", 39, 12, 1), None);
        assert_eq!(recommended_part("tabernacle", 1, 25, 31), Some("lamp"));
        assert_eq!(recommended_part("tabernacle", 1, 27, 2), Some("altar"));
        assert_eq!(recommended_part("tabernacle", 1, 35, 1), None);
        assert_eq!(recommended_part("unknown", 0, 6, 14), None);
    }

    #[test]
    fn today_cards_link_into_a_reader_context_without_a_studies_hub() {
        for (slot, scene) in [(0, "noahs-ark"), (1, "sower"), (2, "tabernacle")] {
            let card = today_card(slot, true);
            assert!(card.contains(&format!("scene={scene}")));
            assert!(!card.contains("Browse all illustrated studies"));
            assert!(!card.contains("/explore/studies"));
        }
    }
}
