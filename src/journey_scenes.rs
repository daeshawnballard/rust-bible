//! Storyboard journeys for passages whose locations are carried by the text.
//! Editorial scenery is kept out of Scripture and audio rendering.
use super::{bible, esc, route_path, slug, NAMES};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug)]
struct Passage {
    chapter: usize,
    first: usize,
    last: usize,
}

#[derive(Clone, Copy, Debug)]
struct Stop {
    id: &'static str,
    label: &'static str,
    anchor_chapter: usize,
    anchor_verse: usize,
    x: u8,
    y: u8,
    sources: &'static [Passage],
    note: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct Intro {
    chapter: usize,
    first: usize,
    last: usize,
    title: &'static str,
    note: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct Scene {
    book: usize,
    id: &'static str,
    title: &'static str,
    coverage: &'static str,
    alt: &'static str,
    path: &'static str,
    stops: &'static [Stop],
    intro: Option<Intro>,
}

const FLOOD_STOPS: &[Stop] = &[
    Stop {
        id: "flood-entering-ark",
        label: "Entering the ark",
        anchor_chapter: 7,
        anchor_verse: 7,
        x: 20,
        y: 56,
        sources: &[Passage {
            chapter: 7,
            first: 1,
            last: 10,
        }],
        note: "Genesis describes Noah's preparation and entry with his household and the animals. The illustration compresses this story phase into a schematic storyboard; the text does not place the ark at a verified modern location.",
    },
    Stop {
        id: "flood-waters-rise",
        label: "Floodwaters",
        anchor_chapter: 7,
        anchor_verse: 17,
        x: 42,
        y: 39,
        sources: &[
            Passage {
                chapter: 7,
                first: 11,
                last: 24,
            },
            Passage {
                chapter: 8,
                first: 1,
                last: 3,
            },
        ],
        note: "The waters rise, cover the earth, and later begin to abate. These verses describe successive moments, shown together here as a compressed storyboard.",
    },
    Stop {
        id: "flood-ark-rests",
        label: "Mountains of Ararat",
        anchor_chapter: 8,
        anchor_verse: 4,
        x: 80,
        y: 14,
        sources: &[Passage {
            chapter: 8,
            first: 4,
            last: 5,
        }],
        note: "Genesis 8:4–5 says the ark rested upon the mountains of Ararat. That textual name does not establish a precise modern site or route.",
    },
    Stop {
        id: "flood-birds",
        label: "Birds sent out",
        anchor_chapter: 8,
        anchor_verse: 8,
        x: 70,
        y: 44,
        sources: &[Passage {
            chapter: 8,
            first: 6,
            last: 12,
        }],
        note: "Noah sends out a raven and then doves as the waters recede. The art groups these moments for reading; it is not a measured map.",
    },
    Stop {
        id: "flood-dry-ground-and-altar",
        label: "Dry ground and altar",
        anchor_chapter: 8,
        anchor_verse: 20,
        x: 82,
        y: 75,
        sources: &[
            Passage {
                chapter: 8,
                first: 13,
                last: 22,
            },
            Passage {
                chapter: 9,
                first: 1,
                last: 17,
            },
        ],
        note: "Noah leaves the ark and builds an altar; the following passage records the covenant. The grouped phases are schematic and do not mark surveyed geography.",
    },
];

const WILDERNESS_STOPS: &[Stop] = &[
    Stop {
        id: "wilderness-departure",
        label: "Departure",
        anchor_chapter: 13,
        anchor_verse: 20,
        x: 16,
        y: 65,
        sources: &[Passage {
            chapter: 13,
            first: 17,
            last: 22,
        }],
        note: "Exodus 13:17–22 describes the departure, the route chosen, and the pillar that guides the people. The drawing is a schematic storyboard, not a reconstructed route.",
    },
    Stop {
        id: "wilderness-sea",
        label: "By the sea",
        anchor_chapter: 14,
        anchor_verse: 21,
        x: 24,
        y: 36,
        sources: &[
            Passage {
                chapter: 14,
                first: 1,
                last: 31,
            },
            Passage {
                chapter: 15,
                first: 1,
                last: 21,
            },
        ],
        note: "Exodus 14 recounts the crossing, and chapter 15 gives the song that follows. The following verse, Exodus 15:22, moves into Shur; that transition is shown without an assigned hotspot.",
    },
    Stop {
        id: "wilderness-marah",
        label: "Marah",
        anchor_chapter: 15,
        anchor_verse: 23,
        x: 58,
        y: 79,
        sources: &[Passage {
            chapter: 15,
            first: 23,
            last: 26,
        }],
        note: "The text names Marah and tells of the bitter waters and their healing. Its modern position is not established by this illustration.",
    },
    Stop {
        id: "wilderness-elim",
        label: "Elim",
        anchor_chapter: 15,
        anchor_verse: 27,
        x: 66,
        y: 52,
        sources: &[Passage {
            chapter: 15,
            first: 27,
            last: 27,
        }],
        note: "Exodus 15:27 names Elim and describes its wells and palm trees. No precise modern location is assigned here.",
    },
    Stop {
        id: "wilderness-sin",
        label: "Wilderness of Sin",
        anchor_chapter: 16,
        anchor_verse: 1,
        x: 70,
        y: 22,
        sources: &[Passage {
            chapter: 16,
            first: 1,
            last: 36,
        }],
        note: "Exodus 16 places the people in the wilderness of Sin and recounts the manna and quails. The label follows the text; the scene does not locate it precisely today.",
    },
    Stop {
        id: "wilderness-rephidim",
        label: "Rephidim",
        anchor_chapter: 17,
        anchor_verse: 1,
        x: 88,
        y: 40,
        sources: &[Passage {
            chapter: 17,
            first: 1,
            last: 16,
        }],
        note: "Exodus 17 names Rephidim and recounts water from the rock and the conflict with Amalek. This storyboard does not assert an exact route or modern coordinate.",
    },
];

const RUTH_STOPS: &[Stop] = &[
    Stop {
        id: "ruth-moab",
        label: "Moab",
        anchor_chapter: 1,
        anchor_verse: 6,
        x: 18,
        y: 19,
        sources: &[Passage {
            chapter: 1,
            first: 1,
            last: 6,
        }],
        note: "Ruth 1:1–6 introduces Naomi's family in Moab and their return after she hears that the famine has ended. The place is named in the story; the drawing is not a geographic map.",
    },
    Stop {
        id: "ruth-road",
        label: "Road to Judah",
        anchor_chapter: 1,
        anchor_verse: 7,
        x: 39,
        y: 59,
        sources: &[Passage {
            chapter: 1,
            first: 7,
            last: 18,
        }],
        note: "Naomi and Ruth travel toward Judah and speak on the way. The text gives no surveyed route, so this connecting line is only a story sequence.",
    },
    Stop {
        id: "ruth-bethlehem",
        label: "Bethlehem",
        anchor_chapter: 1,
        anchor_verse: 22,
        x: 74,
        y: 23,
        sources: &[
            Passage {
                chapter: 1,
                first: 19,
                last: 22,
            },
            Passage {
                chapter: 2,
                first: 1,
                last: 2,
            },
        ],
        note: "Ruth 1:19–22 records their arrival in Bethlehem; Ruth 2:1–2 introduces Boaz and Ruth's request to glean. No city layout or exact field boundary is asserted.",
    },
    Stop {
        id: "ruth-boaz-field",
        label: "Boaz's field",
        anchor_chapter: 2,
        anchor_verse: 8,
        x: 70,
        y: 68,
        sources: &[Passage {
            chapter: 2,
            first: 3,
            last: 23,
        }],
        note: "Ruth 2:3–23 follows Ruth gleaning in Boaz's field. The passage does not identify a particular field boundary or an exact modern location.",
    },
];

const BETHLEHEM_STOPS: &[Stop] = &[
    Stop {
        id: "bethlehem-road",
        label: "Journey to Bethlehem",
        anchor_chapter: 2,
        anchor_verse: 4,
        x: 22,
        y: 62,
        sources: &[Passage {
            chapter: 2,
            first: 4,
            last: 6,
        }],
        note: "Luke 2:4–6 says Joseph went from Galilee to Bethlehem with Mary. The text names the endpoints, not a surveyed travel route.",
    },
    Stop {
        id: "bethlehem-manger",
        label: "Bethlehem and the manger",
        anchor_chapter: 2,
        anchor_verse: 7,
        x: 50,
        y: 49,
        sources: &[
            Passage {
                chapter: 2,
                first: 7,
                last: 7,
            },
            Passage {
                chapter: 2,
                first: 16,
                last: 19,
            },
            Passage { chapter: 2, first: 21, last: 21 },
        ],
        note: "Luke names a manger and later describes the shepherds finding the child there. The passage does not establish a particular stable, cave, or building location.",
    },
    Stop {
        id: "bethlehem-fields",
        label: "Shepherds' fields",
        anchor_chapter: 2,
        anchor_verse: 8,
        x: 20,
        y: 28,
        sources: &[Passage { chapter: 2, first: 8, last: 15 }, Passage { chapter: 2, first: 20, last: 20 }],
        note: "Luke 2:8–15 describes shepherds in the field and the announcement; at 2:20 the shepherds return. The illustration treats this as a separate story setting without fixing its modern location.",
    },
    Stop {
        id: "bethlehem-temple",
        label: "Jerusalem",
        anchor_chapter: 2,
        anchor_verse: 22,
        x: 80,
        y: 17,
        sources: &[
            Passage {
                chapter: 2,
                first: 22,
                last: 38,
            },
            Passage {
                chapter: 2,
                first: 41,
                last: 50,
            },
        ],
        note: "Luke groups the presentation with Simeon and Anna, then returns to Jerusalem for the later temple visit. The recurring place is shown schematically; no architectural reconstruction is claimed.",
    },
    Stop {
        id: "bethlehem-nazareth",
        label: "Nazareth",
        anchor_chapter: 2,
        anchor_verse: 39,
        x: 81,
        y: 63,
        sources: &[
            Passage {
                chapter: 2,
                first: 39,
                last: 40,
            },
            Passage {
                chapter: 2,
                first: 51,
                last: 52,
            },
        ],
        note: "Luke 2:39–40 and 2:51–52 return to Nazareth around the Jerusalem episodes. This stop follows the text's recurring place; it does not claim a route or city plan.",
    },
];

const GALILEE_STOPS: &[Stop] = &[
    Stop {
        id: "galilee-shore",
        label: "Teaching and return shore",
        anchor_chapter: 4,
        anchor_verse: 1,
        x: 22,
        y: 65,
        sources: &[
            Passage {
                chapter: 4,
                first: 1,
                last: 34,
            },
            Passage {
                chapter: 5,
                first: 21,
                last: 37,
            },
        ],
        note: "Mark places teaching by the sea at 4:1 and later records Jesus returning by ship before Jairus's request in 5:21–37. These separate phases reactivate one schematic shore stop.",
    },
    Stop {
        id: "galilee-crossing",
        label: "Crossing",
        anchor_chapter: 4,
        anchor_verse: 35,
        x: 55,
        y: 47,
        sources: &[Passage {
            chapter: 4,
            first: 35,
            last: 41,
        }],
        note: "Mark 4:35–41 recounts the crossing and the storm in the boat. The drawn water and path are schematic, not a modern route map.",
    },
    Stop {
        id: "galilee-opposite-shore",
        label: "Opposite shore",
        anchor_chapter: 5,
        anchor_verse: 1,
        x: 84,
        y: 29,
        sources: &[Passage {
            chapter: 5,
            first: 1,
            last: 20,
        }],
        note: "Mark 5:1–20 names the country of the Gadarenes. The exact modern site is not established here; the storyboard marks only the narrative transition across the water.",
    },
    Stop {
        id: "galilee-jairus-house",
        label: "Jairus's house",
        anchor_chapter: 5,
        anchor_verse: 38,
        x: 22,
        y: 34,
        sources: &[Passage {
            chapter: 5,
            first: 38,
            last: 43,
        }],
        note: "Mark 5:38–43 follows Jesus into Jairus's house. The house is part of the story setting; no present-day building or precise site is identified.",
    },
];

const SCENES: &[Scene] = &[
    Scene {
        book: 0,
        id: "flood",
        title: "Noah and the flood",
        coverage: "Genesis 7–9:17",
        alt: "Illustrated flood story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M20 56 C29 49 34 44 42 39 S70 26 80 14 S76 32 70 44 S78 58 82 75",
        stops: FLOOD_STOPS,
        intro: None,
    },
    Scene {
        book: 1,
        id: "wilderness",
        title: "From the sea to the wilderness",
        coverage: "Exodus 13:17–17:16",
        alt: "Illustrated wilderness story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M16 65 C20 53 22 44 24 36 S45 59 58 79 S65 62 66 52 S69 33 70 22 S80 31 88 40",
        stops: WILDERNESS_STOPS,
        intro: Some(Intro { chapter: 15, first: 22, last: 22, title: "Into the wilderness of Shur", note: "Exodus 15:22 says the people went from the sea into Shur and travelled three days without finding water. This transitional setting has no assigned hotspot; the next named stop is Marah." }),
    },
    Scene {
        book: 7,
        id: "ruth",
        title: "Ruth and Naomi",
        coverage: "Ruth 1–2",
        alt: "Illustrated Ruth story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M18 19 C22 38 30 51 39 59 S62 41 74 23 S75 50 70 68",
        stops: RUTH_STOPS,
        intro: None,
    },
    Scene {
        book: 41,
        id: "bethlehem",
        title: "The early life of Jesus",
        coverage: "Luke 2",
        alt: "Illustrated Luke 2 story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M22 62 C33 57 42 53 50 49 S31 35 20 28 S63 17 80 17 S83 48 81 63",
        stops: BETHLEHEM_STOPS,
        intro: Some(Intro {
            chapter: 2,
            first: 1,
            last: 3,
            title: "Opening verses",
            note: "Luke 2:1–3 introduces the decree and registration before the five stops shown here. These verses are an unassigned introduction, so no stop is marked active.",
        }),
    },
    Scene {
        book: 40,
        id: "galilee",
        title: "Across the sea and back",
        coverage: "Mark 4–5:43",
        alt: "Illustrated shore and crossing story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M22 65 C35 55 44 50 55 47 S74 34 84 29 C70 46 54 49 43 46 S29 38 22 34",
        stops: GALILEE_STOPS,
        intro: None,
    },
];

fn scene_for_chapter(book: usize, chapter: usize) -> Option<&'static Scene> {
    SCENES.iter().find(|scene| {
        scene.book == book
            && (scene
                .stops
                .iter()
                .flat_map(|stop| stop.sources)
                .any(|source| source.chapter == chapter)
                || scene.intro.is_some_and(|intro| intro.chapter == chapter))
    })
}

fn passage_contains(passage: Passage, chapter: usize, verse: usize) -> bool {
    passage.chapter == chapter && (passage.first..=passage.last).contains(&verse)
}

fn stop_at(scene: &Scene, chapter: usize, verse: usize) -> Option<&'static Stop> {
    scene.stops.iter().find(|stop| {
        stop.sources
            .iter()
            .any(|source| passage_contains(*source, chapter, verse))
    })
}

