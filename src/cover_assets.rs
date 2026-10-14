//! Embedded production covers for the native Rust server.
pub fn get(path: &str) -> Option<&'static [u8]> {
    match path {
        "/art/covers/genesis.webp" => {
            Some(include_bytes!("../web/art/covers/genesis.webp").as_slice())
        }
        "/art/covers/exodus.webp" => {
            Some(include_bytes!("../web/art/covers/exodus.webp").as_slice())
        }
        "/art/covers/leviticus.webp" => {
            Some(include_bytes!("../web/art/covers/leviticus.webp").as_slice())
        }
        "/art/covers/numbers.webp" => {
            Some(include_bytes!("../web/art/covers/numbers.webp").as_slice())
        }
        "/art/covers/deuteronomy.webp" => {
            Some(include_bytes!("../web/art/covers/deuteronomy.webp").as_slice())
        }
        "/art/covers/joshua.webp" => {
            Some(include_bytes!("../web/art/covers/joshua.webp").as_slice())
        }
        "/art/covers/judges.webp" => {
            Some(include_bytes!("../web/art/covers/judges.webp").as_slice())
        }
        "/art/covers/ruth.webp" => Some(include_bytes!("../web/art/covers/ruth.webp").as_slice()),
        "/art/covers/1-samuel.webp" => {
            Some(include_bytes!("../web/art/covers/1-samuel.webp").as_slice())
        }
        "/art/covers/2-samuel.webp" => {
            Some(include_bytes!("../web/art/covers/2-samuel.webp").as_slice())
        }
        "/art/covers/1-kings.webp" => {
            Some(include_bytes!("../web/art/covers/1-kings.webp").as_slice())
        }
        "/art/covers/2-kings.webp" => {
            Some(include_bytes!("../web/art/covers/2-kings.webp").as_slice())
        }
        "/art/covers/1-chronicles.webp" => {
            Some(include_bytes!("../web/art/covers/1-chronicles.webp").as_slice())
        }
        "/art/covers/2-chronicles.webp" => {
            Some(include_bytes!("../web/art/covers/2-chronicles.webp").as_slice())
        }
        "/art/covers/ezra.webp" => Some(include_bytes!("../web/art/covers/ezra.webp").as_slice()),
        "/art/covers/nehemiah.webp" => {
            Some(include_bytes!("../web/art/covers/nehemiah.webp").as_slice())
        }
        "/art/covers/esther.webp" => {
            Some(include_bytes!("../web/art/covers/esther.webp").as_slice())
        }
        "/art/covers/job.webp" => Some(include_bytes!("../web/art/covers/job.webp").as_slice()),
        "/art/covers/psalms.webp" => {
            Some(include_bytes!("../web/art/covers/psalms.webp").as_slice())
        }
        "/art/covers/proverbs.webp" => {
            Some(include_bytes!("../web/art/covers/proverbs.webp").as_slice())
        }
        "/art/covers/ecclesiastes.webp" => {
            Some(include_bytes!("../web/art/covers/ecclesiastes.webp").as_slice())
        }
        "/art/covers/song-of-solomon.webp" => {
            Some(include_bytes!("../web/art/covers/song-of-solomon.webp").as_slice())
        }
        "/art/covers/isaiah.webp" => {
            Some(include_bytes!("../web/art/covers/isaiah.webp").as_slice())
        }
        "/art/covers/jeremiah.webp" => {
            Some(include_bytes!("../web/art/covers/jeremiah.webp").as_slice())
        }
        "/art/covers/lamentations.webp" => {
            Some(include_bytes!("../web/art/covers/lamentations.webp").as_slice())
        }
        "/art/covers/ezekiel.webp" => {
            Some(include_bytes!("../web/art/covers/ezekiel.webp").as_slice())
        }
        "/art/covers/daniel.webp" => {
            Some(include_bytes!("../web/art/covers/daniel.webp").as_slice())
        }
        "/art/covers/hosea.webp" => Some(include_bytes!("../web/art/covers/hosea.webp").as_slice()),
        "/art/covers/joel.webp" => Some(include_bytes!("../web/art/covers/joel.webp").as_slice()),
        "/art/covers/amos.webp" => Some(include_bytes!("../web/art/covers/amos.webp").as_slice()),
        "/art/covers/obadiah.webp" => {
            Some(include_bytes!("../web/art/covers/obadiah.webp").as_slice())
        }
        "/art/covers/jonah.webp" => Some(include_bytes!("../web/art/covers/jonah.webp").as_slice()),
        "/art/covers/micah.webp" => Some(include_bytes!("../web/art/covers/micah.webp").as_slice()),
        "/art/covers/nahum.webp" => Some(include_bytes!("../web/art/covers/nahum.webp").as_slice()),
        "/art/covers/habakkuk.webp" => {
            Some(include_bytes!("../web/art/covers/habakkuk.webp").as_slice())
        }
        "/art/covers/zephaniah.webp" => {
            Some(include_bytes!("../web/art/covers/zephaniah.webp").as_slice())
        }
        "/art/covers/haggai.webp" => {
            Some(include_bytes!("../web/art/covers/haggai.webp").as_slice())
        }
        "/art/covers/zechariah.webp" => {
            Some(include_bytes!("../web/art/covers/zechariah.webp").as_slice())
        }
        "/art/covers/malachi.webp" => {
            Some(include_bytes!("../web/art/covers/malachi.webp").as_slice())
        }
        "/art/covers/matthew.webp" => {
            Some(include_bytes!("../web/art/covers/matthew.webp").as_slice())
        }
        "/art/covers/mark.webp" => Some(include_bytes!("../web/art/covers/mark.webp").as_slice()),
        "/art/covers/luke.webp" => Some(include_bytes!("../web/art/covers/luke.webp").as_slice()),
        "/art/covers/john.webp" => Some(include_bytes!("../web/art/covers/john.webp").as_slice()),
        "/art/covers/acts.webp" => Some(include_bytes!("../web/art/covers/acts.webp").as_slice()),
        "/art/covers/romans.webp" => {
            Some(include_bytes!("../web/art/covers/romans.webp").as_slice())
        }
        "/art/covers/1-corinthians.webp" => {
            Some(include_bytes!("../web/art/covers/1-corinthians.webp").as_slice())
        }
        "/art/covers/2-corinthians.webp" => {
            Some(include_bytes!("../web/art/covers/2-corinthians.webp").as_slice())
        }
        "/art/covers/galatians.webp" => {
            Some(include_bytes!("../web/art/covers/galatians.webp").as_slice())
        }
        "/art/covers/ephesians.webp" => {
            Some(include_bytes!("../web/art/covers/ephesians.webp").as_slice())
        }
        "/art/covers/philippians.webp" => {
            Some(include_bytes!("../web/art/covers/philippians.webp").as_slice())
        }
        "/art/covers/colossians.webp" => {
            Some(include_bytes!("../web/art/covers/colossians.webp").as_slice())
        }
        "/art/covers/1-thessalonians.webp" => {
            Some(include_bytes!("../web/art/covers/1-thessalonians.webp").as_slice())
        }
        "/art/covers/2-thessalonians.webp" => {
            Some(include_bytes!("../web/art/covers/2-thessalonians.webp").as_slice())
        }
        "/art/covers/1-timothy.webp" => {
            Some(include_bytes!("../web/art/covers/1-timothy.webp").as_slice())
        }
        "/art/covers/2-timothy.webp" => {
            Some(include_bytes!("../web/art/covers/2-timothy.webp").as_slice())
        }
        "/art/covers/titus.webp" => Some(include_bytes!("../web/art/covers/titus.webp").as_slice()),
        "/art/covers/philemon.webp" => {
            Some(include_bytes!("../web/art/covers/philemon.webp").as_slice())
        }
        "/art/covers/hebrews.webp" => {
            Some(include_bytes!("../web/art/covers/hebrews.webp").as_slice())
        }
        "/art/covers/james.webp" => Some(include_bytes!("../web/art/covers/james.webp").as_slice()),
        "/art/covers/1-peter.webp" => {
            Some(include_bytes!("../web/art/covers/1-peter.webp").as_slice())
        }
        "/art/covers/2-peter.webp" => {
            Some(include_bytes!("../web/art/covers/2-peter.webp").as_slice())
        }
        "/art/covers/1-john.webp" => {
            Some(include_bytes!("../web/art/covers/1-john.webp").as_slice())
        }
        "/art/covers/2-john.webp" => {
            Some(include_bytes!("../web/art/covers/2-john.webp").as_slice())
        }
        "/art/covers/3-john.webp" => {
            Some(include_bytes!("../web/art/covers/3-john.webp").as_slice())
        }
        "/art/covers/jude.webp" => Some(include_bytes!("../web/art/covers/jude.webp").as_slice()),
        "/art/covers/revelation.webp" => {
            Some(include_bytes!("../web/art/covers/revelation.webp").as_slice())
        }
        _ => None,
    }
}
