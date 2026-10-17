//! Rust-rendered book gallery and its server-side filter/sort contract.

use std::collections::HashMap;

use super::{bible, esc, route_path, NAMES};

#[derive(Clone, Copy)]
struct BookDetails {
    category: &'static str,
    tagline: &'static str,
    description: &'static str,
}

const BOOKS: [BookDetails; 66] = [
    BookDetails {
        category: "law",
        tagline: "Beginnings & promises",
        description: "Creation, the patriarchs, and the first promises.",
    },
    BookDetails {
        category: "law",
        tagline: "Deliverance & covenant",
        description: "From slavery in Egypt to the covenant at Sinai.",
    },
    BookDetails {
        category: "law",
        tagline: "Worship & holiness",
        description: "Laws for worship, offerings, and holy living.",
    },
    BookDetails {
        category: "law",
        tagline: "The wilderness journey",
        description: "Israel's journey and trials in the wilderness.",
    },
    BookDetails {
        category: "law",
        tagline: "Remember the covenant",
        description: "Moses' final addresses and the covenant renewed.",
    },
    BookDetails {
        category: "history",
        tagline: "Entering and settling the land",
        description: "Israel's entry into Canaan and the division of the land among the tribes.",
    },
    BookDetails {
        category: "history",
        tagline: "Cycles of conflict and renewal",
        description: "Accounts of Israel's leaders during a time of recurring conflict and change.",
    },
    BookDetails {
        category: "history",
        tagline: "Loyalty in an unsettled time",
        description: "A family story of loss, loyalty, and a new beginning in Bethlehem.",
    },
    BookDetails {
        category: "history",
        tagline: "The first king of Israel",
        description: "The rise of Israel's monarchy, Saul's reign, and David's early years.",
    },
    BookDetails {
        category: "history",
        tagline: "David's reign and its aftermath",
        description:
            "David's kingship, family conflicts, and the transition toward Solomon's rule.",
    },
    BookDetails {
        category: "history",
        tagline: "A kingdom divided",
        description: "Solomon's reign and the early history of the divided kingdoms.",
    },
    BookDetails {
        category: "history",
        tagline: "Two kingdoms, changing fortunes",
        description: "The later kings of Israel and Judah through the fall of both kingdoms.",
    },
    BookDetails {
        category: "history",
        tagline: "Remembering David's line",
        description: "Genealogies and accounts of David's reign, worship, and succession.",
    },
    BookDetails {
        category: "history",
        tagline: "Judah's kings and the temple",
        description: "The history of Judah's kings, the temple, and the kingdom's exile.",
    },
    BookDetails {
        category: "history",
        tagline: "A return and a rebuilt temple",
        description: "The return from exile and the rebuilding of the temple in Jerusalem.",
    },
    BookDetails {
        category: "history",
        tagline: "Jerusalem restored",
        description:
            "Nehemiah's return, the rebuilding of Jerusalem's walls, and community renewal.",
    },
    BookDetails {
        category: "history",
        tagline: "Courage in the Persian court",
        description: "A Jewish community's survival during the reign of a Persian king.",
    },
    BookDetails {
        category: "wisdom-poetry",
        tagline: "Suffering, questions, and wisdom",
        description:
            "A poetic dialogue about suffering, justice, and the limits of human understanding.",
    },
    BookDetails {
        category: "wisdom-poetry",
        tagline: "Songs for many seasons",
        description: "Prayers and songs of praise, lament, thanksgiving, and trust.",
    },
    BookDetails {
        category: "wisdom-poetry",
        tagline: "Wisdom for daily life",
        description: "Short teachings about wisdom, character, relationships, and conduct.",
    },
    BookDetails {
        category: "wisdom-poetry",
        tagline: "Reflections on life and time",
        description: "Reflections on work, pleasure, mortality, and the search for meaning.",
    },
    BookDetails {
        category: "wisdom-poetry",
        tagline: "A poem of love and longing",
        description: "A collection of lyrical poems about love, desire, and delight.",
    },
    BookDetails {
        category: "major-prophets",
        tagline: "Justice, warning, and hope",
        description:
            "Prophetic messages about Judah, surrounding nations, judgment, and restoration.",
    },
    BookDetails {
        category: "major-prophets",
        tagline: "A prophet in a changing city",
        description: "Jeremiah's warnings and laments during the final years of Judah.",
    },
    BookDetails {
        category: "major-prophets",
        tagline: "Poems of grief for Jerusalem",
        description: "Five poems mourning Jerusalem's destruction and expressing grief and hope.",
    },
    BookDetails {
        category: "major-prophets",
        tagline: "Visions among the exiles",
        description: "Ezekiel's prophetic messages and symbolic visions during the exile.",
    },
    BookDetails {
        category: "major-prophets",
        tagline: "Faithfulness in a foreign court",
        description: "Stories of life in exile and visions concerning kingdoms and future events.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Returning to faithful love",
        description:
            "A prophetic account using family imagery to speak about Israel's unfaithfulness.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "A call to return",
        description:
            "A call to turn back to God, framed by a locust crisis and a coming day of reckoning.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Justice and responsibility",
        description: "Warnings about injustice and a vision of renewed hope for Israel.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "A warning to Edom",
        description: "A short prophecy concerning Edom and the restoration of Zion.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "A reluctant messenger",
        description: "The story of Jonah's mission to Nineveh and his response to its people.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Justice, humility, and hope",
        description: "Messages about injustice, coming judgment, and a future of peace.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "A city brought low",
        description: "A prophecy announcing Nineveh's fall and the end of its violence.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Questions in a troubled time",
        description: "Habakkuk brings questions about injustice and receives a vision of judgment.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Judgment and a remnant",
        description:
            "Warnings to Judah and surrounding nations, alongside a promise of restoration.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Rebuilding the temple",
        description: "Messages urging the returned community to resume work on the temple.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Visions of renewal",
        description: "Visions and messages about Jerusalem, its people, and future restoration.",
    },
    BookDetails {
        category: "minor-prophets",
        tagline: "Renewed worship and justice",
        description: "A call to faithful worship, justice, and renewed life in the community.",
    },
    BookDetails {
        category: "gospels",
        tagline: "The story of Jesus the Messiah",
        description: "An account of Jesus' teaching, works, death, and resurrection.",
    },
    BookDetails {
        category: "gospels",
        tagline: "Good news in action",
        description:
            "A concise account of Jesus' ministry, journey to Jerusalem, and resurrection.",
    },
    BookDetails {
        category: "gospels",
        tagline: "An orderly account of Jesus' life",
        description: "An account of Jesus' birth, teaching, ministry, death, and resurrection.",
    },
    BookDetails {
        category: "gospels",
        tagline: "Signs, teaching, and belief",
        description: "Selected signs and teachings that present Jesus' identity and invite belief.",
    },
    BookDetails {
        category: "history",
        tagline: "The early church takes shape",
        description: "The spread of the early Christian movement from Jerusalem to Rome.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Good news and a life of faith",
        description:
            "Paul's letter to the church in Rome about the gospel, faith, and life together.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Unity and order in the church",
        description: "Paul addresses divisions, worship, relationships, and the resurrection.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Reconciliation and encouragement",
        description: "Paul reflects on ministry, reconciliation, generosity, and weakness.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Freedom and a shared promise",
        description: "Paul argues for freedom in Christ and life guided by the Spirit.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Life together in Christ",
        description: "A letter about grace, unity, and the life of the church.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Joy, humility, and steadfastness",
        description: "Paul writes about joy, humility, perseverance, and life centered on Christ.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Christ and the life of the church",
        description: "A letter encouraging the Colossians to live with wisdom and gratitude.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Hope while awaiting Christ",
        description: "Paul encourages a young church in faith, mutual care, and hope.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Steadfastness and daily work",
        description:
            "Paul responds to questions about Christ's return and encourages steady living.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Guidance for a young leader",
        description: "Paul encourages Timothy in teaching, community care, and leadership.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Perseverance in ministry",
        description: "Paul urges Timothy to remain faithful in ministry through hardship.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "Teaching for a young church",
        description: "Paul gives Titus guidance about leadership, teaching, and community life.",
    },
    BookDetails {
        category: "pauline-letters",
        tagline: "A personal appeal for Onesimus",
        description:
            "Paul makes a personal appeal concerning Onesimus and their shared relationship.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Endurance and a better covenant",
        description: "A sustained reflection on perseverance, priesthood, and covenant.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Faith expressed in action",
        description: "Practical teaching about speech, care for others, wisdom, and active faith.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Hope and a holy life",
        description:
            "Peter encourages communities facing hardship to live with hope and integrity.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Remembering the first letter",
        description:
            "A brief letter urging readers to remember earlier teaching and remain steadfast.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Love expressed in community",
        description: "A letter about truth, love, and care for one another.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Walking in truth and love",
        description: "A short personal letter urging hospitality and faithful conduct.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Welcoming faithful messengers",
        description:
            "A short personal letter commending hospitality and warning against harmful leadership.",
    },
    BookDetails {
        category: "general-letters",
        tagline: "Contending for the faith",
        description: "A warning against destructive teaching and a call to remain faithful.",
    },
    BookDetails {
        category: "apocalyptic",
        tagline: "Visions of witness and hope",
        description:
            "Apocalyptic visions addressed to seven churches and depicting judgment and renewal.",
    },
];

