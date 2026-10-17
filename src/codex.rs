//! A concise, reference-led companion index. Notes are editorial; KJV text stays in the reader.
use super::{bible, esc, find_book, json_response, route_path, slug, RouteResponse, NAMES};
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Category {
    People,
    Places,
    Miracles,
    Lessons,
}

impl Category {
    fn key(self) -> &'static str {
        match self {
            Self::People => "people",
            Self::Places => "places",
            Self::Miracles => "miracles",
            Self::Lessons => "lessons",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::People => "People",
            Self::Places => "Places",
            Self::Miracles => "Miracles",
            Self::Lessons => "Study note",
        }
    }
}

struct Entry {
    id: &'static str,
    book: usize,
    category: Category,
    title: &'static str,
    summary: &'static str,
    chapter: usize,
    verse: usize,
}

macro_rules! e {
    ($id:literal, $book:literal, $category:ident, $title:literal, $summary:literal, $chapter:literal, $verse:literal) => {
        Entry {
            id: $id,
            book: $book,
            category: Category::$category,
            title: $title,
            summary: $summary,
            chapter: $chapter,
            verse: $verse,
        }
    };
}

// Three or more entries per book; extra entries are concentrated in books with
// especially broad narrative or teaching material. These are selected guides,
// not an exhaustive index.
static ENTRIES: &[Entry] = &[
    e!("genesis-creation", 0, Lessons, "Creation and human vocation", "The opening account presents creation as ordered and gives humanity a role within it.", 1, 27),
    e!("genesis-eden", 0, Places, "The garden in Eden", "Eden is named as the garden setting; the text does not establish a modern map location.", 2, 8),
    e!("genesis-abraham", 0, People, "Abram's call", "Abram is called to leave his country and go toward a land God will show him.", 12, 1),
    e!("genesis-joseph", 0, People, "Joseph makes himself known", "Joseph identifies himself to his brothers after they come to Egypt for grain.", 45, 4),
    e!("exodus-moses", 1, People, "Moses is sent", "Moses is commissioned to bring the Israelites out of Egypt.", 3, 10),
    e!("exodus-sinai", 1, Places, "Mount Sinai", "The mountain is presented as the place where the people encounter the covenant at Sinai.", 19, 18),
    e!("exodus-sea", 1, Miracles, "The sea crossing", "The narrative describes the sea opening and Israel passing through on dry ground.", 14, 21),
    e!("exodus-covenant", 1, Lessons, "The covenant's opening words", "The commandments begin by recalling deliverance from Egypt.", 20, 2),
    e!("leviticus-worship", 2, Places, "The tent of meeting", "The book opens with instruction spoken from the tent of meeting.", 1, 1),
    e!("leviticus-aaron", 2, People, "Aaron and his sons", "Aaron and his sons are named for the priestly service described in the book.", 8, 2),
    e!("leviticus-neighbor", 2, Lessons, "Love your neighbour", "The instruction joins love of neighbour with the command not to seek vengeance or hold a grudge.", 19, 18),
    e!("numbers-kadesh", 3, Places, "Kadesh in the wilderness", "Kadesh is named in the account of Miriam's death during Israel's wilderness journey.", 20, 1),
    e!("numbers-balaam", 3, People, "Balaam and Balak", "Balak sends for Balaam; their encounter frames the oracles concerning Israel.", 22, 5),
    e!("numbers-serpent", 3, Miracles, "The lifted serpent", "The account says that those who looked at the raised serpent lived after the serpent bites.", 21, 9),
    e!("deuteronomy-moab", 4, Places, "The land of Moab", "Moses speaks to Israel in the land of Moab before the people enter the land.", 1, 5),
    e!("deuteronomy-moses", 4, People, "Moses addresses Israel", "The book frames its speeches as Moses' words to all Israel.", 1, 1),
    e!("deuteronomy-love", 4, Lessons, "Love God wholeheartedly", "Israel is called to love the Lord with heart, soul, and might.", 6, 5),
    e!("joshua-joshua", 5, People, "Joshua commissioned", "Joshua is told to be strong and courageous as he takes up leadership.", 1, 9),
    e!("joshua-jericho", 5, Miracles, "Jericho's walls fall", "The narrative connects the people's shout with the fall of Jericho's wall.", 6, 20),
    e!("joshua-gibeon", 5, Places, "Gibeon", "Gibeon is the setting for the battle in which Joshua asks for the day to be prolonged.", 10, 12),
    e!("judges-deborah", 6, People, "Deborah the judge", "Deborah is identified as a prophetess and judge who leads Israel at that time.", 4, 4),
    e!("judges-ophrah", 6, Places, "Ophrah", "Ophrah is named as the place where Gideon threshes wheat and receives a visitor.", 6, 11),
    e!("judges-gideon-sign", 6, Miracles, "A sign at Gideon's offering", "The narrative describes fire rising from the rock and consuming Gideon's offering.", 6, 21),
    e!("ruth-ruth", 7, People, "Ruth's pledge", "Ruth pledges loyalty to Naomi and to the people and God of Naomi.", 1, 16),
    e!("ruth-bethlehem", 7, Places, "Bethlehem", "Naomi and Ruth arrive in Bethlehem at the beginning of the barley harvest.", 1, 19),
    e!("ruth-boaz", 7, People, "Boaz greets the harvesters", "Boaz arrives from Bethlehem and greets the workers in his field.", 2, 4),
    e!("1-samuel-hannah", 8, People, "Hannah's prayer", "Hannah prays at Shiloh and explains that she has poured out her soul before the Lord.", 1, 15),
    e!("1-samuel-shiloh", 8, Places, "Shiloh", "Shiloh is named as the place of the sanctuary where Eli's sons serve.", 1, 3),
    e!("1-samuel-david", 8, People, "David and Goliath", "The account names David and Goliath and records David's confidence in the Lord.", 17, 45),
    e!("1-samuel-anointing", 8, Lessons, "The heart matters", "Samuel is told that the Lord looks on the heart rather than outward appearance.", 16, 7),
    e!("2-samuel-david", 9, People, "David made king", "The tribes of Israel come to David at Hebron and anoint him king over Israel.", 5, 3),
    e!("2-samuel-jerusalem", 9, Places, "Jerusalem becomes David's city", "David takes the strong hold of Zion, which is identified as the city of David.", 5, 7),
    e!("2-samuel-nathan", 9, People, "Nathan confronts David", "Nathan tells David a parable that exposes the injustice of his actions.", 12, 7),
    e!("1-kings-solomon", 10, People, "Solomon asks for wisdom", "Solomon asks for an understanding heart to judge the people.", 3, 9),
    e!("1-kings-carmel", 10, Places, "Mount Carmel", "Carmel is the setting of Elijah's public challenge to the prophets of Baal.", 18, 19),
    e!("1-kings-fire", 10, Miracles, "Fire at Carmel", "The account says that fire from the Lord consumes the offering, wood, stones, and water.", 18, 38),
    e!("1-kings-sheba", 10, People, "The queen of Sheba", "The queen of Sheba visits Solomon after hearing of his reputation.", 10, 1),
    e!("2-kings-elisha", 11, People, "Elisha succeeds Elijah", "Elisha asks for a double portion of Elijah's spirit before Elijah is taken up.", 2, 9),
    e!("2-kings-samaria", 11, Places, "Samaria under siege", "The king of Syria gathers his forces and besieges Samaria.", 6, 24),
    e!("2-kings-naaman", 11, People, "Naaman washes in Jordan", "Naaman follows Elisha's instruction and washes in the Jordan seven times.", 5, 14),
    e!("1-chronicles-david", 12, People, "David's reign remembered", "The account summarizes David's reign and describes him as king over all Israel.", 11, 1),
    e!("1-chronicles-jerusalem", 12, Places, "Jerusalem", "David and all Israel go to Jerusalem, identified in this verse as Jebus.", 11, 4),
    e!("1-chronicles-worship", 12, Lessons, "Give thanks", "The song calls the people to give thanks and make God's deeds known.", 16, 8),
    e!("2-chronicles-solomon", 13, People, "Solomon dedicates the temple", "Solomon stands before the altar and addresses the gathered people of Israel.", 6, 12),
    e!("2-chronicles-jerusalem", 13, Places, "The temple in Jerusalem", "Solomon's prayer places the temple in Jerusalem, the city chosen for God's name.", 6, 6),
    e!("2-chronicles-fire", 13, Miracles, "Fire at the dedication", "The narrative describes fire coming down and the glory of the Lord filling the house.", 7, 1),
    e!("ezra-jerusalem", 14, Places, "The return to Jerusalem", "The returning families travel from Babylon to Jerusalem and Judah.", 2, 1),
    e!("ezra-zerubbabel", 14, People, "Zerubbabel and Jeshua", "The text names Zerubbabel and Jeshua among the leaders of the returning community.", 2, 2),
    e!("ezra-study", 14, Lessons, "Ezra studies and teaches", "Ezra sets his heart to study, practise, and teach the law of the Lord.", 7, 10),
    e!("nehemiah-nehemiah", 15, People, "Nehemiah hears of Jerusalem", "Nehemiah receives a report about the survivors and the condition of Jerusalem.", 1, 3),
    e!("nehemiah-walls", 15, Places, "Jerusalem's walls", "The people rebuild the wall until its broken places are repaired.", 6, 15),
    e!("nehemiah-reading", 15, Lessons, "The law is explained", "The Levites read from the book and help the people understand what is read.", 8, 8),
    e!("esther-esther", 16, People, "Esther enters the royal house", "Esther is taken to the king's house in Shushan and placed under Hegai's care.", 2, 8),
    e!("esther-susa", 16, Places, "Shushan the palace", "The book locates its opening royal events in Shushan the palace.", 1, 2),
    e!("esther-mordecai", 16, People, "Mordecai urges Esther", "Mordecai asks Esther to consider whether she has come to the kingdom for this moment.", 4, 14),
    e!("job-uz", 17, Places, "The land of Uz", "Uz is named as Job's setting; the book does not locate it precisely on a modern map.", 1, 1),
    e!("job-job", 17, People, "Job's first description", "Job is introduced as a man described by the narrator as upright and God-fearing.", 1, 1),
    e!("job-wisdom", 17, Lessons, "Wisdom and understanding", "The poem distinguishes finding precious things from understanding the way of wisdom.", 28, 28),
    e!("psalms-shepherd", 18, Lessons, "The Lord as shepherd", "The speaker uses the image of a shepherd to express trust and care.", 23, 1),
    e!("psalms-jerusalem", 18, Places, "Jerusalem in song", "The psalmist describes Jerusalem as a city built closely together.", 122, 3),
    e!("psalms-david", 18, People, "David and his seed", "The closing verse names David and his seed in a passage about deliverance and mercy.", 18, 50),
    e!("psalms-lament", 18, Lessons, "A prayer in distress", "The speaker brings distress to God and asks to be heard and answered.", 13, 1),
    e!("proverbs-solomon", 19, People, "Solomon in the opening", "The book's opening identifies these proverbs with Solomon, king of Israel.", 1, 1),
    e!("proverbs-wisdom", 19, Lessons, "The beginning of knowledge", "The opening teaching connects knowledge with the fear of the Lord.", 1, 7),
    e!("proverbs-words", 19, Lessons, "Gentle words", "A soft answer is contrasted with words that stir up anger.", 15, 1),
    e!("ecclesiastes-seasons", 20, Lessons, "A time for every purpose", "The poem reflects on different seasons and activities within human life.", 3, 1),
    e!("ecclesiastes-speaker", 20, People, "The Preacher's inquiry", "The speaker is called the Preacher; this verse does not by itself resolve the figure's identity.", 1, 12),
    e!("ecclesiastes-remember", 20, Lessons, "Remember your Creator", "The speaker urges remembrance of the Creator while a person is young.", 12, 1),
    e!("song-voices", 21, People, "Voices in the poem", "The poem presents lovers' voices without securely identifying them as historical individuals.", 1, 2),
    e!("song-garden", 21, Places, "A garden in the poem", "Garden imagery recurs as a literary setting for desire and delight.", 4, 12),
    e!("song-love", 21, Lessons, "Love's strength", "The poem compares love's strength with death and its jealousy with the grave.", 8, 6),
    e!("isaiah-isaiah", 22, People, "Isaiah's call", "Isaiah sees a vision and responds when the Lord asks whom to send.", 6, 8),
    e!("isaiah-zion", 22, Places, "Zion and Jerusalem", "The vision names the daughter of Zion and describes Jerusalem under siege.", 1, 8),
    e!("isaiah-hezekiah", 22, People, "Hezekiah prays", "Hezekiah turns toward the wall and prays after receiving a message of illness.", 38, 2),
    e!("isaiah-healing", 22, Miracles, "Hezekiah's recovery", "Isaiah announces that Hezekiah will recover and go up to the house of the Lord.", 38, 5),
    e!("jeremiah-jeremiah", 23, People, "Jeremiah's call", "Jeremiah is told that he was known and appointed as a prophet before birth.", 1, 5),
    e!("jeremiah-baruch", 23, People, "Baruch writes the scroll", "Baruch writes the words Jeremiah dictates on a scroll.", 36, 4),
    e!("jeremiah-jerusalem", 23, Places, "Jerusalem warned", "Jeremiah is told to stand in the court of the Lord's house and speak to Judah's cities.", 26, 2),
    e!("lamentations-jerusalem", 24, Places, "Jerusalem mourned", "The poem depicts Jerusalem as lonely after the city's losses.", 1, 1),
    e!("lamentations-hope", 24, Lessons, "Hope amid grief", "The speaker recalls the Lord's mercies while describing affliction and waiting.", 3, 22),
    e!("lamentations-wait", 24, Lessons, "Quiet waiting", "The poem presents waiting quietly for salvation as one response to suffering.", 3, 26),
    e!("ezekiel-ezekiel", 25, People, "Ezekiel among the exiles", "Ezekiel dates his vision to the fifth year of King Jehoiachin's captivity.", 1, 2),
    e!("ezekiel-chebar", 25, Places, "The river Chebar", "The opening vision is set by the river Chebar among the captives.", 1, 1),
    e!("ezekiel-heart", 25, Lessons, "A new heart", "The promise speaks of a new heart and a new spirit for the people.", 36, 26),
    e!("daniel-daniel", 26, People, "Daniel in Babylon", "Daniel and his companions are named among the captives brought to Babylon.", 1, 6),
    e!("daniel-babylon", 26, Places, "Babylon", "The narrative places Daniel's early exile in Babylon.", 1, 1),
    e!("daniel-lions", 26, Miracles, "Daniel in the lions' den", "Daniel says that God sent an angel to shut the lions' mouths.", 6, 22),
    e!("daniel-prayer", 26, Lessons, "Daniel gives thanks", "Daniel continues his regular prayer and thanksgiving after the decree is signed.", 6, 10),
    e!("hosea-hosea", 27, People, "Hosea's family sign", "The book introduces Hosea and describes his marriage as part of its prophetic sign.", 1, 2),
    e!("hosea-gomer", 27, People, "Gomer", "Gomer is named as the wife Hosea takes in the opening sign-act.", 1, 3),
    e!("hosea-jezreel", 27, Places, "The valley of Jezreel", "Jezreel is named in a prophecy concerning the house of Jehu and Israel.", 1, 5),
    e!("joel-joel", 28, People, "Joel's message", "The book introduces its message as the word of the Lord that came to Joel.", 1, 1),
    e!("joel-zion", 28, Places, "Zion", "The people are called to sound an alarm in Zion in the face of the coming day.", 2, 1),
    e!("joel-return", 28, Lessons, "Return with the whole heart", "The call to return includes fasting, weeping, and mourning, with the heart as its focus.", 2, 12),
    e!("amos-amos", 29, People, "Amos the herdsman", "Amos is identified as a herdsman and gatherer of sycomore fruit.", 7, 14),
    e!("amos-bethel", 29, Places, "Bethel", "The prophet is told not to prophesy at Bethel, described as the king's chapel.", 7, 13),
    e!("amos-justice", 29, Lessons, "Let justice run", "The prophet calls for judgment and righteousness to flow like waters and a stream.", 5, 24),
    e!("obadiah-obadiah", 30, People, "The prophet named Obadiah", "The superscription names Obadiah, while the book gives little biographical detail.", 1, 1),
    e!("obadiah-edom", 30, Places, "Edom", "The oracle is addressed to Edom and concerns its conduct toward Judah.", 1, 1),
    e!("obadiah-zion", 30, Places, "Mount Zion", "The closing vision places deliverance on Mount Zion and speaks of the Lord's kingdom.", 1, 17),
    e!("jonah-jonah", 31, People, "Jonah is sent", "Jonah is told to go to Nineveh and cry against the city.", 1, 2),
    e!("jonah-nineveh", 31, Places, "Nineveh", "Nineveh is described as a great city to which Jonah is sent.", 1, 2),
    e!("jonah-great-fish", 31, Miracles, "The great fish", "The narrative says the Lord prepared a great fish to swallow Jonah.", 1, 17),
    e!("micah-micah", 32, People, "Micah's message", "The book identifies Micah as a prophet from Moresheth and names the kings of his time.", 1, 1),
    e!("micah-bethlehem", 32, Places, "Bethlehem Ephratah", "The oracle names Bethlehem Ephratah among Judah's thousands.", 5, 2),
    e!("micah-justice", 32, Lessons, "Do justly and walk humbly", "The verse joins doing justly, loving mercy, and walking humbly with God.", 6, 8),
    e!("nahum-nahum", 33, People, "Nahum's vision", "The superscription names Nahum the Elkoshite; further biographical details are not supplied here.", 1, 1),
    e!("nahum-nineveh", 33, Places, "Nineveh's fall", "The prophecy addresses Nineveh and announces judgment against the city.", 1, 1),
    e!("nahum-good-news", 33, Lessons, "Good tidings of peace", "The poem calls Judah to keep its feasts and vows after announcing that the wicked will pass away.", 1, 15),
    e!("habakkuk-habakkuk", 34, People, "Habakkuk's question", "Habakkuk asks how long he must cry out about violence without a response.", 1, 2),
    e!("habakkuk-chaldeans", 34, Places, "The Chaldeans", "The answer describes the Chaldeans as a bitter and hasty nation.", 1, 6),
    e!("habakkuk-faith", 34, Lessons, "The just shall live by faith", "The vision contrasts a proud soul with the just person who lives by faith.", 2, 4),
    e!("zephaniah-zephaniah", 35, People, "Zephaniah and his lineage", "The opening traces Zephaniah's lineage through Hizkiah and dates his work to Josiah's reign.", 1, 1),
    e!("zephaniah-jerusalem", 35, Places, "Jerusalem's gates", "The prophecy names Jerusalem's fish gate, second quarter, and hills as places of coming trouble.", 1, 10),
    e!("zephaniah-seek", 35, Lessons, "Seek the Lord", "The people are urged to seek the Lord, righteousness, and meekness.", 2, 3),
    e!("haggai-haggai", 36, People, "Haggai speaks to the leaders", "Haggai brings a message to Zerubbabel and Joshua during the second year of Darius.", 1, 1),
    e!("haggai-jerusalem", 36, Places, "The house in Jerusalem", "The prophet asks whether it is time for the people to live in finished houses while the Lord's house lies waste.", 1, 4),
    e!("haggai-work", 36, Lessons, "Consider your ways", "Haggai calls the people to consider their priorities and resume the work.", 1, 7),
    e!("zechariah-zechariah", 37, People, "Zechariah and Joshua", "The vision presents Joshua the high priest before the angel of the Lord.", 3, 1),
    e!("zechariah-jerusalem", 37, Places, "Jerusalem in the visions", "The vision describes Jerusalem as a city that will be inhabited without walls.", 2, 4),
    e!("zechariah-justice", 37, Lessons, "Justice and mercy", "The people are told to speak truth and show judgment and mercy to one another.", 7, 9),
    e!("malachi-worship", 38, Lessons, "Honour in worship", "The prophet challenges the priests over offerings that fail to show honour.", 1, 6),
    e!("malachi-justice", 38, Lessons, "Act justly", "The prophecy names justice, faithfulness, and oppression among the matters God will judge.", 3, 5),
    e!("malachi-tithe", 38, Lessons, "The storehouse", "The passage calls the people to bring all the tithes into the storehouse.", 3, 10),
    e!("matthew-jesus", 39, People, "Jesus is named", "The opening identifies Jesus as the Messiah and son of David and Abraham.", 1, 1),
    e!("matthew-bethlehem", 39, Places, "Bethlehem", "The chief priests and scribes identify Bethlehem in answer to Herod's question.", 2, 6),
    e!("matthew-feeding", 39, Miracles, "The loaves and fishes", "The account says Jesus blesses and breaks the food, and the disciples distribute it.", 14, 19),
    e!("matthew-sermon", 39, Lessons, "The Beatitudes begin", "The sermon opens by describing the blessedness of the poor in spirit.", 5, 3),
    e!("matthew-john-baptist", 39, People, "John the Baptist", "John appears preaching in the wilderness of Judaea and calling people to repentance.", 3, 1),
    e!("mark-jesus", 40, People, "Jesus begins preaching", "Mark opens with Jesus preaching the gospel of the kingdom of God in Galilee.", 1, 14),
    e!("mark-capernaum", 40, Places, "Capernaum", "Jesus enters Capernaum and teaches in the synagogue on the sabbath.", 1, 21),
    e!("mark-deaf-man", 40, Miracles, "A man who cannot hear", "Jesus heals a man who is deaf and has a speech impediment.", 7, 32),
    e!("mark-service", 40, Lessons, "Greatness through service", "Jesus contrasts seeking status with becoming servant of all.", 10, 43),
    e!("luke-jesus", 41, People, "Jesus reads in Nazareth", "Jesus reads from Isaiah in the synagogue and speaks of the passage's fulfilment.", 4, 21),
    e!("luke-nazareth", 41, Places, "Nazareth", "Jesus comes to Nazareth, where he was brought up, and enters the synagogue.", 4, 16),
    e!("luke-widow-son", 41, Miracles, "The widow's son at Nain", "Jesus speaks to a dead young man, and the narrative says he sits up and speaks.", 7, 14),
    e!("luke-mercy", 41, Lessons, "Love your enemies", "Jesus teaches his hearers to love their enemies and do good to those who hate them.", 6, 27),
    e!("john-jesus", 42, People, "The Word made flesh", "The prologue says the Word was made flesh and dwelt among people.", 1, 14),
    e!("john-cana", 42, Places, "Cana of Galilee", "Cana is named as the setting of the wedding where Jesus' first sign takes place.", 2, 1),
    e!("john-water-wine", 42, Miracles, "Water becomes wine", "The narrative identifies the change of water into wine as the beginning of Jesus' signs.", 2, 9),
    e!("john-nicodemus", 42, People, "Nicodemus visits Jesus", "Nicodemus comes to Jesus by night and addresses him as a teacher from God.", 3, 2),
    e!("acts-peter", 43, People, "Peter addresses the crowd", "Peter stands with the eleven and explains the events of Pentecost to the crowd.", 2, 14),
    e!("acts-jerusalem", 43, Places, "The church in Jerusalem", "The early believers continue together in Jerusalem, sharing worship and daily life.", 2, 46),
    e!("acts-pentecost", 43, Miracles, "Pentecost", "The account describes divided tongues like fire and the disciples speaking in other languages.", 2, 3),
    e!("acts-paul", 43, People, "Paul's conversion", "Saul asks who is speaking, and the voice identifies itself as Jesus.", 9, 5),
    e!("romans-paul", 44, People, "Paul's greeting", "Paul introduces himself as a servant of Jesus Christ and an apostle.", 1, 1),
    e!("romans-rome", 44, Places, "The church in Rome", "Paul addresses the letter to all in Rome who are called as saints.", 1, 7),
    e!("romans-renewal", 44, Lessons, "Be transformed", "The letter calls readers to present themselves to God and be transformed by renewed understanding.", 12, 2),
    e!("1-corinthians-paul", 45, People, "Paul and Sosthenes", "The opening names Paul and Sosthenes as the letter's senders.", 1, 1),
    e!("1-corinthians-corinth", 45, Places, "The church at Corinth", "The letter addresses the church of God at Corinth and its members.", 1, 2),
    e!("1-corinthians-love", 45, Lessons, "Love is patient", "The passage describes love through patience, kindness, and the absence of envy and boasting.", 13, 4),
    e!("2-corinthians-paul", 46, People, "Paul's greeting", "Paul and Timothy address the church of God at Corinth.", 1, 1),
    e!("2-corinthians-titus", 46, People, "Titus brings encouragement", "Paul says Titus' arrival brought comfort and joy amid affliction.", 7, 6),
    e!("2-corinthians-reconciliation", 46, Lessons, "The ministry of reconciliation", "The letter describes God reconciling people and entrusting a message of reconciliation.", 5, 19),
    e!("galatians-paul", 47, People, "Paul's opening", "Paul identifies himself as an apostle and names Jesus Christ and God the Father.", 1, 1),
    e!("galatians-antioch", 47, Places, "Antioch", "Paul recounts confronting Peter at Antioch over conduct that affected Gentile believers.", 2, 11),
    e!("galatians-fruit", 47, Lessons, "The fruit of the Spirit", "The letter lists love, joy, peace, and other qualities as fruit of the Spirit.", 5, 22),
    e!("ephesians-paul", 48, People, "Paul's greeting", "The letter opens with Paul's name and addresses the faithful in Ephesus.", 1, 1),
    e!("ephesians-ephesus", 48, Places, "Ephesus", "The salutation locates its original addressees in Ephesus.", 1, 1),
    e!("ephesians-unity", 48, Lessons, "Unity in the body", "The letter urges readers to keep unity through humility, patience, and peace.", 4, 3),
    e!("philippians-paul", 49, People, "Paul and Timothy", "The opening names Paul and Timothy as servants of Jesus Christ.", 1, 1),
    e!("philippians-philippi", 49, Places, "Philippi", "Paul addresses the saints, bishops, and deacons at Philippi.", 1, 1),
    e!("philippians-humility", 49, Lessons, "Look to others' interests", "The letter asks readers to consider others' interests as well as their own.", 2, 4),
    e!("colossians-paul", 50, People, "Paul and Timothy", "The letter opens with Paul as an apostle and Timothy as a brother.", 1, 1),
    e!("colossians-colossae", 50, Places, "Colossae", "The letter addresses the saints and faithful brethren in Christ at Colossae.", 1, 2),
    e!("colossians-epaphras", 50, People, "Epaphras", "Epaphras is described as a faithful minister who made the Colossians' love known.", 1, 7),
    e!("1-thessalonians-paul", 51, People, "Paul, Silvanus, and Timothy", "The opening names Paul, Silvanus, and Timothy as the senders.", 1, 1),
    e!("1-thessalonians-thessalonica", 51, Places, "Thessalonica", "The letter addresses the church of the Thessalonians.", 1, 1),
    e!("1-thessalonians-encourage", 51, Lessons, "Encourage one another", "The letter tells the community to comfort and build one another up.", 5, 11),
    e!("2-thessalonians-paul", 52, People, "The letter's senders", "Paul, Silvanus, and Timothy are named in the greeting.", 1, 1),
    e!("2-thessalonians-thessalonica", 52, Places, "The Thessalonian church", "The greeting addresses the church of the Thessalonians in God the Father and Jesus Christ.", 1, 1),
    e!("2-thessalonians-steadfast", 52, Lessons, "Stand fast", "The letter urges readers to stand fast and hold to the teachings they received.", 2, 15),
    e!("1-timothy-paul", 53, People, "Paul writes to Timothy", "The opening identifies Paul as sender and Timothy as a true son in the faith.", 1, 2),
    e!("1-timothy-ephesus", 53, Places, "Timothy in Ephesus", "Paul says he left Timothy in Ephesus to address teaching problems there.", 1, 3),
    e!("1-timothy-godliness", 53, Lessons, "Train in godliness", "The letter values godliness as useful for life now and the life to come.", 4, 8),
    e!("2-timothy-paul", 54, People, "Paul's final charge", "Paul identifies himself as a prisoner and apostle in his greeting to Timothy.", 1, 1),
    e!("2-timothy-timothy", 54, People, "Timothy's sincere faith", "Paul recalls the faith that first lived in Timothy's grandmother Lois and mother Eunice.", 1, 5),
    e!("2-timothy-scripture", 54, Lessons, "Scripture equips", "The letter says Scripture is useful for teaching and correction and equips the servant of God.", 3, 16),
    e!("titus-paul", 55, People, "Paul and Titus", "The greeting names Paul as sender and Titus as a son after the common faith.", 1, 4),
    e!("titus-crete", 55, Places, "Titus in Crete", "Paul says Titus was left in Crete to set things in order.", 1, 5),
    e!("titus-good-works", 55, Lessons, "Ready for good works", "The letter instructs believers to be ready for every good work.", 3, 1),
    e!("philemon-philemon", 56, People, "Philemon", "Paul addresses Philemon as a beloved fellow worker.", 1, 1),
    e!("philemon-onesimus", 56, People, "Onesimus", "Paul appeals for Onesimus, whom he describes as his son in the faith.", 1, 10),
    e!("philemon-house", 56, Places, "A church in a home", "The letter greets the church that meets in Philemon's house; its location is not given here.", 1, 2),
    e!("hebrews-melchizedek", 57, People, "Melchisedec", "The letter describes Melchisedec as king of Salem and priest of the most high God.", 7, 1),
    e!("hebrews-zion", 57, Places, "Mount Zion", "The passage contrasts Mount Zion and the heavenly Jerusalem with the earlier mountain scene.", 12, 22),
    e!("hebrews-faith", 57, Lessons, "Faith and assurance", "The letter describes faith as the substance of things hoped for and evidence of things unseen.", 11, 1),
    e!("james-james", 58, People, "James introduces himself", "The writer calls himself a servant of God and the Lord Jesus Christ.", 1, 1),
    e!("james-abraham", 58, People, "Abraham's faith and works", "James refers to Abraham offering Isaac when discussing faith and works.", 2, 21),
    e!("james-wisdom", 58, Lessons, "Ask for wisdom", "Readers who lack wisdom are told to ask God, who gives generously.", 1, 5),
    e!("1-peter-peter", 59, People, "Peter writes to scattered communities", "The letter identifies Peter as an apostle and addresses scattered believers.", 1, 1),
    e!("1-peter-babylon", 59, Places, "Babylon in the greeting", "The letter sends greetings from Babylon; whether the name is literal or symbolic is debated.", 5, 13),
    e!("1-peter-hope", 59, Lessons, "A living hope", "The letter connects hope with the resurrection of Jesus Christ from the dead.", 1, 3),
    e!("2-peter-peter", 60, People, "The letter identifies its sender", "The opening names Simon Peter as a servant and apostle of Jesus Christ.", 1, 1),
    e!("2-peter-transfiguration", 60, People, "The mountain witnesses", "The writer recalls hearing the heavenly voice on the holy mount.", 1, 18),
    e!("2-peter-growth", 60, Lessons, "Grow in grace", "Readers are urged to grow in grace and knowledge of Jesus Christ.", 3, 18),
    e!("1-john-light", 61, Lessons, "Walk in the light", "The letter connects walking in the light with fellowship and cleansing from sin.", 1, 7),
    e!("1-john-love", 61, Lessons, "Love one another", "The writer grounds the call to love one another in God's love.", 4, 7),
    e!("1-john-discernment", 61, Lessons, "Test the spirits", "Readers are told to test the spirits because many false prophets have gone out.", 4, 1),
    e!("2-john-elect-lady", 62, People, "The chosen lady", "The addressee is called the elect lady; whether this is an individual or a community is uncertain.", 1, 1),
    e!("2-john-truth", 62, Lessons, "Walk in truth and love", "The writer joins walking in truth with the command to love one another.", 1, 4),
    e!("2-john-teaching", 62, Lessons, "Hold to Christ's teaching", "The letter warns against those who do not abide in the teaching of Christ.", 1, 9),
    e!("3-john-gaius", 63, People, "Gaius", "The elder addresses Gaius as beloved and commends his walking in truth.", 1, 1),
    e!("3-john-diotrephes", 63, People, "Diotrephes", "The writer criticizes Diotrephes for seeking pre-eminence and rejecting the elder's authority.", 1, 9),
    e!("3-john-demetrius", 63, People, "Demetrius", "Demetrius receives a favourable testimony from others and from the truth itself.", 1, 12),
    e!("jude-jude", 64, People, "Jude introduces himself", "The writer calls himself a servant of Jesus Christ and brother of James.", 1, 1),
    e!("jude-michael", 64, People, "Michael the archangel", "Jude recounts a dispute between Michael the archangel and the devil over Moses' body.", 1, 9),
    e!("jude-keep-faith", 64, Lessons, "Contend for the faith", "The letter urges its readers to contend for the faith delivered to the saints.", 1, 3),
    e!("revelation-john", 65, People, "John on Patmos", "John says he was on the isle of Patmos because of the word of God and testimony of Jesus.", 1, 9),
    e!("revelation-seven-churches", 65, Places, "The seven churches in Asia", "The opening names seven churches in Asia to receive the message.", 1, 4),
    e!("revelation-lamb", 65, People, "The Lamb", "The vision presents the Lamb standing as though slain before the throne.", 5, 6),
    e!("revelation-witness", 65, Lessons, "The remnant under attack", "The verse describes the dragon making war on the remnant who keep the commandments of God and have the testimony of Jesus Christ.", 12, 17),
];

