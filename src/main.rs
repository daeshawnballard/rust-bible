//! KJV Bible reader. Data: https://raw.githubusercontent.com/thiagobodruk/bible/master/json/en_kjv.json
use axum::{extract::{Path, Query}, response::{Html, IntoResponse, Redirect, Response}, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, sync::OnceLock};

static RAW: &str = include_str!("../data/kjv.json");

const NAMES: [&str; 66] = ["Genesis","Exodus","Leviticus","Numbers","Deuteronomy","Joshua","Judges","Ruth","1 Samuel","2 Samuel","1 Kings","2 Kings","1 Chronicles","2 Chronicles","Ezra","Nehemiah","Esther","Job","Psalms","Proverbs","Ecclesiastes","Song of Solomon","Isaiah","Jeremiah","Lamentations","Ezekiel","Daniel","Hosea","Joel","Amos","Obadiah","Jonah","Micah","Nahum","Habakkuk","Zephaniah","Haggai","Zechariah","Malachi","Matthew","Mark","Luke","John","Acts","Romans","1 Corinthians","2 Corinthians","Galatians","Ephesians","Philippians","Colossians","1 Thessalonians","2 Thessalonians","1 Timothy","2 Timothy","Titus","Philemon","Hebrews","James","1 Peter","2 Peter","1 John","2 John","3 John","Jude","Revelation"];

const ABBR: &[(&str, usize)] = &[("gen",0),("ge",0),("gn",0),("ex",1),("exo",1),("exod",1),("lev",2),("lv",2),("num",3),("nm",3),("deut",4),("dt",4),("deu",4),("josh",5),("jos",5),("judg",6),("jdg",6),("rth",7),("ru",7),("1sam",8),("1sa",8),("2sam",9),("2sa",9),("1kgs",10),("1ki",10),("1kings",10),("2kgs",11),("2ki",11),("2kings",11),("1chr",12),("1ch",12),("1chron",12),("2chr",13),("2ch",13),("2chron",13),("ezr",14),("neh",15),("est",16),("esth",16),("jb",17),("ps",18),("psa",18),("psalm",18),("pss",18),("prov",19),("pr",19),("prv",19),("eccl",20),("ecc",20),("eccles",20),("qoh",20),("song",21),("sos",21),("songofsongs",21),("canticles",21),("isa",22),("is",22),("jer",23),("lam",24),("ezek",25),("eze",25),("dan",26),("dn",26),("hos",27),("joe",28),("jl",28),("am",29),("obad",30),("ob",30),("jon",31),("jnh",31),("mic",32),("nah",33),("hab",34),("zeph",35),("zep",35),("hag",36),("zech",37),("zec",37),("mal",38),("matt",39),("mt",39),("mat",39),("mk",40),("mrk",40),("mar",40),("lk",41),("luk",41),("jn",42),("jhn",42),("joh",42),("act",43),("rom",44),("rm",44),("1cor",45),("1co",45),("2cor",46),("2co",46),("gal",47),("eph",48),("phil",49),("php",49),("col",50),("1thess",51),("1th",51),("1thes",51),("2thess",52),("2th",52),("2thes",52),("1tim",53),("1ti",53),("2tim",54),("2ti",54),("tit",55),("philem",56),("phm",56),("phlm",56),("heb",57),("jas",58),("jm",58),("1pet",59),("1pe",59),("1pt",59),("2pet",60),("2pe",60),("2pt",60),("1jn",61),("1jo",61),("1joh",61),("2jn",62),("2jo",62),("3jn",63),("3jo",63),("jud",64),("jde",64),("rev",65),("re",65),("rv",65),("revelations",65)];

#[derive(Deserialize)]
struct RawBook { chapters: Vec<Vec<String>> }
struct Bible { books: Vec<Vec<Vec<String>>>, keys: HashMap<String, usize> }

fn norm(s: &str) -> String {
    let s = s.to_lowercase().replace("first ", "1").replace("second ", "2").replace("third ", "3");
    let s = s.replace("iii ", "3").replace("ii ", "2");
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn bible() -> &'static Bible {
    static B: OnceLock<Bible> = OnceLock::new();
    B.get_or_init(|| {
        let raw: Vec<RawBook> = serde_json::from_str(RAW.trim_start_matches('\u{feff}')).expect("bad json");
        let mut keys = HashMap::new();
        for (i, n) in NAMES.iter().enumerate() { keys.insert(norm(n), i); }
        for (a, i) in ABBR { keys.insert(a.to_string(), *i); }
        Bible { books: raw.into_iter().map(|b| b.chapters).collect(), keys }
    })
}

