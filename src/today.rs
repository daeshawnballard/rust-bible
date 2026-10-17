use super::*;

// Edition 1 is an immutable, finite reading sequence, not a claim of new daily content.
const EDITION_1_READINGS: [(&str, &str, &str); 14] = [
    (
        "Psalms 23:1-3",
        "today/psalms",
        "Illustration inspired by Psalms",
    ),
    ("John 1:1-5", "covers/john", "Illustration inspired by John"),
    (
        "Genesis 1:1-5",
        "covers/genesis",
        "Illustration inspired by Genesis",
    ),
    (
        "Isaiah 40:28-31",
        "covers/isaiah",
        "Illustration inspired by Isaiah",
    ),
    (
        "Matthew 6:25-30",
        "covers/matthew",
        "Illustration inspired by Matthew",
    ),
    (
        "Proverbs 3:5-8",
        "covers/proverbs",
        "Illustration inspired by Proverbs",
    ),
    (
        "Philippians 4:4-9",
        "covers/philippians",
        "Illustration inspired by Philippians",
    ),
    (
        "Psalms 121:1-4",
        "today/psalms",
        "Illustration inspired by Psalms",
    ),
    (
        "Exodus 25:31-32",
        "objects/lamp",
        "An illustrated reconstruction of the candlestick",
    ),
    (
        "Micah 6:6-8",
        "covers/micah",
        "Illustration inspired by Micah",
    ),
    (
        "Luke 10:38-42",
        "covers/luke",
        "Illustration inspired by Luke",
    ),
    (
        "Romans 12:9-13",
        "covers/romans",
        "Illustration inspired by Romans",
    ),
    (
        "1 Corinthians 13:4-7",
        "covers/1-corinthians",
        "Illustration inspired by 1 Corinthians",
    ),
    (
        "Revelation 21:1-4",
        "covers/revelation",
        "Illustration inspired by Revelation",
    ),
];