fn in_intro(scene: &Scene, chapter: usize, verse: usize) -> Option<Intro> {
    scene
        .intro
        .filter(|intro| intro.chapter == chapter && (intro.first..=intro.last).contains(&verse))
}

fn ref_label(book: usize, passage: Passage) -> String {
    if passage.first == passage.last {
        format!("{} {}:{}", NAMES[book], passage.chapter, passage.first)
    } else {
        format!(
            "{} {}:{}–{}",
            NAMES[book], passage.chapter, passage.first, passage.last
        )
    }
}

fn read_link(book: usize, passage: Passage, client: bool, label: &str) -> String {
    let book_slug = slug(book);
    let href = route_path(
        &format!(
            "/book/{book_slug}/{}?resume={}&context=journey#v{}",
            passage.chapter, passage.first, passage.first
        ),
        client,
    );
    format!(
        r#"<a class="reader-context-read" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}">{}</a>"#,
        esc(&href),
        esc(&book_slug),
        passage.chapter,
        passage.first,
        esc(label)
    )
}

fn source_links(scene: &Scene, sources: &[Passage], client: bool) -> String {
    let mut html = String::from("<p class=\"story-journey-sources\">Sources: ");
    for (index, passage) in sources.iter().copied().enumerate() {
        if index > 0 {
            html.push_str(" · ");
        }
        let label = format!("Read {}", ref_label(scene.book, passage));
        html.push_str(&read_link(scene.book, passage, client, &label));
    }
    html.push_str("</p>");
    html
}

