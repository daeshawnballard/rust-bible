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

// The selected guide includes at least five entries per book; narrative scenes
// with several distinct movements receive additional references. It is not an
// exhaustive index.
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
    // Coverage expansion: two new references for every book, with two further
    // entries in each of the eight scene-focused books.
    e!("genesis-abram-believes", 0, Lessons, "Abram believes the promise", "The narrative records Abram believing the Lord and that belief being counted to him for righteousness.", 15, 6),
    e!("genesis-joseph-purpose", 0, Lessons, "Joseph reflects on the harm done", "Joseph distinguishes his brothers' intent from the good he says God brought about to preserve many lives.", 50, 20),
    e!("exodus-manna", 1, Lessons, "Daily provision in the wilderness", "The Lord promises bread from heaven and gives the people a daily instruction to follow.", 16, 4),
    e!("exodus-name-of-the-lord", 1, Lessons, "The Lord names his character", "The Lord describes himself to Moses as merciful, gracious, patient, and abundant in goodness and truth.", 34, 6),
    e!("leviticus-day-of-atonement", 2, Lessons, "Cleansing on the Day of Atonement", "The instruction describes atonement and cleansing for the people from their sins before the Lord.", 16, 30),
    e!("leviticus-jubilee", 2, Lessons, "Liberty in the Jubilee year", "The Jubilee instruction calls for liberty throughout the land and each person to return to family property.", 25, 10),
    e!("numbers-priestly-blessing", 3, Lessons, "The priestly blessing", "The Lord gives Aaron and his sons words of blessing asking for care, grace, and peace for Israel.", 6, 24),
    e!("numbers-caleb-and-joshua", 3, People, "Caleb and Joshua urge trust", "Caleb and Joshua say the land can be entered if the Lord delights in Israel and is with them.", 14, 8),
    e!("deuteronomy-what-the-lord-requires", 4, Lessons, "What the Lord asks of Israel", "Moses gathers the call to fear, love, serve, and walk in the Lord's ways.", 10, 12),
    e!("deuteronomy-choose-life", 4, Lessons, "Choose life", "Moses sets life and death before Israel and urges the people to choose life by loving and obeying the Lord.", 30, 19),
    e!("joshua-sanctify-yourselves", 5, Lessons, "Prepare for the crossing", "Joshua tells the people to sanctify themselves before the Lord's promised wonders the next day.", 3, 5),
    e!("joshua-jordan-priests", 5, Places, "The priests stand in the Jordan", "The priests carrying the ark stand on dry ground in the Jordan while Israel crosses.", 3, 17),
    e!("joshua-stones-memory", 5, Lessons, "Stones to remember the crossing", "Joshua says the stones are to help later generations ask about and remember the crossing.", 4, 6),
    e!("joshua-crossing-witness", 5, Lessons, "The crossing made known", "Joshua says the crossing is to make the Lord's hand known and call the people to reverence.", 4, 24),
    e!("judges-deliverers", 6, People, "The judges deliver Israel", "The account says the Lord raised judges who delivered Israel from those who plundered them.", 2, 16),
    e!("judges-deborah-song-close", 6, Lessons, "The song of Deborah closes", "The song calls for the Lord's enemies to perish and those who love him to shine; the land then has rest.", 5, 31),
    e!("ruth-under-the-lords-wings", 7, Lessons, "Ruth seeks refuge under the Lord's wings", "Boaz blesses Ruth for seeking refuge under the wings of the Lord while describing her loyalty to Naomi.", 2, 12),
    e!("ruth-obed", 7, People, "Obed is born", "The narrative records Ruth and Boaz's son Obed and names him as Naomi's descendant.", 4, 13),
    e!("1-samuel-goliath-appears", 8, People, "Goliath challenges Israel", "The narrative introduces Goliath as a Philistine champion who comes forward from the opposing army.", 17, 4),
    e!("1-samuel-david-asks", 8, People, "David asks about the challenge", "David asks what will be done for the man who removes the reproach brought by the Philistine's challenge.", 17, 26),
    e!("1-samuel-david-refuses-armor", 8, Lessons, "David sets aside unfamiliar armor", "David tries Saul's armor, then removes it because he has not proved himself in it.", 17, 39),
    e!("1-samuel-sling-and-stone", 8, People, "David prevails with a sling and stone", "The account says David defeats the Philistine with a sling and a stone, without a sword in his hand.", 17, 50),
    e!("2-samuel-davids-house", 9, Lessons, "A promise concerning David's house", "Nathan's message speaks of a house, kingdom, and throne established for David's line.", 7, 16),
    e!("2-samuel-lord-my-rock", 9, Lessons, "The Lord as rock and fortress", "David's song calls the Lord his rock, fortress, deliverer, and refuge.", 22, 2),
    e!("1-kings-god-and-the-temple", 10, Lessons, "The temple cannot contain God", "In his prayer, Solomon asks whether even heaven can contain the Lord, much less the house he has built.", 8, 27),
    e!("1-kings-kingdom-divides", 10, Lessons, "The tribes reject Rehoboam", "The people answer Rehoboam that they have no share in David's house and depart to their tents.", 12, 16),
    e!("2-kings-prophets-warn-israel", 11, Lessons, "Israel is warned through the prophets", "The text says the Lord warned Israel and Judah through prophets and seers to turn from evil ways.", 17, 13),
    e!("2-kings-book-found", 11, People, "The book is found in the temple", "Hilkiah tells Shaphan the scribe that he has found a book of the law in the house of the Lord.", 22, 8),
    e!("1-chronicles-david-confesses", 12, People, "David confesses after the census", "David acknowledges his wrongdoing and asks that the people not bear the consequence of his sin.", 21, 17),
    e!("1-chronicles-everything-from-god", 12, Lessons, "Everything comes from God", "David says that the people's gifts came from what already belonged to the Lord.", 29, 14),
    e!("2-chronicles-jehoshaphat-prays", 13, People, "Jehoshaphat asks for help", "Jehoshaphat says the people do not know what to do and turns their eyes toward the Lord.", 20, 12),
    e!("2-chronicles-cyrus-decree", 13, Places, "Cyrus permits the return", "The closing words report Cyrus's decree allowing the people to go up to Jerusalem and build the Lord's house.", 36, 23),
    e!("ezra-temple-foundation", 14, Places, "The temple foundation is laid", "The returned community lays the foundation of the house of the Lord in Jerusalem and responds with praise.", 3, 11),
    e!("ezra-fast-at-ahava", 14, Lessons, "A fast before the journey", "Ezra proclaims a fast by the river Ahava so the people can seek a safe way for themselves and their families.", 8, 21),
    e!("nehemiah-call-to-rebuild", 15, People, "Nehemiah calls for rebuilding", "Nehemiah describes Jerusalem's distress and calls the people to rebuild its wall.", 2, 17),
    e!("nehemiah-confronts-exploitation", 15, Lessons, "Nehemiah challenges exploitation", "Nehemiah rebukes the nobles for the way they are treating their fellow Jews and urges them to stop.", 5, 9),
    e!("esther-golden-sceptre", 16, People, "The king extends the sceptre", "The king extends the golden sceptre to Esther, and she approaches to speak with him.", 5, 2),
    e!("esther-remembrance", 16, Lessons, "The days of deliverance are remembered", "The letter establishes days of feasting and giving gifts in remembrance of relief from distress.", 9, 22),
    e!("job-receives-adversity", 17, Lessons, "Job speaks amid loss", "Job asks whether people should receive good from God but refuse adversity, as he sits with his grief.", 2, 10),
    e!("job-heard-and-seen", 17, Lessons, "Job contrasts hearing and seeing", "Near the book's close, Job contrasts what he had heard with what he now says he has seen.", 42, 5),
    e!("psalms-refuge-and-strength", 18, Lessons, "God as refuge and strength", "The psalm calls God a refuge, strength, and present help in trouble.", 46, 1),
    e!("psalms-everything-that-breathes", 18, Lessons, "Let all that breathes praise", "The final psalm closes with a call for everything that has breath to praise the Lord.", 150, 6),
    e!("proverbs-trust-the-lord", 19, Lessons, "Trust the Lord with your heart", "The proverb urges trust in the Lord rather than relying only on one's own understanding.", 3, 5),
    e!("proverbs-path-directed", 19, Lessons, "The Lord directs the path", "The proverb contrasts a person's plans with the Lord's direction of that person's steps.", 16, 9),
    e!("ecclesiastes-adversity-and-prosperity", 20, Lessons, "Consider both prosperity and adversity", "The speaker tells readers to consider the day of prosperity and the day of adversity.", 7, 14),
    e!("ecclesiastes-conclusion", 20, Lessons, "The book's concluding charge", "The closing lines call the reader to fear God and keep his commandments, describing this as the whole duty of humanity.", 12, 13),
    e!("song-winter-past", 21, Lessons, "The winter has passed", "The speaker marks the passing of winter and rain as the beloved's invitation begins.", 2, 11),
    e!("song-mutual-belonging", 21, Lessons, "A language of mutual belonging", "The speaker describes a bond of mutual belonging between the beloved and the lover.", 6, 3),
    e!("isaiah-waiting-renews-strength", 22, Lessons, "Waiting on the Lord", "The poem says those who wait on the Lord renew their strength and continue without fainting.", 40, 31),
    e!("isaiah-gods-ways", 22, Lessons, "God's ways exceed human ways", "The passage compares the heavens above the earth with the distance between God's ways and human ways.", 55, 9),
    e!("jeremiah-potter-and-clay", 23, Places, "The potter's house", "The Lord compares the house of Israel to clay in the potter's hand.", 18, 6),
    e!("jeremiah-law-within", 23, Lessons, "The law written within", "The promise describes a covenant in which the Lord's law is written in the people's hearts.", 31, 33),
    e!("lamentations-yoke-in-youth", 24, Lessons, "Bearing a yoke in youth", "The poem calls it good for a person to bear a yoke in youth within its reflection on suffering and waiting.", 3, 27),
    e!("lamentations-renew-our-days", 24, Lessons, "A prayer for renewal", "The closing prayer asks the Lord to turn the people back and renew their days as before.", 5, 21),
    e!("ezekiel-dry-bones-live", 25, Lessons, "A vision of dry bones", "In a vision, dry bones receive breath and live as a sign concerning the house of Israel.", 37, 5),
    e!("ezekiel-river-brings-life", 25, Places, "A river flows from the temple", "Ezekiel's vision describes water flowing from the house and bringing life where the river goes.", 47, 9),
    e!("daniel-officials-seek-an-accusation", 26, People, "Officials secure a decree", "The officials present a decree limiting petitions to the king for thirty days, with disobedience punishable by the lions' den.", 6, 7),
    e!("daniel-sealed-den", 26, Places, "The den is sealed", "The king has Daniel placed in the lions' den, and a stone is set over its mouth.", 6, 17),
    e!("daniel-brought-from-den", 26, Miracles, "Daniel is brought up unharmed", "The narrative says Daniel is brought up from the den with no hurt found on him because he trusted in God.", 6, 23),
    e!("daniel-darius-decree", 26, People, "Darius issues a decree about God", "Darius's decree tells the peoples of his realm to fear and tremble before Daniel's God.", 6, 26),
    e!("hosea-mercy-and-knowledge", 27, Lessons, "Mercy and knowledge of God", "The prophet says the Lord desires mercy and knowledge of God rather than sacrifice alone.", 6, 6),
    e!("hosea-heal-their-backsliding", 27, Lessons, "A promise of healing", "The closing chapter speaks of the Lord healing Israel's turning away and loving freely.", 14, 4),
    e!("joel-spirit-poured-out", 28, Lessons, "The Spirit poured out", "Joel's prophecy says the Lord will pour out his spirit on all flesh and describes visions and dreams.", 2, 28),
    e!("joel-refuge-for-his-people", 28, Places, "The Lord is hope for his people", "The prophecy describes the Lord as the hope of his people and strength of the children of Israel.", 3, 16),
    e!("amos-reveals-to-prophets", 29, Lessons, "The Lord reveals his purpose", "Amos says the Lord reveals his secret to his servants the prophets before acting.", 3, 7),
    e!("amos-seek-good", 29, Lessons, "Seek good and live", "Amos calls Israel to seek good rather than evil and so live.", 5, 14),
    e!("obadiah-do-not-gloat", 30, Lessons, "Do not gloat over a brother's trouble", "The oracle warns Edom against rejoicing over Judah's distress and taking advantage of it.", 1, 12),
    e!("obadiah-kingdom-is-the-lords", 30, Lessons, "The kingdom is the Lord's", "The book's final line declares that the kingdom shall be the Lord's.", 1, 21),
    e!("jonah-flees-toward-tarshish", 31, People, "Jonah flees from his commission", "Jonah goes toward Tarshish after receiving the command to go to Nineveh.", 1, 3),
    e!("jonah-prays-from-the-fish", 31, People, "Jonah prays from the fish", "The narrative says Jonah prays to the Lord from inside the fish.", 2, 1),
    e!("jonah-nineveh-believes", 31, Places, "Nineveh responds to the warning", "The people of Nineveh believe God, proclaim a fast, and put on sackcloth after hearing Jonah.", 3, 5),
    e!("jonah-explains-his-anger", 31, Lessons, "Jonah explains his anger", "Jonah says he fled because he knew the Lord to be gracious, merciful, patient, and abundant in mercy.", 4, 2),
    e!("micah-filled-with-power", 32, People, "Micah describes his prophetic task", "Micah says he is filled with power, judgment, and might to declare Israel's transgression.", 3, 8),
    e!("micah-who-is-like-thee", 32, Lessons, "Who is like the Lord?", "The closing praise celebrates the Lord's pardon and compassion toward the remnant of his heritage.", 7, 18),
    e!("nahum-lord-is-good", 33, Lessons, "The Lord is good in trouble", "Nahum calls the Lord good, a stronghold in the day of trouble, and one who knows those who trust him.", 1, 7),
    e!("nahum-bloody-city", 33, Places, "A woe against the bloody city", "The prophecy opens a woe against Nineveh, describing the city through violence and deceit.", 3, 1),
    e!("habakkuk-stands-to-watch", 34, People, "Habakkuk waits for an answer", "Habakkuk says he will stand at his watch and look for the answer to his complaint.", 2, 1),
    e!("habakkuk-rejoices-in-the-lord", 34, Lessons, "Rejoicing amid loss", "The speaker says he will rejoice in the Lord even if the fields and herds fail.", 3, 18),
    e!("zephaniah-pure-language", 35, Lessons, "A people call on the Lord", "The prophecy says the Lord will turn the peoples to a pure language so they may call on his name together.", 3, 9),
    e!("zephaniah-rejoices-over-thee", 35, Lessons, "The Lord rejoices over his people", "The poem describes the Lord in the midst of his people as mighty to save and rejoicing over them.", 3, 17),
    e!("haggai-be-strong-and-work", 36, Lessons, "Be strong and work", "Haggai urges Zerubbabel, Joshua, and the people to be strong and work, saying the Lord is with them.", 2, 4),
    e!("haggai-latter-glory", 36, Lessons, "The latter house's glory", "The prophecy says the latter glory of the house will be greater than the former and speaks of peace.", 2, 9),
    e!("zechariah-not-by-might", 37, Lessons, "Not by might, but by the Spirit", "The message to Zerubbabel says the work will be accomplished by the Lord's spirit rather than by human might or power.", 4, 6),
    e!("zechariah-king-comes-lowly", 37, People, "A king comes lowly", "The prophecy pictures Zion's king as just, bringing salvation, and riding upon a donkey.", 9, 9),
    e!("malachi-messenger-prepares-way", 38, People, "A messenger prepares the way", "The prophecy speaks of a messenger sent to prepare the way before the Lord comes to his temple.", 3, 1),
    e!("malachi-sun-of-righteousness", 38, Lessons, "Healing for those who fear the Lord", "The prophecy promises the sun of righteousness with healing to those who fear the Lord's name.", 4, 2),
    e!("matthew-jesus-has-compassion", 39, People, "Jesus has compassion on the crowd", "Jesus sees the gathered crowd, is moved with compassion, and heals their sick.", 14, 14),
    e!("matthew-walks-on-the-sea", 39, Miracles, "Jesus walks on the sea", "The disciples see Jesus walking on the sea as he comes toward their boat during the storm.", 14, 25),
    e!("matthew-peter-cries-for-help", 39, People, "Peter cries out for help", "When Peter begins to sink, he cries out to the Lord to save him.", 14, 30),
    e!("matthew-disciples-worship", 39, Lessons, "The disciples worship Jesus", "After the boat's wind ceases, those in it worship Jesus and call him the Son of God.", 14, 33),
    e!("mark-repent-and-believe", 40, Lessons, "Repent and believe the gospel", "Jesus announces that the time is fulfilled and calls his hearers to repent and believe the gospel.", 1, 15),
    e!("mark-servant-not-served", 40, Lessons, "The Son of Man serves", "Jesus says the Son of Man came to serve and give his life as a ransom for many.", 10, 45),
    e!("luke-remember-his-words", 41, Lessons, "Remember what Jesus said", "At the empty tomb, the messengers remind the women of Jesus's words about his death and rising.", 24, 6),
    e!("luke-road-to-emmaus", 41, Places, "Two disciples travel to Emmaus", "Two disciples walk to the village called Emmaus; the passage does not settle which present-day site is meant.", 24, 13),
    e!("luke-breaking-of-bread", 41, People, "Recognized in the breaking of bread", "The two disciples recognize Jesus when he takes, blesses, breaks, and gives the bread.", 24, 30),
    e!("luke-understanding-opened", 41, Lessons, "The disciples' understanding is opened", "Jesus opens the disciples' understanding so they can understand the Scriptures.", 24, 45),
    e!("john-samaritan-woman", 42, People, "Jesus speaks with a Samaritan woman", "At Jacob's well, Jesus asks a Samaritan woman for a drink and begins a conversation about living water.", 4, 7),
    e!("john-living-water", 42, Lessons, "The promise of living water", "Jesus tells the woman that, if she knew the gift of God, she could ask him for living water.", 4, 10),
    e!("john-woman-tells-the-city", 42, People, "The woman tells her city", "The woman invites people to come see a man who told her all that she had done.", 4, 29),
    e!("john-samaritans-believe", 42, Lessons, "Many Samaritans believe", "Many Samaritans in the city believe because of the woman's testimony.", 4, 39),
    e!("acts-be-of-good-cheer", 43, Lessons, "Paul urges courage during the storm", "Paul tells those aboard to be of good cheer, saying he believes God will keep the promise made to him.", 27, 25),
    e!("acts-all-reach-land", 43, Places, "Everyone reaches land safely", "The account says all aboard escape safely to land after the ship breaks apart.", 27, 44),
    e!("acts-islanders-show-kindness", 43, People, "The islanders welcome the survivors", "The people on the island receive the shipwrecked group with kindness and build a fire against the rain and cold.", 28, 2),
    e!("acts-paul-preaches-under-guard", 43, People, "Paul receives visitors under guard", "The closing lines describe Paul living under guard and receiving those who come to him as he teaches.", 28, 30),
    e!("romans-peace-with-god", 44, Lessons, "Peace with God", "The letter connects justification by faith with peace with God through Jesus Christ.", 5, 1),
    e!("romans-no-condemnation", 44, Lessons, "No condemnation in Christ", "The letter says there is no condemnation for those in Christ Jesus who walk after the Spirit.", 8, 1),
    e!("1-corinthians-faithful-in-temptation", 45, Lessons, "God is faithful in temptation", "The letter says God is faithful and provides a way to endure temptation.", 10, 13),
    e!("1-corinthians-steadfast-work", 45, Lessons, "Steadfast work in the Lord", "Following its teaching about resurrection, the letter urges steadfast work, saying labour in the Lord is not in vain.", 15, 58),
    e!("2-corinthians-treasure-in-clay", 46, Lessons, "Treasure in earthen vessels", "The letter describes its treasure in earthen vessels so the power may be seen as God's.", 4, 7),
    e!("2-corinthians-grace-sufficient", 46, Lessons, "Grace sufficient in weakness", "The letter recounts the Lord's answer that grace is sufficient and strength is made perfect in weakness.", 12, 9),
    e!("galatians-one-in-christ", 47, Lessons, "One in Christ", "The letter says those baptized into Christ are one in him across the distinctions named in the passage.", 3, 28),
    e!("galatians-bear-one-anothers-burdens", 47, Lessons, "Bear one another's burdens", "Paul tells believers to bear one another's burdens and so fulfill the law of Christ.", 6, 2),
    e!("ephesians-saved-by-grace", 48, Lessons, "Saved by grace through faith", "The letter says salvation is by grace through faith and is the gift of God, not a ground for boasting.", 2, 8),
    e!("ephesians-strength-in-the-lord", 48, Lessons, "Be strong in the Lord", "The letter calls readers to be strong in the Lord and in the power of his might.", 6, 10),
    e!("philippians-mind-of-christ", 49, Lessons, "The mind of Christ", "The letter urges the community to share the mind described in the passage about Christ Jesus.", 2, 5),
    e!("philippians-prayer-and-thanksgiving", 49, Lessons, "Bring requests with thanksgiving", "The letter tells readers to bring their requests to God through prayer and supplication with thanksgiving.", 4, 6),
    e!("colossians-image-of-god", 50, Lessons, "Christ as the image of God", "The letter calls Christ the image of the invisible God in its description of his supremacy.", 1, 15),
    e!("colossians-work-heartily", 50, Lessons, "Work wholeheartedly", "The letter tells servants to work heartily as serving the Lord rather than people.", 3, 23),
    e!("1-thessalonians-grief-with-hope", 51, Lessons, "Grief with hope", "The letter tells the community not to grieve as those without hope when speaking about believers who have died.", 4, 13),
    e!("1-thessalonians-give-thanks", 51, Lessons, "Give thanks in every circumstance", "The letter instructs its readers to give thanks in everything and calls this God's will in Christ.", 5, 18),
    e!("2-thessalonians-the-lord-is-faithful", 52, Lessons, "The Lord is faithful", "Paul says the Lord is faithful to establish and keep believers from evil.", 3, 3),
    e!("2-thessalonians-do-not-grow-weary", 52, Lessons, "Do not grow weary in doing good", "The letter urges believers not to grow weary in doing good.", 3, 13),
    e!("1-timothy-pray-for-all", 53, Lessons, "Pray for all people", "The letter urges prayers, intercessions, and thanks to be made for all people.", 2, 1),
    e!("1-timothy-godliness-and-contentment", 53, Lessons, "Godliness with contentment", "The letter describes godliness with contentment as great gain.", 6, 6),
    e!("2-timothy-rightly-divide", 54, Lessons, "Handle the word of truth rightly", "The letter urges Timothy to study so he may present himself approved and rightly handle the word of truth.", 2, 15),
    e!("2-timothy-finished-the-course", 54, People, "The speaker reflects on his course", "Near the letter's end, the speaker says he has fought the good fight, finished the course, and kept the faith.", 4, 7),
    e!("titus-grace-trains-us", 55, Lessons, "Grace trains a way of life", "The letter says God's grace teaches believers to reject ungodliness and live soberly, righteously, and godly.", 2, 12),
    e!("titus-saved-by-mercy", 55, Lessons, "Saved according to mercy", "The letter grounds salvation in God's mercy and speaks of renewal by the Holy Ghost.", 3, 5),
    e!("philemon-beloved-brother", 56, People, "Onesimus as a beloved brother", "The letter asks Philemon to receive Onesimus not merely as a servant but as a beloved brother.", 1, 16),
    e!("philemon-receive-him-as-me", 56, Lessons, "Receive Onesimus as a partner", "The letter asks Philemon to receive Onesimus as he would receive its speaker.", 1, 17),
    e!("hebrews-sympathetic-high-priest", 57, People, "A high priest who knows weakness", "The letter describes Jesus as a high priest able to sympathize with human weakness, yet without sin.", 4, 15),
    e!("hebrews-provoke-to-love", 57, Lessons, "Encourage love and good works", "The letter tells readers to consider how to provoke one another to love and good works.", 10, 24),
    e!("james-small-member", 58, Lessons, "The tongue is a small member", "The letter compares the tongue's small size with its ability to boast of great things.", 3, 5),
    e!("james-confess-and-pray", 58, Lessons, "Confess and pray for one another", "The letter joins confession of faults with prayer for one another and describes prayer as effective.", 5, 16),
    e!("1-peter-chosen-people", 59, Lessons, "A people called to declare God's works", "Peter describes his readers as a chosen people called to show the praises of the one who called them.", 2, 9),
    e!("1-peter-answer-with-meekness", 59, Lessons, "Answer with meekness and reverence", "Peter tells readers to be ready to answer for their hope with meekness and reverence.", 3, 15),
    e!("2-peter-add-to-faith", 60, Lessons, "Add virtue to faith", "The letter urges readers to add virtue to faith and then names further qualities for spiritual growth.", 1, 5),
    e!("2-peter-be-found-in-peace", 60, Lessons, "Be found in peace", "Peter tells readers to be diligent to be found by the Lord in peace, without spot and blameless.", 3, 14),
    e!("1-john-love-in-deed", 61, Lessons, "Love in deed and truth", "The letter says love should be expressed in deed and truth rather than in word and tongue only.", 3, 18),
    e!("1-john-purpose-of-writing", 61, Lessons, "Why the letter is written", "The writer says these things are written so believers may know they have eternal life.", 5, 13),
    e!("2-john-walking-in-love", 62, Lessons, "Walking in love means keeping the commandment", "The letter describes love as walking according to the commandments received from the beginning.", 1, 6),
    e!("2-john-face-to-face", 62, People, "The elder hopes to visit", "The elder says he has more to write but hopes to speak face to face so their joy may be full.", 1, 12),
    e!("3-john-welcoming-travellers", 63, Lessons, "Support those who travel for the truth", "The elder commends Gaius for his faithful treatment of brothers and strangers.", 1, 5),
    e!("3-john-follow-the-good", 63, Lessons, "Follow what is good", "The elder tells Gaius to imitate what is good rather than what is evil.", 1, 11),
    e!("jude-build-up-your-faith", 64, Lessons, "Build one another up in faith", "The letter urges readers to build themselves up in their most holy faith and pray in the Holy Ghost.", 1, 20),
    e!("jude-kept-from-falling", 64, Lessons, "God is able to keep you", "Jude closes by praising God as able to keep believers from falling and present them faultless.", 1, 24),
    e!("revelation-tears-wiped-away", 65, Lessons, "Tears and death are no more", "The vision describes God wiping away tears and says death, sorrow, crying, and pain will be no more.", 21, 4),
    e!("revelation-invitation-to-come", 65, Lessons, "An invitation to come", "The book closes with the Spirit and bride saying come and an invitation to those who thirst.", 22, 17),
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

