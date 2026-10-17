//! Storyboard journeys for passages whose locations are carried by the text.
//! Editorial scenery is kept out of Scripture and audio rendering.
use super::{esc, route_path, slug, NAMES};
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
    sources: &'static [Passage],
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
    intros: &'static [Intro],
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

const JORDAN_STOPS: &[Stop] = &[
    Stop {
        id: "jordan-river-crossing",
        label: "The Jordan crossing",
        anchor_chapter: 3,
        anchor_verse: 17,
        x: 28,
        y: 58,
        sources: &[
            Passage { chapter: 3, first: 1, last: 17 },
            Passage { chapter: 4, first: 10, last: 18 },
        ],
        note: "Joshua 3 recounts the people crossing Jordan on dry ground; Joshua 4:10–18 returns to the priests remaining in the river until the people pass and the waters return. The illustration compresses that sequence and does not claim a modern coordinate or surveyed route.",
    },
    Stop {
        id: "jordan-midstream-stones",
        label: "Stones from the river",
        anchor_chapter: 4,
        anchor_verse: 9,
        x: 48,
        y: 49,
        sources: &[Passage { chapter: 4, first: 1, last: 9 }],
        note: "Joshua 4:1–9 describes twelve stones taken from where the priests stood, carried across, and a second set placed in the midst of Jordan. The text distinguishes those actions; the artwork is a schematic storyboard, not a measured map.",
    },
    Stop {
        id: "jordan-gilgal-memorial",
        label: "Gilgal memorial",
        anchor_chapter: 4,
        anchor_verse: 20,
        x: 83,
        y: 40,
        sources: &[Passage { chapter: 4, first: 19, last: 24 }],
        note: "Joshua 4:19–24 says the people encamped in Gilgal, east of Jericho, and Joshua set up the stones there as a memorial. The illustration does not establish Gilgal's precise modern position or a surveyed path.",
    },
];

const DAVID_STOPS: &[Stop] = &[
    Stop {
        id: "david-camp",
        label: "The armies in the valley",
        anchor_chapter: 17,
        anchor_verse: 2,
        x: 18,
        y: 37,
        sources: &[Passage { chapter: 17, first: 1, last: 39 }],
        note: "First Samuel 17:1–39 introduces the opposing armies by the valley of Elah, Goliath's challenge, David's arrival with provisions, and his exchange with Saul. These successive narrative moments are grouped as one storyboard phase; the drawing does not specify where each conversation occurred.",
    },
    Stop {
        id: "david-brook",
        label: "Stones from the brook",
        anchor_chapter: 17,
        anchor_verse: 40,
        x: 38,
        y: 69,
        sources: &[Passage { chapter: 17, first: 40, last: 40 }],
        note: "First Samuel 17:40 says David chose five smooth stones out of the brook before drawing near to the Philistine. The text does not identify a modern brook or exact location.",
    },
    Stop {
        id: "david-valley-confrontation",
        label: "David and Goliath",
        anchor_chapter: 17,
        anchor_verse: 48,
        x: 58,
        y: 53,
        sources: &[Passage { chapter: 17, first: 41, last: 51 }],
        note: "First Samuel 17:41–51 recounts the confrontation in the valley, from the two drawing near through Goliath's fall. The scene is a narrative illustration and does not claim precise positions or measurements.",
    },
    Stop {
        id: "david-philistine-retreat",
        label: "The Philistines flee",
        anchor_chapter: 17,
        anchor_verse: 52,
        x: 83,
        y: 29,
        sources: &[Passage { chapter: 17, first: 52, last: 53 }],
        note: "First Samuel 17:52–53 says Israel and Judah pursued the Philistines and then returned to spoil their tents. The next verses move to Jerusalem and Saul's questions, so they are left unassigned here.",
    },
];

const DAVID_INTROS: &[Intro] = &[Intro {
    sources: &[Passage { chapter: 17, first: 54, last: 58 }],
    title: "After the battle",
    note: "First Samuel 17:54–58 tells of David bringing Goliath's head to Jerusalem and the ensuing exchange with Saul. This aftermath is not assigned to the retreat hotspot, and the text does not locate the whole exchange at one of the illustrated stops.",
}];