const CATEGORIES: &[(&str, &str)] = &[
    ("law", "Law"),
    ("history", "History"),
    ("wisdom-poetry", "Wisdom & Poetry"),
    ("major-prophets", "Major Prophets"),
    ("minor-prophets", "Minor Prophets"),
    ("gospels", "Gospels"),
    ("pauline-letters", "Pauline Letters"),
    ("general-letters", "General Letters"),
    ("apocalyptic", "Apocalyptic"),
];

// Measured longest-word em advances for the locally hosted display font.
// Recalibrate with assets/imagegen/binding-typography.json if the typeface changes.
const COVER_WORD_EM: [f64; 66] = [
    3.394201, 3.244800, 4.234000, 3.985400, 6.275200, 3.117400, 2.970600, 2.404200, 3.328600,
    3.328600, 2.537200, 2.537200, 5.276800, 5.276800, 2.142000, 4.560600, 3.135401, 1.441600,
    3.128000, 4.159001, 5.703000, 4.046799, 2.849000, 4.066800, 6.308000, 3.319400, 3.107800,
    2.826400, 1.886000, 2.418800, 3.970800, 2.725000, 2.923400, 3.249400, 4.694200, 4.882800,
    3.362200, 4.922400, 3.963800, 4.264400, 2.528400, 2.175000, 2.150800, 2.188400, 3.514200,
    5.757200, 5.757200, 4.706000, 4.427401, 5.119001, 5.051600, 6.648600, 6.648600, 3.865600,
    3.865600, 2.434000, 4.340000, 3.936401, 2.489000, 2.575400, 2.575400, 2.150800, 2.150800,
    2.150800, 1.991600, 5.185400,
];