#[derive(Clone, Copy)]
struct Artwork {
    path: &'static str,
    caption: &'static str,
    contextual: bool,
}

fn artwork(entry: &Entry) -> Option<Artwork> {
    let (path, caption, contextual) = match entry.id {
        // This cover depicts an Eden-like landscape. It is book-level context,
        // rather than a claim that the image reconstructs the named verse.
        "genesis-eden" => ("covers/genesis", "Contextual art inspired by Genesis", true),
        "genesis-abraham" => (
            "journey/genesis-expansion-a-abram-departs",
            "Illustration inspired by Abram's journey",
            false,
        ),
        "genesis-joseph" => (
            "journey/genesis-expansion-c-45-joseph-benjamin-embrace",
            "Illustration inspired by Joseph and Benjamin's reunion",
            false,
        ),
        "exodus-sinai" => (
            "journey/sinai-wilderness",
            "Illustration inspired by the Sinai wilderness",
            false,
        ),
        "exodus-sea" => (
            "journey/ot-exodus-red-sea-crossing",
            "Illustration inspired by Israel's sea crossing",
            false,
        ),
        "judges-deborah" => (
            "journey/ot-judges-deborah-and-barak",
            "Illustration inspired by Deborah and Barak",
            false,
        ),
        "1-samuel-david" => (
            "journey/ot-1samuel-david-goliath",
            "Illustration inspired by David and Goliath",
            false,
        ),
        "2-kings-naaman" => (
            "journey/ot-2kings-naaman-jordan",
            "Illustration inspired by Naaman's healing",
            false,
        ),
        "psalms-shepherd" => (
            "journey/ot-psalms-shepherd",
            "Illustration inspired by the shepherd psalm",
            false,
        ),
        "matthew-feeding" => (
            "journey/loaves-and-sea",
            "Illustration inspired by the feeding of the multitude",
            false,
        ),
        _ => return None,
    };
    Some(Artwork {
        path,
        caption,
        contextual,
    })
}

