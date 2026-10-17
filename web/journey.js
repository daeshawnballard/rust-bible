import { LAST_OPENED_KEY, readProgress, saveProgress, progressHref, matchingVerse, resumeRoute } from "./progress.js";
const LIBRARY_RETURN_KEY = "rust-bible:journey:library-return:v1";
const MAX_SCROLL_Y = 10_000_000;
const mounted = new WeakMap();

function localCalendarDate(now = new Date()) {
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function safeInternalPath(value) {
  if (typeof value !== "string" || !value.startsWith("/") || /[\\\u0000-\u001f\u007f]/.test(value)) return null;
  const pathEnd = value.search(/[?#]/);
  const rawPath = pathEnd < 0 ? value : value.slice(0, pathEnd);
  try {
    const parsed = new URL(value, window.location.origin);
    if (parsed.origin !== window.location.origin || parsed.pathname !== rawPath) return null;
    return `${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return null;
  }
}

export function routeFromHash(hash = window.location.hash) {
  if (typeof hash !== "string" || !hash.startsWith("#/")) return "/";
  return safeInternalPath(hash.slice(1)) || "/";
}

export function resolveTodayDate(route, now = new Date()) {
  const safeRoute = safeInternalPath(route) || "/";
  const split = safeRoute.indexOf("#");
  const fragment = split >= 0 ? safeRoute.slice(split) : "";
  const withoutFragment = split >= 0 ? safeRoute.slice(0, split) : safeRoute;
  const queryAt = withoutFragment.indexOf("?");
  const path = queryAt >= 0 ? withoutFragment.slice(0, queryAt) : withoutFragment;
  if (path !== "/" && path !== "/today") return safeRoute;

  const params = new URLSearchParams(queryAt >= 0 ? withoutFragment.slice(queryAt + 1) : "");
  if (path === "/" || !params.has("date") || !params.get("date")) {
    params.set("date", localCalendarDate(now));
  }
  if (!params.has("edition")) params.set("edition", "2");
  return `${path === "/" ? "/today" : path}?${params.toString()}${fragment}`;
}

function isLibraryRoute(route) {
  const safe = safeInternalPath(route);
  return !!safe && (safe === "/library" || safe.startsWith("/library?"));
}

function bookRouteParts(route) {
  const safe = safeInternalPath(route);
  if (!safe) return null;
  const path = safe.split(/[?#]/, 1)[0];
  const match = /^\/book\/([^/]+)(?:\/([1-9]\d*))?$/.exec(path);
  if (!match || !/^[A-Za-z0-9%\-]+$/.test(match[1])) return null;
  try {
    if (!/^[A-Za-z0-9]+(?:[ -][A-Za-z0-9]+)*$/.test(decodeURIComponent(match[1]))) return null;
  } catch {
    return null;
  }
  return { safe, path, chapter: match[2] ? Number(match[2]) : null };
}

function isReaderRoute(route) {
  const parts = bookRouteParts(route);
  return !!parts?.chapter && parts.safe === parts.path;
}

function isBookNavigationRoute(route) {
  return !!bookRouteParts(route);
}

function isBookSelectionRoute(route) {
  return !!bookRouteParts(route);
}

function isExploreSelectionRoute(route) {
  const safe = safeInternalPath(route);
  return !!safe && /^\/explore\/(?:tabernacle|noahs-ark|sower)\?part=(?:courtyard|altar|basin|tent|table|lamp|veil|ark|construction|door|window|food|path|rocky|thorns|good-soil)&view=(?:scripture|object)$/.test(safe);
}

function readJson(storageName, key) {
  try {
    const storage = window[storageName];
    const value = storage.getItem(key);
    return value ? JSON.parse(value) : null;
  } catch {
    return null;
  }
}

function writeJson(storageName, key, value) {
  try {
    const storage = window[storageName];
    storage.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}

function readLibraryReturn(memoryValue) {
  const value = readJson("sessionStorage", LIBRARY_RETURN_KEY) || memoryValue;
  if (!value || typeof value !== "object") return null;
  const route = safeInternalPath(value.route);
  const scrollY = Number(value.scrollY);
  if (!route || !isLibraryRoute(route) || !Number.isFinite(scrollY) || scrollY < 0 || scrollY > MAX_SCROLL_Y) return null;
  return { route, scrollY, pending: value.pending === true };
}

function routeHref(route) {
  return typeof window.rustBibleApi === "function" ? `#${route}` : route;
}

function currentNativeRoute() {
  const route = `${window.location.pathname}${window.location.search}${window.location.hash}`;
  return safeInternalPath(route) || "/";
}

function nativeNavigationType() {
  try {
    return performance.getEntriesByType("navigation")[0]?.type || "navigate";
  } catch {
    return "navigate";
  }
}

function setHidden(element, hidden) {
  if (element) element.hidden = hidden;
}

function queryAllIncludingRoot(root, selector) {
  const matches = root?.matches?.(selector) ? [root] : [];
  return [...matches, ...(root?.querySelectorAll?.(selector) || [])];
}

function sameBook(left, right) {
  return typeof left === "string" && typeof right === "string"
    && left.trim().toLocaleLowerCase() === right.trim().toLocaleLowerCase();
}

function updateBookActions(root, item, valid) {
  // Covers are reader entry points. Only the book matching the one saved
  // bookmark resumes; other books keep their chapter-one destination.
  for (const card of queryAllIncludingRoot(root, ".library-card[data-book]")) {
    const book = card.dataset.book;
    const matches = valid && item && sameBook(book, item.book);
    for (const link of card.querySelectorAll("[data-book-entry]")) {
      const first = link.dataset.bookFirstRoute || routeFromAnchor(link);
      if (!first) continue;
      link.dataset.bookFirstRoute = first;
      link.setAttribute("href", routeHref(matches ? progressHref(item) : first));
      const label = matches
        ? `Continue ${item.book} ${item.chapter}${item.verse ? `:${item.verse}` : ""}`
        : `Read ${book}`;
      link.setAttribute("aria-label", label);
      if (link.classList.contains("book-read-link")) link.textContent = `${label} →`;
    }
  }
  for (const page of queryAllIncludingRoot(root, "[data-book-page]")) {
    const primary = page.querySelector("[data-book-read-link]");
    if (!primary) continue;
    const label = primary.querySelector("[data-book-read-label]");
    const secondary = page.querySelector("[data-book-start-first]");
    const book = page.dataset.book || "";
    const matches = valid && item && sameBook(book, item.book);
    const displayLabel = item
      ? `${item.book} ${item.chapter}${item.verse ? `:${item.verse}` : ""}`
      : "";

    if (matches) {
      primary.setAttribute("href", routeHref(progressHref(item)));
      primary.setAttribute("aria-label", `Continue reading ${displayLabel}`);
      if (label) label.textContent = `Continue ${displayLabel}`;
      setHidden(secondary, false);
    } else {
      const firstRoute = safeInternalPath(primary.dataset.bookFirstRoute);
      if (firstRoute) primary.setAttribute("href", routeHref(firstRoute));
      primary.setAttribute("aria-label", `Begin reading ${book} chapter 1`);
      if (label) label.textContent = "Begin reading";
      setHidden(secondary, true);
    }
  }
}

async function updateContinuation(root, resolveReaderUri, version, expectedVersion) {
  const item = readProgress();
  let valid = !!item;
  if (item && typeof resolveReaderUri === "function") {
    try {
      valid = !!(await resolveReaderUri(item.route, item));
    } catch {
      valid = false;
    }
  }
  if (version.current !== expectedVersion) return;

  updateBookActions(root, item, valid);
  for (const container of queryAllIncludingRoot(root, "[data-continuation]")) {
    if (container.isConnected === false) continue;
    const link = container.querySelector("[data-continue-link]");
    const label = container.querySelector("[data-continue-label]");
    const empty = container.querySelector("[data-continue-empty]");
    const savedPanel = container.querySelector("[data-continue-saved]");
    const clear = container.querySelector("[data-continue-clear]");

    if (!valid || !item) {
      setHidden(container, container.hasAttribute("data-continue-hide-empty"));
      setHidden(savedPanel, true);
      setHidden(link, true);
      setHidden(empty, false);
      setHidden(clear, true);
      continue;
    }

    const displayLabel = `${item.book} ${item.chapter}${item.verse ? `:${item.verse}` : ""}`;
    if (label) label.textContent = displayLabel;
    if (link) {
      link.setAttribute("href", routeHref(progressHref(item)));
      link.setAttribute("aria-label", `Continue reading ${displayLabel}`);
      if (!label) link.textContent = `Continue reading ${displayLabel} →`;
      link.hidden = false;
    }
    setHidden(savedPanel, false);
    setHidden(empty, true);
    setHidden(clear, false);

    const currentBook = container.closest("[data-book-page]")?.dataset.book;
    const isCurrentBook = container.hasAttribute("data-continue-other-book")
      && sameBook(currentBook, item.book);
    setHidden(container, isCurrentBook);
  }
}

function routeFromAnchor(anchor) {
  const href = anchor?.getAttribute("href");
  if (!href) return null;
  if (href.startsWith("#/")) return safeInternalPath(href.slice(1));
  try {
    const url = new URL(href, window.location.href);
    if (url.origin !== window.location.origin) return null;
    return safeInternalPath(`${url.pathname}${url.search}${url.hash}`);
  } catch {
    return null;
  }
}

export function initJourney(root = document, { resolveReaderUri } = {}) {
  const existing = mounted.get(root);
  if (existing) {
    if (typeof resolveReaderUri === "function") existing.controller.setResolver(resolveReaderUri);
    return existing.controller;
  }

  const eventRoot = root === document ? document : root;
  const controller = new AbortController();
  const { signal } = controller;
  const version = { current: 0 };
  let currentResolver = resolveReaderUri;
  let activeRoute = root === document ? currentNativeRoute() : null;
  let libraryReturnMemory = null;
  let restoreNext = null;

  const persistLibraryReturn = (route, scrollY, pending = false) => {
    const safeRoute = safeInternalPath(route);
    if (!safeRoute || !isLibraryRoute(safeRoute)) return;
    libraryReturnMemory = { route: safeRoute, scrollY: Math.min(MAX_SCROLL_Y, Math.max(0, Number(scrollY) || 0)), pending };
    writeJson("sessionStorage", LIBRARY_RETURN_KEY, libraryReturnMemory);
  };

  const updateReturnLinks = () => {
    const saved = readLibraryReturn(libraryReturnMemory);
    if (!saved) return;
    root.querySelectorAll?.("[data-library-return], .book-breadcrumb a").forEach((link) => {
      link.setAttribute("href", routeHref(saved.route));
    });
  };

  const refreshContinuation = () => {
    version.current += 1;
    const currentVersion = version.current;
    void updateContinuation(root, currentResolver, version, currentVersion);
  };

  let scrollTimer = null, readingDirty = false, restoring = false, scrollIntentUntil = 0;
  const audioOwnsPosition = () => window.biblePlayback?.active?.() || ['starting', 'reading', 'paused'].includes(document.querySelector('[data-global-audio]')?.dataset.audioState);
  const rememberVisibleVerse = () => {
    clearTimeout(scrollTimer);
    if (!readingDirty || restoring || audioOwnsPosition()) return;
    readingDirty = false;
    const toolbar = root.querySelector('.reader-toolbar')?.getBoundingClientRect();
    const panel=root.querySelector('.reader-content');
    const inset=document.body.classList.contains('has-scene')?panel?.getBoundingClientRect():null;
    const line = inset ? inset.top + 35 : Math.max((toolbar?.bottom || 0) + 20, window.innerHeight * .26);
    const verses = [...root.querySelectorAll('[data-reader] .scripture .v')];
    const lastVerse = verses.at(-1);
    const lastRect = lastVerse?.getBoundingClientRect();
    const atDocumentEnd = inset ? panel.scrollTop + panel.clientHeight >= panel.scrollHeight - 4 : window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 4;
    // At the footer the reading line cannot move any farther down. Save the
    // last visible verse instead of leaving the final verses unreachable.
    const reachedLastVerse = atDocumentEnd && lastRect?.top >= 0 && lastRect.bottom <= window.innerHeight;
    const node = (reachedLastVerse ? lastVerse : null) || verses.find(verse => {
      const rect = verse.getBoundingClientRect();
      return rect.bottom > line && rect.top < window.innerHeight - 24;
    }) || verses.filter(verse => {
      const rect = verse.getBoundingClientRect();
      return rect.bottom > (toolbar?.bottom || 0) && rect.top < window.innerHeight;
    }).at(-1);
    if (node) saveProgress(node);
  };
  const readReaderOnPage = (route, response, initial) => {
    clearTimeout(scrollTimer);
    readingDirty = false;
    const reader = root.querySelector('section[data-reader]');
    if (!reader) return null;
    const saved = readProgress();
    const params = new URL(route, window.location.origin).searchParams;
    const requested = Number(params.get(root.querySelector('[data-atlas-page]') ? 'verse' : 'resume'));
    const explicit = response?.anchor && document.getElementById(response.anchor);
    const resumed = requested > 0 ? matchingVerse(root, {route: reader.dataset.readerRoute, verse: requested})
      : initial && !response?.anchor && !params.has('v') && saved?.route === reader.dataset.readerRoute ? matchingVerse(root, saved) : null;
    const playing = audioOwnsPosition() ? matchingVerse(root,window.biblePlayback?.position?.()) : null;
    const node = playing || resumed || (explicit?.matches('.scripture .v') ? explicit : null)
      || reader.querySelector('[data-reader-primary-scripture] .hl') || reader.querySelector('.scripture .v');
    if (node && !audioOwnsPosition()) saveProgress(node, resumed && saved?.source === 'listening' ? 'listening' : 'reading');
    if(node && audioOwnsPosition()) reader.dataset.readerPosition=node.id;
    // The caller scrolls once after rendering. Ignore its scroll events, then
    // begin tracking user scrolls. A new route cannot inherit an old timer.
    restoring = true;
    setTimeout(() => { restoring = false; }, 350);
    return playing?.id || (resumed ? resumed.id : null);
  };

  const rememberLibraryBeforeNavigation = (nextRoute) => {
    const next = safeInternalPath(nextRoute);
    const previous = activeRoute && safeInternalPath(activeRoute);
    if (root.querySelector?.("[data-library]") && previous && isLibraryRoute(previous) && next && isBookNavigationRoute(next)) {
      persistLibraryReturn(previous, window.scrollY, false);
    }
  };

  const handleClick = (event) => {
    const target = event.target instanceof Element ? event.target : event.target?.parentElement;
    const anchor = target?.closest?.("a[href]");
    if (anchor?.matches(".skip-link")) { event.preventDefault(); const main = document.getElementById("main-content"); main?.focus({preventScroll:true}); main?.scrollIntoView({block:"start"}); return; }
    if (anchor && eventRoot.contains(anchor) && anchor.target !== "_blank") {
      const candidateRoute = routeFromAnchor(anchor);
      const candidatePath = candidateRoute?.split(/[?#]/, 1)[0];
      const isTodayLink = anchor.matches("[data-today-link], .site-navigation a[data-today], .site-navigation a[href*='/today']")
        || (anchor.closest(".site-navigation") && anchor.textContent.trim().toLowerCase() === "today" && (candidatePath === "/" || candidatePath === "/today"));
      if (isTodayLink) {
        const date = localCalendarDate();
        const hrefRoute = candidateRoute || "/today";
        const safe = safeInternalPath(hrefRoute) || "/today";
        const split = safe.indexOf("#");
        const pathAndQuery = split >= 0 ? safe.slice(0, split) : safe;
        const path = pathAndQuery.split("?", 1)[0];
        const params = new URLSearchParams(pathAndQuery.split("?")[1] || "");
        params.set("date", date);
        if (!params.has("edition")) params.set("edition", "2");
        const todayRoute = `${path === "/" ? "/today" : path}?${params.toString()}`;
        anchor.setAttribute("href", routeHref(todayRoute));
      }
      const nextRoute = routeFromAnchor(anchor);
      if (nextRoute) { clearTimeout(scrollTimer); readingDirty = false; scrollIntentUntil = 0; rememberLibraryBeforeNavigation(nextRoute); }

      if (anchor.matches("[data-library-return], .book-breadcrumb a")) {
        const saved = readLibraryReturn(libraryReturnMemory);
        if (saved && nextRoute === saved.route) {
          persistLibraryReturn(saved.route, saved.scrollY, true);
        }
      }
    }

    const clear = target?.closest?.("[data-continue-clear]");
    if (clear && eventRoot.contains(clear)) {
      event.preventDefault();
      try { window.localStorage.removeItem(LAST_OPENED_KEY); } catch {}
      refreshContinuation();
    }
  };

  const handleChange = (event) => {
    const select = event.target;
    if (!(select instanceof HTMLSelectElement) || !eventRoot.contains(select)) return;
    const isReaderSelect = select.matches("select[data-reader-book], select[data-reader-chapter]");
    const isExploreSelect = select.matches("select[data-explore-part]");
    if (!isReaderSelect && !isExploreSelect) return;
    const route = select.value.startsWith("#/") ? routeFromHash(select.value) : safeInternalPath(select.value);
    if (!route || (isReaderSelect ? !isBookSelectionRoute(route) : !isExploreSelectionRoute(route))) return;
    if (typeof window.rustBibleApi === "function") {
      const hash = `#${route}`;
      if (window.location.hash !== hash) window.location.hash = hash;
    } else {
      window.location.assign(route);
    }
  };

  const markScrollIntent = () => { scrollIntentUntil = Date.now() + 1800; };
  window.addEventListener('wheel', markScrollIntent, {passive: true, signal});
  window.addEventListener('touchmove', markScrollIntent, {passive: true, signal});
  window.addEventListener('keydown', event => {
    if (['ArrowDown', 'ArrowUp', 'PageDown', 'PageUp', 'Home', 'End', ' '].includes(event.key)
      && !event.target?.closest?.('input, textarea, select, button, [contenteditable]')) markScrollIntent();
  }, {signal});
  window.addEventListener('pointerdown', event => {
    if(event.target?.closest?.('.reader-content'))markScrollIntent();
    if (event.clientX >= document.documentElement.clientWidth - 20) markScrollIntent();
  }, {signal});
  window.addEventListener('scroll', () => {
    if (Date.now() > scrollIntentUntil || restoring || !root.querySelector('[data-reader]') || audioOwnsPosition()) return;
    readingDirty = true;
    clearTimeout(scrollTimer);
    scrollTimer = setTimeout(rememberVisibleVerse, 180);
  }, {passive: true, signal});
  eventRoot.addEventListener('bible:panel-scroll',()=>{if(Date.now()<=scrollIntentUntil&&!audioOwnsPosition()&&!restoring){readingDirty=true;clearTimeout(scrollTimer);scrollTimer=setTimeout(rememberVisibleVerse,200);}},{signal});
  window.addEventListener('pagehide', rememberVisibleVerse, {signal});
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden' && readingDirty && !audioOwnsPosition()) {
      // Use the last rendered geometry even when the page is backgrounded.
      rememberVisibleVerse();
    }
  }, {signal});
  eventRoot.addEventListener("bible:position",refreshContinuation,{signal});
  eventRoot.addEventListener("click", handleClick, { signal });
  eventRoot.addEventListener("change", handleChange, { signal });

  const api = {
    beforeRender(nextRoute) {
      clearTimeout(scrollTimer);
      readingDirty = false;
      scrollIntentUntil = 0;
      const next = safeInternalPath(nextRoute) || "/";
      rememberLibraryBeforeNavigation(next);
      const previous = activeRoute && safeInternalPath(activeRoute);
      const saved = readLibraryReturn(libraryReturnMemory);
      if (previous && isBookNavigationRoute(previous) && isLibraryRoute(next) && saved?.route === next) {
        restoreNext = saved;
        persistLibraryReturn(saved.route, saved.scrollY, true);
      } else {
        restoreNext = null;
      }
    },
    afterRender(route, response, { initial = false } = {}) {
      const safeRoute = safeInternalPath(route) || "/";
      activeRoute = safeRoute;
      const restoreVerseId = readReaderOnPage(safeRoute, response, initial);
      updateReturnLinks();
      refreshContinuation();

      const saved = readLibraryReturn(libraryReturnMemory);
      let scrollY = null;
      if (isLibraryRoute(safeRoute) && saved?.route === safeRoute) {
        if (restoreNext || saved.pending || (initial && nativeNavigationType() === "back_forward")) {
          scrollY = saved.scrollY;
          persistLibraryReturn(saved.route, saved.scrollY, false);
        }
      }
      restoreNext = null;

      if (!initial) {
        const main = root.querySelector?.("main#main-content");
        try { main?.focus({ preventScroll: true }); } catch { main?.focus(); }
      }
      return { restoreVerseId, restoreScrollY: response?.anchor ? null : scrollY };
    },
    dispose() {
      rememberVisibleVerse();
      clearTimeout(scrollTimer);
      controller.abort();
      mounted.delete(root);
    },
  };
  const record = { controller: api };
  mounted.set(root, record);

  Object.defineProperty(api, "setResolver", {
    value(resolver) {
      if (typeof resolver === "function") {
        currentResolver = resolver;
      }
    },
  });
  refreshContinuation();
  return api;
}

export function disposeJourney(root = document) {
  mounted.get(root)?.controller.dispose();
}

function autoInitNativeJourney() {
  if (typeof window.rustBibleApi === "function") return;
  if (!document.querySelector("main#main-content")) return;
  const resumed = resumeRoute(currentNativeRoute());
  if (resumed !== currentNativeRoute()) { window.location.replace(resumed); return; }
  const journey = initJourney(document);
  const result = journey.afterRender(currentNativeRoute(), null, { initial: true });
  if (result.restoreVerseId) {
    requestAnimationFrame(() => document.getElementById(result.restoreVerseId)?.scrollIntoView({block: 'center', behavior: 'instant'}));
  } else if (result.restoreScrollY !== null && result.restoreScrollY !== undefined) {
    requestAnimationFrame(() => window.scrollTo({ top: result.restoreScrollY, left: 0, behavior: "instant" }));
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", autoInitNativeJourney, { once: true });
} else {
  autoInitNativeJourney();
}