fn category(value: Option<&str>) -> Option<Category> {
    match value.unwrap_or_default() {
        "people" => Some(Category::People),
        "places" => Some(Category::Places),
        "miracles" => Some(Category::Miracles),
        "lessons" => Some(Category::Lessons),
        _ => None,
    }
}

fn book_param(value: Option<&String>) -> Option<usize> {
    let value = value?;
    value
        .parse::<usize>()
        .ok()
        .filter(|book| *book < NAMES.len())
        .or_else(|| find_book(value))
}

fn reference(entry: &Entry, client: bool) -> (String, String) {
    let name = format!("{} {}:{}", NAMES[entry.book], entry.chapter, entry.verse);
    let href = route_path(
        &format!(
            "/book/{}/{chapter}?resume={verse}#v{verse}",
            slug(entry.book),
            chapter = entry.chapter,
            verse = entry.verse
        ),
        client,
    );
    (name, href)
}

fn entry_card(
    entry: &Entry,
    client: bool,
    current_chapter: Option<usize>,
    selected: Option<&str>,
    hidden: bool,
) -> String {
    let (reference, href) = reference(entry, client);
    let title_href = route_path(&format!("/codex?entry={}", entry.id), client);
    let state = if selected == Some(entry.id) {
        " aria-current=\"true\""
    } else {
        ""
    };
    let hidden_attr = if hidden { " hidden" } else { "" };
    let relevance = current_chapter
        .map(|chapter| {
            if chapter == entry.chapter {
                " · In this chapter"
            } else {
                ""
            }
        })
        .unwrap_or("");
    format!(
        r#"<article class="codex-card" data-codex-card data-codex-entry-id="{}" data-codex-book="{}" data-codex-category="{}" data-codex-search="{}" id="codex-{}"{}{}><p class="codex-kicker">{} · {}</p><h3><a href="{}" data-codex-entry="{}">{}</a></h3><p class="codex-summary">{}</p><a class="codex-reference" href="{}" data-codex-read data-codex-book="{}" data-codex-chapter="{}" data-codex-verse="{}">Read {}</a>{}</article>"#,
        entry.id,
        slug(entry.book).to_ascii_lowercase(),
        entry.category.key(),
        esc(&format!(
            "{} {} {} {}",
            entry.title, entry.summary, reference, NAMES[entry.book]
        )),
        entry.id,
        state,
        hidden_attr,
        entry.category.label(),
        esc(NAMES[entry.book]),
        esc(&title_href),
        entry.id,
        esc(entry.title),
        esc(entry.summary),
        esc(&href),
        esc(&slug(entry.book)),
        entry.chapter,
        entry.verse,
        esc(&reference),
        relevance
    )
}