const DANIEL_STOPS: &[Stop] = &[
    Stop {
        id: "daniel-palace-prayer",
        label: "The decree and Daniel's prayer",
        anchor_chapter: 6,
        anchor_verse: 10,
        x: 18,
        y: 40,
        sources: &[Passage { chapter: 6, first: 1, last: 10 }],
        note: "Daniel 6:1–9 describes the officials and the decree; verse 10 says Daniel went to his own house and prayed. These story phases are grouped, but Daniel's house is not identified as part of the palace.",
    },
    Stop {
        id: "daniel-decree-court",
        label: "The accusation before the king",
        anchor_chapter: 6,
        anchor_verse: 13,
        x: 45,
        y: 38,
        sources: &[Passage { chapter: 6, first: 11, last: 15 }],
        note: "Daniel 6:11–15 describes the officials reporting Daniel's prayer to the king and Darius trying to deliver him. The illustration marks this court episode without asserting an exact palace plan.",
    },
    Stop {
        id: "daniel-lions-den",
        label: "The lions' den",
        anchor_chapter: 6,
        anchor_verse: 16,
        x: 68,
        y: 79,
        sources: &[
            Passage { chapter: 6, first: 16, last: 18 },
            Passage { chapter: 6, first: 24, last: 24 },
        ],
        note: "Daniel 6:16–18 tells of Daniel being cast into the den and sealed in; verse 24 returns to that den when the accusers are brought there after dawn. The separate moments are grouped without adding details about the den's construction.",
    },
    Stop {
        id: "daniel-dawn-at-den",
        label: "Morning at the den",
        anchor_chapter: 6,
        anchor_verse: 19,
        x: 85,
        y: 48,
        sources: &[Passage { chapter: 6, first: 19, last: 23 }],
        note: "Daniel 6:19–23 says the king went early to the den, called to Daniel, and found him unharmed. The scene follows the text without locating the den in a precise modern place.",
    },
];

const DANIEL_INTROS: &[Intro] = &[Intro {
    sources: &[Passage { chapter: 6, first: 25, last: 28 }],
    title: "The king's decree and Daniel's prosperity",
    note: "Daniel 6:25–28 records Darius's decree and Daniel prospering afterward. These closing verses do not describe another stop in the illustrated sequence, so no hotspot is marked active.",
}];

const JONAH_STOPS: &[Stop] = &[
    Stop {
        id: "jonah-joppa-harbour",
        label: "A ship at Joppa",
        anchor_chapter: 1,
        anchor_verse: 3,
        x: 15,
        y: 60,
        sources: &[Passage { chapter: 1, first: 3, last: 3 }],
        note: "Jonah 1:3 says Jonah went down to Joppa, found a ship bound for Tarshish, and paid its fare. The commission in verses 1–2 is left unassigned because the text does not place that speech at Joppa; no modern harbor layout is claimed.",
    },
    Stop {
        id: "jonah-storm-sea",
        label: "The storm at sea",
        anchor_chapter: 1,
        anchor_verse: 4,
        x: 36,
        y: 40,
        sources: &[Passage { chapter: 1, first: 4, last: 16 }],
        note: "Jonah 1:4–16 recounts the storm, the sailors' response, and Jonah being cast into the sea. The image compresses the shipboard sequence and does not trace a modern route.",
    },
    Stop {
        id: "jonah-shore",
        label: "Jonah on dry land",
        anchor_chapter: 2,
        anchor_verse: 10,
        x: 52,
        y: 70,
        sources: &[
            Passage { chapter: 1, first: 17, last: 17 },
            Passage { chapter: 2, first: 1, last: 10 },
        ],
        note: "Jonah 1:17–2:9 describes Jonah in the great fish; 2:10 says the fish vomited him upon the dry land. The hotspot anchors to that shoreward outcome while the source links preserve the preceding events; no exact coast is identified.",
    },
    Stop {
        id: "jonah-nineveh",
        label: "Nineveh",
        anchor_chapter: 3,
        anchor_verse: 3,
        x: 74,
        y: 37,
        sources: &[Passage { chapter: 3, first: 1, last: 10 }],
        note: "Jonah 3:1–10 says Jonah went into Nineveh, proclaimed its coming overthrow, and records the people's response. The named city follows the text; the illustration does not map its precise extent or modern site.",
    },
    Stop {
        id: "jonah-city-shade",
        label: "Outside the city",
        anchor_chapter: 4,
        anchor_verse: 5,
        x: 86,
        y: 65,
        sources: &[Passage { chapter: 4, first: 5, last: 11 }],
        note: "Jonah 4:5–11 places Jonah east of Nineveh, where he makes a booth and sits in its shadow as the plant and the Lord's question follow. The prior exchange in verses 1–4 is unassigned; this illustration does not claim a surveyed city boundary.",
    },
];

