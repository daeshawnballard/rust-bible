#!/usr/bin/env python3
"""Import a reproducible, KJV-reference Bible atlas from OpenBible.info."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import re
import sys
import urllib.request
from collections import defaultdict
from pathlib import Path
from typing import Any


UPSTREAM_COMMIT = "7eb18a5ee62f27b9b93bd6689ea272d76dd23b8f"
UPSTREAM_BASE = (
    "https://raw.githubusercontent.com/openbibleinfo/"
    f"Bible-Geocoding-Data/{UPSTREAM_COMMIT}/data/"
)
REPOSITORY_URL = (
    "https://github.com/openbibleinfo/Bible-Geocoding-Data/tree/"
    f"{UPSTREAM_COMMIT}"
)
OPENBIBLE_GEO = "https://www.openbible.info/geo"

# OSIS book identifiers in canonical Protestant order. The output uses the
# app's zero-based index: Genesis=0 and Revelation=65.
BOOK_CODES = (
    "Gen Exod Lev Num Deut Josh Judg Ruth 1Sam 2Sam 1Kgs 2Kgs 1Chr 2Chr "
    "Ezra Neh Esth Job Ps Prov Eccl Song Isa Jer Lam Ezek Dan Hos Joel Amos "
    "Obad Jonah Mic Nah Hab Zeph Hag Zech Mal Matt Mark Luke John Acts Rom "
    "1Cor 2Cor Gal Eph Phil Col 1Thess 2Thess 1Tim 2Tim Titus Phlm Heb Jas "
    "1Pet 2Pet 1John 2John 3John Jude Rev"
).split()
BOOK_INDEX = {code: index for index, code in enumerate(BOOK_CODES)}
OSIS_RE = re.compile(r"^([1-3]?[A-Za-z]+)\.(\d+)\.(\d+)$")


class ImportErrorWithContext(RuntimeError):
    pass


def fetch(url: str) -> bytes:
    request = urllib.request.Request(
        url,
        headers={"User-Agent": "rust-bible-atlas-import/1.0"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def parse_jsonl(raw: bytes, label: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line_number, line in enumerate(raw.decode("utf-8").splitlines(), 1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise ImportErrorWithContext(
                f"{label}:{line_number}: invalid JSON: {error}"
            ) from error
        if not isinstance(value, dict):
            raise ImportErrorWithContext(f"{label}:{line_number}: expected object")
        rows.append(value)
    return rows


def parse_osis(reference: str) -> tuple[int, int, int]:
    match = OSIS_RE.fullmatch(reference)
    if not match:
        raise ImportErrorWithContext(f"unsupported OSIS reference: {reference!r}")
    book_code, chapter_text, verse_text = match.groups()
    if book_code not in BOOK_INDEX:
        raise ImportErrorWithContext(f"unknown OSIS book code: {book_code!r}")
    return BOOK_INDEX[book_code], int(chapter_text), int(verse_text)


def parse_lonlat(value: Any, label: str) -> tuple[float, float] | None:
    if not isinstance(value, str):
        return None
    parts = value.split(",")
    if len(parts) != 2:
        return None
    try:
        lon, lat = (float(part) for part in parts)
    except ValueError:
        return None
    if not (math.isfinite(lat) and math.isfinite(lon)):
        return None
    if not (-90 <= lat <= 90 and -180 <= lon <= 180):
        raise ImportErrorWithContext(f"{label}: coordinate outside WGS84 bounds: {value}")
    return lat, lon


def source_description(modern: dict[str, Any]) -> str:
    coordinate_source = modern.get("coordinates_source") or {}
    source_type = coordinate_source.get("type")
    source_id = coordinate_source.get("id")
    source_url = coordinate_source.get("url")
    if not source_url and source_type == "wikidata" and source_id:
        source_url = f"https://www.wikidata.org/wiki/{source_id}"
    if source_url:
        coordinate = f"{source_type or 'coordinate source'}: {source_url}"
    elif source_type and source_id:
        coordinate = f"{source_type} {source_id}"
    elif source_type:
        coordinate = str(source_type)
    else:
        coordinate = "coordinate source unspecified in upstream record"

    precision = modern.get("precision") or {}
    precision_text = precision.get("description")
    meters = precision.get("meters")
    if precision_text and meters is not None:
        precision_text = f"{precision_text} (~{meters} m)"
    elif precision_text and not meters:
        precision_text = f"{precision_text} (distance not quantified)"
    else:
        precision_text = "precision not stated"
    return f"{coordinate}; {precision_text}"


def validate_ref(
    reference: tuple[int, int, int],
    kjv: list[dict[str, Any]],
    context: str,
) -> None:
    book, chapter, verse = reference
    if not 0 <= book < 66:
        raise ImportErrorWithContext(f"{context}: book index outside 0..65")
    chapters = kjv[book].get("chapters")
    if not isinstance(chapters, list) or not 1 <= chapter <= len(chapters):
        raise ImportErrorWithContext(
            f"{context}: chapter {chapter} outside book index {book}"
        )
    verses = chapters[chapter - 1]
    if not isinstance(verses, list) or not 1 <= verse <= len(verses):
        raise ImportErrorWithContext(
            f"{context}: verse {verse} outside book {book} chapter {chapter}"
        )


def confidence_text(association: dict[str, Any], modern: dict[str, Any]) -> str:
    score = association.get("score")
    if isinstance(score, (int, float)) and not isinstance(score, bool):
        if not math.isfinite(score):
            raise ImportErrorWithContext(f"{modern.get('id')}: non-finite adjusted score")
        score_text = f"OpenBible adjusted association score {score:g} (not a probability)"
    else:
        score_text = "OpenBible adjusted association score unavailable"
    return f"{score_text}; {source_description(modern)}"


def build_index(
    ancient_rows: list[dict[str, Any]],
    modern_rows: list[dict[str, Any]],
    kjv: list[dict[str, Any]],
) -> tuple[dict[str, Any], dict[str, int]]:
    if len(BOOK_CODES) != 66 or len(kjv) != 66:
        raise ImportErrorWithContext("expected the 66-book Protestant KJV canon")
    modern_by_id = {row["id"]: row for row in modern_rows if row.get("id")}
    places: dict[str, dict[str, Any]] = {}
    chapter_refs: dict[str, dict[str, set[int]]] = defaultdict(lambda: defaultdict(set))
    candidates_by_ancient: dict[str, list[str]] = {}
    missing_coordinates = 0
    kjv_mentions = 0
    kjv_mentions_without_candidates = 0

    for ancient in ancient_rows:
        ancient_id = ancient.get("id")
        if not isinstance(ancient_id, str):
            continue
        candidate_rows: list[tuple[str, dict[str, Any], dict[str, Any], float, float]] = []
        for modern_id, association in (ancient.get("modern_associations") or {}).items():
            modern = modern_by_id.get(modern_id)
            if not modern:
                continue
            coords = parse_lonlat(modern.get("lonlat"), f"{ancient_id}/{modern_id}")
            if coords is None:
                missing_coordinates += 1
                continue
            lat, lon = coords
            if not isinstance(association, dict):
                continue
            candidate_rows.append((modern_id, association, modern, lat, lon))

        for modern_id, association, modern, lat, lon in candidate_rows:
            stable_id = f"{ancient_id}-{modern_id}"
            ancient_name = ancient.get("friendly_id") or ancient_id
            if ancient_name == "Bethlehem 1":
                ancient_name = "Bethlehem"
            candidate_count = len(candidate_rows)
            candidate_name = modern.get("friendly_id") or association.get("name") or modern_id
            label = (
                str(ancient_name)
                if candidate_count == 1
                else f"{ancient_name} — candidate: {candidate_name}"
            )
            slug = ancient.get("url_slug")
            page_url = f"{OPENBIBLE_GEO}/{slug}" if slug else OPENBIBLE_GEO
            places[stable_id] = {
                "name": label,
                "lat": lat,
                "lon": lon,
                "confidence": confidence_text(association, modern),
                "url": page_url,
            }
        candidates_by_ancient[ancient_id] = [
            f"{ancient_id}-{modern_id}"
            for modern_id, _association, _modern, _lat, _lon in candidate_rows
        ]

        for verse_record in ancient.get("verses", []):
            translations = verse_record.get("translations") or []
            if "kjv" not in translations:
                continue
            osis = (verse_record.get("alternate_verses") or {}).get("kjv") or verse_record.get("osis")
            reference = parse_osis(osis)
            validate_ref(reference, kjv, f"{ancient_id} {osis}")
            kjv_mentions += 1
            if not candidates_by_ancient[ancient_id]:
                kjv_mentions_without_candidates += 1
            book, chapter, verse = reference
            # Revelation's promise/vision of the New Jerusalem is not a
            # geolocated earthly event. OpenBible groups these under the
            # Jerusalem place record, so suppress only those KJV references.
            if (
                ancient.get("friendly_id") == "Jerusalem"
                and book == 65
                and ((chapter == 3 and verse == 12) or (chapter == 21 and verse in {2, 10}))
            ):
                continue
            chapter_key = f"{book}:{chapter}"
            for stable_id in candidates_by_ancient[ancient_id]:
                chapter_refs[chapter_key][stable_id].add(verse)

    chapters: dict[str, list[dict[str, Any]]] = {}
    for chapter_key, by_place in sorted(
        chapter_refs.items(),
        key=lambda item: tuple(int(part) for part in item[0].split(":")),
    ):
        chapters[chapter_key] = [
            {"id": place_id, "verses": sorted(verses)}
            for place_id, verses in sorted(by_place.items())
            if place_id in places and verses
        ]
        if not chapters[chapter_key]:
            del chapters[chapter_key]

    best_place_by_name: dict[str, str] = {}
    for ancient in ancient_rows:
        name = ancient.get("friendly_id")
        ancient_id = ancient.get("id")
        if not name or not ancient_id:
            continue
        candidate_ids = candidates_by_ancient.get(ancient_id, [])
        if not candidate_ids:
            continue
        # Modern associations arrive from the source already ordered as a map;
        # choose the highest adjusted score for deliberately curated focus pins.
        associations = ancient.get("modern_associations") or {}
        candidate_ids.sort(
            key=lambda place_id: (
                -(
                    associations.get(place_id.split("-", 1)[1], {}).get("score")
                    if isinstance(
                        associations.get(place_id.split("-", 1)[1], {}).get("score"),
                        (int, float),
                    )
                    else float("-inf")
                ),
                place_id,
            )
        )
        key = str(name).casefold()
        if key not in best_place_by_name:
            best_place_by_name[key] = candidate_ids[0]

    def focus_item(
        book: int,
        chapter: int,
        start: int,
        end: int,
        name: str,
        note: str,
        source: str,
    ) -> dict[str, Any]:
        place_id = best_place_by_name.get(name.casefold())
        if not place_id:
            raise ImportErrorWithContext(f"curated focus place unavailable: {name}")
        validate_ref((book, chapter, start), kjv, f"focus {name} start")
        validate_ref((book, chapter, end), kjv, f"focus {name} end")
        if end < start:
            raise ImportErrorWithContext(f"focus {name}: end verse precedes start")
        return {
            "book": book,
            "chapter": chapter,
            "start": start,
            "end": end,
            "place": place_id,
            "note": note,
            "source": source,
        }

    editorial_source = "OpenBible.info numeric KJV place references; curated narrative span"
    focus = [
        focus_item(43, 16, 11, 11, "Troas", "Verse 11 names Troas; the verse itself also carries same-verse travel waypoints.", editorial_source),
        focus_item(43, 16, 11, 11, "Samothrace", "Same verse waypoint; within-verse route timing is schematic. KJV uses the spelling Samothracia.", editorial_source),
        focus_item(43, 16, 11, 11, "Neapolis", "Same verse waypoint; within-verse route timing is schematic.", editorial_source),
        focus_item(43, 16, 12, 40, "Philippi", "The narrative remains at Philippi through the end of this chapter.", editorial_source),
        focus_item(43, 17, 1, 9, "Thessalonica", "The Thessalonica episode; the mapped name is a place reference, not a claim about every verse's physical setting.", editorial_source),
        focus_item(43, 17, 10, 14, "Berea", "The Berea episode, bounded to the chapter's stated arrival and departure.", editorial_source),
        focus_item(43, 17, 15, 34, "Athens", "Paul is brought to Athens and remains through the chapter's closing scene.", editorial_source),
        focus_item(41, 2, 4, 7, "Bethlehem 1", "The birth narrative's Bethlehem section.", editorial_source),
        focus_item(41, 2, 22, 38, "Jerusalem", "The presentation and temple encounter section.", editorial_source),
        focus_item(41, 2, 39, 40, "Nazareth", "Return to Nazareth after the Jerusalem visit.", editorial_source),
        focus_item(41, 2, 41, 50, "Jerusalem", "The Passover visit and temple encounter section.", editorial_source),
        focus_item(41, 2, 51, 52, "Nazareth", "Return to Nazareth; no claim about an exact dwelling point.", editorial_source),
        focus_item(65, 1, 9, 9, "Patmos", "John locates himself on Patmos; visionary places are not mapped as earthly events.", editorial_source),
    ]

    def chosen_place(name: str) -> str:
        place_id = best_place_by_name.get(name.casefold())
        if not place_id:
            raise ImportErrorWithContext(f"curated journey stop unavailable: {name}")
        return place_id

    journeys = [
        {
            "id": "acts-16-aegean",
            "book": 43,
            "chapter": 16,
            "title": "Across the Aegean to Macedonia",
            "note": "Schematic connections; exact ancient route unknown.",
            "stops": [
                {"place": chosen_place("Troas"), "verse": 11, "label": "Troas"},
                {"place": chosen_place("Samothrace"), "verse": 11, "label": "Samothracia"},
                {"place": chosen_place("Neapolis"), "verse": 11, "label": "Neapolis"},
                {"place": chosen_place("Philippi"), "verse": 12, "label": "Philippi"},
            ],
        }
    ]

    index = {
        "sources": [
            {
                "name": "OpenBible.info Bible Geocoding Data",
                "url": REPOSITORY_URL,
                "license": "CC BY 4.0; credit OpenBible.info",
            },
            {
                "name": "OpenStreetMap (only for records marked OSM-derived upstream)",
                "url": "https://www.openstreetmap.org/copyright",
                "license": "ODbL 1.0; retain attribution where OSM-derived data is used",
            },
            {
                "name": "Wikidata structured data (where used as a coordinate source upstream)",
                "url": "https://www.wikidata.org/wiki/Wikidata:Licensing",
                "license": "CC0 1.0 for structured data",
            },
        ],
        "places": places,
        "chapters": chapters,
        "focus": focus,
        "journeys": journeys,
    }
    stats = {
        "ancient_records": len(ancient_rows),
        "modern_records": len(modern_rows),
        "places": len(places),
        "chapters": len(chapters),
        "kjv_mention_records": kjv_mentions,
        "kjv_mention_records_without_coordinate_candidates": kjv_mentions_without_candidates,
        "coordinate_candidates_without_lonlat": missing_coordinates,
    }
    validate_index(index, kjv)
    return index, stats


def validate_index(index: dict[str, Any], kjv: list[dict[str, Any]]) -> None:
    expected_keys = {"sources", "places", "chapters", "focus", "journeys"}
    if set(index) != expected_keys:
        raise ImportErrorWithContext("output top-level keys differ from atlas schema")
    places = index["places"]
    for place_id, place in places.items():
        if not isinstance(place_id, str) or not place_id:
            raise ImportErrorWithContext("place IDs must be nonempty strings")
        for axis, low, high in (("lat", -90, 90), ("lon", -180, 180)):
            value = place.get(axis)
            if isinstance(value, bool) or not isinstance(value, (int, float)):
                raise ImportErrorWithContext(f"{place_id}: {axis} must be numeric")
            if not math.isfinite(value) or not low <= value <= high:
                raise ImportErrorWithContext(f"{place_id}: invalid {axis} {value!r}")
        if not all(isinstance(place.get(key), str) and place[key] for key in ("name", "confidence", "url")):
            raise ImportErrorWithContext(f"{place_id}: name/confidence/url required")
    for chapter_key, entries in index["chapters"].items():
        match = re.fullmatch(r"(\d+):(\d+)", chapter_key)
        if not match:
            raise ImportErrorWithContext(f"invalid chapter key {chapter_key!r}")
        book, chapter = map(int, match.groups())
        if not 0 <= book < 66 or not 1 <= chapter <= len(kjv[book]["chapters"]):
            raise ImportErrorWithContext(f"chapter key outside KJV bounds: {chapter_key}")
        seen: set[str] = set()
        for entry in entries:
            place_id = entry.get("id")
            if place_id not in places or place_id in seen:
                raise ImportErrorWithContext(f"{chapter_key}: missing or repeated place {place_id!r}")
            seen.add(place_id)
            verses = entry.get("verses")
            if not isinstance(verses, list) or not verses or verses != sorted(set(verses)):
                raise ImportErrorWithContext(f"{chapter_key}/{place_id}: invalid verse list")
            for verse in verses:
                validate_ref((book, chapter, verse), kjv, f"{chapter_key}/{place_id}")
    for item in index["focus"]:
        place_id = item.get("place")
        if place_id not in places:
            raise ImportErrorWithContext(f"focus references missing place {place_id!r}")
        book, chapter = item.get("book"), item.get("chapter")
        validate_ref((book, chapter, item.get("start")), kjv, "focus start")
        validate_ref((book, chapter, item.get("end")), kjv, "focus end")
    for journey in index["journeys"]:
        book, chapter = journey.get("book"), journey.get("chapter")
        for stop in journey.get("stops", []):
            if stop.get("place") not in places:
                raise ImportErrorWithContext(
                    f"journey references missing place {stop.get('place')!r}"
                )
            validate_ref((book, chapter, stop.get("verse")), kjv, "journey stop")
    if any("text" in entry for entry in index["places"].values()):
        raise ImportErrorWithContext("place records must not contain verse text")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "data" / "atlas" / "index.json",
        help="destination JSON path (default: data/atlas/index.json)",
    )
    parser.add_argument(
        "--kjv",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "data" / "kjv.json",
        help="local KJV JSON used only to validate numeric verse bounds",
    )
    args = parser.parse_args()

    raw_inputs = {
        filename: fetch(UPSTREAM_BASE + filename)
        for filename in ("ancient.jsonl", "modern.jsonl")
    }
    input_hashes = {
        filename: hashlib.sha256(content).hexdigest()
        for filename, content in raw_inputs.items()
    }
    kjv = json.loads(args.kjv.read_text(encoding="utf-8"))
    index, stats = build_index(
        parse_jsonl(raw_inputs["ancient.jsonl"], "ancient.jsonl"),
        parse_jsonl(raw_inputs["modern.jsonl"], "modern.jsonl"),
        kjv,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(index, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    books_with_places = {
        int(key.split(":", 1)[0]) for key in index["chapters"]
    }
    stats["books_with_mapped_places"] = len(books_with_places)
    stats["books_without_mapped_places"] = 66 - len(books_with_places)
    print(f"Wrote {args.output}")
    fetched_date = datetime.now(timezone.utc).date().isoformat()
    print(f"Fetched: {fetched_date} UTC (pinned commit {UPSTREAM_COMMIT})")
    for filename, digest in input_hashes.items():
        print(f"SHA-256 {filename}: {digest}")
    for name, count in stats.items():
        print(f"{name}: {count}")
    print("mapped chapters per book (zero means empty state):")
    counts = [0] * 66
    for chapter_key in index["chapters"]:
        book = int(chapter_key.split(":", 1)[0])
        counts[book] += 1
    for book, code in enumerate(BOOK_CODES):
        print(f"  {book:02d} {code}: {counts[book]}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ImportErrorWithContext, OSError, json.JSONDecodeError, urllib.error.URLError) as error:
        print(f"atlas import failed: {error}", file=sys.stderr)
        raise SystemExit(1)