fn entry_detail(entry: &Entry, client: bool) -> String {
    let (reference, href) = reference(entry, client);
    let back = route_path(
        &format!("/codex?book={}", slug(entry.book).to_ascii_lowercase()),
        client,
    );
    format!(
        r#"<article class="codex-detail" data-codex-detail data-codex-entry-id="{}"><nav class="codex-breadcrumb" aria-label="Breadcrumb"><a href="{}">Codex</a><span aria-hidden="true">/</span><span>{}</span></nav><p class="codex-kicker">{} · {}</p><h1>{}</h1><p class="codex-summary">{}</p><p class="codex-editorial-note">A concise study note linked to the King James Version source. This explanation is not Scripture.</p><a class="codex-reference" href="{}" data-codex-read data-codex-book="{}" data-codex-chapter="{}" data-codex-verse="{}">Read {}</a><a class="codex-back" href="{}">Browse this book's Codex entries</a></article>"#,
        entry.id,
        esc(&back),
        esc(NAMES[entry.book]),
        entry.category.label(),
        esc(NAMES[entry.book]),
        esc(entry.title),
        esc(entry.summary),
        esc(&href),
        esc(&slug(entry.book)),
        entry.chapter,
        entry.verse,
        esc(&reference),
        esc(&back)
    )
}