fn render_intro(scene: &Scene, intro: Intro, client: bool) -> Value {
    let source = Passage {
        chapter: intro.chapter,
        first: intro.first,
        last: intro.last,
    };
    let source_list = source_links(scene, &[source], client);
    render_html(scene, None, intro.title, intro.note, &source_list, client)
}

fn render_stop(scene: &Scene, stop: &Stop, client: bool) -> Value {
    let source_list = source_links(scene, stop.sources, client);
    render_html(
        scene,
        Some(stop),
        stop.label,
        stop.note,
        &source_list,
        client,
    )
}

fn render_html(
    scene: &Scene,
    active: Option<&Stop>,
    current_title: &str,
    current_note: &str,
    current_sources: &str,
    client: bool,
) -> Value {
    let active_id = active.map(|stop| stop.id);
    let stage = active_id.unwrap_or("intro");
    let book_slug = slug(scene.book);
    let mut html = format!(
        r#"<section class="story-journey" data-story-scene="{}" data-story-stage="{}"><p class="reader-context-kind">Illustrated Journey · {}</p><figure class="story-journey-art"><img src="/art/journey/{}.webp" width="1536" height="1024" alt="{}"><svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true"><path d="{}"></path></svg>"#,
        esc(scene.id),
        esc(stage),
        esc(scene.coverage),
        esc(scene.id),
        esc(scene.alt),
        scene.path
    );
    for stop in scene.stops {
        let anchor = Passage {
            chapter: stop.anchor_chapter,
            first: stop.anchor_verse,
            last: stop.anchor_verse,
        };
        let href = route_path(
            &format!(
                "/book/{book_slug}/{}?resume={}&context=journey#v{}",
                stop.anchor_chapter, stop.anchor_verse, stop.anchor_verse
            ),
            client,
        );
        let current = if Some(stop.id) == active_id {
            " aria-current=\"step\""
        } else {
            ""
        };
        let aria_label = format!("Read {}: {}", ref_label(scene.book, anchor), stop.label);
        html.push_str(&format!(
            r#"<a class="story-journey-pin" href="{}" data-context-read data-context-book="{}" data-context-chapter="{}" data-context-verse="{}" style="left:{}%;top:{}%" aria-label="{}"{}><span aria-hidden="true">●</span><span>{}</span></a>"#,
            esc(&href),
            esc(&book_slug),
            stop.anchor_chapter,
            stop.anchor_verse,
            stop.x,
            stop.y,
            esc(&aria_label),
            current,
            esc(stop.label)
        ));
    }
    html.push_str("</figure><p class=\"story-journey-caption\">Artistic storyboard · Times and places compressed</p>");
    html.push_str(&format!(
        "<div class=\"story-journey-current\"><h3>{}</h3>",
        esc(current_title)
    ));
    if let Some(stop) = active {
        let source = Passage {
            chapter: stop.anchor_chapter,
            first: stop.anchor_verse,
            last: stop.anchor_verse,
        };
        html.push_str(&read_link(
            scene.book,
            source,
            client,
            &format!("Read {} →", ref_label(scene.book, source)),
        ));
    }
    html.push_str("</div><details class=\"story-journey-details\"><summary>Context &amp; journey stops</summary>");
    html.push_str(&format!(
        "<div class=\"reader-context-copy\"><p class=\"reader-context-kind\">Explanation</p><p>{}</p>{}</div>",
        esc(current_note),
        current_sources
    ));
    html.push_str(&format!(
        r#"<nav class="story-journey-stops" aria-label="{} journey stops">"#,
        esc(&book_slug)
    ));
    for stop in scene.stops {
        let current = Some(stop.id) == active_id;
        let current_class = if current { " class=\"is-current\"" } else { "" };
        let aria_current = if current {
            " aria-current=\"step\""
        } else {
            ""
        };
        let anchor = Passage {
            chapter: stop.anchor_chapter,
            first: stop.anchor_verse,
            last: stop.anchor_verse,
        };
        let label = format!("{} · {}", stop.label, ref_label(scene.book, anchor));
        let link = read_link(
            scene.book,
            Passage {
                chapter: stop.anchor_chapter,
                first: stop.anchor_verse,
                last: stop.anchor_verse,
            },
            client,
            &label,
        );
        let link = if aria_current.is_empty() {
            link
        } else {
            link.replacen(">", &format!("{}>", aria_current), 1)
        };
        html.push_str(&format!("<span{current_class}>{link}</span>"));
    }
    html.push_str("</nav></details></section>");
    json!({
        "kind": "story",
        "title": scene.title,
        "html": html,
        "active": active_id,
        "scene": scene.id
    })
}

