#!/usr/bin/env python3
"""Build the Exodus chapter companion catalog and its native SVG diagrams.

For initial authoring, pass the three reviewed plan files with --early-plan,
--middle-plan, and --late-plan. Later runs can omit them and rebuild SVGs and
provenance from the checked-in chapter companion catalog.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import textwrap
from pathlib import Path
from xml.sax.saxutils import escape


ROOT = Path(__file__).resolve().parents[1]
DATA_PATH = ROOT / "data/journey/exodus-chapter-companions.json"
PROVENANCE_PATH = ROOT / "assets/diagrams/exodus-chapter-companions-provenance.json"
SVG_DIR = ROOT / "web/art/journey"
KJV_PATH = ROOT / "data/kjv.json"
CURRENT_SCENES_PATH = ROOT / "data/journey/passage-scenes.json"
REFERENCE_SVG_PATH = ROOT / "web/art/journey/exodus-signs-rods.svg"
TABERNACLE_SOURCE_PATH = ROOT / "src/tabernacle.rs"

CAPTION = "Editorial chapter diagram · events and instructions are separate reading stages, not one scene"
SVG_WIDTH = 1536
CARD_X = 120
CARD_WIDTH = 430
CARD_CENTER_X = CARD_X + CARD_WIDTH / 2
CARD_PIN_X = 88
CARD_PIN_PERCENT = round(CARD_PIN_X / SVG_WIDTH * 100, 2)
CARD_FOCUS_PERCENT = round(CARD_CENTER_X / SVG_WIDTH * 100, 2)
TITLE_WRAP = 22
BODY_WRAP = 27
TITLE_LINE_HEIGHT = 36
BODY_LINE_HEIGHT = 30
MIN_CARD_HEIGHT = 170
CARD_GAP = 70
CARD_START_Y = 245


SHORT_TITLES = {
    "Jacob’s households enter Egypt; Joseph’s generation dies; Israel multiplies": "Israel grows; Joseph’s generation dies",
    "Zipporah circumcises her son during the way-station encounter": "Zipporah circumcises her son",
    "Moses and Aaron ask Pharaoh to let Israel go; Pharaoh refuses": "Moses asks for release; Pharaoh refuses",
    "One more plague is foretold; the people are told to ask for silver and gold": "One more plague; Israel asks for Egypt’s goods",
    "Genealogies of Reuben, Simeon, Levi, Moses, Aaron, and their descendants": "Reuben, Simeon, and Levi genealogy",
    "The rod sign before Pharaoh; imitation and Pharaoh’s refusal": "Rod sign; magicians imitate; Pharaoh refuses",
    "The waters are struck and turned to blood; fish die and water cannot be drunk": "Water turns to blood; fish die",
    "Frogs are announced, Aaron instructed, and frogs cover Egypt": "Frogs are announced and cover Egypt",
    "Dust becomes lice; the magicians cannot reproduce the sign": "Dust becomes lice; magicians fail",
    "Swarms are announced, a distinction promised for Goshen, and swarms enter Egyptian houses": "Swarms enter Egypt; Goshen is set apart",
    "Pharaoh bargains; Moses prays; swarms depart; Pharaoh refuses again": "Pharaoh bargains; swarms depart; refusal resumes",
    "Livestock plague announced and fulfilled; Pharaoh checks Israel’s cattle": "Livestock plague; Israel’s cattle are spared",
    "Hail, thunder, and fire strike Egypt; Goshen has no hail": "Hail and fire in Egypt; Goshen has no hail",
    "An east wind brings locusts that cover Egypt and consume what the hail left": "Locusts cover Egypt and consume what hail left",
    "Three days of darkness in Egypt; Israel’s dwellings have light": "Darkness in Egypt; Israel has light",
    "Moses predicts the servants’ plea and exits Pharaoh; refusal is summarized": "Servants will plead for release; Pharaoh refuses",
    "The ordinance is repeated; future children will be told its meaning; people worship": "Passover ordinance; children learn its meaning",
    "Passover lamb, blood sign, meal, readiness, and announced judgment": "Passover meal and blood sign; judgment announced",
    "A new king who did not know Joseph fears Israel’s growth": "A new king fears Israel’s growth",
    "Taskmasters set over Israel; treasure cities are named": "Taskmasters build Pithom and Raamses",
    "Pharaoh commands that every son be cast into the river": "Pharaoh orders Hebrew sons cast into the Nile",
    "The basket is found; Pharaoh’s daughter has compassion": "Basket found; Pharaoh’s daughter shows compassion",
    "Moses helps the seven daughters water their father’s flock": "Moses helps the seven daughters at the well",
    "Moses leads the flock to Horeb, the mountain of God": "Moses leads the flock to Horeb",
    "The bush burns without being consumed; Moses hides his face": "The bush burns; Moses hides his face",
    "The LORD has seen, heard, and known Israel’s sorrows": "The LORD sees and hears Israel’s sorrow",
    "The LORD says he will bring Israel out to a good land": "The LORD promises a good land",
    "Moses asks who he is; the LORD promises to be with him": "Moses asks who he is; God promises help",
    "The LORD identifies himself as God of Abraham, Isaac, and Jacob": "The God of Abraham, Isaac, and Jacob",
    "Gather Israel’s elders and report the LORD’s message": "Israel’s elders hear the LORD’s message",
    "Israel will not go empty; silver, gold, and raiment are foretold": "Israel will leave with Egypt’s goods",
    "Moses says Israel may not believe that the LORD appeared": "Moses fears Israel will not believe",
    "Moses’s hand becomes leprous as snow, then is restored": "Moses’s hand is made leprous, then restored",
    "Water from the river is foretold to become blood on dry land": "River water will become blood on dry land",
    "Moses takes his wife, sons, and the rod toward Egypt": "Moses returns toward Egypt with family and rod",
    "Wonders before Pharaoh and a hardened heart are foretold": "Wonders foretold; Pharaoh’s heart will harden",
    "Israel is called the LORD’s firstborn son; Egypt’s firstborn is warned of": "Israel as firstborn; Egypt’s firstborn warned",
    "The people gather stubble under the unchanged daily task": "Stubble gathering under unchanged quotas",
    "The officers ask Pharaoh why they are treated this way": "Officers question Pharaoh’s treatment",
    "Moses asks why he was sent and why the people suffer": "Moses questions the worsening suffering",
    "The LORD says Pharaoh will let Israel go and drive them out": "Pharaoh will release Israel",
    "The LORD promises to bring out, redeem, and take Israel as a people": "Israel will be brought out and redeemed",
    "Israel does not hearken for anguish of spirit and cruel bondage": "Israel does not hear because of anguish",
    "Moses points to Israel’s failure to hear and his own speech difficulty": "Moses cites Israel’s silence and his speech",
    "Moses and Aaron are identified as those charged to bring Israel out": "Moses and Aaron charged to bring Israel out",
    "The LORD again tells Moses to speak all that is commanded": "Moses is again commanded to speak",
    "The rod-to-serpent sign is instructed as a sign to Pharaoh": "A rod sign is instructed for Pharaoh",
    "Aaron is told to stretch his hand over Egypt’s waters": "Aaron is told to strike Egypt’s waters",
    "The magicians imitate the sign and Pharaoh does not hearken": "Magicians imitate; Pharaoh will not hear",
    "Seven days are fulfilled after the river is smitten": "Seven days after the river is struck",
    "The LORD warns Pharaoh of frogs if Israel is not released": "Frogs warned if Israel is not released",
    "Frogs are said to reach Pharaoh, his people, and servants": "Frogs reach Pharaoh and his servants",
    "Aaron is commanded to stretch his hand over the waters": "Aaron is commanded to strike the waters",
    "The magicians bring up frogs; Pharaoh asks Moses and Aaron to entreat": "Magicians bring frogs; Pharaoh asks relief",
    "Moses is told to stand before Pharaoh and announce swarms": "Swarms are announced before Pharaoh",
    "Moses says the people must go three days into the wilderness": "Three days in wilderness for worship",
    "Pharaoh hardens his heart and will not let the people go": "Pharaoh hardens his heart again",
    "The plague is announced for Egypt’s cattle in the field": "The cattle plague is announced",
    "The LORD makes a distinction: Israel’s cattle live, while Egypt’s die": "Egypt’s cattle die; Israel’s are spared",
    "All Egypt’s cattle die; none of Israel’s cattle die": "Egypt’s cattle die; Israel’s are spared",
    "Moses and Aaron take furnace ashes; Moses scatters them": "Moses and Aaron scatter furnace ashes",
    "Some servants bring their servants and cattle inside": "Some servants shelter people and cattle",
    "Others leave their servants and cattle in the field": "Others leave people and cattle outside",
    "Hail, thunder, and fire strike the land and break trees": "Hail, thunder, and fire break trees",
    "Pharaoh says he has sinned and asks for the storm to cease": "Pharaoh confesses and asks for relief",
    "Barley and flax are smitten; wheat and rye are not grown up": "Hail destroys barley and flax, not wheat",
    "Pharaoh asks who will go, then restricts the departure to men": "Pharaoh asks who may go, then limits them",
    "The children of Israel have light in their dwellings": "Israel has light in their dwellings",
    "Pharaoh says the people may go but livestock must stay": "Pharaoh offers release but keeps livestock",
    "One more plague is foretold before Pharaoh releases Israel": "One more plague is foretold",
    "Men and women are told to ask their neighbours for goods": "Israelites ask neighbors for goods",
    "Moses announces that the firstborn will die at midnight": "Firstborn death is announced for midnight",
    "Pharaoh’s servants are predicted to ask Israel to leave": "Pharaoh’s servants will demand Israel leave",
    "The chapter summarizes wonders and Pharaoh’s refusal": "Chapter summary: wonders and Pharaoh’s refusal",
    "The lamb is kept until the fourteenth day and killed in the evening": "The lamb is kept until the fourteenth day",
    "Blood on the side posts and upper post is commanded": "Blood is placed on the doorposts",
    "Moses calls the elders and instructs them to take and kill a lamb": "Moses instructs elders to kill a lamb",
    "The people are told to use hyssop and remain inside until morning": "Hyssop, blood, and staying indoors until dawn",
    "At midnight the firstborn and firstborn of cattle die": "The firstborn die at midnight",
    "A great cry is heard; Pharaoh calls Moses and Aaron by night": "A great cry; Pharaoh orders Israel to leave",
    "This night is to be observed through Israel’s generations": "Passover night kept through generations",
    "Passover participation rules for household, servant, and foreigner": "Passover eligibility and household rules",
    "One law is stated for homeborn and sojourning stranger": "One law for homeborn and foreigner",
    "The LORD brings Israel out of Egypt on that selfsame day": "The LORD brings Israel out that day",
    "Breastplate, twelve stones, names, Urim and Thummim": "Breastplate, stones, names, Urim, and Thummim",
    "Wash, clothe, anoint, and consecrate Aaron and his sons": "Wash, clothe, and consecrate Aaron’s sons",
    "Garment succession, holy meal, and seven-day altar consecration": "Garment succession and seven-day consecration",
    "Daily lambs and the promise of dwelling among Israel": "Daily lambs and promise of God’s dwelling",
    "Golden incense altar, placement, and service restrictions": "Golden incense altar and service rules",
    "Moses sees the calf, breaks the tablets, and destroys it": "Moses breaks tablets after seeing the calf",
    "Promise of an angel, warning of absence, and mourning": "Angel promised; warning of absence and mourning",
    "A tent outside camp, cloud at its door, and Moses and Joshua": "Tent outside camp; cloud at its door",
    "Moses asks for the LORD’s presence to accompany Israel": "Moses asks for the LORD’s presence",
    "Moses asks to see glory; the LORD describes a guarded passing": "Moses asks for glory; passing is guarded",
    "New tablets, the LORD’s proclamation, and Moses’ plea": "New tablets and the LORD’s proclamation",
    "Offering materials and a full list of work to be made": "Materials and work to be made",
    "People bring gifts; women spin; rulers bring stones and spices": "Willing gifts, spinning, stones, and spices",
    "Bezalel and Aholiab named to lead and teach skilled work": "Bezalel and Aholiab lead skilled work",
    "Work begins; offerings exceed need and further giving is stopped": "Work begins; further giving is stopped",
    "Curtains, loops, clasps, and outer coverings are made": "Curtains, loops, clasps, and coverings",
    "Boards, sockets, corner supports, bars, and gold overlay are made": "Boards, supports, bars, and gold overlay",
    "Ark, rings, carrying poles, mercy seat, and cherubim are made": "Ark, poles, mercy seat, and cherubim",
    "Incense altar, carrying poles, anointing oil, and incense are made": "Incense altar, oil, and incense are made",
    "Bronze altar and its tools and carrying hardware": "Bronze altar and carrying hardware",
    "Bronze altar and its tools and carrying hardware are made": "Bronze altar, tools, and carrying hardware",
    "Breastplate and its stones, names, chains, and fastening are made": "Breastplate and fastening are made",
    "Coats, headwear, girdle, and gold forehead plate": "Coats, headwear, girdle, and gold plate",
    "Coats, headwear, girdle, and gold forehead plate are made": "Coats, headwear, girdle, and gold plate",
    "Work is finished, brought to Moses, checked, and blessed": "Finished work is inspected and blessed",
    "Commanded setup sequence and relative placement of the furnishings": "Commanded setup and furniture placement",
    "Moses erects, furnishes, and completes the court and service": "Moses assembles the court and furnishings",
    "Cloud and glory fill the Tabernacle; cloud and fire guide travel": "Cloud fills Tabernacle; cloud and fire guide",
}

EARLY_SUMMARY_OVERRIDES = {
    "Verses 1–5 name the households and count the people; verses 6–7 summarize deaths and later growth.": "Verses 1–5 name households and count Israel; verses 6–7 summarize death and growth.",
    "The king’s order, the midwives’ refusal, the exchange, and the outcome are distinct narrated steps.": "Midwives refuse the order; the king questions them.",
    "The bush, call, shoe instruction, identity statement, and Moses’s response are linked to verses 2–6.": "Bush, call, and Moses’s response are linked.",
    "Preserves the crop distinction and source wording without substituting modern crop states.": "Preserves the crop distinction without modern substitutions.",
    "Verse 12 commands; verse 13 narrates the wind and arrival; verses 14–15 report the coverage and eating.": "Verse 12 commands; verse 13 brings locusts; verses 14–15 describe their extent.",
    "The children’s question is future; verse 27 reports the explanation and the people’s bowing and worship.": "The children’s question is future; the people bow and worship.",
    "Makes the decree and its stated distinction between sons and daughters visible without depicting harm to infants.": "Keeps the decree’s distinction between sons and daughters clear without depicting harm.",
    "The elders’ meeting, Pharaoh’s refusal, wonders, and departure with goods are planned or predicted in this speech, not completed here.": "The elders’ meeting and departure with goods are predicted, not completed.",
    "The second hand sign is narrated in verses 6–7; the river-water sign in verses 8–9 is conditional and predicted, not performed here.": "The hand sign occurs; the river-water sign is only predicted.",
    "The brief passage has ambiguous referents; preserve its words and sequence without supplying an interpretation of who is sought or what “his feet” means.": "Ambiguous referents; preserve the sequence without resolving whose feet are meant.",
    "The promises of bringing out, redeeming, taking as a people, and bringing into the land are future speech, not completed events here.": "Deliverance and land promises are spoken of as future events.",
    "The existing diagram links only verses 16, 18, and 20. The remaining names and family links stay unrepresented until added.": "The genealogy names Reuben, Simeon, Levi, Moses, Aaron, and descendants.",
    "Verses 26–29 repeat the identification and commission; verse 30 returns to Moses’s objection. No new action before Pharaoh is narrated.": "The commission is repeated; Moses’s speech objection returns.",
    "The hail is forecast for tomorrow; servants who fear the word bring servants and cattle inside, while others leave them in the field.": "Hail is foretold; servants respond differently to the warning.",
    "Verse 22 is instruction; verses 23–25 narrate the storm and damage; verse 26 reports no hail in Goshen. Keep locations in distinct panels, not a map.": "The storm strikes Egypt; Goshen remains free of hail.",
    "Keep Pharaoh’s speech, Moses’s stated response, crop report, prayer, storm cessation, and Pharaoh’s renewed refusal in order.": "Pharaoh asks for relief; hail stops, then refusal resumes.",
    "Verses 1–2 explain the stated purpose; verses 3–6 announce locusts; verses 7–11 record servant speech and Pharaoh’s restriction.": "Locusts are announced as Pharaoh restricts who may leave.",
    "Verses 3–11 prescribe household actions; verses 12–13 announce judgment and explain the sign. The blood application is not narrated as performed here.": "Household rites are prescribed; judgment and the sign are announced.",
    "Verses 21–22 give commands; verse 23 states what the LORD will do. The text does not independently narrate the blood being applied.": "Elders receive instructions; the LORD’s action is stated.",
    "This is the fulfilled midnight event and urgent release. Use a text-first, non-graphic sequence; do not merge it with the warning in chapter 11.": "At midnight the firstborn die and urgent release begins.",
    "The unit moves from carried dough to goods received, journey, accompanying people and flocks, and cakes made later. Do not show the entire sequence as one documentary instant.": "Dough, goods, journey, and later cakes form a sequence.",
    "The text records 430 years and a night to be observed; do not reconcile or extend the chronology beyond Exodus 12:40–42.": "The text records 430 years and a night to observe.",
    "These verses are ordinances, not a narrative event; preserve each condition and the concluding one-law statement.": "Passover ordinances set participation and household conditions.",
}

MIDDLE_SUMMARY_OVERRIDES = {
    "Starts the wilderness journey and preserves the route choice, Joseph's bones, and continuing guidance as distinct details.": "The route, Joseph’s bones, and continuing guidance remain distinct.",
    "Makes the question-and-answer teaching pattern visible without blending it into the rule.": "A child’s question frames teaching about the rule.",
    "Records the actual provision and equal measure without treating the announcement as fulfillment.": "Records actual provision and measured portions.",
    "Shows the coordinated ground battle and hill actions as a sequence, not a single undifferentiated tableau.": "Shows the ground battle and Moses’s hilltop support.",
    "Separates the immediate battle result from its memorial and forward-looking declaration.": "Separates the battle result, memorial, and future conflict.",
    "Keeps the messenger's announced role and obedience condition distinct from later land-outcome promises.": "Messenger role and obedience condition remain distinct.",
    "Keeps this distinct encounter and meal separate from the public covenant rite and Moses' later ascent.": "This encounter and meal precede covenant rite and ascent.",
}

LATE_SUMMARY_OVERRIDES = {
    "Introduces purpose and materials before the detailed object specifications; makes clear that a generated reconstruction is not the pattern itself.": "Introduces the sanctuary’s purpose and materials before object specifications.",
    "Preserves the abrupt transition from skilled work to rest and then to the tablets, rather than treating the whole chapter as a construction list.": "Separates skilled work, the Sabbath boundary, and the tablets.",
    "Provides a clear narrative endpoint after the covenant words without conflating this veil with the Tabernacle’s inner veil.": "Marks the endpoint; this veil is not the Tabernacle’s inner veil.",
    "Links the execution account back to its earlier specifications and exposes the one major furnishing with no dedicated close-up asset.": "Links the work to earlier specifications; the altar has no close-up art.",
    "Pairs the object work with the passage’s explicit inventory instead of treating the quantities as decoration.": "Pairs the work with the passage’s explicit inventory.",
    "An order chart makes the placement relationships and distinction between command (1-15) and performed setup (16-33) legible.": "Shows placement order, separating command from performed setup.",
    "A linked register separates four neighboring but different instructions and shows where the later execution is recorded.": "Separates neighboring instructions and points to later execution.",
    "A narrative beat chart preserves cause, response, intercession, and consequence in a chapter with several scene shifts.": "Preserves cause, response, intercession, and consequence.",
    "Makes the chapter’s distinct setting and the argument about presence visible without confusing it with the later completed sanctuary.": "Keeps the tent’s setting and argument about presence distinct.",
    "A source-ordered checklist makes the multi-stage ritual and its daily continuation easier to follow.": "Source order clarifies the ritual and its daily continuation.",
    "A sequence chart can separate the mountain encounter, covenant terms, festivals, and writing of the tablets.": "Separates the mountain encounter, terms, feasts, and tablets.",
}


def short_title(value: str) -> str:
    title = SHORT_TITLES.get(value, value)
    if len(title) > 50:
        raise ValueError(f"Display title exceeds 50 characters ({len(title)}): {title}")
    if len(wrap_lines(title, TITLE_WRAP)) > 3:
        raise ValueError(f"Display title wraps to more than three lines: {title}")
    return title


def compact_summary(value: str, overrides: dict[str, str]) -> str:
    summary = overrides.get(value, " ".join(value.strip().split()))
    if len(summary) > 108:
        parts = re.split(r"[;.!?]", summary, maxsplit=1)
        if len(parts) > 1 and len(parts[0].strip()) >= 24:
            summary = parts[0].strip().rstrip(" ,:—–-") + "."
    if len(summary) > 108:
        raise ValueError(f"Diagram summary exceeds four lines ({len(summary)} characters): {summary}")
    if len(wrap_lines(summary, BODY_WRAP)) > 4:
        raise ValueError(f"Diagram summary wraps to more than four lines: {summary}")
    return finish_sentence(summary)


MIDDLE_TITLES = {
    "ex13-firstborn-claim": "Firstborn dedicated to the LORD",
    "ex13-unleavened-memory": "Unleavened bread recalls deliverance",
    "ex13-firstborn-redemption-rule": "Firstborn redemption rules",
    "ex13-child-asks-why": "A child asks why",
    "ex13-departure-guidance": "Departure guided by cloud and fire",
    "ex14-encampment-announcement": "Sea encampment and announced pursuit",
    "ex14-pharaoh-pursues": "Pharaoh's army catches Israel",
    "ex14-fear-and-assurance": "Israel fears; Moses reassures",
    "ex14-go-forward-and-cloud": "The cloud moves between the camps",
    "ex14-sea-divides-israel-crosses": "The sea opens; Israel crosses",
    "ex14-egyptian-pursuit-hindered": "Egypt follows into the sea",
    "ex14-waters-return-and-israel-sees": "The waters return; Israel is saved",
    "ex15-song-victory": "A song of victory over Egypt",
    "ex15-song-guidance-and-reign": "Guidance and the LORD's reign",
    "ex15-crossing-summary": "The crossing recalled in prose",
    "ex15-miriam-answers": "Miriam leads the women's song",
    "ex15-shur-marah-water": "Bitter water at Marah",
    "ex15-elim": "Israel camps at Elim",
    "ex16-hunger-complaint": "Hunger in the wilderness",
    "ex16-provision-announced": "Manna announced, not yet received",
    "ex16-quail-manna-and-measure": "Quail and measured manna",
    "ex16-daily-gathering-test": "Manna left overnight spoils",
    "ex16-sixth-day-and-sabbath": "Sabbath gathering and rest",
    "ex16-manna-memorial-and-duration": "Manna kept as a memorial",
    "ex17-thirst-and-complaint": "Thirst at Rephidim",
    "ex17-rock-water": "Water from the rock at Horeb",
    "ex17-amalek-battle": "Joshua fights; Moses is supported",
    "ex17-memorial-and-ongoing-conflict": "Battle memorial and ongoing conflict",
    "ex18-jethro-arrives": "Jethro arrives with Moses' family",
    "ex18-reunion-and-shared-worship": "Reunion, report, and shared meal",
    "ex18-moses-hears-cases": "Moses hears cases",
    "ex18-jethro-proposes-delegation": "Jethro proposes shared judging",
    "ex18-delegation-implemented": "Moses appoints rulers",
    "ex19-arrive-at-sinai": "Israel camps before Sinai",
    "ex19-covenant-proposal": "Covenant proposal and promised identity",
    "ex19-people-answer-and-cloud-announced": "The people answer; a cloud is announced",
    "ex19-consecration-and-boundaries": "Preparation and mountain boundary",
    "ex19-sinai-revelation": "The LORD descends on Sinai",
    "ex19-boundaries-reaffirmed": "The mountain boundary is reaffirmed",
    "ex20-liberation-preamble": "Deliverance before the commands",
    "ex20-worship-name-sabbath": "Worship, name, and Sabbath commands",
    "ex20-family-and-neighbor-commands": "Commands toward family and neighbor",
    "ex20-people-request-mediation": "The people ask Moses to mediate",
    "ex20-no-silver-or-gold-gods": "No gods of silver or gold",
    "ex20-altar-instructions": "Instructions for the altar",
    "ex21-servants-and-household-provisions": "Household service and release rules",
    "ex21-life-and-family-offenses": "Life and family offenses",
    "ex21-injury-and-compensation": "Injury and compensation",
    "ex21-animal-and-pit-liability": "Animals and pit liability",
    "ex22-theft-and-property-damage": "Theft and property damage",
    "ex22-entrusted-and-borrowed-property": "Entrusted or borrowed property",
    "ex22-sexual-obligation": "Obligation after enticement",
    "ex22-forbidden-practices": "Forbidden practices",
    "ex22-protect-strangers-and-vulnerable": "Protect strangers and vulnerable neighbors",
    "ex22-reverence-first-gifts-and-holiness": "First gifts and holy conduct",
    "ex23-justice-testimony-and-neighbor-help": "Justice, testimony, and neighbor help",
    "ex23-sabbatical-year-and-weekly-rest": "Seventh-year and weekly rest",
    "ex23-annual-feasts-and-offerings": "Annual feasts and offerings",
    "ex23-messenger-and-obedience-promise": "Messenger and conditional promise",
    "ex23-worship-and-blessing": "Worship and promised blessing",
    "ex23-land-outcome-and-boundaries": "Future land and boundaries",
    "ex23-no-covenant-with-local-gods": "No covenant with local gods",
    "ex24-approach-instructions": "Instructions for approaching",
    "ex24-covenant-read-and-ratified": "The covenant is read and ratified",
    "ex24-elders-see-and-eat": "The elders see and eat",
    "ex24-moses-enters-cloud": "Moses enters the cloud",
}


OBJECTS = {
    "ark": {
        "path": "/art/objects/ark.webp",
        "alt": "Illustration of the gold-covered Ark of the testimony and mercy seat",
    },
    "table": {
        "path": "/art/objects/table.webp",
        "alt": "Illustration of the gold-covered table holding the shewbread",
    },
    "lamp": {
        "path": "/art/objects/lamp.webp",
        "alt": "Illustration of the golden seven-branched candlestick and its lamps",
    },
    "veil": {
        "path": "/art/objects/veil.webp",
        "alt": "Illustration of the embroidered veil between the holy place and the most holy",
    },
    "tent": {
        "path": "/art/objects/tent.webp",
        "alt": "Illustration of the Tabernacle tent and its layered coverings",
    },
    "courtyard": {
        "path": "/art/objects/courtyard.webp",
        "alt": "Illustration of the Tabernacle courtyard, linen boundary, and east gate",
    },
    "altar": {
        "path": "/art/objects/altar.webp",
        "alt": "Illustration of the bronze-covered altar with projecting horns",
    },
    "basin": {
        "path": "/art/objects/basin.webp",
        "alt": "Illustration of the bronze laver and its foot",
    },
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_local_range(value: str, chapter: int) -> tuple[int, int]:
    value = value.strip()
    match = re.fullmatch(r"(?:Exodus\s+)?(?:(\d+):)?(\d+)(?:-(\d+))?", value)
    if not match:
        raise ValueError(f"Unsupported source range: {value!r}")
    range_chapter = int(match.group(1)) if match.group(1) else chapter
    first = int(match.group(2))
    last = int(match.group(3) or first)
    if range_chapter != chapter or first < 1 or last < first:
        raise ValueError(f"Range is not local to Exodus {chapter}: {value}")
    return first, last


def range_object(chapter: int, first: int, last: int) -> dict[str, int]:
    return {"chapter": chapter, "first": first, "last": last}


def ranges_partition(ranges: list[tuple[int, int]], first: int, last: int) -> bool:
    counts = [0] * (last - first + 1)
    for start, end in ranges:
        if start < first or end > last or end < start:
            return False
        for verse in range(start, end + 1):
            counts[verse - first] += 1
    return bool(counts) and all(count == 1 for count in counts)


def finish_sentence(value: str) -> str:
    value = " ".join(value.strip().split())
    if not value:
        return value
    if value[-1] not in ".!?;:":
        value += "."
    return value


def join_notes(*parts: str | None) -> str:
    return " ".join(finish_sentence(part) for part in parts if part and part.strip())


def early_records(plan: dict) -> list[dict]:
    nodes = []
    for chapter_row in plan["chapters"]:
        chapter = chapter_row["chapter"]
        for unit in chapter_row["units"]:
            parent_first, parent_last = parse_local_range(unit["range"], chapter)
            proposals = unit.get("proposed_nodes") or []
            parsed = []
            flat_sources = []
            for proposal in proposals:
                sources = [parse_local_range(span, chapter) for span in proposal.get("sources", [])]
                parsed.append((proposal, sources))
                flat_sources.extend(sources)
            can_split = bool(parsed) and ranges_partition(flat_sources, parent_first, parent_last)

            if can_split:
                for index, (proposal, sources) in enumerate(parsed, start=1):
                    if not sources:
                        raise ValueError(f"Proposed node has no sources: Exodus {chapter} {unit['range']}")
                    source_objects = [range_object(chapter, first, last) for first, last in sources]
                    first = sources[0][0]
                    last = sources[-1][1]
                    note = join_notes(
                        f"Timing: {proposal.get('timing', 'source-defined reading stage')}",
                        proposal.get("why_useful"),
                    )
                    nodes.append({
                        "chapter": chapter,
                        "first": first,
                        "last": last,
                        "id": f"exodus-chapter-{chapter:02d}-node-{len(nodes) + 1:02d}",
                        "title": short_title(proposal["label"]),
                        "note": note,
                        "diagram_note": compact_summary(proposal.get("why_useful", proposal.get("timing", "Reading stage")), EARLY_SUMMARY_OVERRIDES),
                        "sources": source_objects,
                        "detail": None,
                    })
            else:
                note = join_notes(
                    f"Timing: {unit.get('timing_note', 'source-defined reading stage')}",
                    unit.get("note"),
                )
                diagram_summary = unit.get("timing_note", "Source-defined reading stage")
                nodes.append({
                    "chapter": chapter,
                    "first": parent_first,
                    "last": parent_last,
                    "id": f"exodus-chapter-{chapter:02d}-node-{len(nodes) + 1:02d}",
                    "title": short_title(unit["label"]),
                    "note": note,
                    "diagram_note": compact_summary(diagram_summary, EARLY_SUMMARY_OVERRIDES),
                    "sources": [range_object(chapter, parent_first, parent_last)],
                    "detail": None,
                })
    return nodes


def middle_records(plan: dict) -> list[dict]:
    nodes = []
    for chapter_row in plan["chapter_plan"]:
        chapter = chapter_row["chapter"]
        for unit in chapter_row["units"]:
            node_id = unit["node_id"]
            if node_id not in MIDDLE_TITLES:
                raise ValueError(f"Missing display title for middle-plan node {node_id}")
            first, last = unit["first"], unit["last"]
            note = join_notes(
                f"Stage type: {unit['node_type'].replace('-', ' ')}",
                unit["paraphrase"],
                unit["why_useful"],
            )
            nodes.append({
                "chapter": chapter,
                "first": first,
                "last": last,
                "id": f"exodus-chapter-{chapter:02d}-node-{len(nodes) + 1:02d}",
                "title": short_title(MIDDLE_TITLES[node_id]),
                "note": note,
                "diagram_note": compact_summary(unit["why_useful"], MIDDLE_SUMMARY_OVERRIDES),
                "sources": [range_object(chapter, first, last)],
                "detail": None,
            })
    return nodes


def late_status(chapter: int, unit_id: str) -> str:
    if chapter <= 30:
        return "Instruction"
    if chapter == 31:
        return "Commission and covenant boundary"
    if chapter <= 34:
        return "Narrative"
    if chapter == 35:
        return "Sabbath and offering preparation"
    if chapter <= 38:
        return "Construction report"
    if chapter == 39:
        return "Completion and handover report"
    if unit_id in {"setup-command", "anointing-command"}:
        return "Setup instruction"
    if unit_id == "glory-and-guidance":
        return "Closing account"
    return "Setup report"


def late_records(plan: dict) -> list[dict]:
    chart_nodes = {node["id"]: node for node in plan["proposed_chart_nodes"]}
    known_objects = {obj["id"] for obj in plan["existing_interactive_objects"]}
    nodes = []
    for chapter_row in plan["exhaustive_chapters"]:
        chapter = chapter_row["chapter"]
        for unit in chapter_row["units"]:
            first, last = unit["first_verse"], unit["last_verse"]
            mapped_nodes = re.findall(r"node:([a-z0-9-]+)", unit["map_to"])
            if any(node_id not in chart_nodes for node_id in mapped_nodes):
                raise ValueError(f"Unresolved late chapter map_to in Exodus {chapter}: {unit['map_to']}")
            object_ids = []
            for token in re.findall(r"objects?:([a-z0-9, ]+)", unit["map_to"]):
                object_ids.extend(part.strip() for part in token.split(",") if part.strip())
            if any(object_id not in known_objects for object_id in object_ids):
                raise ValueError(f"Unresolved object map_to in Exodus {chapter}: {unit['map_to']}")

            note_parts = [f"{late_status(chapter, unit['id'])}: {unit['title']}"]
            if mapped_nodes:
                labels = [chart_nodes[node_id]["label"] for node_id in mapped_nodes]
                note_parts.append(
                    f"This stage belongs to the wider {', '.join(labels)} outline; only Exodus {chapter}:{first}–{last} is assigned to this card"
                )

            detail_object = None
            if len(object_ids) == 1:
                detail_object = object_ids[0]
            elif "veil" in object_ids and unit["id"] == "veil-and-inner-layout":
                # The veil is one visible object in this mixed placement unit;
                # the note keeps the ark, table, and lamp placements distinct.
                detail_object = "veil"

            detail = None
            if detail_object:
                art = OBJECTS[detail_object]
                detail = {
                    "path": art["path"],
                    "width": 1536,
                    "height": 1024,
                    "alt": art["alt"],
                }
                note_parts.append(
                    f"The detail image reuses the existing artist reconstruction of the {detail_object}; it illustrates that object only, not the whole unit or a measured pattern"
                )
            elif unit["id"] == "incense-altar":
                note_parts.append(
                    "No dedicated golden incense-altar image exists; altar.webp shows the bronze burnt-offering altar and is not a substitute"
                )
            elif unit["id"] == "levites-and-account":
                note_parts.append(
                    "Keep the passage's account of the Levites' violence visible and do not add motives beyond the text"
                )
            elif unit["id"] == "tent-outside-camp":
                note_parts.append(
                    "The text calls this the tabernacle of the congregation; it does not identify it as the later fully built sanctuary"
                )
            elif unit["id"] == "request-for-glory":
                note_parts.append(
                    "Do not portray the LORD as a human figure; retain the stated rock, concealment, and passing glory"
                )
            elif chapter == 31:
                note_parts.append(
                    "The chapter groups commissioning, Sabbath, and tablets; it does not establish that every task happened in that same order"
                )
            elif chapter == 34:
                note_parts.append(
                    "Keep covenant terms distinct from the narrated encounter and its aftermath"
                )
            elif chapter == 35:
                note_parts.append(
                    "The chapter begins with a Sabbath boundary; the work and offerings are not assigned to the Sabbath"
                )
            elif unit["id"] in {"boards-bars", "boards-and-bars"}:
                note_parts.append(
                    "The outline is not a measured frame plan and supplies no exact joints, hidden structure, or complete build schedule"
                )
            elif unit["id"] in {"curtains-and-covers"}:
                note_parts.append("The textile account is not a pattern or stitching diagram")
            elif unit["id"] in {"metal-account"}:
                note_parts.append("The illustration is not to scale or a materials audit; quantities remain in the linked KJV passage")
            elif unit["id"] == "glory-and-guidance":
                note_parts.append("The existing Tabernacle overview is a still reconstruction, not a depiction of this event")

            sources = [range_object(chapter, first, last)]
            if detail_object:
                diagram_note = "Object detail only; not the whole unit or a measured pattern."
            elif mapped_nodes:
                diagram_note = compact_summary(chart_nodes[mapped_nodes[0]]["why_useful"], LATE_SUMMARY_OVERRIDES)
            else:
                diagram_note = "A source-defined reading stage in the chapter outline."
            nodes.append({
                "chapter": chapter,
                "first": first,
                "last": last,
                "id": f"exodus-chapter-{chapter:02d}-node-{len(nodes) + 1:02d}",
                "title": short_title(unit["title"]),
                "note": join_notes(*note_parts),
                "diagram_note": diagram_note,
                "sources": sources,
                "detail": detail,
            })
    return nodes


def scene_from_nodes(chapter: int, verse_count: int, nodes: list[dict]) -> dict:
    hotspots = []
    for node in nodes:
        hotspots.append({
            "id": node["id"],
            "title": node["title"],
            "x": CARD_PIN_PERCENT,
            "focus_x": CARD_FOCUS_PERCENT,
            "y": 0.0,
            "active": [range_object(chapter, node["first"], node["last"])],
            "anchor": range_object(chapter, node["first"], node["first"]),
            "sources": node["sources"],
            "note": node["note"],
            "diagram_note": node["diagram_note"],
            "detail": node["detail"],
        })
    return {
        "id": f"exodus-chapter-{chapter:02d}",
        "book": 1,
        "title": f"Exodus {chapter} chapter outline",
        "coverage": f"Chapter outline · Exodus {chapter}:1–{verse_count}",
        "caption": CAPTION,
        "ranges": [range_object(chapter, 1, verse_count)],
        "art": {
            "path": f"/art/journey/exodus-chapter-{chapter:02d}.svg",
            "width": SVG_WIDTH,
            "height": 1024,
            "alt": (
                f"Editorial chapter diagram for Exodus {chapter}: {len(nodes)} source-based cards "
                "partition the chapter into reading stages. This is not a geographical map or a single scene."
            ),
        },
        "intro": {
            "title": f"Exodus {chapter} · chapter outline",
            "note": (
                "This native text diagram separates the chapter into source-based reading stages. "
                "Events, instructions, songs, laws, and summaries follow the text; the cards do not claim to depict one place or moment."
            ),
        },
        "hotspots": hotspots,
    }


def scenes_from_plans(early: dict, middle: dict, late: dict, verse_counts: list[int]) -> dict:
    records = early_records(early) + middle_records(middle) + late_records(late)
    grouped: dict[int, list[dict]] = {chapter: [] for chapter in range(1, 41)}
    for record in records:
        grouped[record["chapter"]].append(record)
    scenes = [scene_from_nodes(chapter, verse_counts[chapter - 1], grouped[chapter]) for chapter in range(1, 41)]
    return {"schema_version": 1, "scenes": scenes}


def wrap_lines(value: str, width: int) -> list[str]:
    lines = textwrap.wrap(
        value,
        width=width,
        break_long_words=True,
        break_on_hyphens=False,
        replace_whitespace=True,
        drop_whitespace=True,
    )
    return lines or [""]


def display_range(first: int, last: int) -> str:
    return str(first) if first == last else f"{first}–{last}"


def card_layout(hotspot: dict, chapter: int, top: int) -> dict:
    title_lines = wrap_lines(hotspot["title"], TITLE_WRAP)
    note_lines = wrap_lines(hotspot.get("diagram_note") or hotspot["note"], BODY_WRAP)
    height = max(
        MIN_CARD_HEIGHT,
        28 + len(title_lines) * TITLE_LINE_HEIGHT + 7 + 25 + 8 + len(note_lines) * BODY_LINE_HEIGHT + 22,
    )
    reference = f"Exodus {chapter}:{display_range(hotspot['active'][0]['first'], hotspot['active'][0]['last'])}"
    return {
        "top": top,
        "height": height,
        "bottom": top + height,
        "center": top + height / 2,
        "title_lines": title_lines,
        "note_lines": note_lines,
        "reference": reference,
    }


def render_svg(scene: dict) -> tuple[str, list[dict]]:
    chapter = scene["ranges"][0]["chapter"]
    hotspots = scene["hotspots"]
    layouts = []
    cursor = CARD_START_Y
    for hotspot in hotspots:
        layout = card_layout(hotspot, chapter, cursor)
        layouts.append(layout)
        cursor = layout["bottom"] + CARD_GAP
    total_height = max(1024, int(cursor - CARD_GAP + 95))
    scene["art"]["height"] = total_height
    for hotspot, layout in zip(hotspots, layouts):
        hotspot["x"] = CARD_PIN_PERCENT
        hotspot["focus_x"] = CARD_FOCUS_PERCENT
        hotspot["y"] = round(layout["center"] / total_height * 100, 4)
        if hotspot["x"] != CARD_PIN_PERCENT:
            raise ValueError(f"{hotspot['id']} pin x does not match its visible card")
        if hotspot["focus_x"] != CARD_FOCUS_PERCENT:
            raise ValueError(f"{hotspot['id']} focus_x does not match its visible card center")
        if hotspot["y"] != round(layout["center"] / total_height * 100, 4):
            raise ValueError(f"{hotspot['id']} pin y does not match its visible card")

    verse_count = scene["ranges"][0]["last"]
    title = f"Exodus {chapter} · Chapter outline"
    subtitle = f"Exodus {chapter}:1–{verse_count} · {len(hotspots)} source-based reading stages"
    desc = (
        f"A vertical column of {len(hotspots)} text cards partitions all {verse_count} verses in Exodus {chapter}. "
        "The diagram separates reading stages and does not depict one scene or a geographic route."
    )
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_WIDTH}" height="{total_height}" viewBox="0 0 {SVG_WIDTH} {total_height}" role="img" aria-labelledby="title desc">',
        f'  <title id="title">{escape(title)}</title>',
        f'  <desc id="desc">{escape(desc)}</desc>',
        f'  <rect width="{SVG_WIDTH}" height="{total_height}" fill="#f7f5ee"/>',
        '  <defs><marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10Z" fill="#29664d"/></marker></defs>',
        f'  <text x="120" y="90" fill="#172d27" font-family="Georgia,serif" font-size="40">{escape(title)}</text>',
        f'  <text x="120" y="142" fill="#59685f" font-family="Arial,sans-serif" font-size="28">{escape(subtitle)}</text>',
    ]
    for index, layout in enumerate(layouts):
        y = layout["top"]
        h = layout["height"]
        center = layout["center"]
        lines.append(
            f'  <line x1="{CARD_PIN_X}" y1="{center:g}" x2="{CARD_X}" y2="{center:g}" stroke="#29664d" stroke-width="3"/>'
        )
        lines.append(f'  <rect x="{CARD_X}" y="{y}" width="{CARD_WIDTH}" height="{h}" rx="18" fill="#eceee4" stroke="#b8cbbb" stroke-width="2"/>')
        lines.append(
            f'  <circle cx="{CARD_PIN_X}" cy="{center:g}" r="10" fill="#29664d" stroke="#f7f5ee" stroke-width="3"/>'
        )
        title_y = y + 43
        for line_index, text in enumerate(layout["title_lines"]):
            baseline = title_y + line_index * TITLE_LINE_HEIGHT
            lines.append(f'  <text x="150" y="{baseline}" fill="#172d27" font-family="Georgia,serif" font-size="30">{escape(text)}</text>')
        ref_y = title_y + len(layout["title_lines"]) * TITLE_LINE_HEIGHT + 3
        lines.append(f'  <text x="150" y="{ref_y}" fill="#29664d" font-family="Arial,sans-serif" font-size="22">{escape(layout["reference"])}</text>')
        note_y = ref_y + 34
        for line_index, text in enumerate(layout["note_lines"]):
            baseline = note_y + line_index * BODY_LINE_HEIGHT
            lines.append(f'  <text x="150" y="{baseline}" fill="#59685f" font-family="Arial,sans-serif" font-size="24">{escape(text)}</text>')
        if index + 1 < len(layouts):
            arrow_start = layout["bottom"] + 10
            arrow_end = layouts[index + 1]["top"] - 10
            lines.append(f'  <path d="M{CARD_CENTER_X:g} {arrow_start}V{arrow_end}" fill="none" stroke="#29664d" stroke-width="3" marker-end="url(#arrow)"/>')
    footer_y = total_height - 35
    footer = "Each card follows its cited range; the diagram is a reading outline, not a scene or map."
    lines.append(f'  <text x="120" y="{footer_y}" fill="#59685f" font-family="Arial,sans-serif" font-size="22">{escape(footer)}</text>')
    lines.append("</svg>")
    return "\n".join(lines) + "\n", layouts


def load_kjv() -> tuple[dict, list[int]]:
    data = json.loads(KJV_PATH.read_text(encoding="utf-8"))
    book = data[1]
    if book.get("name") != "Êxodo" or len(book.get("chapters", [])) != 40:
        raise ValueError("Expected KJV corpus book index 1 to be Exodus with 40 chapters")
    return data, [len(chapter) for chapter in book["chapters"]]


def validate_catalog(catalog: dict, verse_counts: list[int]) -> int:
    if catalog.get("schema_version") != 1:
        raise ValueError("Exodus companion catalog must use schema_version 1")
    scenes = catalog.get("scenes")
    if not isinstance(scenes, list) or len(scenes) != 40:
        raise ValueError("The companion catalog must contain exactly 40 chapters")
    existing = json.loads(CURRENT_SCENES_PATH.read_text(encoding="utf-8"))
    existing_ids = {scene["id"] for scene in existing["scenes"]}
    scene_ids = set()
    hotspot_ids = set()
    total_nodes = 0
    for expected_chapter, scene in enumerate(scenes, start=1):
        expected_id = f"exodus-chapter-{expected_chapter:02d}"
        if scene.get("id") != expected_id or scene.get("book") != 1:
            raise ValueError(f"Expected {expected_id} for Exodus book index 1")
        if scene["id"] in existing_ids or scene["id"] in scene_ids:
            raise ValueError(f"Scene ID collision: {scene['id']}")
        scene_ids.add(scene["id"])
        count = verse_counts[expected_chapter - 1]
        expected_range = range_object(expected_chapter, 1, count)
        if scene.get("ranges") != [expected_range]:
            raise ValueError(f"{expected_id} does not declare the full KJV chapter range")
        if scene.get("coverage") != f"Chapter outline · Exodus {expected_chapter}:1–{count}":
            raise ValueError(f"{expected_id} has an incorrect coverage label")
        if scene.get("caption") != CAPTION:
            raise ValueError(f"{expected_id} has an unexpected caption")
        art_path = scene.get("art", {}).get("path", "")
        if art_path != f"/art/journey/exodus-chapter-{expected_chapter:02d}.svg":
            raise ValueError(f"{expected_id} has an unexpected art path")
        hotspots = scene.get("hotspots")
        if not hotspots:
            raise ValueError(f"{expected_id} has no reading-stage cards")
        active_ranges = []
        previous_first = 0
        for hotspot in hotspots:
            if hotspot["id"] in hotspot_ids:
                raise ValueError(f"Duplicate hotspot id {hotspot['id']}")
            hotspot_ids.add(hotspot["id"])
            active = hotspot.get("active", [])
            if len(active) != 1 or active[0].get("chapter") != expected_chapter:
                raise ValueError(f"{hotspot['id']} must have one contiguous active range in its chapter")
            first, last = active[0]["first"], active[0]["last"]
            if first <= previous_first:
                raise ValueError(f"{expected_id} cards are not in source order")
            previous_first = first
            active_ranges.append((first, last))
            anchor = hotspot.get("anchor", {})
            if anchor != range_object(expected_chapter, first, first):
                raise ValueError(f"{hotspot['id']} anchor must begin its active source range")
            if hotspot.get("sources") != [range_object(expected_chapter, first, last)]:
                raise ValueError(f"{hotspot['id']} source range must match its active range")
            if not hotspot.get("title") or not hotspot.get("note") or not hotspot.get("diagram_note"):
                raise ValueError(f"{hotspot['id']} is missing readable title or explanation")
            if hotspot.get("x") != CARD_PIN_PERCENT or hotspot.get("focus_x") != CARD_FOCUS_PERCENT:
                raise ValueError(f"{hotspot['id']} must use the left-gutter pin and centered focus coordinate")
            if len(hotspot["title"]) > 50 or len(hotspot["diagram_note"]) > 108:
                raise ValueError(f"{hotspot['id']} card copy exceeds its title or summary budget")
            if len(wrap_lines(hotspot["title"], TITLE_WRAP)) > 3:
                raise ValueError(f"{hotspot['id']} title exceeds three SVG lines")
            if len(wrap_lines(hotspot["diagram_note"], BODY_WRAP)) > 4:
                raise ValueError(f"{hotspot['id']} summary exceeds four SVG lines")
            detail = hotspot.get("detail")
            if detail:
                if detail.get("width") != 1536 or detail.get("height") != 1024:
                    raise ValueError(f"{hotspot['id']} detail dimensions do not match the Pillow check")
                detail_file = ROOT / "web" / detail["path"].removeprefix("/")
                if not detail_file.is_file():
                    raise ValueError(f"{hotspot['id']} detail asset is missing: {detail['path']}")
        if not ranges_partition(active_ranges, 1, count):
            raise ValueError(f"{expected_id} active ranges do not partition all {count} KJV verses exactly once")
        total_nodes += len(hotspots)
    if scene_ids != {f"exodus-chapter-{chapter:02d}" for chapter in range(1, 41)}:
        raise ValueError("Chapter companion scene IDs are incomplete")
    return total_nodes


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_outputs(catalog: dict, verse_counts: list[int]) -> None:
    for scene in catalog.get("scenes", []):
        for hotspot in scene.get("hotspots", []):
            hotspot["x"] = CARD_PIN_PERCENT
            hotspot["focus_x"] = CARD_FOCUS_PERCENT
    total_nodes = validate_catalog(catalog, verse_counts)
    SVG_DIR.mkdir(parents=True, exist_ok=True)
    diagram_rows = []
    for scene in catalog["scenes"]:
        svg, layouts = render_svg(scene)
        svg_path = ROOT / "web" / scene["art"]["path"].removeprefix("/")
        svg_path.write_text(svg, encoding="utf-8")
        node_centers = []
        for hotspot, layout in zip(scene["hotspots"], layouts):
            node_centers.append({
                "id": hotspot["id"],
                "title": hotspot["title"],
                "x": hotspot["x"],
                "focus_x": hotspot["focus_x"],
                "y": hotspot["y"],
                "pixel_x": CARD_PIN_X,
                "focus_pixel_x": CARD_CENTER_X,
                "pixel_y": round(layout["center"], 2),
                "card_top": layout["top"],
                "card_height": layout["height"],
            })
        diagram_rows.append({
            "path": scene["art"]["path"],
            "scene_id": scene["id"],
            "references": scene["coverage"],
            "svg_sha256": sha256(svg_path),
            "width": scene["art"]["width"],
            "height": scene["art"]["height"],
            "node_centers": node_centers,
            "exact_alt_text": scene["art"]["alt"],
        })

    source_paths = [KJV_PATH, CURRENT_SCENES_PATH, REFERENCE_SVG_PATH, TABERNACLE_SOURCE_PATH]
    provenance = {
        "schema_version": 1,
        "method": "Deterministic native SVG text diagrams rendered from the separate Exodus chapter-companion catalog; no ImageGen or raster placeholders.",
        "source_review": {
            "basis": "Three reviewed Exodus chapter plans for chapters 1–12, 13–24, and 25–40; reviewed planning files are private and are not copied into this repository.",
            "source_rules": [
                "Active ranges are disjoint and cover every KJV verse in each chapter exactly once.",
                "Exodus 1–12 proposed subnodes are used only when their source spans partition the parent unit; otherwise the full parent unit remains one card.",
                "Exodus 13–24 uses the reviewed verse units; Exodus 25–40 uses exhaustive chapter units with their mapped explanatory limits.",
                "Object-detail art reuses existing Tabernacle object reconstructions only for matching object subjects and is not evidence for a complete unit, measured pattern, or geography.",
            ],
            "early_plan_partition": {
                "parent_units": 68,
                "safe_proposed_subnodes": 122,
                "unsplittable_parent_units_kept_whole": 23,
                "unsplittable_reason": "Their proposed source spans were incomplete, overlapping, outside the parent unit, or absent; the reviewed parent ranges still partition the chapter.",
                "resulting_nodes": 145,
            },
            "phase_node_counts": {"exodus_1_12": 145, "exodus_13_24": 66, "exodus_25_40": 70},
            "detail_image_check": "Pillow confirmed all eight reused WebP files are 1536x1024 RGB. WebP metadata contains no alt field; exact alt text is copied from src/tabernacle.rs.",
        },
        "scripture_source": {
            "path": "data/kjv.json",
            "sha256": sha256(KJV_PATH),
            "translation": "KJV as stored in the local corpus",
            "book_index": 1,
            "chapters": 40,
            "verses": sum(verse_counts),
        },
        "repository_sources": [
            {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
            for path in source_paths
        ],
        "scope": "Forty editorial chapter diagrams partition all 1,213 Exodus verses into source-based reading stages. They are chapter outlines, not rich narrative paintings, maps, or evidence that distinct events and instructions form one scene.",
        "interaction_coordinates": {
            "pin_x_pixel": CARD_PIN_X,
            "pin_x_percent": CARD_PIN_PERCENT,
            "focus_x_pixel": CARD_CENTER_X,
            "focus_x_percent": CARD_FOCUS_PERCENT,
            "y": "The pin and focus share the card-center y coordinate.",
        },
        "catalog": {"path": "data/journey/exodus-chapter-companions.json", "scene_count": 40, "node_count": total_nodes},
        "assets": diagram_rows,
    }
    write_json(PROVENANCE_PATH, provenance)
    write_json(DATA_PATH, catalog)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--early-plan", type=Path)
    parser.add_argument("--middle-plan", type=Path)
    parser.add_argument("--late-plan", type=Path)
    args = parser.parse_args()
    supplied = [args.early_plan, args.middle_plan, args.late_plan]
    if any(supplied) and not all(supplied):
        parser.error("--early-plan, --middle-plan, and --late-plan must be supplied together")
    _, verse_counts = load_kjv()
    if all(supplied):
        catalog = scenes_from_plans(
            json.loads(args.early_plan.read_text(encoding="utf-8")),
            json.loads(args.middle_plan.read_text(encoding="utf-8")),
            json.loads(args.late_plan.read_text(encoding="utf-8")),
            verse_counts,
        )
    elif DATA_PATH.is_file():
        catalog = json.loads(DATA_PATH.read_text(encoding="utf-8"))
    else:
        parser.error("provide the three reviewed plans for first build, or build the catalog first")
    build_outputs(catalog, verse_counts)
    node_count = sum(len(scene["hotspots"]) for scene in catalog["scenes"])
    print(f"Built {len(catalog['scenes'])} Exodus chapter diagrams and {node_count} source-linked nodes.")


if __name__ == "__main__":
    main()
