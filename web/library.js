const LIBRARY_SELECTOR = "section.library-page[data-library]";
const MOBILE_QUERY = "(max-width: 760px)";
const TOOLS_STATE_KEY = "rust-bible-library-tools-open";
const DEFAULTS = {
  testament: "all",
  category: "all",
  sort: "bible",
  size: "240",
};

const mountedLibraries = new WeakMap();
let rememberedToolsOpen = null;
let mobileCollapseApplied = false;

function readRememberedToolsOpen() {
  try {
    const stored = window.sessionStorage.getItem(TOOLS_STATE_KEY);
    if (stored === "open") return true;
    if (stored === "closed") return false;
  } catch {
    // The in-memory state still covers client-side route changes.
  }
  return rememberedToolsOpen;
}

function rememberToolsOpen(isOpen) {
  rememberedToolsOpen = isOpen;
  try {
    window.sessionStorage.setItem(TOOLS_STATE_KEY, isOpen ? "open" : "closed");
  } catch {
    // The in-memory state still covers client-side route changes.
  }
}

function findLibrary(root) {
  if (root instanceof Element && root.matches(LIBRARY_SELECTOR)) return root;
  return root?.querySelector?.(LIBRARY_SELECTOR) ?? null;
}

function hashRoutingIsActive() {
  return typeof window.rustBibleApi === "function";
}

function libraryHash(form) {
  const query = new URLSearchParams();
  for (const [name, rawValue] of new FormData(form)) {
    const value = String(rawValue);
    if (value === (DEFAULTS[name] ?? "")) continue;
    query.set(name, value);
  }
  const suffix = query.toString();
  return `#/library${suffix ? `?${suffix}` : ""}`;
}

function updateCoverWidth(grid, slider) {
  if (!grid || !slider) return;
  const value = Number(slider.value);
  if (!Number.isFinite(value)) return;
  const min = Number(slider.min) || 160;
  const max = Number(slider.max) || 340;
  const size = Math.min(max, Math.max(min, value));
  grid.style.setProperty("--cover-width", `${size}px`);
  const output = slider.closest("label")?.querySelector("output");
  if (output) output.textContent = `${size}px`;
}

function submitOrRoute(form) {
  if (hashRoutingIsActive()) {
    window.location.hash = libraryHash(form);
  } else {
    form.requestSubmit();
  }
}

export function disposeLibrary(root = document) {
  const library = findLibrary(root);
  if (!library) return;
  mountedLibraries.get(library)?.dispose();
}

export function initLibrary(root = document) {
  const library = findLibrary(root);
  if (!library) return () => {};

  const mounted = mountedLibraries.get(library);
  if (mounted) return mounted.dispose;

  const form = library.querySelector("#library-tools-form");
  const tools = library.querySelector("details.library-tools[data-library-tools]");
  const grid = library.querySelector(".library-grid");
  const slider = form?.querySelector('input[type="range"][name="size"]');
  const controller = new AbortController();
  const { signal } = controller;

  updateCoverWidth(grid, slider);

  if (tools) {
    let wasMobile = window.matchMedia(MOBILE_QUERY).matches;
    const savedToolsOpen = readRememberedToolsOpen();
    if (savedToolsOpen !== null) {
      tools.open = savedToolsOpen;
    } else if (wasMobile && !mobileCollapseApplied) {
      tools.open = false;
      mobileCollapseApplied = true;
    } else {
      tools.open = true;
    }
    rememberToolsOpen(tools.open);

    tools.addEventListener("toggle", () => {
      rememberToolsOpen(tools.open);
    }, { signal });

    window.addEventListener("resize", () => {
      const isMobile = window.matchMedia(MOBILE_QUERY).matches;
      if (isMobile === wasMobile) return;
      wasMobile = isMobile;
      if (!isMobile) {
        tools.open = true;
        rememberToolsOpen(true);
      }
    }, { signal });
  }

  if (form) {
    form.addEventListener("change", (event) => {
      if (tools) rememberToolsOpen(tools.open);
      const target = event.target;
      if (target instanceof HTMLSelectElement) {
        submitOrRoute(form);
      } else if (target === slider) {
        updateCoverWidth(grid, slider);
        submitOrRoute(form);
      }
    }, { signal });

    form.addEventListener("input", (event) => {
      if (event.target === slider) updateCoverWidth(grid, slider);
    }, { signal });

    form.addEventListener("submit", (event) => {
      if (tools) rememberToolsOpen(tools.open);
      if (!hashRoutingIsActive()) return;
      event.preventDefault();
      window.location.hash = libraryHash(form);
    }, { signal });

    form.querySelector("[data-library-reset]")?.addEventListener("click", (event) => {
      event.preventDefault();
      if (hashRoutingIsActive()) {
        window.location.hash = "#/";
      } else {
        window.location.assign("/");
      }
    }, { signal });
  }

  const dispose = () => {
    if (tools) rememberToolsOpen(tools.open);
    controller.abort();
    mountedLibraries.delete(library);
  };
  mountedLibraries.set(library, { dispose });
  return dispose;
}

function autoInit() {
  initLibrary(document);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", autoInit, { once: true });
} else {
  autoInit();
}