const JONAH_INTROS: &[Intro] = &[
    Intro {
        sources: &[Passage { chapter: 1, first: 1, last: 2 }],
        title: "The commission",
        note: "Jonah 1:1–2 records the Lord's word to Jonah and the command concerning Nineveh. The text does not identify the speaker's physical location, so no hotspot is assigned to these verses.",
    },
    Intro {
        sources: &[Passage { chapter: 4, first: 1, last: 4 }],
        title: "Jonah's displeasure",
        note: "Jonah 4:1–4 recounts Jonah's anger and his exchange with the Lord before the narrative says he went out and sat east of the city in verse 5. No physical stop is assigned to this passage.",
    },
];

const LOAVES_STOPS: &[Stop] = &[
    Stop {
        id: "loaves-isolated-shore",
        label: "The solitary place",
        anchor_chapter: 14,
        anchor_verse: 13,
        x: 18,
        y: 77,
        sources: &[Passage { chapter: 14, first: 13, last: 21 }],
        note: "Matthew 14:13–21 says Jesus went by ship to a desert place apart, the multitude followed, and he fed them. The image uses an isolated shore as an artistic setting; the text does not name a precise shoreline.",
    },
    Stop {
        id: "loaves-boat-crossing",
        label: "The boat crossing",
        anchor_chapter: 14,
        anchor_verse: 24,
        x: 49,
        y: 48,
        sources: &[
            Passage { chapter: 14, first: 22, last: 22 },
            Passage { chapter: 14, first: 24, last: 33 },
        ],
        note: "Matthew 14:22 records the disciples setting out by ship; verses 24–33 follow the boat, Jesus walking on the sea, and their arrival at him. Verse 23, where Jesus goes up a mountain to pray, is left unassigned rather than placed on the water.",
    },
    Stop {
        id: "loaves-far-shore",
        label: "The land of Gennesaret",
        anchor_chapter: 14,
        anchor_verse: 34,
        x: 80,
        y: 34,
        sources: &[Passage { chapter: 14, first: 34, last: 36 }],
        note: "Matthew 14:34–36 says they came into the land of Gennesaret and recounts people seeking healing. The scene follows the named destination without assigning it a precise modern coordinate.",
    },
];

const LOAVES_INTROS: &[Intro] = &[
    Intro {
        sources: &[Passage { chapter: 14, first: 1, last: 12 }],
        title: "Herod hears of Jesus",
        note: "Matthew 14:1–12 recounts Herod, John the Baptist, and John's death. This account precedes the journey scene and has no assigned hotspot in this storyboard.",
    },
    Intro {
        sources: &[Passage { chapter: 14, first: 23, last: 23 }],
        title: "Jesus goes up the mountain",
        note: "Matthew 14:23 says Jesus went up into a mountain apart to pray after sending the multitude away. That setting has no hotspot in this scene, so the verse remains unassigned.",
    },
];

const EMMAUS_STOPS: &[Stop] = &[
    Stop {
        id: "emmaus-empty-tomb",
        label: "The empty sepulchre",
        anchor_chapter: 24,
        anchor_verse: 12,
        x: 15,
        y: 57,
        sources: &[Passage { chapter: 24, first: 1, last: 12 }],
        note: "Luke 24:1–12 recounts the women finding the stone rolled away and the tomb empty, their report, and Peter's visit. The source range follows the empty-tomb episode without claiming that every report was spoken at the tomb.",
    },
    Stop {
        id: "emmaus-road",
        label: "The road to Emmaus",
        anchor_chapter: 24,
        anchor_verse: 13,
        x: 49,
        y: 68,
        sources: &[Passage { chapter: 24, first: 13, last: 27 }],
        note: "Luke 24:13–27 follows two disciples walking toward Emmaus and their conversation with Jesus on the way. The text names the destination and the journey, not a surveyed road or exact modern path.",
    },
    Stop {
        id: "emmaus-meal",
        label: "The meal at Emmaus",
        anchor_chapter: 24,
        anchor_verse: 30,
        x: 89,
        y: 68,
        sources: &[Passage { chapter: 24, first: 28, last: 32 }],
        note: "Luke 24:28–31 describes entering the village, the meal, and the disciples recognizing Jesus; verse 32 records their immediate reflection. The passage does not identify a particular house or modern Emmaus site.",
    },
    Stop {
        id: "emmaus-jerusalem-return",
        label: "Return to Jerusalem",
        anchor_chapter: 24,
        anchor_verse: 33,
        x: 81,
        y: 23,
        sources: &[
            Passage { chapter: 24, first: 33, last: 49 },
            Passage { chapter: 24, first: 52, last: 53 },
        ],
        note: "Luke 24:33–49 recounts the disciples returning to Jerusalem and the gathering there; verses 52–53 say the disciples returned after the events at Bethany. The intervening movement and ascension in verses 50–51 are left unassigned rather than placed in Jerusalem.",
    },
];