/// Return a storyboard for a passage that falls inside one of the authored ranges.
/// The unassigned Luke 2:1–3 introduction is rendered with no active stop.
pub(super) fn render(book: usize, chapter: usize, verse: usize, client: bool) -> Option<Value> {
    let scene = scene_for_chapter(book, chapter)?;
    if let Some(stop) = stop_at(scene, chapter, verse) {
        Some(render_stop(scene, stop, client))
    } else {
        in_intro(scene, chapter, verse).map(|intro| render_intro(scene, intro, client))
    }
}

/// Whether a chapter contains any specifically assigned story stop or intro.
pub(super) fn available(book: usize, chapter: usize) -> bool {
    scene_for_chapter(book, chapter).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all_scenes() -> &'static [Scene] {
        SCENES
    }

    fn stop_id(book: usize, chapter: usize, verse: usize) -> Option<&'static str> {
        let scene = scene_for_chapter(book, chapter)?;
        stop_at(scene, chapter, verse).map(|stop| stop.id)
    }

    #[test]
    fn every_range_and_anchor_is_present_in_the_embedded_kjv_and_ranges_do_not_overlap() {
        let mut ids = HashSet::new();
        for scene in all_scenes() {
            let mut assigned_verses = HashSet::new();
            for stop in scene.stops {
                assert!(ids.insert(stop.id), "duplicate stop id: {}", stop.id);
                let anchor_is_sourced = stop.sources.iter().any(|source| {
                    source.chapter == stop.anchor_chapter
                        && (source.first..=source.last).contains(&stop.anchor_verse)
                });
                assert!(
                    anchor_is_sourced,
                    "anchor is outside source ranges: {}",
                    stop.id
                );
                assert!(
                    bible().books[scene.book].len() >= stop.anchor_chapter,
                    "missing anchor chapter: {}",
                    stop.id
                );
                assert!(
                    bible().books[scene.book][stop.anchor_chapter - 1].len() >= stop.anchor_verse,
                    "missing anchor verse: {}",
                    stop.id
                );
                for source in stop.sources {
                    assert!(source.first > 0 && source.first <= source.last);
                    assert!(bible().books[scene.book].len() >= source.chapter);
                    let verses = &bible().books[scene.book][source.chapter - 1];
                    assert!(
                        verses.len() >= source.last,
                        "missing range end: {}",
                        stop.id
                    );
                    for verse in source.first..=source.last {
                        assert!(
                            assigned_verses.insert((source.chapter, verse)),
                            "overlapping range at {} {}:{}",
                            NAMES[scene.book],
                            source.chapter,
                            verse
                        );
                    }
                }
            }
            if let Some(intro) = scene.intro {
                let verses = &bible().books[scene.book][intro.chapter - 1];
                assert!(intro.first > 0 && intro.first <= intro.last);
                assert!(verses.len() >= intro.last);
                for verse in intro.first..=intro.last {
                    assert!(
                        assigned_verses.insert((intro.chapter, verse)),
                        "intro overlaps a stop at {} {}:{}",
                        NAMES[scene.book],
                        intro.chapter,
                        verse
                    );
                }
            }
        }
    }

    #[test]
    fn scene_transitions_and_recurring_places_follow_the_authored_ranges() {
        for (book, chapter, verse, expected) in [
            (0, 7, 7, "flood-entering-ark"),
            (0, 7, 17, "flood-waters-rise"),
            (0, 8, 4, "flood-ark-rests"),
            (0, 8, 8, "flood-birds"),
            (0, 9, 17, "flood-dry-ground-and-altar"),
            (1, 13, 20, "wilderness-departure"),
            (1, 14, 21, "wilderness-sea"),
            (1, 15, 23, "wilderness-marah"),
            (1, 15, 27, "wilderness-elim"),
            (1, 16, 1, "wilderness-sin"),
            (1, 17, 1, "wilderness-rephidim"),
            (7, 1, 6, "ruth-moab"),
            (7, 1, 7, "ruth-road"),
            (7, 1, 22, "ruth-bethlehem"),
            (7, 1, 19, "ruth-bethlehem"),
            (7, 2, 3, "ruth-boaz-field"),
            (7, 2, 4, "ruth-boaz-field"),
            (41, 2, 4, "bethlehem-road"),
            (41, 2, 7, "bethlehem-manger"),
            (41, 2, 8, "bethlehem-fields"),
            (41, 2, 20, "bethlehem-fields"),
            (41, 2, 22, "bethlehem-temple"),
            (41, 2, 39, "bethlehem-nazareth"),
            (41, 2, 41, "bethlehem-temple"),
            (41, 2, 51, "bethlehem-nazareth"),
            (40, 4, 34, "galilee-shore"),
            (40, 4, 35, "galilee-crossing"),
            (40, 5, 1, "galilee-opposite-shore"),
            (40, 5, 21, "galilee-shore"),
            (40, 5, 38, "galilee-jairus-house"),
        ] {
            assert_eq!(stop_id(book, chapter, verse), Some(expected));
        }
    }

    #[test]
    fn unassigned_boundaries_do_not_carry_a_nearby_stop_forward() {
        assert!(render(0, 9, 18, true).is_none());
        assert!(render(1, 13, 16, true).is_none());
        assert!(render(40, 5, 44, true).is_none());
        assert!(!available(0, 6));
        assert!(!available(40, 3));

        let intro = render(41, 2, 1, true).unwrap();
        assert!(intro["active"].is_null());
        assert_eq!(intro["scene"], "bethlehem");
        assert!(intro["html"].as_str().unwrap().contains("Opening verses"));
        assert_eq!(stop_id(41, 2, 3), None);
        assert!(render(1, 15, 22, true).unwrap()["active"].is_null());
        assert_eq!(stop_id(7, 1, 18), Some("ruth-road"));
    }

    #[test]
    fn journey_html_has_live_reader_links_and_separate_editorial_notes() {
        for (book, chapter, verse, active, link) in [
            (
                0,
                7,
                7,
                "flood-entering-ark",
                "#/book/Genesis/7?resume=7&amp;context=journey#v7",
            ),
            (
                7,
                2,
                4,
                "ruth-boaz-field",
                "#/book/Ruth/2?resume=3&amp;context=journey#v3",
            ),
            (
                41,
                2,
                8,
                "bethlehem-fields",
                "#/book/Luke/2?resume=8&amp;context=journey#v8",
            ),
            (
                40,
                4,
                35,
                "galilee-crossing",
                "#/book/Mark/4?resume=35&amp;context=journey#v35",
            ),
        ] {
            let state = render(book, chapter, verse, true).unwrap();
            let html = state["html"].as_str().unwrap();
            assert_eq!(state["kind"], "story");
            assert_eq!(state["active"], active);
            assert!(html.contains("class=\"story-journey-pin\""));
            assert!(html.contains("aria-current=\"step\""));
            assert!(html.contains("data-context-read"));
            assert!(html.contains(&format!("href=\"{link}\"")));
            assert!(html.contains("width=\"1536\" height=\"1024\""));
            assert!(html.contains("/art/journey/"));
            assert!(html.contains("<svg viewBox=\"0 0 100 100\""));
            assert!(html.contains("class=\"reader-context-copy\""));
            assert!(html.contains("class=\"story-journey-stops\""));
            assert!(!html.contains("class=\"scripture"));
            assert!(!html.contains("data-reader "));
            assert!(!html.contains("data-audio"));
            assert!(!html.contains("class=\"player"));
        }

        let native = render(0, 7, 7, false).unwrap();
        let native_html = native["html"].as_str().unwrap();
        assert!(native_html.contains("href=\"/book/Genesis/7?resume=7&amp;context=journey#v7\""));
        assert!(native_html.contains("data-context-book=\"Genesis\""));
    }
}