fn options(selected: Option<usize>) -> String {
    let mut html = String::from("<option value=\"all\">All books</option>");
    for (index, name) in NAMES.iter().enumerate() {
        html.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            slug(index).to_ascii_lowercase(),
            if selected == Some(index) {
                " selected"
            } else {
                ""
            },
            esc(name)
        ));
    }
    html
}

/// Render the searchable book-by-book Codex page or an individual entry.
pub(super) fn page(params: &HashMap<String, String>, client: bool) -> String {
    let selected_book = book_param(params.get("book"));
    if let Some(entry) = params
        .get("entry")
        .and_then(|id| ENTRIES.iter().find(|entry| entry.id == id))
    {
        return format!(
            r#"<section class="codex-page" data-codex-page><header class="codex-heading"><p class="codex-eyebrow">A companion to the King James Version</p><h1>Codex</h1></header>{}</section>"#,
            entry_detail(entry, client)
        );
    }
    let selected_category = category(params.get("category").map(String::as_str));
    let query = params.get("q").map(String::as_str).unwrap_or_default();
    let query_folded = query.to_lowercase();
    let mut html = format!(
        r#"<section class="codex-page" data-codex-page data-codex-client="{}"><header class="codex-heading"><p class="codex-eyebrow">A companion to the King James Version</p><h1>Codex</h1><p>Explore people, places, miracles, and concise study notes alongside the books of the Bible.</p><p class="codex-editorial-note">A selected guide to the text, not an exhaustive index. Study notes are editorial explanations; Scripture remains in the reader.</p></header><form class="codex-filters" data-codex-filters role="search"><label>Find in the Codex<input type="search" data-codex-query value="{}" placeholder="Name, place, or theme"></label><label>Book<select data-codex-book-filter>{}</select></label><label>Category<select data-codex-category-filter><option value="all">All categories</option><option value="people"{}>People</option><option value="places"{}>Places</option><option value="miracles"{}>Miracles</option><option value="lessons"{}>Study notes</option></select></label></form><p class="codex-count" data-codex-count aria-live="polite"></p><div class="codex-results" data-codex-results>"#,
        client,
        esc(query),
        options(selected_book),
        if selected_category == Some(Category::People) {
            " selected"
        } else {
            ""
        },
        if selected_category == Some(Category::Places) {
            " selected"
        } else {
            ""
        },
        if selected_category == Some(Category::Miracles) {
            " selected"
        } else {
            ""
        },
        if selected_category == Some(Category::Lessons) {
            " selected"
        } else {
            ""
        },
    );
    let mut visible_count = 0;
    for book in 0..NAMES.len() {
        let book_entries = ENTRIES.iter().filter(|entry| entry.book == book);
        let book_entries: Vec<&Entry> = book_entries.collect();
        if book_entries.is_empty() {
            continue;
        }
        let visible_in_book = book_entries
            .iter()
            .filter(|entry| {
                let searchable = format!("{} {} {}", entry.title, entry.summary, NAMES[entry.book])
                    .to_lowercase();
                selected_book.map_or(true, |filter| filter == entry.book)
                    && selected_category.map_or(true, |filter| filter == entry.category)
                    && (query_folded.is_empty() || searchable.contains(&query_folded))
            })
            .count();
        visible_count += visible_in_book;
        html.push_str(&format!(
            r#"<section class="codex-book-group" data-codex-book-group="{}" aria-label="{}"{}><h2>{}</h2>"#,
            slug(book).to_ascii_lowercase(),
            esc(NAMES[book]),
            if visible_in_book == 0 { " hidden" } else { "" },
            esc(NAMES[book])
        ));
        for entry in book_entries {
            let searchable =
                format!("{} {} {}", entry.title, entry.summary, NAMES[entry.book]).to_lowercase();
            let visible = selected_book.map_or(true, |filter| filter == entry.book)
                && selected_category.map_or(true, |filter| filter == entry.category)
                && (query_folded.is_empty() || searchable.contains(&query_folded));
            html.push_str(&entry_card(entry, client, None, None, !visible));
        }
        html.push_str("</section>");
    }
    let empty = if visible_count == 0 { "" } else { " hidden" };
    html.push_str(&format!(
        r#"</div><p class="codex-empty" data-codex-empty{}>No entries match these filters.</p></section>"#,
        empty
    ));
    html
}