const EMMAUS_INTROS: &[Intro] = &[Intro {
    sources: &[Passage { chapter: 24, first: 50, last: 51 }],
    title: "Toward Bethany",
    note: "Luke 24:50–51 says Jesus led the disciples out as far as Bethany and was parted from them. This narrated movement is left unassigned because the scene has no Bethany hotspot.",
}];

const SAMARIA_STOPS: &[Stop] = &[
    Stop {
        id: "samaria-well",
        label: "Jacob's well near Sychar",
        anchor_chapter: 4,
        anchor_verse: 6,
        x: 21,
        y: 67,
        sources: &[
            Passage { chapter: 4, first: 5, last: 27 },
            Passage { chapter: 4, first: 31, last: 38 },
        ],
        note: "John 4:5–27 places Jesus at Jacob's well near Sychar and recounts the conversation there; verses 31–38 follow the disciples' return and the conversation about the harvest. The text names the well and city, but not a precise modern site.",
    },
    Stop {
        id: "samaria-town",
        label: "Sychar and its people",
        anchor_chapter: 4,
        anchor_verse: 28,
        x: 49,
        y: 34,
        sources: &[
            Passage { chapter: 4, first: 28, last: 30 },
            Passage { chapter: 4, first: 39, last: 42 },
        ],
        note: "John 4:28–30 says the woman went into the city and the people came out to Jesus; verses 39–42 recount their belief and request that he stay. The sources follow movement between town and Jesus without placing every exchange inside the town.",
    },
    Stop {
        id: "samaria-galilee-cana",
        label: "Cana of Galilee",
        anchor_chapter: 4,
        anchor_verse: 46,
        x: 93,
        y: 23,
        sources: &[Passage { chapter: 4, first: 43, last: 54 }],
        note: "John 4:43–54 says Jesus departed for Galilee, came again to Cana, and healed the nobleman's son at a distance. Cana is named in the text; the drawing does not claim a precise route or modern coordinate.",
    },
];

const SAMARIA_INTROS: &[Intro] = &[Intro {
    sources: &[Passage { chapter: 4, first: 1, last: 4 }],
    title: "Leaving Judaea",
    note: "John 4:1–4 recounts Jesus leaving Judaea and needing to pass through Samaria before the narrative names Sychar and Jacob's well. This transition has no assigned hotspot.",
}];

const PAUL_STOPS: &[Stop] = &[
    Stop {
        id: "paul-crete-haven",
        label: "Fair Havens in Crete",
        anchor_chapter: 27,
        anchor_verse: 8,
        x: 15,
        y: 32,
        sources: &[Passage { chapter: 27, first: 7, last: 13 }],
        note: "Acts 27:7–13 recounts sailing under Crete, reaching a place called Fair Havens, and the decision to sail on in hope of wintering at Phenice. The text does not say they reached Phenice before the storm; no exact modern route is drawn.",
    },
    Stop {
        id: "paul-storm-sea",
        label: "The storm at sea",
        anchor_chapter: 27,
        anchor_verse: 14,
        x: 45,
        y: 28,
        sources: &[Passage { chapter: 27, first: 14, last: 38 }],
        note: "Acts 27:14–38 follows the tempest, the ship's drift, Paul's assurance, and the crew's actions while at sea. The image is a storyboard and does not identify an exact track across the water.",
    },
    Stop {
        id: "paul-malta-shore",
        label: "The shore of Melita",
        anchor_chapter: 28,
        anchor_verse: 1,
        x: 80,
        y: 70,
        sources: &[
            Passage { chapter: 27, first: 39, last: 44 },
            Passage { chapter: 28, first: 1, last: 10 },
        ],
        note: "Acts 27:39–44 recounts the ship breaking and everyone reaching land; Acts 28:1 names the island as Melita and describes the reception and events there. The art does not identify a particular modern shoreline or building.",
    },
    Stop {
        id: "paul-rome",
        label: "Paul arrives in Rome",
        anchor_chapter: 28,
        anchor_verse: 16,
        x: 85,
        y: 19,
        sources: &[Passage { chapter: 28, first: 16, last: 31 }],
        note: "Acts 28:16–31 says Paul came to Rome, met with local Jewish leaders, and dwelt two years in his own hired house. The preceding voyage and welcome outside the city are left unassigned; the drawing does not reconstruct a Roman street or house.",
    },
];