fn find_book(s: &str) -> Option<usize> {
    let k = norm(s);
    if k.is_empty() { return None; }
    let b = bible();
    if let Some(i) = b.keys.get(&k) { return Some(*i); }
    // unique prefix match on full names
    let m: Vec<usize> = (0..66).filter(|&i| norm(NAMES[i]).starts_with(&k)).collect();
    if m.len() == 1 { Some(m[0]) } else { None }
}

#[derive(Debug, Clone, Copy)]
struct Ref { book: usize, ch: usize, v1: Option<usize>, v2: Option<usize> }

fn parse_ref(input: &str) -> Option<Ref> {
    let s = input.trim();
    // split book part from trailing chapter[:verse[-verse]]
    let idx = s.char_indices().rev().find(|(_, c)| c.is_whitespace()).map(|(i, _)| i);
    let (bk, rest) = match idx {
        Some(i) => (&s[..i], s[i..].trim()),
        None => {
            // e.g. "John" alone or "jn3:16"
            let p = s.char_indices().skip(1).find(|(_, c)| c.is_ascii_digit()).map(|(i, _)| i);
            match p { Some(i) => (&s[..i], &s[i..]), None => (s, "1") }
        }
    };
    let (bk, rest) = if rest.chars().next().map_or(false, |c| c.is_ascii_digit()) { (bk, rest) } else { (s, "1") };
    let book = find_book(bk)?;
    let rest = rest.replace(['–', '—'], "-").replace('.', ":");
    let (ch, vs) = match rest.split_once(':') { Some((c, v)) => (c, Some(v)), None => (rest.as_str(), None) };
    let ch: usize = ch.trim().parse().ok()?;
    let chapters = &bible().books[book];
    if ch == 0 || ch > chapters.len() { return None; }
    let n = chapters[ch - 1].len();
    let (v1, v2) = match vs {
        None => (None, None),
        Some(v) => {
            let (a, b) = match v.split_once('-') { Some((a, b)) => (a, Some(b)), None => (v, None) };
            let a: usize = a.trim().parse().ok()?;
            let b: usize = match b { Some(b) => b.trim().parse().ok()?, None => a };
            if a == 0 || a > n || b < a { return None; }
            (Some(a), Some(b.min(n)))
        }
    };
    Some(Ref { book, ch, v1, v2 })
}

fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }
fn slug(b: usize) -> String { NAMES[b].replace(' ', "-") }
fn ref_label(r: &Ref) -> String {
    match (r.v1, r.v2) {
        (Some(a), Some(b)) if a != b => format!("{} {}:{}-{}", NAMES[r.book], r.ch, a, b),
        (Some(a), _) => format!("{} {}:{}", NAMES[r.book], r.ch, a),
        _ => format!("{} {}", NAMES[r.book], r.ch),
    }
}