// Edition 2 is a second finite, curated sequence anchored to the same date.
const EDITION_2_READINGS: [(&str, &str, &str); 42] = [
    (
        "Genesis 6:14-16",
        "learn/ark/scene",
        "An illustrated reconstruction of Noah’s Ark",
    ),
    (
        "Matthew 13:3-9",
        "learn/sower/scene",
        "An illustrated landscape inspired by the parable of the sower",
    ),
    (
        "Exodus 25:23-26",
        "objects/table",
        "An illustrated reconstruction of the table of showbread",
    ),
    (
        "Genesis 12:1-4",
        "covers/genesis",
        "Illustration inspired by Genesis",
    ),
    (
        "Exodus 3:1-5",
        "covers/exodus",
        "Illustration inspired by Exodus",
    ),
    (
        "Joshua 1:5-9",
        "covers/joshua",
        "Illustration inspired by Joshua",
    ),
    (
        "Ruth 1:16-18",
        "covers/ruth",
        "Illustration inspired by Ruth",
    ),
    (
        "1 Samuel 17:45-47",
        "covers/1-samuel",
        "Illustration inspired by 1 Samuel",
    ),
    (
        "1 Kings 19:11-13",
        "covers/1-kings",
        "Illustration inspired by 1 Kings",
    ),
    ("Job 19:23-27", "covers/job", "Illustration inspired by Job"),
    (
        "Psalms 46:1-3",
        "today/psalms",
        "Illustration inspired by Psalms",
    ),
    (
        "Proverbs 16:1-4",
        "covers/proverbs",
        "Illustration inspired by Proverbs",
    ),
    (
        "Ecclesiastes 3:1-5",
        "covers/ecclesiastes",
        "Illustration inspired by Ecclesiastes",
    ),
    (
        "Isaiah 9:2-6",
        "covers/isaiah",
        "Illustration inspired by Isaiah",
    ),
    (
        "Jeremiah 29:11-13",
        "covers/jeremiah",
        "Illustration inspired by Jeremiah",
    ),
    (
        "Daniel 3:16-18",
        "covers/daniel",
        "Illustration inspired by Daniel",
    ),
    (
        "Jonah 1:15-17",
        "covers/jonah",
        "Illustration inspired by Jonah",
    ),
    (
        "Micah 5:2-5",
        "covers/micah",
        "Illustration inspired by Micah",
    ),
    (
        "Matthew 5:14-16",
        "covers/matthew",
        "Illustration inspired by Matthew",
    ),
    (
        "Mark 4:35-39",
        "covers/mark",
        "Illustration inspired by Mark",
    ),
    (
        "Luke 2:8-12",
        "covers/luke",
        "Illustration inspired by Luke",
    ),
    (
        "John 10:11-15",
        "covers/john",
        "Illustration inspired by John",
    ),
    ("Acts 2:1-4", "covers/acts", "Illustration inspired by Acts"),
    (
        "Romans 8:31-35",
        "covers/romans",
        "Illustration inspired by Romans",
    ),
    (
        "1 Corinthians 15:3-7",
        "covers/1-corinthians",
        "Illustration inspired by 1 Corinthians",
    ),
    (
        "2 Corinthians 4:6-9",
        "covers/2-corinthians",
        "Illustration inspired by 2 Corinthians",
    ),
    (
        "Galatians 5:22-26",
        "covers/galatians",
        "Illustration inspired by Galatians",
    ),
    (
        "Ephesians 6:10-14",
        "covers/ephesians",
        "Illustration inspired by Ephesians",
    ),
    (
        "Philippians 2:5-8",
        "covers/philippians",
        "Illustration inspired by Philippians",
    ),
    (
        "Colossians 3:12-15",
        "covers/colossians",
        "Illustration inspired by Colossians",
    ),
    (
        "1 Thessalonians 4:13-17",
        "covers/1-thessalonians",
        "Illustration inspired by 1 Thessalonians",
    ),
    (
        "1 Timothy 6:6-10",
        "covers/1-timothy",
        "Illustration inspired by 1 Timothy",
    ),
    (
        "Hebrews 11:1-6",
        "covers/hebrews",
        "Illustration inspired by Hebrews",
    ),
    (
        "James 1:2-5",
        "covers/james",
        "Illustration inspired by James",
    ),
    (
        "1 Peter 2:9-12",
        "covers/1-peter",
        "Illustration inspired by 1 Peter",
    ),
    (
        "1 John 4:7-11",
        "covers/1-john",
        "Illustration inspired by 1 John",
    ),
    (
        "Jude 1:20-23",
        "covers/jude",
        "Illustration inspired by Jude",
    ),
    (
        "Revelation 22:1-5",
        "covers/revelation",
        "Illustration inspired by Revelation",
    ),
    (
        "Isaiah 55:6-9",
        "covers/isaiah",
        "Illustration inspired by Isaiah",
    ),
    (
        "Luke 15:4-7",
        "covers/luke",
        "Illustration inspired by Luke",
    ),
    (
        "Matthew 6:9-13",
        "covers/matthew",
        "Illustration inspired by Matthew",
    ),
    (
        "Genesis 22:13-16",
        "covers/genesis",
        "Illustration inspired by Genesis",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Day {
    year: i64,
    month: i64,
    day: i64,
}

impl Day {
    fn parse(value: &str) -> Option<Self> {
        if value.len() != 10 || value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' {
            return None;
        }
        let year = value.get(..4)?.parse().ok()?;
        let month = value.get(5..7)?.parse().ok()?;
        let day = value.get(8..)?.parse().ok()?;
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let lengths = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        if !(1800..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || day < 1
            || day > lengths[(month - 1) as usize]
        {
            return None;
        }
        Some(Self { year, month, day })
    }
    fn ordinal(self) -> i64 {
        let year = self.year - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let yoe = year - era * 400;
        let month = self.month + if self.month > 2 { -3 } else { 9 };
        let doy = (153 * month + 2) / 5 + self.day - 1;
        era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
    }
    fn key(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
    fn label(self) -> String {
        let weekdays = [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];
        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        format!(
            "{}, {} {}, {}",
            weekdays[(self.ordinal() + 4).rem_euclid(7) as usize],
            months[(self.month - 1) as usize],
            self.day,
            self.year
        )
    }
}

fn default_day() -> Day {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = seconds as i64 / 86400;
        let mut year = 1970 + days / 366;
        while (Day {
            year: year + 1,
            month: 1,
            day: 1,
        })
        .ordinal()
            <= days
        {
            year += 1;
        }
        let mut month = 1;
        while month < 12
            && (Day {
                year,
                month: month + 1,
                day: 1,
            })
            .ordinal()
                <= days
        {
            month += 1;
        }
        return Day {
            year,
            month,
            day: days
                - (Day {
                    year,
                    month,
                    day: 1,
                })
                .ordinal()
                + 1,
        };
    }
    // The WASM adapter supplies the browser's local civil date before rendering.
    #[cfg(target_arch = "wasm32")]
    Day {
        year: 2026,
        month: 10,
        day: 15,
    }
}

pub(super) fn page(params: &HashMap<String, String>, client: bool) -> RouteResponse {
    let date = match params.get("date") {
        Some(raw) => Day::parse(raw),
        None => Some(default_day()),
    };
    let edition = params.get("edition").map(|s| s.as_str()).unwrap_or("2");
    if date.is_none() || !matches!(edition, "1" | "2") {
        return html_response(404, "Reading date not found", "", &format!("<section class=\"recovery-page\"><h1>Reading date not found</h1><p>Use a valid calendar date and reading edition 1 or 2.</p><a href=\"{}\">Return to Today</a></section>", esc(&route_path("/", client))), client, None);
    }
    let date = date.unwrap();
    let origin = Day {
        year: 2026,
        month: 10,
        day: 15,
    };
    let readings = match edition {
        "1" => &EDITION_1_READINGS[..],
        "2" => &EDITION_2_READINGS[..],
        _ => unreachable!("edition was validated above"),
    };
    let index = (date.ordinal() - origin.ordinal()).rem_euclid(readings.len() as i64) as usize;
    let (reference, image, caption) = readings[index];
    let passage = parse_ref(reference).expect("reviewed readings reference the embedded corpus");
    let book = NAMES[passage.book];
    let chapter = passage.ch;
    let title = ref_label(&passage);
    let full_chapter = route_path(&format!("/book/{}/{chapter}", slug(passage.book)), client);
    let archive = route_path(
        &format!("/today?date={}&edition={edition}", date.key()),
        client,
    );
    let library = route_path("/library", client);
    let learning_slot = if NAMES[passage.book] == "Genesis" && (6..=8).contains(&chapter) {
        0
    } else if NAMES[passage.book] == "Matthew" && chapter == 13 {
        1
    } else if NAMES[passage.book] == "Exodus" && (25..=27).contains(&chapter) {
        2
    } else {
        date.ordinal().rem_euclid(3) as usize
    };
    let learning_card = super::learning::today_card(learning_slot, client);
    let icon =
        include_str!("../web/icons/book-open.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let mut text = String::new();
    for (number, verse) in verses(&passage) {
        text.push_str(&format!("<span class=\"v\" data-verse=\"{number}\" data-reference=\"{}\"><sup>{number}</sup>{}</span>", esc(&title), esc(verse)));
    }
    let controls = reader::listen_controls(&title, "passage");
    let rotation = match edition {
        "1" => "A curated 14-day reading rotation",
        "2" => "A curated 42-day reading rotation",
        _ => unreachable!("edition was validated above"),
    };
    let body = format!(
        r#"<section class="today-page" data-today data-today-date="{date_key}"><div class="today-hero"><div class="today-reading"><div class="today-heading"><p class="today-date"><time datetime="{date_key}">{date_label}</time></p><h1>Today</h1><section class="continuation-row continuation-row--today" data-continuation>{icon}<div data-continue-empty><p>Begin with a book</p><a href="{library}">Browse the Library →</a></div><div data-continue-saved hidden><p>Continue reading</p><span data-continue-label></span></div><a data-continue-link hidden href="{library}" aria-label="Continue reading">→</a><details class="continue-options"><summary aria-label="Saved reading options" title="Saved reading options"><span aria-hidden="true">•••</span></summary><button data-continue-clear type="button" hidden aria-label="Clear saved reading location">Clear saved location</button></details></section></div><p class="today-kicker">Today’s passage <span aria-hidden="true">·</span> King James Version</p><h2 class="today-reference">{title}</h2><div class="scripture today-scripture">{text}</div><div class="today-actions"><a class="today-read" href="{full_chapter}">{icon}<span>Read {book} {chapter}</span><span aria-hidden="true">→</span></a>{controls}</div></div><figure class="today-art"><img src="/art/{image}.webp" width="1536" height="1280" alt="{caption}" fetchpriority="high"><figcaption>{caption}</figcaption></figure></div><div class="today-support">{learning_card}</div><p class="today-rotation">{rotation} <span aria-hidden="true">·</span> <a href="{archive}">Link to this day</a></p></section>"#,
        date_key = date.key(),
        date_label = date.label(),
        title = esc(&title),
        image = esc(image),
        caption = esc(caption),
        full_chapter = esc(&full_chapter),
        library = esc(&library),
        learning_card = learning_card,
        rotation = rotation,
        archive = esc(&archive)
    );
    html_response(200, "Today · The Bible", "", &body, client, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next_day(day: Day) -> Day {
        let leap = day.year % 4 == 0 && (day.year % 100 != 0 || day.year % 400 == 0);
        let lengths = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let max_day = lengths[(day.month - 1) as usize];
        if day.day < max_day {
            Day {
                day: day.day + 1,
                ..day
            }
        } else if day.month < 12 {
            Day {
                month: day.month + 1,
                day: 1,
                ..day
            }
        } else {
            Day {
                year: day.year + 1,
                month: 1,
                day: 1,
            }
        }
    }

    fn scripture_html(body: &str) -> &str {
        let marker = "<div class=\"scripture today-scripture\">";
        let start = body
            .find(marker)
            .expect("today page has a Scripture region")
            + marker.len();
        let end = body[start..]
            .find("</div>")
            .expect("today Scripture region closes")
            + start;
        &body[start..end]
    }

    #[test]
    fn civil_dates_include_leap_boundaries_and_known_weekday() {
        assert!(Day::parse("2024-02-29").is_some());
        assert!(Day::parse("2026-02-29").is_none());
        assert!(Day::parse("2026-04-31").is_none());
        assert!(Day::parse("2026-0x-15").is_none());
        assert_eq!(
            Day::parse("2026-10-15").unwrap().label(),
            "Thursday, October 15, 2026"
        );
        assert_eq!(
            Day::parse("2026-03-09").unwrap().ordinal()
                - Day::parse("2026-03-08").unwrap().ordinal(),
            1
        );
    }

    #[test]
    fn edition_one_keeps_its_original_dated_mapping_and_links() {
        let expected = [
            "Psalms 23:1-3",
            "John 1:1-5",
            "Genesis 1:1-5",
            "Isaiah 40:28-31",
            "Matthew 6:25-30",
            "Proverbs 3:5-8",
            "Philippians 4:4-9",
            "Psalms 121:1-4",
            "Exodus 25:31-32",
            "Micah 6:6-8",
            "Luke 10:38-42",
            "Romans 12:9-13",
            "1 Corinthians 13:4-7",
            "Revelation 21:1-4",
        ];
        let mut date = Day {
            year: 2026,
            month: 10,
            day: 15,
        };
        for reference in expected {
            let response = page(
                &HashMap::from([("date".into(), date.key()), ("edition".into(), "1".into())]),
                true,
            );
            assert_eq!(response.status, 200);
            assert!(
                response.body.contains(reference),
                "{}: {reference}",
                date.key()
            );
            assert!(response.body.contains("edition=1"));
            assert!(response.body.contains("A curated 14-day reading rotation"));
            if reference == "Exodus 25:31-32" {
                assert!(response.body.contains("/art/objects/lamp.webp"));
            }
            date = next_day(date);
        }
        let a = page(
            &HashMap::from([
                ("date".into(), "2026-10-15".into()),
                ("edition".into(), "1".into()),
            ]),
            true,
        );
        let b = page(
            &HashMap::from([
                ("date".into(), "2026-10-29".into()),
                ("edition".into(), "1".into()),
            ]),
            true,
        );
        assert!(a.body.contains("data-continue-empty"));
        assert!(a.body.contains("/art/today/psalms.webp"));
        assert!(a.body.contains(&esc(&bible().books[18][22][0])));
        assert_eq!(scripture_html(&a.body), scripture_html(&b.body));
    }

    #[test]
    fn today_continue_row_sits_below_the_heading_before_daily_content() {
        let response = page(
            &HashMap::from([("date".into(), "2026-10-15".into())]),
            false,
        );
        let markup_start = response
            .body
            .find("<section class=\"today-page\"")
            .expect("today page markup exists");
        let markup = &response.body[markup_start..];
        let heading = markup.find("<h1>Today</h1>").unwrap();
        let continuation = markup.find("continuation-row--today").unwrap();
        let reference = markup.find("class=\"today-reference\"").unwrap();
        let art = markup.find("class=\"today-art\"").unwrap();
        let scripture = markup.find("class=\"scripture today-scripture\"").unwrap();
        assert!(heading < continuation);
        assert!(continuation < reference && continuation < art && continuation < scripture);
        assert_eq!(markup.matches("data-continuation").count(), 1);
    }

    #[test]
    fn edition_two_has_42_unique_valid_verbatim_readings_and_available_art() {
        use std::collections::HashSet;

        let mut references = HashSet::new();
        let mut date = Day {
            year: 2026,
            month: 10,
            day: 15,
        };
        let edition_one_references: HashSet<_> = EDITION_1_READINGS
            .iter()
            .map(|(reference, _, _)| *reference)
            .collect();
        for (reference, image, caption) in EDITION_2_READINGS {
            assert!(
                references.insert(reference),
                "duplicate reading: {reference}"
            );
            assert!(
                !edition_one_references.contains(reference),
                "edition 2 repeats an edition 1 reading: {reference}"
            );
            let passage = parse_ref(reference).expect("reading resolves in the embedded KJV");
            let expected_verses = verses(&passage);
            assert!((3..=7).contains(&expected_verses.len()), "{reference}");

            let image_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("web/art")
                .join(format!("{image}.webp"));
            assert!(
                image_path.is_file(),
                "missing artwork for {reference}: {}",
                image_path.display()
            );

            let response = page(
                &HashMap::from([("date".into(), date.key()), ("edition".into(), "2".into())]),
                true,
            );
            assert_eq!(response.status, 200, "{reference}");
            let title = ref_label(&passage);
            let expected = expected_verses
                .iter()
                .map(|(number, verse)| {
                    format!(
                        "<span class=\"v\" data-verse=\"{number}\" data-reference=\"{}\"><sup>{number}</sup>{}</span>",
                        esc(&title),
                        esc(verse)
                    )
                })
                .collect::<String>();
            assert_eq!(scripture_html(&response.body), expected, "{reference}");
            assert!(
                response
                    .body
                    .contains(&format!("src=\"/art/{image}.webp\"")),
                "missing art link for {reference}"
            );
            assert!(
                response.body.contains(&format!("alt=\"{}\"", esc(caption))),
                "missing art description for {reference}"
            );
            date = next_day(date);
        }
        assert_eq!(references.len(), 42);
    }

    #[test]
    fn edition_two_defaults_at_the_origin_and_repeats_after_42_days() {
        let origin = page(&HashMap::from([("date".into(), "2026-10-15".into())]), true);
        let repeat = page(
            &HashMap::from([
                ("date".into(), "2026-11-26".into()),
                ("edition".into(), "2".into()),
            ]),
            true,
        );
        assert_eq!(origin.status, 200);
        assert!(origin.body.contains("Genesis 6:14-16"));
        assert!(origin.body.contains("A curated 42-day reading rotation"));
        assert!(origin.body.contains("/art/learn/ark/scene.webp"));
        assert_eq!(scripture_html(&origin.body), scripture_html(&repeat.body));
        assert!(repeat.body.contains("Genesis 6:14-16"));
        assert!(repeat.body.contains("Link to this day"));
        assert!(repeat.body.contains("edition=2"));
    }

    #[test]
    fn unknown_editions_and_invalid_dates_return_a_recovery_page() {
        let unknown = page(
            &HashMap::from([
                ("date".into(), "2026-10-15".into()),
                ("edition".into(), "3".into()),
            ]),
            true,
        );
        let invalid_date = page(
            &HashMap::from([
                ("date".into(), "2026-02-29".into()),
                ("edition".into(), "1".into()),
            ]),
            true,
        );
        assert_eq!(unknown.status, 404);
        assert_eq!(invalid_date.status, 404);
        assert!(unknown.body.contains("edition 1 or 2"));
        assert!(unknown.body.contains("Return to Today"));
    }
}
