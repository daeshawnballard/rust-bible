// Source inventory only. Runtime acceptance and legacy story/object worlds are separate.
import fs from 'node:fs';
const catalog = JSON.parse(fs.readFileSync('data/journey/passage-scenes.json', 'utf8'));
const bible = JSON.parse(fs.readFileSync('data/kjv.json', 'utf8'));
const namesBlock = fs.readFileSync('src/lib.rs', 'utf8').match(/const NAMES: \[&str; 66\] = \[([\s\S]*?)\];/)[1];
const names = [...namesBlock.matchAll(/"([^"]+)"/g)].map(match => match[1]);
if (names.length !== 66 || bible.length !== 66) throw Error('Expected the 66-book corpus.');
const intervals = values => {
  const runs = [];
  for (const value of values) {
    const last = runs.at(-1);
    if (last && value === last.last + 1) last.last = value;
    else runs.push({ first: value, last: value });
  }
  return runs;
};
const books = names.map((name, book) => {
  const scenes = catalog.scenes.filter(scene => scene.book === book);
  const chapters = bible[book].chapters.map((verses, i) => {
    const selected = new Set();
    const chapterScenes = scenes.filter(scene => scene.ranges.some(r => r.chapter === i + 1));
    for (const scene of chapterScenes) for (const r of scene.ranges.filter(r => r.chapter === i + 1)) {
      if (r.first < 1 || r.last > verses.length || r.first > r.last) throw Error(`Invalid range in ${scene.id}`);
      for (let v = r.first; v <= r.last; v++) {
        if (selected.has(v)) throw Error(`Overlapping catalog range at ${name} ${i + 1}:${v}`);
        selected.add(v);
      }
    }
    return { chapter: i + 1, verse_count: verses.length, scene_ids: chapterScenes.map(s => s.id), selected_verse_count: selected.size,
      selected_ranges: intervals([...selected].sort((a, b) => a - b)),
      verses_without_catalog_scene: intervals(verses.map((_, v) => v + 1).filter(v => !selected.has(v))) };
  });
  return { book, name, chapter_count: chapters.length, catalog_scenes: scenes.length,
    catalog_hotspots: scenes.reduce((n, s) => n + s.hotspots.length, 0),
    chapters_with_catalog_scene: chapters.filter(c => c.scene_ids.length).length, chapters };
});
const output = { schema_version: 1, scope: 'Consolidated focused-scene catalog only. Chapter diagrams, legacy story worlds, Eden anchors, object studies and broad book context are excluded. An unassigned catalog verse may have one of those existing companions. Source inventory is not live verification.',
  source_files: ['data/journey/passage-scenes.json', 'data/kjv.json'],
  summary: { books: books.length, chapters: books.reduce((n, b) => n + b.chapter_count, 0), scenes: catalog.scenes.length, hotspots: books.reduce((n, b) => n + b.catalog_hotspots, 0) }, books };
fs.writeFileSync('data/journey/catalog-coverage.json', JSON.stringify(output, null, 2) + '\n');
let markdown = '# Focused Journey catalog coverage\n\nGenerated with `node scripts/journey-coverage.mjs` from the embedded KJV and consolidated scene catalog. This counts authored source ranges, not live acceptance or complete chapters. Chapter diagrams, legacy worlds, object studies and broad context are separate; a chapter absent here can still have those companions. Genesis’s complete selected-episode ledger includes its legacy scenes in [Genesis chapter coverage](genesis-chapter-coverage.md).\n\n';
markdown += `The current catalog contains **${output.summary.scenes} scenes and ${output.summary.hotspots} hotspots across all 66 books**. The underlying KJV has ${output.summary.chapters} chapters.\n\n`;
markdown += '| Book | Chapters | Chapters with catalog episodes | Scenes | Hotspots |\n| --- | ---: | ---: | ---: | ---: |\n';
for (const b of books) markdown += `| ${b.name} | ${b.chapter_count} | ${b.chapters_with_catalog_scene} | ${b.catalog_scenes} | ${b.catalog_hotspots} |\n`;
markdown += '\nThe exact selected intervals and their complements are in `data/journey/catalog-coverage.json`. Focused coverage is selected verse intervals, not a claim that the whole chapter has painted artwork.\n\n## Chapter companions\n\nExodus has a separate complete chapter-diagram layer: 40 authored diagrams and 281 source-linked nodes. It covers the reading intervals between focused scenes without expanding a painting beyond its source bounds. See [Exodus chapter companions](exodus-chapter-companions.md). Leviticus adds a separate 27-chapter layer with 169 source-linked nodes; see [Leviticus chapter companions](leviticus-chapter-companions.md). These source inventories do not establish publication or user-surface acceptance.\n';
fs.writeFileSync('docs/journey-coverage.md', markdown);
console.log(JSON.stringify(output.summary));
