const LAST_OPENED_KEY = "rust-bible:journey:last-opened:v1";
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
  if (!params.has("edition")) params.set("edition", "1");
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
  return !!safe && /^\/explore\/tabernacle\?part=(?:courtyard|altar|basin|tent|table|lamp|veil|ark)&view=(?:scripture|object)$/.test(safe);
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

function canonicalSavedReader(value) {
  if (!value || typeof value !== "object") return null;
  const route = safeInternalPath(value.route);
  if (!route || !isReaderRoute(route)) return null;
  const book = typeof value.book === "string" ? value.book.trim() : "";
  if (!book || book.length > 80) return null;
  const routeChapter = Number(route.match(/\/([1-9]\d*)$/)?.[1]);
  const chapter = Number.isInteger(value.chapter) && value.chapter > 0 ? value.chapter : routeChapter;
  if (!Number.isInteger(chapter) || chapter !== routeChapter) return null;
  const result = {
    route,
    book,
    chapter,
    savedAt: Number.isFinite(value.savedAt) ? value.savedAt : 0,
  };
  const bookIndex = Number(value.bookIndex);
  if (Number.isInteger(bookIndex) && bookIndex >= 0 && bookIndex < 66) result.bookIndex = bookIndex;
  return result;
}

function savedReader() {
  return canonicalSavedReader(readJson("localStorage", LAST_OPENED_KEY));
}

function saveReader(section) {
  const route = safeInternalPath(section?.dataset.readerRoute);
  const book = section?.dataset.bookName?.trim();
  if (!route || !isReaderRoute(route) || !book || book.length > 80) return;
  const routeChapter = Number(route.match(/\/([1-9]\d*)$/)?.[1]);
  const dataChapter = Number(section.dataset.chapter);
  if (Number.isFinite(dataChapter) && dataChapter > 0 && dataChapter !== routeChapter) return;
  const record = { route, book, chapter: routeChapter, savedAt: Date.now() };
  const bookIndex = Number(section.dataset.bookIndex);
  if (Number.isInteger(bookIndex) && bookIndex >= 0 && bookIndex < 66) record.bookIndex = bookIndex;
  writeJson("localStorage", LAST_OPENED_KEY, record);
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

async function updateContinuation(root, resolveReaderUri, version, expectedVersion) {
  const container = root.querySelector("[data-continuation]");
  if (!container) return;
  const link = container.querySelector("[data-continue-link]");
  const label = container.querySelector("[data-continue-label]");
  const empty = container.querySelector("[data-continue-empty]");
  const savedPanel = container.querySelector("[data-continue-saved]");
  const clear = container.querySelector("[data-continue-clear]");
  const item = savedReader();
  let valid = !!item;
  if (item && typeof resolveReaderUri === "function") {
    try {
      valid = !!(await resolveReaderUri(item.route));
    } catch {
      valid = false;
    }
  }
  if (!container.isConnected || version.current !== expectedVersion) return;
  if (!valid || !item) {
    setHidden(savedPanel, true);
    setHidden(link, true);
    setHidden(empty, false);
    setHidden(clear, true);
    return;
  }
  const displayLabel = `${item.book} ${item.chapter}`;
  if (label) label.textContent = displayLabel;
  if (link) {
    link.setAttribute("href", routeHref(item.route));
    link.setAttribute("aria-label", `Continue reading ${displayLabel}`);
    if (!label) link.textContent = `Continue reading ${displayLabel} →`;
    link.hidden = false;
  }
  setHidden(savedPanel, false);
  setHidden(empty, true);
  setHidden(clear, false);
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

  const readReaderOnPage = () => {
    const section = root.querySelector?.("section[data-reader]");
    if (section) saveReader(section);
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
        if (!params.has("edition")) params.set("edition", "1");
        const todayRoute = `${path === "/" ? "/today" : path}?${params.toString()}`;
        anchor.setAttribute("href", routeHref(todayRoute));
      }
      const nextRoute = routeFromAnchor(anchor);
      if (nextRoute) rememberLibraryBeforeNavigation(nextRoute);

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

  eventRoot.addEventListener("click", handleClick, { signal });
  eventRoot.addEventListener("change", handleChange, { signal });

  const api = {
    beforeRender(nextRoute) {
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
      readReaderOnPage();
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
      return { restoreScrollY: response?.anchor ? null : scrollY };
    },
    dispose() {
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
  const journey = initJourney(document);
  const result = journey.afterRender(currentNativeRoute(), null, { initial: true });
  if (result.restoreScrollY !== null && result.restoreScrollY !== undefined) {
    requestAnimationFrame(() => window.scrollTo({ top: result.restoreScrollY, left: 0, behavior: "instant" }));
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", autoInitNativeJourney, { once: true });
} else {
  autoInitNativeJourney();
}
