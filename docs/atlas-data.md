# Bible atlas data

`data/atlas/index.json` supplies named place candidates, numeric KJV place
references, editorial focus ranges, and a schematic Acts 16 route. The map uses
these records as referenced places; a reference alone does not establish that
the verse's narrated event happened at that coordinate.

## Source and attribution

The importer reads the `ancient.jsonl` and `modern.jsonl` files from
[OpenBible.info's Bible Geocoding Data repository](https://github.com/openbibleinfo/Bible-Geocoding-Data/tree/7eb18a5ee62f27b9b93bd6689ea272d76dd23b8f).
The upstream project describes its data as a catalog of possible modern
locations, with confidence information, and licenses the dataset under
[CC BY 4.0](https://github.com/openbibleinfo/Bible-Geocoding-Data/blob/7eb18a5ee62f27b9b93bd6689ea272d76dd23b8f/license.txt).
Credit OpenBible.info. The upstream README calls out an OpenStreetMap-derived
subset under [ODbL 1.0](https://www.openstreetmap.org/copyright); records
marked as OSM-derived retain that source link in their `confidence` field.
Where a coordinate comes from Wikidata, its item URL is included; Wikidata's
structured data is released under [CC0 1.0](https://www.wikidata.org/wiki/Wikidata:Licensing).

Each place `url` opens the corresponding OpenBible.info ancient-place entry.
The `confidence` field keeps the upstream adjusted association score, the
coordinate source, and the stated point precision. The score is a ranking
measure, not a probability or a guarantee of an ancient site's exact
identification. Some coordinates indicate a modern settlement, a region, or
visible remains rather than a pinpoint location.

## References and map behavior

The upstream reference records describe occurrences across ten English
translations and use ESV-based references when another translation has a
different verse number. This importer keeps a reference only when the upstream
`translations` array includes `kjv`, then applies `alternate_verses.kjv` when
present. It converts the resulting OSIS reference to the app's zero-based book
index and stores only chapter and verse numbers. It never copies Scripture
text or translation prose into the atlas bundle. Existing local KJV data is
read solely to reject out-of-bounds chapter and verse references.

Each `places` ID combines an upstream ancient-place ID and one of its modern
location candidate IDs. When OpenBible lists several coordinate candidates,
the bundle keeps each candidate with its own score and source details. A
chapter entry groups the verse numbers for each candidate. The upstream has
three KJV place-reference records without a coordinate candidate—Bamah
(Ezekiel 20:29), Biziothiah (Joshua 15:28), and Nod (Genesis 4:16)—so they have
no map point. No coordinates are invented for them.

The `focus` records are small editorial ranges for Acts 16–17, Luke 2, and
Revelation 1:9. Acts 16:11 has Troas, Samothrace (KJV spelling: Samothracia),
and Neapolis as same-verse waypoints; the route has no asserted timing within
that verse. Philippi begins at Acts 16:12. Luke's ranges follow the named
Bethlehem, Jerusalem, and Nazareth episodes. Revelation focus is Patmos at
1:9; the visionary New Jerusalem references in Revelation 3:12, 21:2, and
21:10 are deliberately not rendered as earthly map pins. The journey note is
part of the data: “Schematic connections; exact ancient route unknown.”

An absent `chapters` key means this index has no mappable KJV place reference
for that chapter. It should yield an empty map state. This includes all
chapters in Philemon, James, 1 John, 2 John, and 3 John; it does not mean those
books or chapters are missing from the Bible reader.

## Rebuilding and source receipt

Run from the repository root:

```sh
python3 scripts/import-atlas.py
```

The importer pins an immutable upstream commit, downloads the two source files,
prints their SHA-256 hashes and fetch date, validates coordinate ranges and
all generated references against local KJV chapter lengths, then writes the
JSON bundle. To update the source, deliberately change `UPSTREAM_COMMIT`, rerun
the importer, and refresh the receipt below.

Current generated receipt:

- Fetched: 2026-10-16 UTC
- OpenBible.info commit: `7eb18a5ee62f27b9b93bd6689ea272d76dd23b8f` (2021-11-01)
- `ancient.jsonl` SHA-256: `b8187aa4737e8517ccc090f765d2be11da4c548cd2a59d3cdcb62e952cb8c0f2`
- `modern.jsonl` SHA-256: `da731f6e110bac4ea66a9f037a0a31cfb11c4f1efc1206aa9e109092b2c60087`
- 1,342 ancient records and 1,596 modern records read; 4,314 ancient-to-modern
  candidate place records written; 7,600 KJV-qualified place-reference
  records read, with three lacking any coordinate candidate.
- 824 chapters have mapped candidates across 61 books. The other five books
  have no mapped place chapters.

Mapped chapter counts by zero-based book index:

| Index | Book | Chapters | Index | Book | Chapters |
|---:|---|---:|---:|---|---:|
| 0 | Genesis | 43 | 33 | Nahum | 3 |
| 1 | Exodus | 28 | 34 | Habakkuk | 3 |
| 2 | Leviticus | 10 | 35 | Zephaniah | 3 |
| 3 | Numbers | 25 | 36 | Haggai | 1 |
| 4 | Deuteronomy | 30 | 37 | Zechariah | 13 |
| 5 | Joshua | 24 | 38 | Malachi | 4 |
| 6 | Judges | 21 | 39 | Matthew | 21 |
| 7 | Ruth | 3 | 40 | Mark | 14 |
| 8 | 1 Samuel | 30 | 41 | Luke | 19 |
| 9 | 2 Samuel | 23 | 42 | John | 15 |
| 10 | 1 Kings | 22 | 43 | Acts | 28 |
| 11 | 2 Kings | 25 | 44 | Romans | 5 |
| 12 | 1 Chronicles | 27 | 45 | 1 Corinthians | 3 |
| 13 | 2 Chronicles | 36 | 46 | 2 Corinthians | 6 |
| 14 | Ezra | 10 | 47 | Galatians | 3 |
| 15 | Nehemiah | 12 | 48 | Ephesians | 1 |
| 16 | Esther | 7 | 49 | Philippians | 2 |
| 17 | Job | 9 | 50 | Colossians | 3 |
| 18 | Psalms | 54 | 51 | 1 Thessalonians | 4 |
| 19 | Proverbs | 1 | 52 | 2 Thessalonians | 1 |
| 20 | Ecclesiastes | 2 | 53 | 1 Timothy | 1 |
| 21 | Song of Solomon | 8 | 54 | 2 Timothy | 3 |
| 22 | Isaiah | 56 | 55 | Titus | 2 |
| 23 | Jeremiah | 51 | 56 | Philemon | 0 |
| 24 | Lamentations | 4 | 57 | Hebrews | 7 |
| 25 | Ezekiel | 37 | 58 | James | 0 |
| 26 | Daniel | 11 | 59 | 1 Peter | 3 |
| 27 | Hosea | 13 | 60 | 2 Peter | 1 |
| 28 | Joel | 2 | 61 | 1 John | 0 |
| 29 | Amos | 9 | 62 | 2 John | 0 |
| 30 | Obadiah | 1 | 63 | 3 John | 0 |
| 31 | Jonah | 3 | 64 | Jude | 1 |
| 32 | Micah | 7 | 65 | Revelation | 10 |