fn selected(value: &str, current: &str) -> &'static str {
    if value == current {
        " selected"
    } else {
        ""
    }
}

fn cover_size(params: &HashMap<String, String>) -> usize {
    params
        .get("size")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(240)
        .clamp(160, 340)
}

fn category_is_valid(value: &str) -> bool {
    value == "all" || CATEGORIES.iter().any(|(slug, _)| *slug == value)
}

fn filtered_books(testament: &str, category: &str, sort: &str) -> Vec<usize> {
    let testament = match testament {
        "old" | "new" => testament,
        _ => "all",
    };
    let category = if category_is_valid(category) {
        category
    } else {
        "all"
    };
    let sort = match sort {
        "title" | "title-desc" | "chapters-desc" => sort,
        _ => "bible",
    };
    let mut books = (0..NAMES.len())
        .filter(|&index| {
            let testament_matches = testament == "all"
                || (testament == "old" && index < 39)
                || (testament == "new" && index >= 39);
            let category_matches = category == "all" || BOOKS[index].category == category;
            testament_matches && category_matches
        })
        .collect::<Vec<_>>();

    match sort {
        "title" => books.sort_by(|a, b| {
            NAMES[*a]
                .to_ascii_lowercase()
                .cmp(&NAMES[*b].to_ascii_lowercase())
        }),
        "title-desc" => books.sort_by(|a, b| {
            NAMES[*b]
                .to_ascii_lowercase()
                .cmp(&NAMES[*a].to_ascii_lowercase())
        }),
        "chapters-desc" => books.sort_by(|a, b| {
            bible().books[*b]
                .len()
                .cmp(&bible().books[*a].len())
                .then_with(|| a.cmp(b))
        }),
        _ => {}
    }
    books
}

