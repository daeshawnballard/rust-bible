const mounted = new WeakMap();

function currentURL() {
  const route = location.hash.startsWith('#/') ? location.hash.slice(1) : `${location.pathname}${location.search}${location.hash}`;
  return new URL(route, location.origin);
}

function writeFilters(page, { replace = true } = {}) {
  const url = currentURL();
  url.pathname = '/codex';
  url.search = '';
  const book = page.querySelector('[data-codex-book-filter]')?.value;
  const category = page.querySelector('[data-codex-category-filter]')?.value;
  const query = page.querySelector('[data-codex-query]')?.value.trim();
  if (book && book !== 'all') url.searchParams.set('book', book);
  if (category && category !== 'all') url.searchParams.set('category', category);
  if (query) url.searchParams.set('q', query);
  const route = `${url.pathname}${url.search}`;
  const next = location.hash.startsWith('#/') ? `#${route}` : route;
  if (replace) history.replaceState(history.state, '', next);
  else location.hash = route;
}

export function disposeCodex(root = document) {
  mounted.get(root)?.();
  mounted.delete(root);
}

export function initCodex(root = document) {
  disposeCodex(root);
  const page = root.matches?.('.codex-page[data-codex-page]')
    ? root
    : root.querySelector('.codex-page[data-codex-page]');
  if (!page) return;
  const abort = new AbortController();
  const { signal } = abort;
  const cards = [...page.querySelectorAll('[data-codex-card]')];
  const groups = [...page.querySelectorAll('[data-codex-book-group]')];
  const count = page.querySelector('[data-codex-count]');
  const empty = page.querySelector('[data-codex-empty]');
  let timer;

  const applyFilters = (writeURL = true) => {
    const book = page.querySelector('[data-codex-book-filter]')?.value || 'all';
    const category = page.querySelector('[data-codex-category-filter]')?.value || 'all';
    const query = page.querySelector('[data-codex-query]')?.value.trim().toLocaleLowerCase() || '';
    let visible = 0;
    for (const card of cards) {
      const matches = (book === 'all' || card.dataset.codexBook === book)
        && (category === 'all' || card.dataset.codexCategory === category)
        && (!query || card.dataset.codexSearch.toLocaleLowerCase().includes(query));
      card.hidden = !matches;
      if (matches) visible += 1;
    }
    for (const group of groups) {
      group.hidden = !group.querySelector('[data-codex-card]:not([hidden])');
    }
    if (count) count.textContent = `${visible} ${visible === 1 ? 'entry' : 'entries'}`;
    if (empty) empty.hidden = visible > 0;
    if (writeURL) writeFilters(page);
  };

  const scheduleSearch = () => {
    clearTimeout(timer);
    timer = setTimeout(() => applyFilters(), 100);
  };
  page.addEventListener('input', event => {
    if (event.target.matches('[data-codex-query]')) scheduleSearch();
  }, { signal });
  page.addEventListener('change', event => {
    if (event.target.matches('[data-codex-book-filter],[data-codex-category-filter]')) applyFilters();
  }, { signal });
  page.addEventListener('submit', event => {
    if (event.target.matches('[data-codex-filters]')) {
      event.preventDefault();
      applyFilters();
    }
  }, { signal });
  applyFilters(false);
  mounted.set(root, () => { clearTimeout(timer); abort.abort(); });
}