fn parse_current(
    params: &HashMap<String, String>,
    selected: Option<&Entry>,
) -> Result<(Option<usize>, Option<usize>, Option<usize>), &'static str> {
    let book = match params.get("book") {
        Some(_) => Some(book_param(params.get("book")).ok_or("Choose a valid book.")?),
        None => selected.map(|entry| entry.book),
    };
    let chapter = params
        .get("chapter")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "Choose a valid chapter.")?;
    let verse = params
        .get("verse")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "Choose a valid verse.")?;
    if verse.is_some() && chapter.is_none() {
        return Err("Choose a chapter with the verse.");
    }
    if let Some(book) = book {
        if let Some(chapter) = chapter {
            let Some(verses) = bible()
                .books
                .get(book)
                .and_then(|chapters| chapters.get(chapter.saturating_sub(1)))
            else {
                return Err("Choose a valid chapter.");
            };
            if chapter == 0 || verse.is_some_and(|verse| verse == 0 || verse > verses.len()) {
                return Err("Choose a valid verse.");
            }
        } else if verse.is_some() {
            return Err("Choose a valid passage.");
        }
    } else if chapter.is_some() || verse.is_some() {
        return Err("Choose a valid book.");
    }
    Ok((book, chapter, verse))
}

fn context_html(
    book: Option<usize>,
    chapter: Option<usize>,
    selected: Option<&Entry>,
    client: bool,
) -> String {
    let mut html = String::from(
        r#"<section class="codex-context" aria-label="Codex passage notes"><p class="codex-eyebrow">Codex · Study companion</p>"#,
    );
    if let Some(book) = book {
        html.push_str(&format!("<h3>{}</h3>", esc(NAMES[book])));
    } else {
        html.push_str("<h3>Selected entry</h3>");
    }
    if let Some(selected) = selected {
        html.push_str(&format!(
            r#"<p class="codex-selected-label">Selected {} · {}</p>"#,
            selected.category.label(),
            esc(NAMES[selected.book])
        ));
        let entry_href = route_path(&format!("/codex?entry={}", selected.id), client);
        html.push_str(&format!(r#"<article class="codex-card codex-card-selected" data-codex-entry-id="{}"><h4><a href="{}" data-codex-entry="{}">{}</a></h4><p>{}</p>{}</article>"#, selected.id, esc(&entry_href), selected.id, esc(selected.title), esc(selected.summary), reference_link(selected, client)));
    }
    let mut count = 0;
    if let Some(book) = book {
        html.push_str("<div class=\"codex-context-list\">");
        for entry in ENTRIES.iter().filter(|entry| {
            entry.book == book && selected.map_or(true, |chosen| chosen.id != entry.id)
        }) {
            html.push_str(&entry_card(entry, client, chapter, None, false));
            count += 1;
        }
        html.push_str("</div>");
    }
    if count == 0 && selected.is_none() {
        html.push_str("<p>There are no curated entries for this book yet.</p>");
    }
    html.push_str("</section>");
    html
}

fn reference_link(entry: &Entry, client: bool) -> String {
    let (reference, href) = reference(entry, client);
    format!(
        r#"<a class="codex-reference" href="{}" data-codex-read data-codex-book="{}" data-codex-chapter="{}" data-codex-verse="{}">Read {}</a>"#,
        esc(&href),
        esc(&slug(entry.book)),
        entry.chapter,
        entry.verse,
        esc(&reference)
    )
}

/// Return JSON for the reader companion. `entry` selects one note; book/chapter/verse
/// identify the current passage and are echoed to keep reader integrations explicit.
pub(super) fn api(params: &HashMap<String, String>, client: bool) -> RouteResponse {
    let selected = params
        .get("entry")
        .and_then(|id| ENTRIES.iter().find(|entry| entry.id == id));
    if params.contains_key("entry") && selected.is_none() {
        return json_response(404, json!({"error":"Codex entry not found."}).to_string());
    }
    if selected.is_none() && !params.contains_key("book") {
        return json_response(
            400,
            json!({"error":"Choose a book, passage, or Codex entry."}).to_string(),
        );
    }
    let (book, chapter, verse) = match parse_current(params, selected) {
        Ok(current) => current,
        Err(error) => return json_response(400, json!({"error":error}).to_string()),
    };
    let title = selected
        .map(|entry| entry.title.to_string())
        .unwrap_or_else(|| {
            book.map(|index| format!("Codex · {}", NAMES[index]))
                .unwrap_or_else(|| "Codex".into())
        });
    let html = context_html(
        book.or_else(|| selected.map(|entry| entry.book)),
        chapter,
        selected,
        client,
    );
    let reference = book.zip(chapter).map(|(book, chapter)| match verse {
        Some(verse) => format!("{} {chapter}:{verse}", NAMES[book]),
        None => format!("{} {chapter}", NAMES[book]),
    });
    json_response(200, json!({
        "html": html,
        "title": title,
        "book": book,
        "book_name": book.map(|index| NAMES[index]),
        "chapter": chapter,
        "verse": verse,
        "reference": reference,
        "entry": selected.map(|entry| entry.id),
        "entries": ENTRIES.iter().filter(|entry| Some(entry.book) == book).map(|entry| entry.id).collect::<Vec<_>>(),
    }).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_codex_anchor_is_a_verse_in_the_embedded_kjv() {
        let mut ids = HashSet::new();
        let mut per_book = [0usize; 66];
        for entry in ENTRIES {
            assert!(ids.insert(entry.id), "duplicate Codex id: {}", entry.id);
            assert!(entry.book < NAMES.len(), "invalid book for {}", entry.id);
            let verses = bible()
                .books
                .get(entry.book)
                .and_then(|chapters| chapters.get(entry.chapter.saturating_sub(1)))
                .unwrap_or_else(|| panic!("invalid chapter for {}", entry.id));
            assert!(
                entry.chapter > 0 && entry.verse > 0 && entry.verse <= verses.len(),
                "invalid verse for {}: {} {}:{}",
                entry.id,
                NAMES[entry.book],
                entry.chapter,
                entry.verse
            );
            assert!(!entry.title.trim().is_empty() && !entry.summary.trim().is_empty());
            per_book[entry.book] += 1;
        }
        assert!(
            ENTRIES.len() >= 200,
            "expected a meaningful curated set, got {}",
            ENTRIES.len()
        );
        assert!(
            per_book.iter().all(|count| *count >= 2),
            "every book needs at least two entries: {per_book:?}"
        );
    }

    #[test]
    fn every_book_has_selected_notes_without_duplicating_scripture() {
        let html = page(&HashMap::new(), true);
        for name in NAMES {
            assert!(html.contains(&format!("aria-label=\"{}\"", name)));
        }
        assert!(html.contains("Study note"));
        assert!(!html.contains("class=\"scripture\""));
        assert!(!html.contains("class=\"v\""));
    }

    #[test]
    fn reader_api_returns_book_entries_and_validates_the_live_position() {
        let params = HashMap::from([
            ("book".to_string(), "42".to_string()),
            ("chapter".to_string(), "2".to_string()),
            ("verse".to_string(), "9".to_string()),
        ]);
        let response = api(&params, true);
        assert_eq!(response.status, 200);
        let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["reference"], "John 2:9");
        assert!(body["html"]
            .as_str()
            .unwrap()
            .contains("Water becomes wine"));
        let invalid = HashMap::from([
            ("book".to_string(), "John".to_string()),
            ("chapter".to_string(), "2".to_string()),
            ("verse".to_string(), "999".to_string()),
        ]);
        assert_eq!(api(&invalid, true).status, 400);
    }

    #[test]
    fn entry_api_selects_a_note_without_returning_source_text() {
        let params = HashMap::from([("entry".to_string(), "john-water-wine".to_string())]);
        let response = api(&params, false);
        assert_eq!(response.status, 200);
        let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["entry"], "john-water-wine");
        assert!(body["html"].as_str().unwrap().contains("data-codex-read"));
        assert!(!body["html"]
            .as_str()
            .unwrap()
            .contains("class=\"scripture\""));
    }
}