fn book_slug(name: &str) -> String {
    name.to_ascii_lowercase().replace(' ', "-")
}

/// Book overview, chapter destinations, and KJV-only device speech data share
/// the same corpus in the native server and browser WASM builds.
pub(super) fn render_book(index: usize, client: bool) -> String {
    let name = NAMES[index];
    let detail = BOOKS[index];
    let chapters = &bible().books[index];
    let count = chapters.len();
    let slug = book_slug(name);
    let reader_book_route = format!("/book/{}", super::slug(index));
    let category = CATEGORIES
        .iter()
        .find(|(key, _)| *key == detail.category)
        .map(|(_, label)| *label)
        .unwrap_or("Book");
    let testament = if index < 39 {
        "Old Testament"
    } else {
        "New Testament"
    };
    let title_class = if name.len() > 10 {
        " long-title"
    } else if name.len() > 7 {
        " medium-title"
    } else {
        ""
    };
    let home = esc(&route_path("/library", client));
    let first_route = format!("/book/{slug}/1");
    let first = esc(&route_path(&first_route, client));
    let first_route = esc(&first_route);
    let read_icon =
        include_str!("../web/icons/book-open.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let listen_icon =
        include_str!("../web/icons/headphones.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let speech = chapters
        .iter()
        .enumerate()
        .flat_map(|(ch, verses)| {
            verses.iter().enumerate().map(move |(v, text)| {
                serde_json::json!({
                    "text": text, "chapter": ch + 1, "verse": v + 1
                })
            })
        })
        .collect::<Vec<_>>();
    let speech = serde_json::to_string(&speech)
        .expect("serializable KJV verses")
        .replace('<', "\\u003c");
    let mut html = format!(
        r#"<section class="book-page" data-book-page data-book="{name}"><nav class="book-breadcrumb" aria-label="Breadcrumb"><a href="{home}" data-library-return>Library</a><span aria-hidden="true">/</span><span aria-current="page">{name}</span></nav><div class="book-layout"><div class="book-artwork"><div class="book-cover book-volume" role="img" aria-label="Illustrated {name} book cover"><img class="cover-art" src="/art/covers/{slug}.webp" alt="" width="768" height="1152" fetchpriority="high"><img class="cover-binding" src="/art/book-binding.webp" alt="" aria-hidden="true" width="1080" height="1440"><span class="cover-title{title_class}" style="--cover-word-em:{word_em}">{name}</span><span class="cover-tagline">{tagline}</span></div></div><div class="book-overview"><h1 class="book-title{title_class}">{name}</h1><p class="book-metadata">{testament} <span>·</span> {category} <span>·</span> {count} {chapter_word}</p><p class="book-introduction">{description}</p><section class="continuation-row continuation-row--book" data-continuation data-continue-hide-empty data-continue-other-book hidden>{read_icon}<div data-continue-saved hidden><p>Last opened on this device</p><span data-continue-label></span></div><a data-continue-link hidden href="{first}">Continue reading →</a><button data-continue-clear type="button" hidden aria-label="Clear saved reading location">Clear</button></section><div class="book-audio" data-read-aloud data-book-audio data-book-index="{index}" data-book-route="{reader_book_route}" data-reference="{name}"><div class="book-actions"><a class="book-begin" data-book-read-link data-book-first-route="{first_route}" href="{first}" aria-label="Begin reading {name} chapter 1">{read_icon}<span data-book-read-label>Begin reading</span><span aria-hidden="true">→</span></a><button type="button" class="book-listen" data-audio-action="play">{listen_icon}<span data-audio-label>Listen to {name}</span></button></div><p class="book-start-first" data-book-start-first hidden><a data-book-start-link href="{first}">Start at chapter 1 →</a></p><p class="book-version">King James Version · Free natural voice<br><small>First listen downloads about 100 MB.</small></p><div class="book-playback-settings" hidden><p data-audio-status role="status" aria-live="polite">Ready. Reads the KJV text only.</p><div class="book-playback-actions"><button type="button" data-audio-action="pause">Pause</button><button type="button" data-audio-action="stop">Stop</button><label>Speed<select data-audio-rate aria-label="Speech speed"><option value="0.75">0.75×</option><option value="1" selected>1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label><label>Voice<select data-audio-voice aria-label="Voice"><option value="kokoro:af_heart">Heart · Natural American</option></select></label></div><small>Synthetic Kokoro voice, generated on this device. Device voices are also available. Reads Scripture only. Resumes your saved verse in this book, or starts at chapter 1.</small></div><script type="application/json" data-book-scripture>{speech}</script></div><section class="book-chapters" aria-labelledby="chapter-heading"><h2 id="chapter-heading">Choose a chapter</h2><nav class="chs book-chapter-grid" aria-label="{name} chapters">"#,
        word_em = COVER_WORD_EM[index],
        tagline = esc(detail.tagline),
        description = esc(detail.description),
        chapter_word = if count == 1 { "chapter" } else { "chapters" }
    );
    for chapter in 1..=count {
        let path = esc(&route_path(&format!("/book/{slug}/{chapter}"), client));
        html.push_str(&format!(
            "<a href=\"{path}\" aria-label=\"{name} chapter {chapter}\"{}>{chapter}</a>",
            if chapter == 1 {
                " class=\"first-chapter\""
            } else {
                ""
            }
        ));
    }
    html.push_str("</nav></section></div></div></section>");
    html
}

fn render_controls(testament: &str, category: &str, sort: &str, size: usize) -> String {
    let mut html = String::from(
        "<details class=\"library-tools\" data-library-tools><summary>Library tools</summary><form id=\"library-tools-form\" action=\"/library\" method=\"get\"><div class=\"tools-fields\"><label>Testament<select name=\"testament\" aria-label=\"Testament\">",
    );
    for (value, label) in [
        ("all", "All testaments"),
        ("old", "Old Testament"),
        ("new", "New Testament"),
    ] {
        html.push_str(&format!(
            "<option value=\"{value}\"{}>{label}</option>",
            selected(testament, value)
        ));
    }
    html.push_str(&format!("</select></label><label>Category<select name=\"category\" aria-label=\"Category\"><option value=\"all\"{}>All categories</option>", selected(category, "all")));
    for (value, label) in CATEGORIES {
        html.push_str(&format!(
            "<option value=\"{value}\"{}>{label}</option>",
            selected(category, value)
        ));
    }
    html.push_str("</select></label><label>Sort<select name=\"sort\" aria-label=\"Sort books\">");
    for (value, label) in [
        ("bible", "Bible order"),
        ("title", "Title A-Z"),
        ("title-desc", "Title Z-A"),
        ("chapters-desc", "Chapters most"),
    ] {
        html.push_str(&format!(
            "<option value=\"{value}\"{}>{label}</option>",
            selected(sort, value)
        ));
    }
    html.push_str(&format!(
        "</select></label><label>Cover size<input id=\"library-size\" type=\"range\" name=\"size\" min=\"160\" max=\"340\" step=\"20\" value=\"{size}\" aria-label=\"Cover size\"><output for=\"library-size\">{size}px</output></label></div><div class=\"tools-actions\"><button type=\"button\" data-library-reset>Reset</button><button type=\"submit\">Apply</button></div></form></details>"
    ));
    html
}

pub(super) fn render(params: &HashMap<String, String>, client: bool) -> String {
    let raw_testament = params.get("testament").map(String::as_str).unwrap_or("all");
    let testament = match raw_testament {
        "old" | "new" => raw_testament,
        _ => "all",
    };
    let raw_category = params.get("category").map(String::as_str).unwrap_or("all");
    let category = if category_is_valid(raw_category) {
        raw_category
    } else {
        "all"
    };
    let raw_sort = params.get("sort").map(String::as_str).unwrap_or("bible");
    let sort = match raw_sort {
        "title" | "title-desc" | "chapters-desc" => raw_sort,
        _ => "bible",
    };
    let size = cover_size(params);
    let books = filtered_books(testament, category, sort);
    let total = NAMES.len();
    let count = if books.len() == total {
        format!("Showing all {total} books")
    } else {
        format!("Showing {} of {total} books", books.len())
    };
    let read_icon =
        include_str!("../web/icons/book-open.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let continuation = format!(
        r#"<section class="continuation-row continuation-row--library" data-continuation data-continue-hide-empty hidden>{read_icon}<div data-continue-saved hidden><p>Continue reading</p><span data-continue-label></span></div><a data-continue-link hidden href="{}" aria-label="Continue reading">→</a><details class="continue-options"><summary aria-label="Saved reading options" title="Saved reading options"><span aria-hidden="true">•••</span></summary><button data-continue-clear type="button" hidden aria-label="Clear saved reading location">Clear saved location</button></details></section>"#,
        esc(&route_path("/library", client))
    );
    let mut html = format!(
        "<section class=\"library-page\" data-library><div class=\"library-topline\"><div class=\"library-heading\"><p class=\"library-meta\">66 books <span aria-hidden=\"true\">·</span> King James Version</p><h1>Library</h1><a class=\"library-codex-link\" href=\"{}\">Codex reference →</a></div>{continuation}</div>{}<p class=\"library-count\" role=\"status\">{count}</p>",
        esc(&route_path("/codex", client)),
        render_controls(testament, category, sort, size),
    );

    if books.is_empty() {
        html.push_str(&format!(
            "<div class=\"library-empty\"><h2>No books match these filters</h2><p>Clear the selections to browse all 66 books.</p><a href=\"{}\">Reset filters</a></div>",
            esc(&route_path("/library", client))
        ));
    } else {
        html.push_str(&format!(
            "<div class=\"library-grid\" style=\"--cover-width:{size}px\">"
        ));
        for index in books {
            let name = NAMES[index];
            let detail = BOOKS[index];
            let testament = if index < 39 { "old" } else { "new" };
            let category_label = CATEGORIES
                .iter()
                .find(|(slug, _)| *slug == detail.category)
                .map(|(_, label)| *label)
                .unwrap_or("Book");
            let book_path = format!("/book/{}", book_slug(name));
            let chapter_path = format!("{book_path}/1");
            let title_class = if name.len() > 10 {
                "cover-title long-title"
            } else if name.len() > 7 {
                "cover-title medium-title"
            } else {
                "cover-title"
            };
            let cover = format!("/art/covers/{}.webp", book_slug(name));
            html.push_str(&format!(
                "<article class=\"library-card\" data-book=\"{}\" data-index=\"{index}\" data-testament=\"{testament}\" data-category=\"{}\"><a class=\"book-cover\" data-book-entry href=\"{}\" aria-label=\"Read {}\"><img class=\"cover-art\" src=\"{}\" alt=\"\" loading=\"lazy\" width=\"768\" height=\"1152\"><img class=\"cover-binding\" src=\"/art/book-binding.webp\" alt=\"\" aria-hidden=\"true\" width=\"1080\" height=\"1440\"><h2 class=\"{}\" style=\"--cover-word-em:{}\">{}</h2><p class=\"cover-tagline\">{}</p></a><p class=\"book-description\">{}</p><p class=\"book-category\">{} · {category_label}</p><a class=\"book-read-link\" data-book-entry href=\"{}\">Read {} →</a></article>",
                esc(name), detail.category, esc(&route_path(&chapter_path, client)), esc(name),
                cover, title_class, COVER_WORD_EM[index], esc(name), esc(detail.tagline), esc(detail.description),
                if testament == "old" { "Old Testament" } else { "New Testament" },
                esc(&route_path(&chapter_path, client)), esc(name)
            ));
        }
        html.push_str("</div>");
    }
    html.push_str("</section>");
    html
}

#[cfg(test)]
mod tests {
    use super::{book_slug, cover_size, filtered_books, render_book, BOOKS, CATEGORIES};
    use crate::NAMES;
    use std::collections::HashMap;

    #[test]
    fn every_book_audio_payload_is_exact_complete_kjv_with_valid_chapter_destinations() {
        for (index, name) in NAMES.iter().enumerate() {
            let html = render_book(index, true);
            let data = html
                .split("data-book-scripture>")
                .nth(1)
                .unwrap()
                .split("</script>")
                .next()
                .unwrap();
            let verses: Vec<serde_json::Value> = serde_json::from_str(data).unwrap();
            assert!(html.contains(&format!(
                "data-book-index=\"{index}\" data-book-route=\"/book/{}\"",
                crate::slug(index)
            )));
            let expected = &crate::bible().books[index];
            assert_eq!(verses.len(), expected.iter().map(Vec::len).sum::<usize>());
            let mut offset = 0;
            for (chapter, text) in expected.iter().enumerate() {
                let path = format!("#/book/{}/{}", book_slug(name), chapter + 1);
                assert!(html.contains(&format!("href=\"{path}\"")));
                for (verse, value) in text.iter().enumerate() {
                    assert_eq!(verses[offset]["text"], *value);
                    assert_eq!(verses[offset]["chapter"], chapter + 1);
                    assert_eq!(verses[offset]["verse"], verse + 1);
                    offset += 1;
                }
            }
            let native = render_book(index, false);
            assert!(native.contains(&format!("href=\"/book/{}/1\"", book_slug(name))));
            assert!(!native.contains("href=\"#/book/"));
        }
    }

    #[test]
    fn metadata_covers_the_canonical_66_books_and_known_genres() {
        assert_eq!(BOOKS.len(), 66);
        assert_eq!(NAMES.len(), 66);
        assert_eq!(
            filtered_books("all", "all", "bible"),
            (0..66).collect::<Vec<_>>()
        );
        assert_eq!(NAMES[0], "Genesis");
        assert_eq!(NAMES[38], "Malachi");
        assert_eq!(NAMES[39], "Matthew");
        assert_eq!(NAMES[65], "Revelation");
        for book in BOOKS {
            assert!(CATEGORIES.iter().any(|(slug, _)| *slug == book.category));
            assert!(!book.tagline.trim().is_empty());
            assert!(!book.description.trim().is_empty());
        }
    }

    #[test]
    fn filters_combine_testament_and_genre_and_preserve_canonical_indices() {
        let old_law = filtered_books("old", "law", "bible");
        assert_eq!(old_law, vec![0, 1, 2, 3, 4]);
        let new_letters = filtered_books("new", "pauline-letters", "bible");
        assert_eq!(new_letters, (44..=56).collect::<Vec<_>>());
        assert!(filtered_books("old", "gospels", "bible").is_empty());
    }

    #[test]
    fn unknown_filter_and_sort_values_fall_back_to_the_full_canonical_list() {
        assert_eq!(
            filtered_books("unknown", "unknown", "unknown"),
            (0..66).collect::<Vec<_>>()
        );
    }

    #[test]
    fn sorts_follow_title_or_real_chapter_counts() {
        let ascending = filtered_books("all", "all", "title");
        let descending = filtered_books("all", "all", "title-desc");
        assert_eq!(NAMES[ascending[0]], "1 Chronicles");
        assert_eq!(NAMES[*descending.last().unwrap()], "1 Chronicles");
        let by_chapters = filtered_books("all", "all", "chapters-desc");
        assert_eq!(NAMES[by_chapters[0]], "Psalms");
        assert!(
            crate::bible().books[by_chapters[0]].len()
                >= crate::bible().books[by_chapters[1]].len()
        );
    }

    #[test]
    fn cover_size_uses_the_default_and_clamps_to_the_supported_range() {
        assert_eq!(cover_size(&HashMap::new()), 240);
        assert_eq!(
            cover_size(&HashMap::from([("size".into(), "40".into())])),
            160
        );
        assert_eq!(
            cover_size(&HashMap::from([("size".into(), "900".into())])),
            340
        );
        assert_eq!(
            cover_size(&HashMap::from([("size".into(), "large".into())])),
            240
        );
    }

    #[test]
    fn continue_rows_precede_the_library_gallery_and_preserve_chapter_one() {
        let library = super::render(&HashMap::new(), false);
        let heading = library.find("<h1>Library</h1>").unwrap();
        let continuation = library
            .find("data-continuation data-continue-hide-empty hidden")
            .unwrap();
        let gallery = library.find("class=\"library-grid\"").unwrap();
        assert!(library.contains("66 books <span aria-hidden=\"true\">·</span> King James Version"));
        assert!(heading < continuation && continuation < gallery);

        let book = render_book(0, false);
        assert!(book.contains(
            "data-book-read-link data-book-first-route=\"/book/genesis/1\" href=\"/book/genesis/1\""
        ));
        assert!(book.contains("data-book-start-first hidden"));
        assert!(book.contains("Start at chapter 1 →"));
        assert!(book.contains(
            "data-continuation data-continue-hide-empty data-continue-other-book hidden"
        ));
    }
}