const PAUL_INTROS: &[Intro] = &[
    Intro {
        sources: &[Passage { chapter: 27, first: 1, last: 6 }],
        title: "Before Crete",
        note: "Acts 27:1–6 recounts the voyage through Sidon and Myra before the ship sails under Crete. Those earlier stages have no assigned hotspot in this scene.",
    },
    Intro {
        sources: &[Passage { chapter: 28, first: 11, last: 15 }],
        title: "From Malta toward Rome",
        note: "Acts 28:11–15 recounts the voyage from Malta and the believers meeting Paul before Rome. These intermediate stages are left unassigned; the Rome hotspot begins at verse 16, when the party comes to Rome.",
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
        intros: &[],
    },
    Scene {
        book: 1,
        id: "wilderness",
        title: "From the sea to the wilderness",
        coverage: "Exodus 13:17–17:16",
        alt: "Illustrated wilderness story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M16 65 C20 53 22 44 24 36 S45 59 58 79 S65 62 66 52 S69 33 70 22 S80 31 88 40",
        stops: WILDERNESS_STOPS,
        intros: &[Intro { sources: &[Passage { chapter: 15, first: 22, last: 22 }], title: "Into the wilderness of Shur", note: "Exodus 15:22 says the people went from the sea into Shur and travelled three days without finding water. This transitional setting has no assigned hotspot; the next named stop is Marah." }],
    },
    Scene {
        book: 7,
        id: "ruth",
        title: "Ruth and Naomi",
        coverage: "Ruth 1–2",
        alt: "Illustrated Ruth story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M18 19 C22 38 30 51 39 59 S62 41 74 23 S75 50 70 68",
        stops: RUTH_STOPS,
        intros: &[],
    },
    Scene {
        book: 41,
        id: "bethlehem",
        title: "The early life of Jesus",
        coverage: "Luke 2",
        alt: "Illustrated Luke 2 story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M22 62 C33 57 42 53 50 49 S31 35 20 28 S63 17 80 17 S83 48 81 63",
        stops: BETHLEHEM_STOPS,
        intros: &[Intro {
            sources: &[Passage { chapter: 2, first: 1, last: 3 }],
            title: "Opening verses",
            note: "Luke 2:1–3 introduces the decree and registration before the five stops shown here. These verses are an unassigned introduction, so no stop is marked active.",
        }],
    },
    Scene {
        book: 40,
        id: "galilee",
        title: "Across the sea and back",
        coverage: "Mark 4–5:43",
        alt: "Illustrated shore and crossing story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M22 65 C35 55 44 50 55 47 S74 34 84 29 C70 46 54 49 43 46 S29 38 22 34",
        stops: GALILEE_STOPS,
        intros: &[],
    },
    Scene {
        book: 5,
        id: "jordan",
        title: "Across Jordan to Gilgal",
        coverage: "Joshua 3:1–4:24",
        alt: "Illustrated Jordan crossing and memorial phases gathered into one symbolic storyboard, not a geographic map",
        path: "M28 58 C34 53 41 50 48 49 S65 45 83 40",
        stops: JORDAN_STOPS,
        intros: &[],
    },
    Scene {
        book: 8,
        id: "david-goliath",
        title: "David and Goliath",
        coverage: "1 Samuel 17:1–53 (17:54–58 unassigned)",
        alt: "Illustrated story phases of David and Goliath gathered into one symbolic storyboard, not a geographic map",
        path: "M18 37 C25 47 31 60 38 69 S50 59 58 53 S75 38 83 29",
        stops: DAVID_STOPS,
        intros: DAVID_INTROS,
    },
    Scene {
        book: 26,
        id: "daniel",
        title: "Daniel and the lions' den",
        coverage: "Daniel 6:1–24 (6:25–28 unassigned)",
        alt: "Illustrated Daniel story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M18 40 C27 38 37 37 45 38 S60 58 68 79 S79 59 85 48",
        stops: DANIEL_STOPS,
        intros: DANIEL_INTROS,
    },
    Scene {
        book: 31,
        id: "jonah",
        title: "Jonah's journey",
        coverage: "Jonah 1:1–4:11 (1:1–2 and 4:1–4 unassigned)",
        alt: "Illustrated Jonah story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M15 60 C23 53 30 45 36 40 S45 57 52 70 S65 52 74 37 S84 49 86 65",
        stops: JONAH_STOPS,
        intros: JONAH_INTROS,
    },
    Scene {
        book: 39,
        id: "loaves-and-sea",
        title: "Bread, sea, and Gennesaret",
        coverage: "Matthew 14:1–36 (14:1–12 and 14:23 unassigned)",
        alt: "Illustrated feeding, crossing, and arrival phases gathered into one symbolic storyboard, not a geographic map",
        path: "M18 77 C29 64 39 50 49 48 S66 41 80 34",
        stops: LOAVES_STOPS,
        intros: LOAVES_INTROS,
    },
    Scene {
        book: 41,
        id: "emmaus",
        title: "The road to Emmaus",
        coverage: "Luke 24:1–53 (24:50–51 unassigned)",
        alt: "Illustrated resurrection-morning story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M15 57 C26 63 38 69 49 68 S72 70 89 68 S84 39 81 23",
        stops: EMMAUS_STOPS,
        intros: EMMAUS_INTROS,
    },
    Scene {
        book: 42,
        id: "samaria",
        title: "From Sychar to Cana",
        coverage: "John 4:1–54 (4:1–4 unassigned)",
        alt: "Illustrated well, town, and Cana story phases gathered into one symbolic storyboard, not a geographic map",
        path: "M21 67 C29 53 38 39 49 34 S76 31 93 23",
        stops: SAMARIA_STOPS,
        intros: SAMARIA_INTROS,
    },
    Scene {
        book: 43,
        id: "paul-voyage",
        title: "Paul's voyage to Rome",
        coverage: "Acts 27:1–28:31 (27:1–6 and 28:11–15 unassigned)",
        alt: "Illustrated sea voyage and arrival phases gathered into one symbolic storyboard, not a geographic map",
        path: "M15 32 C26 35 36 28 45 28 S66 50 80 70 S84 37 85 19",
        stops: PAUL_STOPS,
        intros: PAUL_INTROS,
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
                || scene
                    .intros
                    .iter()
                    .flat_map(|intro| intro.sources)
                    .any(|source| source.chapter == chapter))
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

fn in_intro(scene: &Scene, chapter: usize, verse: usize) -> Option<&'static Intro> {
    scene.intros.iter().find(|intro| {
        intro
            .sources
            .iter()
            .any(|source| passage_contains(*source, chapter, verse))
    })
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

fn render_intro(scene: &Scene, intro: &Intro, client: bool) -> Value {
    let source_list = source_links(scene, intro.sources, client);
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
    let (image_width, image_height) = if matches!(scene.id, "loaves-and-sea" | "emmaus") {
        (1672, 941)
    } else {
        (1536, 1024)
    };
    let active_id = active.map(|stop| stop.id);
    let stage = active_id.unwrap_or("intro");
    let book_slug = slug(scene.book);
    let mut html = format!(
        r#"<section class="story-journey" data-story-scene="{}" data-story-stage="{}"><p class="reader-context-kind">Illustrated Journey · {}</p><figure class="story-journey-art"><img src="/art/journey/{}.webp" width="{image_width}" height="{image_height}" alt="{}"><svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true"><path d="{}"></path></svg>"#,
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
/// Explicit unassigned intervals render without an active stop.
pub(super) fn render(book: usize, chapter: usize, verse: usize, client: bool) -> Option<Value> {
    let scene = scene_for_chapter(book, chapter)?;
    if let Some(stop) = stop_at(scene, chapter, verse) {
        Some(render_stop(scene, stop, client))
    } else {
        in_intro(scene, chapter, verse).map(|intro| render_intro(scene, intro, client))
    }
}

/// Whether a chapter contains any specifically assigned stop or unassigned interval.
pub(super) fn available(book: usize, chapter: usize) -> bool {
    scene_for_chapter(book, chapter).is_some()
}

#[cfg(test)]
mod tests {
    use super::super::bible;
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
        let mut scene_ids = HashSet::new();
        let mut art_paths = HashSet::new();
        for scene in all_scenes() {
            assert!(
                scene_ids.insert(scene.id),
                "duplicate scene id: {}",
                scene.id
            );
            assert!(
                art_paths.insert(scene.id),
                "duplicate journey art path: {}",
                scene.id
            );
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
            for intro in scene.intros {
                for source in intro.sources {
                    assert!(source.chapter > 0);
                    assert!(source.first > 0 && source.first <= source.last);
                    assert!(bible().books[scene.book].len() >= source.chapter);
                    let verses = &bible().books[scene.book][source.chapter - 1];
                    assert!(
                        verses.len() >= source.last,
                        "missing unassigned range end in {} {}:{}–{}",
                        NAMES[scene.book],
                        source.chapter,
                        source.first,
                        source.last
                    );
                    for verse in source.first..=source.last {
                        assert!(
                            assigned_verses.insert((source.chapter, verse)),
                            "unassigned interval overlaps another range at {} {}:{}",
                            NAMES[scene.book],
                            source.chapter,
                            verse
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn all_new_scene_ranges_and_gaps_follow_the_embedded_corpus() {
        for scene in SCENES.iter().filter(|scene| {
            matches!(
                scene.id,
                "jordan"
                    | "david-goliath"
                    | "daniel"
                    | "jonah"
                    | "loaves-and-sea"
                    | "emmaus"
                    | "samaria"
                    | "paul-voyage"
            )
        }) {
            for stop in scene.stops {
                for source in stop.sources {
                    for verse in source.first..=source.last {
                        assert_eq!(
                            stop_id(scene.book, source.chapter, verse),
                            Some(stop.id),
                            "{} {}:{} should activate {}",
                            scene.id,
                            source.chapter,
                            verse,
                            stop.id
                        );
                    }
                    let html = render(scene.book, source.chapter, source.first, true).unwrap()
                        ["html"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                    assert!(
                        html.contains(&format!("/art/journey/{}.webp", scene.id)),
                        "{} does not render its own artwork",
                        scene.id
                    );
                    assert!(
                        html.contains(&format!("<a class=\"reader-context-read\"")),
                        "{} has no deliberate Read source link",
                        stop.id
                    );
                    assert!(
                        html.contains(&format!("Read {}</a>", ref_label(scene.book, *source))),
                        "{} does not expose its exact {} source",
                        stop.id,
                        ref_label(scene.book, *source)
                    );
                }
            }
            for intro in scene.intros {
                for source in intro.sources {
                    for verse in source.first..=source.last {
                        let state = render(scene.book, source.chapter, verse, true)
                            .unwrap_or_else(|| panic!("{} lost an unassigned interval", scene.id));
                        assert!(
                            state["active"].is_null(),
                            "{} marked a gap active",
                            scene.id
                        );
                        let html = state["html"].as_str().unwrap();
                        assert!(html.contains(intro.title));
                        assert!(
                            html.contains(&format!("Read {}</a>", ref_label(scene.book, *source)))
                        );
                    }
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
            (5, 3, 1, "jordan-river-crossing"),
            (5, 3, 17, "jordan-river-crossing"),
            (5, 4, 1, "jordan-midstream-stones"),
            (5, 4, 9, "jordan-midstream-stones"),
            (5, 4, 10, "jordan-river-crossing"),
            (5, 4, 18, "jordan-river-crossing"),
            (5, 4, 19, "jordan-gilgal-memorial"),
            (5, 4, 24, "jordan-gilgal-memorial"),
            (8, 17, 1, "david-camp"),
            (8, 17, 39, "david-camp"),
            (8, 17, 40, "david-brook"),
            (8, 17, 41, "david-valley-confrontation"),
            (8, 17, 51, "david-valley-confrontation"),
            (8, 17, 52, "david-philistine-retreat"),
            (8, 17, 53, "david-philistine-retreat"),
            (26, 6, 1, "daniel-palace-prayer"),
            (26, 6, 10, "daniel-palace-prayer"),
            (26, 6, 11, "daniel-decree-court"),
            (26, 6, 15, "daniel-decree-court"),
            (26, 6, 16, "daniel-lions-den"),
            (26, 6, 18, "daniel-lions-den"),
            (26, 6, 19, "daniel-dawn-at-den"),
            (26, 6, 23, "daniel-dawn-at-den"),
            (26, 6, 24, "daniel-lions-den"),
            (31, 1, 3, "jonah-joppa-harbour"),
            (31, 1, 4, "jonah-storm-sea"),
            (31, 1, 16, "jonah-storm-sea"),
            (31, 1, 17, "jonah-shore"),
            (31, 2, 10, "jonah-shore"),
            (31, 3, 1, "jonah-nineveh"),
            (31, 3, 10, "jonah-nineveh"),
            (31, 4, 5, "jonah-city-shade"),
            (31, 4, 11, "jonah-city-shade"),
            (39, 14, 13, "loaves-isolated-shore"),
            (39, 14, 21, "loaves-isolated-shore"),
            (39, 14, 22, "loaves-boat-crossing"),
            (39, 14, 24, "loaves-boat-crossing"),
            (39, 14, 33, "loaves-boat-crossing"),
            (39, 14, 34, "loaves-far-shore"),
            (39, 14, 36, "loaves-far-shore"),
            (41, 24, 1, "emmaus-empty-tomb"),
            (41, 24, 12, "emmaus-empty-tomb"),
            (41, 24, 13, "emmaus-road"),
            (41, 24, 27, "emmaus-road"),
            (41, 24, 28, "emmaus-meal"),
            (41, 24, 32, "emmaus-meal"),
            (41, 24, 33, "emmaus-jerusalem-return"),
            (41, 24, 49, "emmaus-jerusalem-return"),
            (41, 24, 52, "emmaus-jerusalem-return"),
            (41, 24, 53, "emmaus-jerusalem-return"),
            (42, 4, 5, "samaria-well"),
            (42, 4, 27, "samaria-well"),
            (42, 4, 28, "samaria-town"),
            (42, 4, 30, "samaria-town"),
            (42, 4, 31, "samaria-well"),
            (42, 4, 38, "samaria-well"),
            (42, 4, 39, "samaria-town"),
            (42, 4, 42, "samaria-town"),
            (42, 4, 43, "samaria-galilee-cana"),
            (42, 4, 46, "samaria-galilee-cana"),
            (42, 4, 54, "samaria-galilee-cana"),
            (43, 27, 7, "paul-crete-haven"),
            (43, 27, 13, "paul-crete-haven"),
            (43, 27, 14, "paul-storm-sea"),
            (43, 27, 38, "paul-storm-sea"),
            (43, 27, 39, "paul-malta-shore"),
            (43, 27, 44, "paul-malta-shore"),
            (43, 28, 1, "paul-malta-shore"),
            (43, 28, 10, "paul-malta-shore"),
            (43, 28, 16, "paul-rome"),
            (43, 28, 31, "paul-rome"),
        ] {
            assert_eq!(stop_id(book, chapter, verse), Some(expected));
        }
    }

    #[test]
    fn unassigned_boundaries_do_not_carry_a_nearby_stop_forward() {
        assert!(render(0, 9, 18, true).is_none());
        assert!(render(1, 13, 16, true).is_none());
        assert!(render(40, 5, 44, true).is_none());
        assert!(render(5, 4, 25, true).is_none());
        assert!(render(8, 17, 59, true).is_none());
        assert!(render(39, 14, 37, true).is_none());
        assert!(render(42, 5, 1, true).is_none());
        assert!(!available(0, 6));
        assert!(!available(40, 3));

        let intro = render(41, 2, 1, true).unwrap();
        assert!(intro["active"].is_null());
        assert_eq!(intro["scene"], "bethlehem");
        assert!(intro["html"].as_str().unwrap().contains("Opening verses"));
        assert_eq!(stop_id(41, 2, 3), None);
        assert!(render(1, 15, 22, true).unwrap()["active"].is_null());
        assert_eq!(stop_id(7, 1, 18), Some("ruth-road"));
        for (book, chapter, verse, title) in [
            (8, 17, 54, "After the battle"),
            (26, 6, 25, "The king's decree and Daniel's prosperity"),
            (31, 1, 1, "The commission"),
            (31, 4, 1, "Jonah's displeasure"),
            (39, 14, 1, "Herod hears of Jesus"),
            (39, 14, 23, "Jesus goes up the mountain"),
            (41, 24, 50, "Toward Bethany"),
            (42, 4, 1, "Leaving Judaea"),
            (43, 27, 1, "Before Crete"),
            (43, 28, 11, "From Malta toward Rome"),
        ] {
            let gap = render(book, chapter, verse, true).unwrap();
            assert!(
                gap["active"].is_null(),
                "gap unexpectedly activated a stop: {book} {chapter}:{verse}"
            );
            assert!(gap["html"].as_str().unwrap().contains(title));
        }
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
