// One on-device bookmark shared by reading and listening. No reading history
// is sent to the server. A legacy chapter-only record remains valid.
export const LAST_OPENED_KEY = 'rust-bible:journey:last-opened:v1';

export function canonicalProgress(value) {
  if (!value || typeof value !== 'object') return null;
  const match = typeof value.route === 'string' && /^\/book\/([A-Za-z0-9%-]+)\/([1-9]\d*)$/.exec(value.route);
  if (!match) return null;
  let slug;
  try { slug = decodeURIComponent(match[1]).replaceAll('-', ' '); } catch { return null; }
  const book = typeof value.book === 'string' ? value.book.trim() : '';
  if (!book || !/^[A-Za-z0-9]+(?: [A-Za-z0-9]+)*$/.test(book) || slug.toLowerCase() !== book.toLowerCase()) return null;
  const chapter = Number(match[2]);
  if (chapter > 150 || (value.chapter !== undefined && value.chapter !== chapter)) return null;
  const result = {route: value.route, book, chapter, savedAt: Number.isFinite(value.savedAt) ? value.savedAt : 0};
  if (value.bookIndex !== undefined) {
    if (!Number.isInteger(value.bookIndex) || value.bookIndex < 0 || value.bookIndex >= 66) return null;
    result.bookIndex = value.bookIndex;
  }
  if (value.verse !== undefined) {
    if (!Number.isInteger(value.verse) || value.verse < 1 || value.verse > 176) return null;
    result.verse = value.verse;
  }
  if (typeof value.entryRoute === 'string' && /^\/book\/[A-Za-z0-9%-]+\/[1-9]\d*$/.test(value.entryRoute)) result.entryRoute = value.entryRoute;
  result.source = value.source === 'listening' ? 'listening' : 'reading';
  return result;
}

export function readProgress() {
  try { return canonicalProgress(JSON.parse(localStorage.getItem(LAST_OPENED_KEY))); } catch { return null; }
}

export function positionForVerse(node) {
  const section = node?.closest('[data-reader-route]');
  if (!section || !node.matches('.scripture .v')) return null;
  return canonicalProgress({route: section.dataset.readerRoute, book: section.dataset.bookName,
    bookIndex: Number(section.dataset.bookIndex), chapter: Number(section.dataset.chapter), verse: Number(node.dataset.verse)});
}

export function progressHref(item) {
  return item.verse ? `${item.route}?resume=${item.verse}#v${item.verse}` : item.route;
}

export function resumeRoute(route, item = readProgress(), {reload = false} = {}) {
  // Resume bare reloads without losing the companion the reader selected.
  if (typeof route !== 'string' || !item?.verse) return route;
  const match = /^(\/book\/[A-Za-z0-9%-]+\/[1-9]\d*)(?:\?([^#]*))?(?:#(.*))?$/.exec(route);
  if (!match || item.route !== match[1] && item.entryRoute !== match[1]) return route;
  const params = new URLSearchParams(match[2] || '');
  if (params.has('v')) return route;
  if (match[3] && !params.has('resume')) return route;
  if (!reload && params.has('resume') && Number(params.get('resume')) !== item.verse) return route;
  for (const key of params.keys()) if (!['context','scene','part','resume'].includes(key)) return route;
  params.set('resume', item.verse);
  return `${item.route}?${params}#v${item.verse}`;
}

export function matchingVerse(root, item) {
  if (!item) return null;
  return [...root.querySelectorAll('[data-reader] .scripture .v')].find(node => {
    const position = positionForVerse(node);
    return position?.route === item.route && position.verse === (item.verse || 1);
  }) || null;
}

export function saveProgress(node, source = 'reading') {
  const position = positionForVerse(node);
  if (!position) return false;
  const reader = node.closest('[data-reader]');
  const record = {...position, entryRoute: reader?.dataset.readerRoute || position.route, source, savedAt: Date.now()};
  let persisted = false;
  try { localStorage.setItem(LAST_OPENED_KEY, JSON.stringify(record)); persisted = true; } catch {}
  if (reader) { reader.dataset.readerPosition = node.id; reader.dataset.positionSaved = String(persisted); }
  node.dispatchEvent(new CustomEvent('bible:position', {bubbles: true, detail: {record, persisted, nodeId: node.id}}));
  return persisted;
}

export function saveBookProgress(controls, chapter, verse) {
  const record = canonicalProgress({route: `${controls.dataset.bookRoute}/${chapter}`,
    book: controls.dataset.reference, bookIndex: Number(controls.dataset.bookIndex), chapter, verse,
    source: 'listening', savedAt: Date.now()});
  if (!record) return false;
  try { localStorage.setItem(LAST_OPENED_KEY, JSON.stringify(record)); return true; } catch { return false; }
}
