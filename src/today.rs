use super::*;

// Edition 1 is an immutable, finite reading sequence, not a claim of new daily content.
const READINGS: [(&str, &str, &str); 14] = [
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
    if date.is_none() || params.get("edition").is_some_and(|v| v != "1") {
        return html_response(404, "Reading date not found", "", &format!("<section class=\"recovery-page\"><h1>Reading date not found</h1><p>Use a valid calendar date and reading edition 1.</p><a href=\"{}\">Return to Today</a></section>", esc(&route_path("/", client))), client, None);
    }
    let date = date.unwrap();
    let origin = Day {
        year: 2026,
        month: 10,
        day: 15,
    };
    let index = (date.ordinal() - origin.ordinal()).rem_euclid(READINGS.len() as i64) as usize;
    let (reference, image, caption) = READINGS[index];
    let passage = parse_ref(reference).expect("reviewed readings reference the embedded corpus");
    let book = NAMES[passage.book];
    let chapter = passage.ch;
    let title = ref_label(&passage);
    let full_chapter = route_path(&format!("/book/{}/{chapter}", slug(passage.book)), client);
    let archive = route_path(&format!("/today?date={}&edition=1", date.key()), client);
    let library = route_path("/library", client);
    let explore = route_path("/explore", client);
    let icon =
        include_str!("../web/icons/book-open.svg").replace("<svg ", "<svg aria-hidden=\"true\" ");
    let mut text = String::new();
    for (number, verse) in verses(&passage) {
        text.push_str(&format!("<span class=\"v\" data-verse=\"{number}\" data-reference=\"{}\"><sup>{number}</sup>{}</span>", esc(&title), esc(verse)));
    }
    let controls = reader::listen_controls(&title, "passage");
    let body = format!(
        r#"<section class="today-page" data-today data-today-date="{date_key}"><div class="today-hero"><div class="today-reading"><p class="today-date"><time datetime="{date_key}">{date_label}</time></p><h1>Today’s reading</h1><p class="today-reference">{title} <span>·</span> King James Version</p><div class="scripture today-scripture">{text}</div><div class="today-actions"><a class="today-read" href="{full_chapter}">{icon}<span>Read {book} {chapter}</span><span aria-hidden="true">→</span></a>{controls}</div></div><figure class="today-art"><img src="/art/{image}.webp" width="1536" height="1280" alt="{caption}" fetchpriority="high"><figcaption>{caption}</figcaption></figure></div><div class="today-support"><section class="continuation-row" data-continuation>{icon}<div data-continue-empty><p>Start with a book</p><a href="{library}">Browse the Library →</a></div><div data-continue-saved hidden><p>Saved on this device</p><span data-continue-label></span></div><a data-continue-link hidden href="{library}">Continue reading →</a><button data-continue-clear type="button" hidden aria-label="Clear saved reading location">Clear</button></section><section class="today-explore">{icon}<div><a href="{explore}">Explore the Tabernacle →</a><p>An illustrated reconstruction · Exodus 25–27</p></div></section></div><p class="today-rotation">A curated 14-day reading rotation <span aria-hidden="true">·</span> <a href="{archive}">Link to this day</a></p></section>"#,
        date_key = date.key(),
        date_label = date.label(),
        title = esc(&title),
        image = esc(image),
        caption = esc(caption),
        full_chapter = esc(&full_chapter),
        library = esc(&library),
        explore = esc(&explore),
        archive = esc(&archive)
    );
    html_response(200, "Today · The Bible", "", &body, client, None)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn entire_rotation_resolves_to_exact_corpus_and_stable_dated_links() {
        for (reference, _, _) in READINGS {
            assert!(parse_ref(reference).is_some(), "{reference}");
        }
        let a = page(&HashMap::from([("date".into(), "2026-10-15".into())]), true);
        let b = page(&HashMap::from([("date".into(), "2026-10-29".into())]), true);
        assert!(a.body.contains("Psalms 23:1-3") && b.body.contains("Psalms 23:1-3"));
        assert!(a.body.contains("edition=1") && a.body.contains("data-continue-empty"));
        assert!(a.body.contains(&esc(&bible().books[18][22][0])));
    }
}
