#!/usr/bin/env python3
"""Build the Leviticus chapter-companion catalog and native SVG diagrams.

For first authoring, pass the folder containing the three private reviewed
chapter plans with --plan-dir. Later runs can omit it and rebuild from the
checked-in companion catalog.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import textwrap
from pathlib import Path
from xml.sax.saxutils import escape


ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = Path(__file__).resolve()
DATA_PATH = ROOT / "data/journey/leviticus-chapter-companions.json"
PROVENANCE_PATH = ROOT / "assets/diagrams/leviticus-chapter-companions-provenance.json"
SVG_DIR = ROOT / "web/art/journey"
KJV_PATH = ROOT / "data/kjv.json"
CURRENT_SCENES_PATH = ROOT / "data/journey/passage-scenes.json"

BOOK_INDEX = 2
BOOK_NAME = "Levítico"
BOOK_LABEL = "Leviticus"
CAPTION = "Editorial chapter diagram · the cards follow the source text; they are not a single scene"

SVG_WIDTH = 1536
CARD_X = 120
CARD_WIDTH = 430
CARD_CENTER_X = CARD_X + CARD_WIDTH / 2
CARD_PIN_X = 88
CARD_PIN_PERCENT = round(CARD_PIN_X / SVG_WIDTH * 100, 2)
CARD_FOCUS_PERCENT = round(CARD_CENTER_X / SVG_WIDTH * 100, 2)
TITLE_WRAP = 22
BODY_WRAP = 29
TITLE_LINE_HEIGHT = 36
BODY_LINE_HEIGHT = 30
MIN_CARD_HEIGHT = 170
CARD_GAP = 70
CARD_START_Y = 245

PLAN_GROUPS = (
    ("chapters-01-09.json", 1, 9),
    ("chapters-10-18.json", 10, 18),
    ("chapters-19-27.json", 19, 27),
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def range_object(chapter: int, first: int, last: int) -> dict[str, int]:
    return {"chapter": chapter, "first": first, "last": last}


def finish_sentence(value: str) -> str:
    value = " ".join(value.strip().split())
    if not value:
        return value
    if value[-1] not in ".!?;:":
        value += "."
    return value


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


def plan_catalog(plan_dir: Path, verse_counts: list[int]) -> dict:
    chapters_by_number: dict[int, dict] = {}
    for filename, first_chapter, last_chapter in PLAN_GROUPS:
        path = plan_dir / filename
        if not path.is_file():
            raise FileNotFoundError(f"Required reviewed chapter plan is missing: {filename}")
        plan = json.loads(path.read_text(encoding="utf-8"))
        if plan.get("book") != BOOK_INDEX:
            raise ValueError(f"{filename} must declare book {BOOK_INDEX}")
        chapters = plan.get("chapters")
        expected = list(range(first_chapter, last_chapter + 1))
        if not isinstance(chapters, list) or [row.get("chapter") for row in chapters] != expected:
            raise ValueError(f"{filename} must contain chapters {first_chapter}–{last_chapter} in order")
        for row in chapters:
            chapter = row["chapter"]
            if chapter in chapters_by_number:
                raise ValueError(f"Duplicate plan chapter {chapter}")
            chapters_by_number[chapter] = row

    if set(chapters_by_number) != set(range(1, 28)):
        raise ValueError("The three Leviticus plans must cover chapters 1–27 exactly once")

    scenes = []
    for chapter in range(1, 28):
        plan_chapter = chapters_by_number[chapter]
        verse_count = verse_counts[chapter - 1]
        if plan_chapter.get("verse_count") != verse_count:
            raise ValueError(
                f"Leviticus {chapter} plan says {plan_chapter.get('verse_count')} verses; local KJV has {verse_count}"
            )
        chapter_title = plan_chapter.get("title")
        units = plan_chapter.get("units")
        if not isinstance(chapter_title, str) or not chapter_title.strip():
            raise ValueError(f"Leviticus {chapter} has no chapter title")
        if not isinstance(units, list) or not units:
            raise ValueError(f"Leviticus {chapter} has no reading units")

        hotspots = []
        previous_last = 0
        for index, unit in enumerate(units, start=1):
            first, last = unit.get("first"), unit.get("last")
            if not isinstance(first, int) or isinstance(first, bool) or not isinstance(last, int) or isinstance(last, bool):
                raise ValueError(f"Leviticus {chapter} unit {index} needs integer verse bounds")
            if first != previous_last + 1 or last < first or last > verse_count:
                raise ValueError(
                    f"Leviticus {chapter} unit {index} breaks ordered verse coverage at {first}–{last}"
                )
            previous_last = last

            title = unit.get("title")
            summary = unit.get("summary")
            note = unit.get("note")
            kind = unit.get("kind")
            anchor = unit.get("anchor")
            detail = unit.get("detail")
            if not isinstance(title, str) or not title.strip() or len(title) > 50:
                raise ValueError(f"Leviticus {chapter} unit {index} title must be 1–50 characters")
            if not isinstance(summary, str) or not summary.strip() or len(summary) > 170:
                raise ValueError(f"Leviticus {chapter} unit {index} summary must be 1–170 characters")
            if not isinstance(note, str) or not note.strip():
                raise ValueError(f"Leviticus {chapter} unit {index} needs its full Context note")
            if not isinstance(kind, str) or not kind.strip():
                raise ValueError(f"Leviticus {chapter} unit {index} needs a reading kind")
            if not isinstance(anchor, int) or isinstance(anchor, bool) or not first <= anchor <= last:
                raise ValueError(f"Leviticus {chapter} unit {index} anchor must lie inside {first}–{last}")
            if detail is not None:
                validate_detail(detail, chapter, index)

            active_range = range_object(chapter, first, last)
            hotspots.append(
                {
                    "id": f"leviticus-chapter-{chapter:02d}-unit-{index:02d}",
                    "title": title.strip(),
                    "x": CARD_PIN_PERCENT,
                    "focus_x": CARD_FOCUS_PERCENT,
                    "y": 0.0,
                    "active": [active_range],
                    "anchor": range_object(chapter, anchor, anchor),
                    "sources": [active_range],
                    "note": note.strip(),
                    "summary": summary.strip(),
                    "kind": kind.strip(),
                    "detail": detail,
                }
            )

        if previous_last != verse_count:
            raise ValueError(f"Leviticus {chapter} plan stops at verse {previous_last}; expected {verse_count}")

        coverage = f"Chapter outline · {BOOK_LABEL} {chapter}:1–{verse_count}"
        scene_title = f"{BOOK_LABEL} {chapter} · {chapter_title.strip()}"
        scenes.append(
            {
                "id": f"leviticus-chapter-{chapter:02d}",
                "book": BOOK_INDEX,
                "title": scene_title,
                "coverage": coverage,
                "caption": CAPTION,
                "ranges": [range_object(chapter, 1, verse_count)],
                "art": {
                    "path": f"/art/journey/leviticus-chapter-{chapter:02d}.svg",
                    "width": SVG_WIDTH,
                    "height": 1024,
                    "alt": (
                        f"Editorial chapter diagram for {BOOK_LABEL} {chapter}: {len(hotspots)} source-based cards "
                        "partition the chapter into reading units. This is not a geographical map or a single scene."
                    ),
                },
                "intro": {
                    "title": scene_title,
                    "note": (
                        f"This native text diagram separates {BOOK_LABEL} {chapter} into source-based reading units. "
                        "Ritual, law, and narrative follow their cited ranges; the cards do not claim to depict one place or moment."
                    ),
                },
                "hotspots": hotspots,
            }
        )
    return {"schema_version": 1, "scenes": scenes}


def validate_detail(detail: dict, chapter: int, index: int) -> None:
    if not isinstance(detail, dict):
        raise ValueError(f"Leviticus {chapter} unit {index} detail must be an Art object or null")
    path = detail.get("path")
    width, height, alt = detail.get("width"), detail.get("height"), detail.get("alt")
    if not isinstance(path, str) or not path.startswith("/art/") or ".." in Path(path).parts:
        raise ValueError(f"Leviticus {chapter} unit {index} detail path must be a repository /art/ URL")
    if (
        not isinstance(width, int)
        or isinstance(width, bool)
        or width <= 0
        or not isinstance(height, int)
        or isinstance(height, bool)
        or height <= 0
    ):
        raise ValueError(f"Leviticus {chapter} unit {index} detail needs positive dimensions")
    if not isinstance(alt, str) or not alt.strip():
        raise ValueError(f"Leviticus {chapter} unit {index} detail needs alt text")
    web_root = (ROOT / "web").resolve()
    asset = (web_root / path.lstrip("/")).resolve()
    if not asset.is_relative_to(web_root) or not asset.is_file():
        raise FileNotFoundError(f"Leviticus {chapter} unit {index} detail asset is absent: {path}")


def load_kjv() -> tuple[list[int], int]:
    books = json.loads(KJV_PATH.read_text(encoding="utf-8"))
    book = books[BOOK_INDEX]
    chapters = book.get("chapters")
    if book.get("name") != BOOK_NAME or not isinstance(chapters, list) or len(chapters) != 27:
        raise ValueError(f"Expected local KJV book index {BOOK_INDEX} to be {BOOK_NAME} with 27 chapters")
    verse_counts = [len(chapter) for chapter in chapters]
    return verse_counts, sum(verse_counts)


def card_layout(hotspot: dict, chapter: int, top: int) -> dict:
    title_lines = wrap_lines(hotspot["title"], TITLE_WRAP)
    note_lines = wrap_lines(hotspot.get("summary") or hotspot["note"], BODY_WRAP)
    height = max(
        MIN_CARD_HEIGHT,
        28
        + len(title_lines) * TITLE_LINE_HEIGHT
        + 7
        + 25
        + 8
        + len(note_lines) * BODY_LINE_HEIGHT
        + 22,
    )
    active = hotspot["active"][0]
    reference = f"{BOOK_LABEL} {chapter}:{display_range(active['first'], active['last'])}"
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
    verse_count = scene["ranges"][0]["last"]
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
            raise ValueError(f"{hotspot['id']} pin y does not match its visible card center")

    title = f"{BOOK_LABEL} {chapter} · Chapter outline"
    chapter_title = scene["title"].split(" · ", maxsplit=1)[-1]
    subtitle = f"{chapter_title} · {BOOK_LABEL} {chapter}:1–{verse_count} · {len(hotspots)} source-based units"
    subtitle_lines = wrap_lines(subtitle, 70)
    desc = (
        f"A vertical column of {len(hotspots)} text cards partitions all {verse_count} verses in {BOOK_LABEL} {chapter}. "
        "The diagram follows source ranges and does not depict one scene or a geographic route."
    )
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_WIDTH}" height="{total_height}" viewBox="0 0 {SVG_WIDTH} {total_height}" role="img" aria-labelledby="title desc">',
        f'  <title id="title">{escape(title)}</title>',
        f'  <desc id="desc">{escape(desc)}</desc>',
        f'  <rect width="{SVG_WIDTH}" height="{total_height}" fill="#f7f5ee"/>',
        '  <defs><marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10Z" fill="#29664d"/></marker></defs>',
        f'  <text x="120" y="90" fill="#172d27" font-family="Georgia,serif" font-size="40">{escape(title)}</text>',
    ]
    for i, line in enumerate(subtitle_lines):
        lines.append(
            f'  <text x="120" y="{142 + i * 32}" fill="#59685f" font-family="Arial,sans-serif" font-size="26">{escape(line)}</text>'
        )
    for index, layout in enumerate(layouts):
        y, height = layout["top"], layout["height"]
        center = layout["center"]
        lines.append(
            f'  <line x1="{CARD_PIN_X}" y1="{center:g}" x2="{CARD_X}" y2="{center:g}" stroke="#29664d" stroke-width="3"/>'
        )
        lines.append(
            f'  <rect x="{CARD_X}" y="{y}" width="{CARD_WIDTH}" height="{height}" rx="18" fill="#eceee4" stroke="#b8cbbb" stroke-width="2"/>'
        )
        lines.append(
            f'  <circle cx="{CARD_PIN_X}" cy="{center:g}" r="10" fill="#29664d" stroke="#f7f5ee" stroke-width="3"/>'
        )
        title_y = y + 43
        for line_index, text in enumerate(layout["title_lines"]):
            baseline = title_y + line_index * TITLE_LINE_HEIGHT
            lines.append(
                f'  <text x="150" y="{baseline}" fill="#172d27" font-family="Georgia,serif" font-size="30">{escape(text)}</text>'
            )
        ref_y = title_y + len(layout["title_lines"]) * TITLE_LINE_HEIGHT + 3
        lines.append(
            f'  <text x="150" y="{ref_y}" fill="#29664d" font-family="Arial,sans-serif" font-size="22">{escape(layout["reference"])}</text>'
        )
        note_y = ref_y + 34
        for line_index, text in enumerate(layout["note_lines"]):
            baseline = note_y + line_index * BODY_LINE_HEIGHT
            lines.append(
                f'  <text x="150" y="{baseline}" fill="#59685f" font-family="Arial,sans-serif" font-size="24">{escape(text)}</text>'
            )
        if index + 1 < len(layouts):
            arrow_start = layout["bottom"] + 10
            arrow_end = layouts[index + 1]["top"] - 10
            lines.append(
                f'  <path d="M{CARD_CENTER_X:g} {arrow_start}V{arrow_end}" fill="none" stroke="#29664d" stroke-width="3" marker-end="url(#arrow)"/>'
            )
    footer_y = total_height - 35
    footer = "Each card follows its cited range; this diagram is a reading outline, not a scene or map."
    lines.append(
        f'  <text x="120" y="{footer_y}" fill="#59685f" font-family="Arial,sans-serif" font-size="22">{escape(footer)}</text>'
    )
    lines.append("</svg>")
    return "\n".join(lines) + "\n", layouts


def validate_catalog(catalog: dict, verse_counts: list[int]) -> int:
    if catalog.get("schema_version") != 1:
        raise ValueError("Leviticus companion catalog must use schema_version 1")
    scenes = catalog.get("scenes")
    if not isinstance(scenes, list) or len(scenes) != 27:
        raise ValueError("Leviticus companion catalog must contain exactly 27 chapters")

    existing = json.loads(CURRENT_SCENES_PATH.read_text(encoding="utf-8"))
    existing_ids = {scene["id"] for scene in existing["scenes"]}
    seen_scene_ids = set()
    seen_unit_ids = set()
    total_units = 0
    for chapter, scene in enumerate(scenes, start=1):
        expected_id = f"leviticus-chapter-{chapter:02d}"
        if scene.get("id") != expected_id or scene.get("book") != BOOK_INDEX:
            raise ValueError(f"Expected Leviticus scene {expected_id} for book index {BOOK_INDEX}")
        if scene["id"] in existing_ids or scene["id"] in seen_scene_ids:
            raise ValueError(f"Scene ID collision: {scene['id']}")
        seen_scene_ids.add(scene["id"])
        count = verse_counts[chapter - 1]
        full_range = range_object(chapter, 1, count)
        if scene.get("ranges") != [full_range]:
            raise ValueError(f"{expected_id} does not declare the full KJV chapter range")
        if scene.get("coverage") != f"Chapter outline · {BOOK_LABEL} {chapter}:1–{count}":
            raise ValueError(f"{expected_id} has an incorrect coverage label")
        if scene.get("caption") != CAPTION:
            raise ValueError(f"{expected_id} has an unexpected caption")
        if scene.get("art", {}).get("path") != f"/art/journey/leviticus-chapter-{chapter:02d}.svg":
            raise ValueError(f"{expected_id} has an unexpected SVG path")
        if scene.get("art", {}).get("width") != SVG_WIDTH:
            raise ValueError(f"{expected_id} has an unexpected SVG width")

        hotspots = scene.get("hotspots")
        if not isinstance(hotspots, list) or not hotspots:
            raise ValueError(f"{expected_id} has no reading units")
        previous_last = 0
        for hotspot in hotspots:
            unit_id = hotspot.get("id")
            if not isinstance(unit_id, str) or unit_id in seen_unit_ids:
                raise ValueError(f"Missing or duplicate unit ID: {unit_id}")
            seen_unit_ids.add(unit_id)
            active = hotspot.get("active", [])
            if len(active) != 1 or active[0].get("chapter") != chapter:
                raise ValueError(f"{unit_id} must have one active range in its chapter")
            first, last = active[0].get("first"), active[0].get("last")
            if (
                not isinstance(first, int)
                or isinstance(first, bool)
                or not isinstance(last, int)
                or isinstance(last, bool)
            ):
                raise ValueError(f"{unit_id} active bounds must be integers")
            if first != previous_last + 1 or last < first or last > count:
                raise ValueError(f"{unit_id} breaks ordered exact verse coverage")
            previous_last = last
            anchor = hotspot.get("anchor")
            if not isinstance(anchor, dict):
                raise ValueError(f"{unit_id} must declare a one-verse anchor")
            anchor_verse = anchor.get("first")
            if (
                not isinstance(anchor_verse, int)
                or isinstance(anchor_verse, bool)
                or anchor != range_object(chapter, anchor_verse, anchor_verse)
            ):
                raise ValueError(f"{unit_id} anchor must be one verse in the same chapter")
            if not first <= anchor_verse <= last:
                raise ValueError(f"{unit_id} anchor falls outside its active range")
            if hotspot.get("sources") != [range_object(chapter, first, last)]:
                raise ValueError(f"{unit_id} source range must match the active range")
            if not hotspot.get("title") or len(hotspot["title"]) > 50:
                raise ValueError(f"{unit_id} title is missing or exceeds 50 characters")
            if not hotspot.get("summary") or len(hotspot["summary"]) > 170:
                raise ValueError(f"{unit_id} summary is missing or exceeds 170 characters")
            if not hotspot.get("note") or not hotspot.get("kind"):
                raise ValueError(f"{unit_id} needs a full note and reading kind")
            if hotspot.get("x") != CARD_PIN_PERCENT or hotspot.get("focus_x") != CARD_FOCUS_PERCENT:
                raise ValueError(f"{unit_id} must use the left-gutter pin and centered focus coordinate")
            if len(wrap_lines(hotspot["title"], TITLE_WRAP)) > 3:
                raise ValueError(f"{unit_id} title wraps to more than three card lines")
            detail = hotspot.get("detail")
            if detail:
                validate_detail(detail, chapter, unit_id)
        if previous_last != count:
            raise ValueError(f"{expected_id} covers through verse {previous_last}, expected {count}")
        total_units += len(hotspots)
    return total_units


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_outputs(catalog: dict, verse_counts: list[int], total_verses: int) -> None:
    for scene in catalog.get("scenes", []):
        for hotspot in scene.get("hotspots", []):
            hotspot["x"] = CARD_PIN_PERCENT
            hotspot["focus_x"] = CARD_FOCUS_PERCENT
    total_units = validate_catalog(catalog, verse_counts)
    SVG_DIR.mkdir(parents=True, exist_ok=True)
    asset_rows = []
    for scene in catalog["scenes"]:
        svg, layouts = render_svg(scene)
        svg_path = ROOT / "web" / scene["art"]["path"].removeprefix("/")
        svg_path.parent.mkdir(parents=True, exist_ok=True)
        svg_path.write_text(svg, encoding="utf-8")
        card_centers = []
        for hotspot, layout in zip(scene["hotspots"], layouts):
            card_centers.append(
                {
                    "id": hotspot["id"],
                    "title": hotspot["title"],
                    "kind": hotspot["kind"],
                    "x": hotspot["x"],
                    "focus_x": hotspot["focus_x"],
                    "y": hotspot["y"],
                    "pixel_x": CARD_PIN_X,
                    "focus_pixel_x": CARD_CENTER_X,
                    "pixel_y": round(layout["center"], 2),
                    "card_top": layout["top"],
                    "card_height": layout["height"],
                }
            )
        asset_rows.append(
            {
                "path": scene["art"]["path"],
                "scene_id": scene["id"],
                "references": scene["coverage"],
                "svg_sha256": sha256(svg_path),
                "width": scene["art"]["width"],
                "height": scene["art"]["height"],
                "node_centers": card_centers,
                "exact_alt_text": scene["art"]["alt"],
            }
        )

    write_json(DATA_PATH, catalog)
    source_paths = [KJV_PATH, SCRIPT_PATH]
    count_by_chapter = [len(scene["hotspots"]) for scene in catalog["scenes"]]
    provenance = {
        "schema_version": 1,
        "method": "Deterministic native SVG text diagrams rendered from a separate Leviticus chapter-companion catalog.",
        "source_review": {
            "plan_path_policy": "Reviewed chapter plans remain private and are not copied into repository outputs.",
            "chapter_plan_groups": [
                {"first_chapter": first, "last_chapter": last}
                for _, first, last in PLAN_GROUPS
            ],
            "active_ranges": "Each chapter is partitioned into ordered, disjoint source ranges covering every local KJV verse exactly once.",
            "anchors": "Every unit anchor is a single verse inside the unit's active range.",
            "card_copy": "The plan summary is stored separately from the full note and is used for SVG card text.",
            "visual_style": {
                "reference_path": "web/art/journey/exodus-chapter-01.svg",
                "applied": "Ivory background, forest-green connectors, muted sage cards, and a left-column reading flow.",
            },
            "detail_scope": "Optional Art references are passed through only when their repository asset exists; the text diagram itself makes no claim to depict the detail or whole passage.",
            "units_per_chapter": count_by_chapter,
            "detail_instances": sum(1 for scene in catalog["scenes"] for hotspot in scene["hotspots"] if hotspot.get("detail")),
        },
        "scripture_source": {
            "path": "data/kjv.json",
            "sha256": sha256(KJV_PATH),
            "translation": "KJV as stored in the local corpus",
            "book_index": BOOK_INDEX,
            "book_name_in_corpus": BOOK_NAME,
            "chapters": len(verse_counts),
            "verses": total_verses,
            "verses_per_chapter": verse_counts,
        },
        "repository_sources": [
            {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
            for path in source_paths
        ],
        "scope": f"Twenty-seven editorial chapter diagrams partition all {total_verses} Leviticus verses into source-based reading units. They are chapter outlines, not a single scene or map.",
        "interaction_coordinates": {
            "pin_x_pixel": CARD_PIN_X,
            "pin_x_percent": CARD_PIN_PERCENT,
            "focus_x_pixel": CARD_CENTER_X,
            "focus_x_percent": CARD_FOCUS_PERCENT,
            "y": "The pin and focus share the card-center y coordinate.",
        },
        "catalog": {
            "path": "data/journey/leviticus-chapter-companions.json",
            "sha256": sha256(DATA_PATH),
            "scene_count": len(catalog["scenes"]),
            "unit_count": total_units,
        },
        "assets": asset_rows,
    }
    write_json(PROVENANCE_PATH, provenance)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plan-dir",
        type=Path,
        help="Private folder containing chapters-01-09.json, chapters-10-18.json, and chapters-19-27.json",
    )
    args = parser.parse_args()
    verse_counts, total_verses = load_kjv()
    if args.plan_dir:
        catalog = plan_catalog(args.plan_dir, verse_counts)
    else:
        if not DATA_PATH.is_file():
            parser.error("--plan-dir is required before the Leviticus catalog exists")
        catalog = json.loads(DATA_PATH.read_text(encoding="utf-8"))
    build_outputs(catalog, verse_counts, total_verses)
    total_units = sum(len(scene["hotspots"]) for scene in catalog["scenes"])
    print(f"Built 27 Leviticus chapter diagrams and {total_units} source-linked units across {total_verses} KJV verses.")


if __name__ == "__main__":
    main()
