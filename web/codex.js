import { mountHtml } from './site-paths.js';

const mounted = new WeakMap();

function currentURL() {
  const route = location.hash.startsWith('#/')
    ? location.hash.slice(1)
    : `${location.pathname}${location.search}${location.hash}`;
  return new URL(route, location.origin);
}

function writeFilters(page, { replace = true, includeSelection = true } = {}) {
  const url = currentURL();
  url.pathname = '/codex';
  url.search = '';
  const book = page.querySelector('[data-codex-book-filter]')?.value;
  const category = page.querySelector('[data-codex-category-filter][aria-pressed="true"]')?.value;
  const query = page.querySelector('[data-codex-query]')?.value.trim();
  if (book && book !== 'all') url.searchParams.set('book', book);
  if (category && category !== 'all') url.searchParams.set('category', category);
  if (query) url.searchParams.set('q', query);
  const selected = includeSelection ? page.dataset.codexSelected : '';
  if (selected) url.searchParams.set('entry', selected);
  const route = `${url.pathname}${url.search}`;
  const next = location.hash.startsWith('#/') ? `#${route}` : route;
  if (replace) history.replaceState(history.state, '', next);
  else history.pushState(history.state, '', next);
}

function detailMarkup(id) {
  const apiURL = `/api/codex?entry=${encodeURIComponent(id)}`;
  if (typeof window.rustBibleApi === 'function') {
    return Promise.resolve(window.rustBibleApi(apiURL).detail_html);
  }
  return fetch(apiURL, { headers: { Accept: 'application/json' } }).then(response => {
    if (!response.ok) throw new Error(`Codex detail returned HTTP ${response.status}.`);
    return response.json();
  }).then(response => response.detail_html);
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
  const preview = page.querySelector('[data-codex-preview]');
  let timer;
  let renderId = 0;

  const setSelection = async (id, {
    writeURL = true,
    replaceURL = false,
    open = true,
    load = true,
  } = {}) => {
    const card = cards.find(item => item.dataset.codexEntryId === id && !item.hidden);
    if (!card) id = '';
    const previous = page.dataset.codexSelected || '';
    page.dataset.codexSelected = id;
    page.dataset.codexSelectionOpen = String(Boolean(id && open));
    for (const item of cards) {
      const link = item.querySelector('[data-codex-entry]');
      if (item.dataset.codexEntryId === id) link?.setAttribute('aria-current', 'page');
      else link?.removeAttribute('aria-current');
    }
    if (writeURL) writeFilters(page, { replace: replaceURL });

    const currentDetail = preview?.querySelector('[data-codex-detail]');
    if (!id) {
      renderId += 1;
      if (preview) preview.hidden = true;
    } else if (preview) {
      preview.hidden = false;
      if (load && (previous !== id || currentDetail?.dataset.codexEntryId !== id)) {
        const thisRender = ++renderId;
        preview.setAttribute('aria-busy', 'true');
        preview.innerHTML = '<p role="status" class="codex-detail-status">Opening study note…</p>';
        try {
          const html = await detailMarkup(id);
          if (!html) throw new Error('Codex detail is unavailable.');
          if (thisRender === renderId && page.dataset.codexSelected === id) {
            preview.innerHTML = mountHtml(html);
          }
        } catch {
          if (thisRender === renderId && page.dataset.codexSelected === id) {
            preview.innerHTML = '<p role="status" class="codex-detail-status">This note could not load. Select it again to retry.</p>';
          }
        } finally {
          if (thisRender === renderId) preview.removeAttribute('aria-busy');
        }
      }
    }

    return card;
  };

  const applyFilters = (writeURL = true) => {
    const book = page.querySelector('[data-codex-book-filter]')?.value || 'all';
    const category = page.querySelector('[data-codex-category-filter][aria-pressed="true"]')?.value || 'all';
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

    const oldSelection = page.dataset.codexSelected || '';
    const selectedCard = cards.find(card => card.dataset.codexEntryId === oldSelection && !card.hidden);
    const nextCard = selectedCard || cards.find(card => !card.hidden);
    const selectionChanged = (nextCard?.dataset.codexEntryId || '') !== oldSelection;
    if (selectionChanged) {
      void setSelection(nextCard?.dataset.codexEntryId || '', {
        writeURL: false,
        open: page.dataset.codexSelectionOpen === 'true',
      });
    }
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
    if (event.target.matches('[data-codex-book-filter]')) applyFilters();
  }, { signal });
  page.addEventListener('click', event => {
    const back = event.target.closest('[data-codex-back-to-index]');
    if (back && page.contains(back)) {
      event.preventDefault();
      void setSelection(page.dataset.codexSelected || '', { writeURL: false, open: false, load: false });
      writeFilters(page, { replace: true, includeSelection: false });
      page.querySelector('.codex-index')?.scrollIntoView({ block: 'start', behavior: 'smooth' });
      page.querySelector(`[data-codex-entry="${page.dataset.codexSelected}"]`)?.focus({ preventScroll: true });
      return;
    }
    const tab = event.target.closest('[data-codex-category-filter]');
    if (tab && page.contains(tab)) {
      for (const button of page.querySelectorAll('[data-codex-category-filter]')) {
        const active = button === tab;
        button.setAttribute('aria-pressed', String(active));
        button.classList.toggle('is-active', active);
      }
      applyFilters();
      return;
    }
    const entry = event.target.closest('a[data-codex-entry]');
    if (entry && page.contains(entry)) {
      event.preventDefault();
      const id = entry.dataset.codexEntry;
      void setSelection(id, { writeURL: true, replaceURL: false, open: true });
      if (window.matchMedia('(max-width: 760px)').matches) {
        requestAnimationFrame(() => preview?.scrollIntoView({ block: 'start', behavior: 'smooth' }));
      }
      return;
    }
    if (event.target.closest('[data-codex-read]')) window.biblePlayback?.stop();
  }, { signal });
  page.addEventListener('submit', event => {
    if (event.target.matches('[data-codex-filters]')) {
      event.preventDefault();
      applyFilters();
    }
  }, { signal });
  window.addEventListener('popstate', () => {
    const url = currentURL();
    const book = page.querySelector('[data-codex-book-filter]');
    const query = page.querySelector('[data-codex-query]');
    if (book) book.value = url.searchParams.get('book') || 'all';
    if (query) query.value = url.searchParams.get('q') || '';
    const category = url.searchParams.get('category') || 'all';
    for (const button of page.querySelectorAll('[data-codex-category-filter]')) {
      const active = button.value === category;
      button.setAttribute('aria-pressed', String(active));
      button.classList.toggle('is-active', active);
    }
    applyFilters(false);
    const entry = url.searchParams.get('entry');
    const fallback = cards.find(card => !card.hidden && card.querySelector('.codex-row-art'))
      || cards.find(card => !card.hidden);
    void setSelection(entry || fallback?.dataset.codexEntryId || '', {
      writeURL: false,
      open: Boolean(entry),
    });
  }, { signal });

  const selected = page.dataset.codexSelected || '';
  if (selected) {
    for (const card of cards) {
      const link = card.querySelector('[data-codex-entry]');
      if (card.dataset.codexEntryId === selected) link?.setAttribute('aria-current', 'page');
    }
  }
  applyFilters(false);
  mounted.set(root, () => { clearTimeout(timer); abort.abort(); });
}
