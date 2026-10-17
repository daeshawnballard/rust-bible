import { initExperience } from './experience.js';
import { disposeLibrary, initLibrary } from './library.js';
import { initJourney, resolveTodayDate, routeFromHash } from './journey.js';
import { resumeRoute } from './progress.js';
import { initReaderContext, disposeReaderContext, resolveLegacyReaderRoute } from './reader-context.js';
const app = document.getElementById("app");
const encoder = new TextEncoder();
const decoder = new TextDecoder();
let journey = null;

function showError(message) {
  disposeReaderContext(app);
  disposeLibrary(app);
  app.innerHTML = "";
  const heading = document.createElement("h1");
  heading.textContent = "Reader couldn't start";
  const detail = document.createElement("p");
  detail.className = "muted";
  detail.textContent = message;
  app.append(heading, detail);
}

function createRustBridge(exports) {
  const { memory, alloc, dealloc, render, response_ptr, response_len } = exports;

  function invoke(uri) {
    const input = encoder.encode(uri);
    const inputPtr = alloc(input.length);
    new Uint8Array(memory.buffer, inputPtr, input.length).set(input);
    try {
      render(inputPtr, input.length);
    } finally {
      dealloc(inputPtr, input.length);
    }
    const outputPtr = response_ptr() >>> 0;
    const outputLen = response_len() >>> 0;
    const output = new Uint8Array(memory.buffer, outputPtr, outputLen).slice();
    return JSON.parse(decoder.decode(output));
  }

  return invoke;
}

function installPage(response, route, { initial = false } = {}) {
  disposeReaderContext(app);
  disposeLibrary(app);
  if (response.location) {
    const target = `#${routeFromHash(response.location)}`;
    if (window.location.hash !== target) window.location.hash = target;
    return;
  }

  document.title = response.title || "The Bible";
  if (response.content_type.startsWith("application/json")) {
    app.textContent = response.body;
    journey?.afterRender(route, response, { initial });
    return;
  }
  app.innerHTML = response.body;
  initLibrary(app);

  const form = app.querySelector("#lookup");
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const query = new FormData(form).get("q")?.toString().trim() || "";
    window.location.hash = `#/go?q=${encodeURIComponent(query)}`;
  });

  const journeyResult = journey?.afterRender(route, response, { initial }) || {};
  initExperience(app);
  initReaderContext(app);
  if (app.querySelector('[data-atlas-page]')) {
    window.scrollTo({top:0,left:0,behavior:'instant'});
  } else if (journeyResult.restoreVerseId) {
    requestAnimationFrame(() => document.getElementById(journeyResult.restoreVerseId)?.scrollIntoView({block: "center", behavior: "instant"}));
  } else if (response.anchor) {
    requestAnimationFrame(() => document.getElementById(response.anchor)?.scrollIntoView());
  } else if (journeyResult.restoreScrollY !== null && journeyResult.restoreScrollY !== undefined) {
    requestAnimationFrame(() => window.scrollTo({top: journeyResult.restoreScrollY, left: 0, behavior: 'instant'}));
  } else if (!initial && route.startsWith('/explore/') && route.includes('part=') && window.matchMedia('(max-width:760px)').matches) {
    requestAnimationFrame(() => app.querySelector('.study-panel')?.scrollIntoView({block: 'start', behavior: 'instant'}));
  } else if (!initial) {
    window.scrollTo({top: 0, left: 0, behavior: 'instant'});
  }
}

async function start() {
  if (!("WebAssembly" in window)) {
    throw new Error("This browser does not support WebAssembly.");
  }
  const wasmUrl = new URL("./rust_bible.wasm", import.meta.url);
  const wasmBytes = await fetch(wasmUrl).then((response) => {
    if (!response.ok) throw new Error(`WebAssembly file returned HTTP ${response.status}.`);
    return response.arrayBuffer();
  });
  const { instance } = await WebAssembly.instantiate(wasmBytes, {});
  const invoke = createRustBridge(instance.exports);

  // Static hosts cannot route /api/* to Axum. This method calls the same Rust
  // API handlers in WASM and returns their decoded JSON object in this page.
  window.rustBibleApi = (uri) => {
    const response = invoke(uri);
    if (!response.content_type.startsWith("application/json")) {
      throw new Error("rustBibleApi expects a Rust JSON API URI.");
    }
    const value = JSON.parse(response.body);
    if (response.status >= 400) throw Object.assign(new Error(value.error || "Bible API request failed."), { response: value, status: response.status });
    return value;
  };

  journey = initJourney(app, {
    resolveReaderUri(uri, position) {
      const response = invoke(uri);
      return response.status === 200
        && response.content_type.startsWith("text/html")
        && response.body.includes("data-reader")
        && (!position?.verse || invoke(`/api/verse?ref=${encodeURIComponent(`${position.book} ${position.chapter}:${position.verse}`)}`).status === 200);
    },
  });

  let initial = true;
  const renderCurrent = () => {
    try {
      let route = resolveTodayDate(routeFromHash(window.location.hash));
      const mapped=resolveLegacyReaderRoute(route);
      if(mapped!==route){window.history.replaceState(null,'',`#${mapped}`);route=mapped;}
      if (initial) {
        const resumed = resumeRoute(route, undefined, {reload: performance.getEntriesByType('navigation')[0]?.type === 'reload'});
        if (resumed !== route) {
          window.history.replaceState(null, '', `#${resumed}`);
          route = resumed;
        }
      }
      journey.beforeRender(route);
      const response = invoke(route);
      installPage(response, route, { initial });
      if (!response.location) initial = false;
    } catch (error) {
      showError(error instanceof Error ? error.message : "An unexpected reader error occurred.");
    }
  };
  window.addEventListener("hashchange", renderCurrent);
  renderCurrent();
}

start().catch((error) => {
  showError(error instanceof Error ? error.message : "An unexpected reader error occurred.");
});