fn artwork_html(entry: &Entry, compact: bool) -> String {
    let Some(art) = artwork(entry) else {
        return String::new();
    };
    let size = if art.path.starts_with("covers/") {
        (768, 1152)
    } else {
        (1536, 1024)
    };
    let contextual_class = if art.contextual {
        " codex-art-contextual"
    } else {
        ""
    };
    let loading = if compact { "lazy" } else { "eager" };
    format!(
        r#"<figure class="codex-art{}"><img src="/art/{}.webp" width="{}" height="{}" alt="{}" loading="{}" decoding="async"><figcaption>{}</figcaption></figure>"#,
        contextual_class,
        esc(art.path),
        size.0,
        size.1,
        esc(art.caption),
        loading,
        esc(art.caption)
    )
}

fn index_entry(entry: &Entry, client: bool, selected: Option<&str>, hidden: bool) -> String {
    let (reference, _) = reference(entry, client);
    let href = route_path(&format!("/codex?entry={}", entry.id), client);
    let current = if selected == Some(entry.id) {
        " aria-current=\"page\""
    } else {
        ""
    };
    let hidden_attr = if hidden { " hidden" } else { "" };
    let art = artwork(entry)
        .map(|_| {
            format!(
                r#"<span class="codex-row-art">{}</span>"#,
                artwork_html(entry, true)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<article class="codex-index-entry" data-codex-card data-codex-entry-id="{}" data-codex-book="{}" data-codex-category="{}" data-codex-search="{}" id="codex-{}"{}><a class="codex-entry-row{}" href="{}" data-codex-entry="{}"{}>{}<span class="codex-row-copy"><span class="codex-kicker">{}</span><span class="codex-row-title">{}</span><span class="codex-row-reference">{}</span></span></a></article>"#,
        entry.id,
        slug(entry.book).to_ascii_lowercase(),
        entry.category.key(),
        esc(&format!(
            "{} {} {} {}",
            entry.title, entry.summary, reference, NAMES[entry.book]
        )),
        entry.id,
        hidden_attr,
        if art.is_empty() {
            " codex-entry-row-text-only"
        } else {
            ""
        },
        esc(&href),
        entry.id,
        current,
        art,
        entry.category.label(),
        esc(entry.title),
        esc(&reference)
    )
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
    format!(
        r#"<article class="codex-detail" data-codex-detail data-codex-entry-id="{}"><a class="codex-back" data-codex-back-to-index href="{}"><span aria-hidden="true">←</span> Back to entries</a>{}<p class="codex-kicker">{} · {}</p><h2>{}</h2><p class="codex-summary">{}</p><p class="codex-editorial-note"><strong>Study note</strong><span aria-hidden="true"> · </span>Editorial commentary. Scripture remains in the reader.</p><p class="codex-source-label">Source passage</p><a class="codex-reference" href="{}" data-codex-read data-codex-book="{}" data-codex-chapter="{}" data-codex-verse="{}">Read {}</a></article>"#,
        entry.id,
        esc(&route_path("/codex", client)),
        artwork_html(entry, false),
        entry.category.label(),
        esc(NAMES[entry.book]),
        esc(entry.title),
        esc(entry.summary),
        esc(&href),
        esc(&slug(entry.book)),
        entry.chapter,
        entry.verse,
        esc(&reference)
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

fn category_tabs(selected: Option<Category>) -> String {
    let categories = [
        ("all", "All", None),
        ("people", "People", Some(Category::People)),
        ("places", "Places", Some(Category::Places)),
        ("miracles", "Miracles", Some(Category::Miracles)),
        ("lessons", "Study notes", Some(Category::Lessons)),
    ];
    categories
        .iter()
        .map(|(value, label, category)| {
            let state = if selected == *category {
                " aria-pressed=\"true\" class=\"is-active\""
            } else {
                " aria-pressed=\"false\""
            };
            format!(
                r#"<button type="button" value="{}" data-codex-category-filter{}>{}</button>"#,
                value, state, label
            )
        })
        .collect::<String>()
}

/// Render the searchable book-by-book Codex index and its selected note preview.
pub(super) fn page(params: &HashMap<String, String>, client: bool) -> String {
    let requested_entry = params
        .get("entry")
        .and_then(|id| ENTRIES.iter().find(|entry| entry.id == id));
    let selected_book =
        book_param(params.get("book")).or_else(|| requested_entry.map(|entry| entry.book));
    let selected_category = category(params.get("category").map(String::as_str));
    let query = params.get("q").map(String::as_str).unwrap_or_default();
    let query_folded = query.to_lowercase();
    let matches_filters = |entry: &Entry| {
        let source = reference(entry, client).0;
        let searchable = format!(
            "{} {} {} {}",
            entry.title, entry.summary, source, NAMES[entry.book]
        )
        .to_lowercase();
        selected_book.map_or(true, |filter| filter == entry.book)
            && selected_category.map_or(true, |filter| filter == entry.category)
            && (query_folded.is_empty() || searchable.contains(&query_folded))
    };
    let selected_entry = requested_entry
        .filter(|entry| matches_filters(entry))
        .or_else(|| {
            ENTRIES
                .iter()
                .find(|entry| matches_filters(entry) && artwork(entry).is_some())
        })
        .or_else(|| ENTRIES.iter().find(|entry| matches_filters(entry)));
    let selected_id = selected_entry.map(|entry| entry.id);
    let mobile_open = params.contains_key("entry");
    let mut html = format!(
        r#"<section class="codex-page" data-codex-page data-codex-client="{}" data-codex-selected="{}" data-codex-selection-open="{}"><form class="codex-filters" data-codex-filters role="search"><div class="codex-filter-row"><label class="codex-search-label"><span class="codex-sr-only">Search the Codex</span><input type="search" data-codex-query value="{}" placeholder="Search people, places, miracles, or study notes" aria-label="Search people, places, miracles, or study notes"></label><label class="codex-book-label"><span>Book</span><select data-codex-book-filter>{}</select></label></div><nav class="codex-category-tabs" aria-label="Filter by category">{}</nav></form><div class="codex-index-layout"><section class="codex-index" aria-label="Codex entries"><p class="codex-count" data-codex-count aria-live="polite">{} entries</p><div class="codex-results" data-codex-results>"#,
        client,
        selected_id.unwrap_or_default(),
        mobile_open,
        esc(query),
        options(selected_book),
        category_tabs(selected_category),
        ENTRIES
            .iter()
            .filter(|entry| matches_filters(entry))
            .count(),
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
            .filter(|entry| matches_filters(entry))
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
            let visible = matches_filters(entry);
            html.push_str(&index_entry(entry, client, selected_id, !visible));
        }
        html.push_str("</section>");
    }
    let empty = if visible_count == 0 { "" } else { " hidden" };
    html.push_str(&format!(
        r#"</div><p class="codex-empty" data-codex-empty{}>No entries match these filters.</p></section>"#,
        empty
    ));
    html.push_str(&format!(
        r#"<section class="codex-preview" data-codex-preview aria-label="Selected Codex note" aria-live="polite"{}>"#,
        if selected_entry.is_some() { "" } else { " hidden" }
    ));
    if let Some(entry) = selected_entry {
        html.push_str(&entry_detail(entry, client));
    }
    html.push_str("</section></div></section>");
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
        "detail_html": selected.map(|entry| entry_detail(entry, client)),
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
        let mut anchors = HashSet::new();
        let mut per_book = [0usize; 66];
        let mut per_category = [0usize; 4];
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
            assert!(
                entry.summary.trim().len() >= 24,
                "summary is too thin for {}",
                entry.id
            );
            anchors.insert((entry.book, entry.chapter, entry.verse));
            per_category[match entry.category {
                Category::People => 0,
                Category::Places => 1,
                Category::Miracles => 2,
                Category::Lessons => 3,
            }] += 1;
            per_book[entry.book] += 1;
        }
        assert!(
            ENTRIES.len() >= 360,
            "expected at least 360 selected entries, got {}",
            ENTRIES.len()
        );
        const MIN_PER_BOOK: [usize; 66] = [
            6, 6, 5, 5, 5, 7, 5, 5, 8, 5, 6, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 6, 5, 5, 5, 8, 5, 5,
            5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 9, 6, 8, 8, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 6,
        ];
        assert!(
            per_book
                .iter()
                .zip(MIN_PER_BOOK)
                .all(|(count, minimum)| *count >= minimum),
            "every book must retain its expansion coverage: {per_book:?}"
        );
        assert!(
            anchors.len() >= 352,
            "expected new entries to use distinct verse anchors while preserving legacy repeats, got {} unique anchors",
            anchors.len()
        );
        assert!(
            per_category.iter().all(|count| *count > 0),
            "all four Codex categories should remain represented: {per_category:?}"
        );
    }

    #[test]
    fn every_book_has_selected_notes_without_duplicating_scripture() {
        let html = page(&HashMap::new(), true);
        for name in NAMES {
            assert!(html.contains(&format!("aria-label=\"{}\"", name)));
        }
        assert_eq!(html.matches("data-codex-card").count(), ENTRIES.len());
        assert_eq!(html.matches("data-codex-book-group=").count(), 66);
        assert!(html.contains("data-codex-selected=\"genesis-eden\""));
        assert!(html.contains("Study note"));
        assert!(!html.contains("class=\"codex-heading\""));
        assert!(!html.contains("<h1>Codex</h1>"));
        assert!(!html.contains("class=\"scripture\""));
        assert!(!html.contains("class=\"v\""));
    }

    #[test]
    fn direct_entry_keeps_the_filtered_index_and_renders_its_note_preview() {
        let params = HashMap::from([
            ("entry".to_string(), "genesis-abraham".to_string()),
            ("category".to_string(), "people".to_string()),
            ("q".to_string(), "abram".to_string()),
        ]);
        let html = page(&params, true);
        assert!(html.contains("data-codex-selected=\"genesis-abraham\""));
        assert!(html.contains("data-codex-selection-open=\"true\""));
        assert!(html.contains("value=\"abram\""));
        assert!(html.contains("value=\"people\" data-codex-category-filter aria-pressed=\"true\""));
        assert!(html.contains("/art/journey/genesis-expansion-a-abram-departs.webp"));
        assert!(html.contains("Study note"));
        assert!(html.contains("href=\"#/book/Genesis/12?resume=1#v1\" data-codex-read"));
        assert!(html.contains("href=\"#/codex\""));
        assert!(!html.contains("<h1>Codex</h1>"));
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
        assert!(body["detail_html"]
            .as_str()
            .unwrap()
            .contains("data-codex-back-to-index"));
        assert!(!body["html"]
            .as_str()
            .unwrap()
            .contains("class=\"scripture\""));
    }
}