const CSS: &str = r#"
:root{--bg:#fbf9f4;--fg:#222;--muted:#6b6b6b;--accent:#7a3e12;--card:#fff;--border:#e5e0d5}
@media (prefers-color-scheme:dark){:root{--bg:#16171a;--fg:#e6e3dc;--muted:#9a978f;--accent:#e0a96d;--card:#1f2024;--border:#33343a}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif}
header{border-bottom:1px solid var(--border);background:var(--card)}
.wrap{max-width:46rem;margin:0 auto;padding:1rem}
header .wrap{display:flex;flex-wrap:wrap;gap:.75rem;align-items:center;justify-content:space-between}
a{color:var(--accent);text-decoration:none}a:hover{text-decoration:underline}
.brand{font-family:Georgia,'Times New Roman',serif;font-size:1.3rem;font-weight:bold;color:var(--fg)}
form{display:flex;gap:.4rem;flex:1;min-width:15rem}
input{flex:1;padding:.45rem .6rem;border:1px solid var(--border);border-radius:6px;background:var(--bg);color:var(--fg);font-size:1rem}
button{padding:.45rem .8rem;border:0;border-radius:6px;background:var(--accent);color:var(--bg);font-size:.95rem;cursor:pointer}
h1,h2{font-family:Georgia,'Times New Roman',serif;font-weight:normal}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(9.5rem,1fr));gap:.4rem}
.grid a,.chs a{display:block;padding:.45rem .6rem;background:var(--card);border:1px solid var(--border);border-radius:6px}
.chs{display:grid;grid-template-columns:repeat(auto-fill,minmax(3.2rem,1fr));gap:.4rem;text-align:center}
.scripture{font-family:Georgia,'Iowan Old Style','Times New Roman',serif;font-size:1.2rem;line-height:1.75}
.scripture sup{color:var(--muted);font-family:system-ui,sans-serif;font-size:.65em;margin-right:.2em;vertical-align:super}
.v{scroll-margin-top:1rem}.hl{background:rgba(224,169,109,.25);border-radius:3px}
.nav{display:flex;justify-content:space-between;margin:1.5rem 0;gap:1rem}
.muted{color:var(--muted)}.res p{margin:.2rem 0 1rem}.res .scripture{font-size:1.05rem}
mark{background:rgba(224,169,109,.45);color:inherit}
"#;

fn page(title: &str, q: &str, body: &str) -> Html<String> {
    Html(format!(r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{t}</title><style>{CSS}</style></head><body><header><div class="wrap"><a class="brand" href="/">KJV Bible</a><form action="/go"><input name="q" value="{q}" placeholder="John 3:16, 1 Cor 13:4-7, or search words" aria-label="Lookup or search"><button>Go</button></form></div></header><main class="wrap">{body}</main><footer class="wrap muted" style="font-size:.85rem">King James Version (public domain).</footer></body></html>"#, t = esc(title), q = esc(q)))
}

async fn home() -> Html<String> {
    let mut s = String::from("<h2>Old Testament</h2><div class=\"grid\">");
    for i in 0..66 {
        if i == 39 { s.push_str("</div><h2>New Testament</h2><div class=\"grid\">"); }
        s.push_str(&format!("<a href=\"/book/{}\">{}</a>", slug(i), NAMES[i]));
    }
    s.push_str("</div>");
    page("KJV Bible", "", &s)
}

async fn book(Path(name): Path<String>) -> Response {
    let Some(b) = find_book(&name.replace('-', " ")) else { return not_found(&name) };
    let n = bible().books[b].len();
    let mut s = format!("<h1>{}</h1><div class=\"chs\">", NAMES[b]);
    for c in 1..=n { s.push_str(&format!("<a href=\"/book/{}/{c}\">{c}</a>", slug(b))); }
    s.push_str("</div>");
    page(NAMES[b], "", &s).into_response()
}

fn not_found(what: &str) -> Response {
    (axum::http::StatusCode::NOT_FOUND, page("Not found", what, &format!("<h1>Not found</h1><p>Couldn't find “{}”.</p>", esc(what)))).into_response()
}

#[derive(Deserialize)] struct HlQ { v: Option<String> }

async fn chapter(Path((name, ch)): Path<(String, usize)>, Query(h): Query<HlQ>) -> Response {
    let Some(b) = find_book(&name.replace('-', " ")) else { return not_found(&name) };
    let chs = &bible().books[b];
    if ch == 0 || ch > chs.len() { return not_found(&format!("{} {}", NAMES[b], ch)); }
    let (h1, h2) = h.v.as_deref().and_then(|v| {
        let (a, c) = v.split_once('-').unwrap_or((v, v));
        Some((a.parse::<usize>().ok()?, c.parse::<usize>().ok()?))
    }).unwrap_or((0, 0));
    let mut s = format!("<h1>{} {}</h1><div class=\"scripture\">", NAMES[b], ch);
    for (i, v) in chs[ch - 1].iter().enumerate() {
        let n = i + 1;
        let cls = if n >= h1 && n <= h2 { "v hl" } else { "v" };
        s.push_str(&format!("<span class=\"{cls}\" id=\"v{n}\"><sup>{n}</sup>{} </span>", esc(v)));
    }
    s.push_str("</div><nav class=\"nav\">");
    // prev / next across book boundaries
    let prev = if ch > 1 { Some((b, ch - 1)) } else if b > 0 { Some((b - 1, bible().books[b - 1].len())) } else { None };
    let next = if ch < chs.len() { Some((b, ch + 1)) } else if b < 65 { Some((b + 1, 1)) } else { None };
    s.push_str(&match prev { Some((pb, pc)) => format!("<a rel=\"prev\" href=\"/book/{}/{pc}\">← {} {pc}</a>", slug(pb), NAMES[pb]), None => "<span></span>".into() });
    s.push_str(&format!("<a href=\"/book/{}\">All chapters</a>", slug(b)));
    s.push_str(&match next { Some((nb, nc)) => format!("<a rel=\"next\" href=\"/book/{}/{nc}\">{} {nc} →</a>", slug(nb), NAMES[nb]), None => "<span></span>".into() });
    s.push_str("</nav>");
    page(&format!("{} {}", NAMES[b], ch), "", &s).into_response()
}

#[derive(Deserialize)] struct QQ { q: Option<String> }

async fn go(Query(q): Query<QQ>) -> Response {
    let q = q.q.unwrap_or_default();
    let q = q.trim();
    if q.is_empty() { return Redirect::to("/").into_response(); }
    if let Some(r) = parse_ref(q) {
        let mut url = format!("/book/{}/{}", slug(r.book), r.ch);
        if let (Some(a), Some(b)) = (r.v1, r.v2) { url.push_str(&format!("?v={a}-{b}#v{a}")); }
        // show passage page for verse refs, full chapter otherwise
        if r.v1.is_some() { return passage_page(&r, q, &url); }
        return Redirect::to(&url).into_response();
    }
    search_page(q).into_response()
}

fn verses(r: &Ref) -> Vec<(usize, &'static str)> {
    let c = &bible().books[r.book][r.ch - 1];
    let (a, b) = (r.v1.unwrap_or(1), r.v2.unwrap_or(c.len()));
    (a..=b).map(|n| (n, c[n - 1].as_str())).collect()
}

fn passage_page(r: &Ref, q: &str, url: &str) -> Response {
    let mut s = format!("<h1>{}</h1><div class=\"scripture\">", esc(&ref_label(r)));
    for (n, t) in verses(r) { s.push_str(&format!("<span class=\"v\"><sup>{n}</sup>{} </span>", esc(t))); }
    s.push_str(&format!("</div><p><a href=\"{}\">Read full chapter →</a></p>", esc(url)));
    page(&ref_label(r), q, &s).into_response()
}

const MAX_RESULTS: usize = 200;

fn search(q: &str) -> (Vec<(usize, usize, usize, &'static str)>, usize) {
    let terms: Vec<String> = q.to_lowercase().split_whitespace().map(String::from).collect();
    let mut out = Vec::new();
    let mut total = 0;
    if terms.is_empty() { return (out, 0); }
    for (b, book) in bible().books.iter().enumerate() {
        for (c, ch) in book.iter().enumerate() {
            for (v, t) in ch.iter().enumerate() {
                let lt = t.to_lowercase();
                if terms.iter().all(|w| lt.contains(w.as_str())) {
                    total += 1;
                    if out.len() < MAX_RESULTS { out.push((b, c + 1, v + 1, t.as_str())); }
                }
            }
        }
    }
    (out, total)
}

fn highlight(text: &str, q: &str) -> String {
    let mut s = esc(text);
    for w in q.split_whitespace() {
        let w = esc(w);
        let lower = s.to_lowercase();
        let wl = w.to_lowercase();
        if wl.is_empty() || lower.len() != s.len() { continue; }
        let mut r = String::new(); let mut last = 0;
        for (i, _) in lower.match_indices(&wl) {
            if i < last { continue; }
            r.push_str(&s[last..i]); r.push_str("<mark>"); r.push_str(&s[i..i + wl.len()]); r.push_str("</mark>");
            last = i + wl.len();
        }
        r.push_str(&s[last..]); s = r;
    }
    s
}

fn search_page(q: &str) -> Html<String> {
    let (res, total) = search(q);
    let mut s = format!("<h1>Search: “{}”</h1><p class=\"muted\">{} verse{} found{}.</p><div class=\"res\">", esc(q), total, if total == 1 { "" } else { "s" },
        if total > MAX_RESULTS { format!(" — showing first {MAX_RESULTS}") } else { String::new() });
    for (b, c, v, t) in res {
        s.push_str(&format!("<p><a href=\"/book/{}/{c}?v={v}-{v}#v{v}\">{} {c}:{v}</a><br><span class=\"scripture\">{}</span></p>", slug(b), NAMES[b], highlight(t, q)));
    }
    s.push_str("</div>");
    page(&format!("Search: {q}"), q, &s)
}

#[derive(Deserialize)] struct RefQ { r#ref: Option<String> }

async fn api_verse(Query(q): Query<RefQ>) -> Response {
    let input = q.r#ref.unwrap_or_default();
    match parse_ref(&input) {
        Some(r) => Json(json!({
            "reference": ref_label(&r), "book": NAMES[r.book], "chapter": r.ch,
            "verses": verses(&r).iter().map(|(n, t)| json!({"verse": n, "text": t})).collect::<Vec<_>>(),
            "text": verses(&r).iter().map(|(_, t)| *t).collect::<Vec<_>>().join(" "),
            "translation": "KJV"
        })).into_response(),
        None => (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": format!("could not parse reference: {input}")}))).into_response(),
    }
}

async fn api_search(Query(q): Query<QQ>) -> Json<serde_json::Value> {
    let q = q.q.unwrap_or_default();
    let (res, total) = search(&q);
    Json(json!({"query": q, "total": total, "returned": res.len(), "limit": MAX_RESULTS,
        "results": res.iter().map(|(b, c, v, t)| json!({"reference": format!("{} {c}:{v}", NAMES[*b]), "book": NAMES[*b], "chapter": c, "verse": v, "text": t})).collect::<Vec<_>>()}))
}

#[tokio::main]
async fn main() {
    let b = bible();
    println!("Loaded {} books", b.books.len());
    let app = Router::new()
        .route("/", get(home))
        .route("/go", get(go))
        .route("/book/{name}", get(book))
        .route("/book/{name}/{ch}", get(chapter))
        .route("/api/verse", get(api_verse))
        .route("/api/search", get(api_search));
    let l = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(l, app).await.unwrap();
}
