import { readProgress, positionForVerse } from './progress.js';
import { LocalNarrator, DEFAULT_NARRATOR, splitSpeech } from './local-voice.js';

const RUNTIME_KEY = Symbol.for('rust-bible.global-audio.v1');
const RATE_KEY = 'bible.rate';
const VOICE_KEY = 'bible.narrator';
const LOOKAHEAD_SEGMENT_COUNT = 2;
const LOOKAHEAD_SAMPLE_LIMIT = 24000 * 60;
const INITIAL_BUFFER_TIMEOUT_MS = 8000;
const ACTIVE_STATES = new Set(['preparing', 'playing', 'paused']);
const SOURCE_BUTTONS = new WeakSet();

const readPreference = (key, fallback) => {
  try { return localStorage.getItem(key) ?? fallback; } catch { return fallback; }
};

const writePreference = (key, value) => {
  try { localStorage.setItem(key, String(value)); } catch {}
};

function currentRoute() {
  if (window.location.hash.startsWith('#/')) return window.location.hash.slice(1);
  if (typeof window.rustBibleApi === 'function') return '/';
  return `${window.location.pathname}${window.location.search}${window.location.hash}`;
}

function appHref(route) {
  return typeof window.rustBibleApi === 'function' || window.location.hash.startsWith('#/') ? `#${route}` : route;
}

function routeURL(route = currentRoute()) {
  try { return new URL(route, window.location.origin); } catch { return new URL('/', window.location.origin); }
}

function bookSlug(book) {
  return encodeURIComponent(String(book || '').trim().replace(/\s+/g, '-'));
}

function routeFor(book, chapter) {
  return `/book/${bookSlug(book)}/${Number(chapter)}`;
}

function parseReference(reference) {
  const match = /^(.+?)\s+(\d+)(?::(\d+)(?:\s*[-–]\s*(\d+))?)?$/.exec(String(reference || '').trim());
  if (!match) return null;
  return {book: match[1].trim(), chapter: Number(match[2]), verse: match[3] ? Number(match[3]) : 1,
    lastVerse: match[4] ? Number(match[4]) : null};
}

function numberOrNull(value) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}

function formatTime(value) {
  if (!Number.isFinite(value) || value < 0) return '—:—';
  const seconds = Math.floor(value);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

function makeDock(runtime) {
  const existing = document.getElementById('global-audio-dock');
  if (existing) return existing;

  const dock = document.createElement('section');
  dock.id = 'global-audio-dock';
  dock.className = 'audio-dock';
  dock.setAttribute('aria-label', 'Audio playback');
  dock.hidden = true;
  dock.innerHTML = `
    <button class="audio-dock-toggle" type="button" data-dock-toggle aria-label="Play audio">
      <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path data-icon-play d="M8 4 21 12 8 20Z"></path><path data-icon-pause d="M6 5h4v14H6zm8 0h4v14h-4z"></path></svg>
    </button>
    <div class="audio-dock-info">
      <a class="audio-dock-return" data-dock-return href="#" aria-label="Return to the passage"></a>
      <div class="audio-dock-options">
        <button class="audio-dock-switch" data-dock-switch type="button" hidden>Listen here</button>
      </div>
      <span class="audio-dock-status" data-dock-status role="status" aria-live="polite">Preparing the free natural voice…</span>
    </div>
    <div class="audio-dock-timeline" data-dock-timeline>
      <span class="audio-dock-time" data-dock-current-time aria-hidden="true">—:—</span>
      <div class="audio-dock-wave-wrap">
        <canvas class="audio-dock-wave" data-dock-wave aria-hidden="true"></canvas>
        <input class="audio-dock-seek" data-dock-seek type="range" min="0" max="1" step="0.01" value="0" aria-label="Seek audio" disabled>
      </div>
      <span class="audio-dock-time" data-dock-duration aria-hidden="true">—:—</span>
    </div>
    <label class="audio-dock-voice"><span>Voice</span><select data-dock-voice aria-label="Narrator voice"></select></label>
    <label class="audio-dock-rate" aria-label="Playback speed">
      <select data-dock-rate aria-label="Playback speed">
        <option value="0.75">0.75×</option><option value="1">1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option><option value="2">2×</option>
      </select>
    </label>`;
  document.body.append(dock);

  runtime.dock = dock;
  runtime.ui = {
    toggle: dock.querySelector('[data-dock-toggle]'),
    playIcon: dock.querySelector('[data-icon-play]'),
    pauseIcon: dock.querySelector('[data-icon-pause]'),
    returnLink: dock.querySelector('[data-dock-return]'),
    status: dock.querySelector('[data-dock-status]'),
    canvas: dock.querySelector('[data-dock-wave]'),
    seek: dock.querySelector('[data-dock-seek]'),
    currentTime: dock.querySelector('[data-dock-current-time]'),
    duration: dock.querySelector('[data-dock-duration]'),
    rate: dock.querySelector('[data-dock-rate]'),
    voice: dock.querySelector('[data-dock-voice]'),
    switchSource: dock.querySelector('[data-dock-switch]'),
  };

  runtime.ui.rate.value = String(runtime.rate);
  populateVoices(runtime);

  runtime.ui.toggle.addEventListener('click', () => togglePlayback(runtime));
  runtime.ui.switchSource.addEventListener('click', () => {
    const root = runtime.currentRoot;
    const controls = root?.querySelector?.('[data-audio-source-button]')?.closest('[data-read-aloud]');
    if (controls) void startSource(runtime, sourceConfig(controls, root));
  });
  runtime.ui.rate.addEventListener('change', () => {
    runtime.rate = Number(runtime.ui.rate.value) || 1;
    writePreference(RATE_KEY, runtime.rate);
    runtime.narrator.setRate(runtime.rate);
    if (ACTIVE_STATES.has(runtime.state)) {
      setStatus(runtime, 'Playback speed updated. The current narration continues at the selected speed.');
    }
  });
  runtime.ui.voice.addEventListener('change', () => {
    runtime.voice = runtime.ui.voice.value || DEFAULT_NARRATOR;
    writePreference(VOICE_KEY, runtime.voice);
    if (ACTIVE_STATES.has(runtime.state)) {
      runtime.narrator.cancelPrefetch();
      clearLookahead(runtime);
      settleInitialBuffer(runtime);
      setStatus(runtime, 'Voice selected. It will be used for the next verse.');
      const keepCurrentRoute = runtime.utteranceDispatched && runtime.position && runtime.currentVerseVoice
        ? chapterRouteKey(runtime.position, runtime.currentVerseVoice) : null;
      for (const key of runtime.chapterRoutes.keys()) if (key !== keepCurrentRoute) runtime.chapterRoutes.delete(key);
      if (runtime.state === 'preparing' && !runtime.utteranceDispatched) runtime.currentVerseVoice = runtime.voice;
      const preparingCurrent = runtime.state === 'preparing' && !runtime.utteranceDispatched;
      const nextIndex = preparingCurrent || !runtime.position ? runtime.index : runtime.index + 1;
      const nextRecord = runtime.queue[nextIndex];
      if (nextRecord) {
        void chapterRoutesFor(runtime, nextRecord, runtime.voice, runtime.generation, 'prefetch').catch(() => {});
        if (preparingCurrent) speakCurrent(runtime, runtime.generation);
      } else if (runtime.source?.continuous && runtime.source.next) {
        void runtime.narrator.prefetchChapter(runtime.source.next.bookIndex, runtime.source.next.chapter, runtime.voice).catch(() => {});
      }
      if (runtime.state === 'playing') maintainLookahead(runtime, runtime.generation);
    }
  });
  runtime.ui.seek.addEventListener('input', () => {
    if (runtime.narrator.seek(Number(runtime.ui.seek.value))) updateTimeline(runtime);
  });

  const media = runtime.narrator.audio;
  for (const event of ['timeupdate', 'durationchange', 'loadedmetadata', 'play', 'pause', 'ended']) {
    media.addEventListener(event, () => updateTimeline(runtime));
  }
  runtime.narrator.addEventListener('audiobuffer', event => {
    runtime.samples = event.detail.samples || null;
    runtime.peaks = event.detail.peaks || null;
    runtime.sampleRate = event.detail.sampleRate || 0;
    runtime.waveDuration = event.detail.duration;
    drawWaveform(runtime);
    updateTimeline(runtime);
  });
  // Preparation/error messages can grow the title group. Reserve the actual
  // player height so the document and Journey pane always clear the dock.
  runtime.syncDockClearance = () => {
    if (dock.hidden) {
      document.body.style.removeProperty('--global-audio-dock-clearance');
      return;
    }
    const rect = dock.getBoundingClientRect();
    const bottom = Math.max(0, window.innerHeight - rect.bottom);
    document.body.style.setProperty('--global-audio-dock-clearance', `${Math.ceil(rect.height + bottom + 12)}px`);
  };
  if (typeof ResizeObserver === 'function') new ResizeObserver(runtime.syncDockClearance).observe(dock);
  window.addEventListener('resize', () => {
    runtime.syncDockClearance();
    drawWaveform(runtime);
  }, {passive: true});
  updateDock(runtime);
  return dock;
}

function getRuntime() {
  if (window[RUNTIME_KEY]) return window[RUNTIME_KEY];
  const runtime = {
    narrator: null, dock: null, ui: null, source: null, previewConfig: null, queue: [], index: 0, chunkIndex: 0,
    currentVerseVoice: null, utteranceDispatched: false, initialBufferPending: false, initialBuffer: null,
    lookahead: new Map(), lookaheadSamples: 0, lookaheadWork: null, lookaheadRefresh: false, lookaheadEpoch: 0,
    generation: 0, state: 'idle', position: null, samples: null, peaks: null, sampleRate: 0,
    waveDuration: 0, rate: Number(readPreference(RATE_KEY, '1')) || 1,
    voice: readPreference(VOICE_KEY, DEFAULT_NARRATOR),
    follow: true,
    currentRoot: document, startTimer: null, loadingNext: null, loadError: null, chapterRoutes: new Map(),
  };
  runtime.narrator = new LocalNarrator({onProgress: progress => {
    if (runtime.state !== 'preparing') return;
    if (runtime.initialBuffer && !runtime.initialBuffer.settled) return;
    const total = Number(progress.total) || 0;
    const loaded = Number(progress.loaded) || 0;
    const message = total > 0
      ? `Downloading the free natural voice · ${Math.round(loaded / total * 100)}% (${Math.ceil(loaded / 1048576)} / ${Math.ceil(total / 1048576)} MB)`
      : progress.message || 'Preparing the free natural voice…';
    setStatus(runtime, message);
  }});
  window[RUNTIME_KEY] = runtime;
  window.biblePlayback = Object.freeze({
    active: () => ACTIVE_STATES.has(runtime.state),
    position: () => runtime.position ? {...runtime.position} : null,
    stop: () => stopPlayback(runtime),
    returnHref: () => runtime.source?.returnHref || currentRoute(),
    setRate: value => {
      runtime.rate = Number(value) || 1;
      runtime.narrator.setRate(runtime.rate);
      if (runtime.ui) runtime.ui.rate.value = String(runtime.rate);
      writePreference(RATE_KEY, runtime.rate);
    },
  });
  makeDock(runtime);
  return runtime;
}

function populateVoices(runtime) {
  if (!runtime.ui?.voice) return;
  const selected = runtime.voice || DEFAULT_NARRATOR;
  const voices = runtime.narrator.getVoices();
  runtime.ui.voice.replaceChildren();
  for (const voice of voices) {
    const option = document.createElement('option');
    option.value = voice.voiceURI;
    option.textContent = voice.name.split(' · ')[0];
    runtime.ui.voice.append(option);
  }
  runtime.ui.voice.value = [...runtime.ui.voice.options].some(option => option.value === selected)
    ? selected : DEFAULT_NARRATOR;
  runtime.voice = runtime.ui.voice.value;
  writePreference(VOICE_KEY, runtime.voice);
}

function setStatus(runtime, message) {
  if (runtime.ui?.status) runtime.ui.status.textContent = message;
  if (runtime.dock) runtime.dock.dataset.state = runtime.state;
}

function updateDock(runtime) {
  if (!runtime.dock) return;
  syncSpeakingHighlight(runtime);
  const activeSource = Boolean(runtime.source && ACTIVE_STATES.has(runtime.state));
  const retainedSource = Boolean(runtime.source && ['error', 'finished'].includes(runtime.state)
    && (!runtime.previewConfig || runtime.previewConfig.key === runtime.source.key));
  const source = activeSource || retainedSource ? runtime.source : runtime.previewConfig;
  const previewOnly = Boolean(source && source === runtime.previewConfig);
  const visible = Boolean(source);
  runtime.dock.hidden = !visible;
  runtime.dock.dataset.state = previewOnly ? 'idle' : runtime.state;
  document.body.classList.toggle('has-global-audio', visible);
  if (visible) {
    const record = activeSource ? runtime.position : null;
    const reference = record ? `${record.book} ${record.chapter}:${record.verse}` : source.reference;
    runtime.ui.returnLink.textContent = reference ? `${activeSource && runtime.state === 'playing' ? 'Listening' : activeSource && runtime.state === 'paused' ? 'Paused' : activeSource && runtime.state === 'preparing' ? 'Preparing' : 'Listen'} · ${reference}` : 'Return to passage';
    runtime.ui.returnLink.setAttribute('aria-label', `Return to ${reference || 'the passage'}`);
    runtime.ui.returnLink.href = appHref(source.returnHref || currentRoute());
    runtime.ui.rate.value = String(runtime.rate);
    if (previewOnly) runtime.ui.status.textContent = 'Ready to listen';
  }
  const playing = activeSource && runtime.state === 'playing';
  runtime.ui.playIcon.toggleAttribute('hidden', playing);
  runtime.ui.pauseIcon.toggleAttribute('hidden', !playing);
  const label = activeSource && runtime.state === 'playing' ? 'Pause reading'
    : activeSource && runtime.state === 'paused' ? 'Resume reading'
      : activeSource && runtime.state === 'preparing' ? 'Cancel audio preparation'
        : retainedSource && runtime.state === 'finished' ? 'Replay passage'
          : retainedSource && runtime.state === 'error' ? 'Try listening again' : 'Play audio';
  runtime.ui.toggle.setAttribute('aria-label', label);
  runtime.ui.toggle.title = label;
  updateTimeline(runtime);
  drawWaveform(runtime);
  updateSourceButton(runtime);
  runtime.syncDockClearance?.();
}

function updateTimeline(runtime) {
  if (!runtime.ui) return;
  const media = runtime.narrator;
  const mediaAvailable = Boolean(runtime.source && ACTIVE_STATES.has(runtime.state));
  const seekable = mediaAvailable && media.seekable;
  const duration = seekable ? media.duration : 0;
  const currentTime = seekable ? media.currentTime : 0;
  runtime.ui.currentTime.textContent = seekable ? formatTime(currentTime) : '—:—';
  runtime.ui.duration.textContent = seekable ? formatTime(duration) : '—:—';
  runtime.ui.seek.disabled = !seekable;
  runtime.ui.seek.max = String(Math.max(duration, 0.01));
  if (!runtime.ui.seek.matches(':active')) runtime.ui.seek.value = String(currentTime);
  runtime.ui.seek.setAttribute('aria-valuetext', seekable
    ? `${formatTime(currentTime)} of ${formatTime(duration)}` : 'Seeking is available during natural voice playback');
  drawWaveform(runtime, seekable ? runtime.samples : null);
}

function drawWaveform(runtime, samples = runtime.samples) {
  const canvas = runtime.ui?.canvas;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  if (!rect.width || !rect.height) return;
  const scale = Math.max(1, window.devicePixelRatio || 1);
  const width = Math.round(rect.width * scale);
  const height = Math.round(rect.height * scale);
  if (canvas.width !== width || canvas.height !== height) { canvas.width = width; canvas.height = height; }
  const context = canvas.getContext('2d');
  if (!context) return;
  context.clearRect(0, 0, width, height);
  const palette = getComputedStyle(canvas);
  const unplayed = palette.getPropertyValue('--wave-muted').trim() || '#b7c8bf';
  const played = palette.getPropertyValue('--wave-active').trim() || '#29664d';
  if (!runtime.source || !ACTIVE_STATES.has(runtime.state)) {
    context.fillStyle = unplayed;
    context.fillRect(0, Math.floor(height / 2), width, Math.max(1, Math.round(scale)));
    return;
  }
  const bins = Array.isArray(runtime.peaks) ? runtime.peaks : samples;
  if (!(bins instanceof Float32Array) && !Array.isArray(bins)) return;
  if (!bins.length) return;
  const bars = Math.max(24, Math.min(112, Math.floor(rect.width / 5)));
  const gap = Math.max(1, Math.round(2 * scale));
  const barWidth = Math.max(1, Math.floor(width / bars) - gap);
  const progress = runtime.narrator.seekable && runtime.narrator.duration > 0
    ? runtime.narrator.currentTime / runtime.narrator.duration : 0;
  const middle = height / 2;
  for (let index = 0; index < bars; index++) {
    const start = Math.floor(index * bins.length / bars);
    const end = Math.max(start + 1, Math.floor((index + 1) * bins.length / bars));
    let peak = 0;
    for (let sample = start; sample < end; sample++) peak = Math.max(peak, Array.isArray(bins) ? bins[sample] : Math.abs(bins[sample]));
    const barHeight = Math.max(Math.round(2 * scale), Math.round(peak * height * 0.86));
    const x = Math.floor(index * width / bars);
    context.fillStyle = index / bars <= progress ? played : unplayed;
    context.fillRect(x, middle - barHeight / 2, barWidth, barHeight);
  }
}

function isAtCurrentReader(runtime) {
  if (!runtime.position || !runtime.source?.readerMode) return false;
  const here = routeURL();
  return here.pathname === runtime.source.readerRoute;
}

function findVisibleVerse(record) {
  if (!record) return null;
  return [...document.querySelectorAll('.scripture .v')].find(node => {
    const position = positionForVerse(node);
    if (position) return position.bookIndex === record.bookIndex
      && position.chapter === record.chapter && position.verse === record.verse;
    const reference = parseReference(node.dataset.reference);
    return reference?.book.toLowerCase() === record.book.toLowerCase()
      && reference.chapter === record.chapter && Number(node.dataset.verse) === record.verse;
  }) || null;
}

function syncSpeakingHighlight(runtime) {
  const node = runtime.source && ['playing', 'paused'].includes(runtime.state)
    ? findVisibleVerse(runtime.position) : null;
  document.querySelectorAll('.is-speaking, [data-narration-state]').forEach(active => {
    if (active === node) return;
    active.classList.remove('is-speaking');
    delete active.dataset.narrationState;
  });
  if (node) {
    node.classList.add('is-speaking');
    node.dataset.narrationState = runtime.state;
  }
  return node;
}

function followVerse(runtime, record) {
  const node = syncSpeakingHighlight(runtime);
  if (!node || !runtime.follow) return;
  const atSource = isAtCurrentReader(runtime) || (runtime.source?.kind === 'range'
    && Boolean(node.closest('[data-today]')));
  if (!atSource) return;
  const bounds = node.getBoundingClientRect();
  const toolbar = document.querySelector('.reader-toolbar')?.getBoundingClientRect();
  const pane=document.body.classList.contains('has-scene') ? node.closest('.reader-content') : null;
  if(pane?.hidden)return;
  const viewport=pane?.getBoundingClientRect();
  if (bounds.top < (viewport?.top ?? Math.max(20, toolbar?.bottom || 0)) + 12 || bounds.bottom > (viewport?.bottom ?? window.innerHeight - 130) - 12) {
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    node.scrollIntoView({block: 'center', behavior: reduce ? 'instant' : 'smooth'});
  }
}

function preserveContextHref(source, record) {
  if (!source || !record?.route) return;
  const stored = routeURL(source.returnHref);
  const visible = routeURL();
  const sameReader = visible.pathname === record.route || visible.pathname === source.readerRoute || visible.pathname === source.lastFollowedRoute;
  const context = sameReader ? visible : stored;
  const target = new URL(record.route, window.location.origin);
  for (const key of ['context', 'scene', 'part', 'art', 'focus']) {
    if (context.searchParams.has(key)) target.searchParams.set(key, context.searchParams.get(key));
  }
  target.searchParams.set('resume',record.verse);
  target.hash = `v${record.verse}`;
  source.returnHref = `${target.pathname}${target.search}${target.hash}`;
  source.lastFollowedRoute = target.pathname;
}

function publishPosition(runtime, queueRecord) {
  const record = {
    book: queueRecord.book,
    bookIndex: queueRecord.bookIndex,
    chapter: queueRecord.chapter,
    verse: queueRecord.verse,
    route: queueRecord.route,
    source: 'listening',
    savedAt: Date.now(),
  };
  if (!record.book || !Number.isInteger(record.bookIndex) || !record.route
    || !Number.isInteger(record.chapter) || !Number.isInteger(record.verse)) return null;
  runtime.position = record;
  preserveContextHref(runtime.source, record);
  window.dispatchEvent(new CustomEvent('bible:audio-position', {detail: {record: {...record}}}));
  updateDock(runtime);
  followVerse(runtime, record);
  return record;
}

function updateSourceButton(runtime) {
  const root = runtime.currentRoot;
  const controls = root?.querySelector?.('[data-audio-source-button]')?.closest('[data-read-aloud]');
  const button = controls?.querySelector?.('[data-audio-source-button]');
  if (runtime.ui?.switchSource) runtime.ui.switchSource.hidden = true;
  if (runtime.dock) runtime.dock.dataset.switchAvailable = 'false';
  if (!button) return;
  const sameReaderChapter = runtime.source?.readerMode && button.dataset.audioSourceKind === 'reader'
    && runtime.position && Number(button.dataset.audioSourceBookIndex) === runtime.position.bookIndex
    && Number(button.dataset.audioSourceChapter) === runtime.position.chapter;
  const sameBookSource = runtime.source?.bookMode && button.dataset.audioSourceKind === 'book'
    && Number(button.dataset.audioSourceBookIndex) === runtime.source.bookIndex;
  const sameSource = Boolean((runtime.source?.key && button.dataset.audioSourceKey === runtime.source.key)
    || sameReaderChapter || sameBookSource);
  const activeElsewhere = ACTIVE_STATES.has(runtime.state) && !sameSource;
  const defaultInDock = button.dataset.audioSourceDefaultContext === 'true';
  const text = activeElsewhere ? button.dataset.audioSourceSwitchLabel
    : sameSource && ACTIVE_STATES.has(runtime.state) ? 'Listening to this passage'
      : button.dataset.audioSourceIdleLabel;
  controls.hidden = defaultInDock || sameSource;
  if (defaultInDock && activeElsewhere && runtime.ui?.switchSource) {
    const reference = controls.dataset.reference || 'this passage';
    runtime.ui.switchSource.hidden = false;
    runtime.ui.switchSource.setAttribute('aria-label', `Listen to ${reference} instead`);
    runtime.ui.switchSource.title = `Switch narration to ${reference}`;
    runtime.dock.dataset.switchAvailable = 'true';
  }
  (button.querySelector('span') || button).textContent = text;
  button.disabled = sameSource && ACTIVE_STATES.has(runtime.state);
  button.setAttribute('aria-label', text);
}

function togglePlayback(runtime) {
  if (runtime.state === 'playing') {
    runtime.narrator.pause();
    if (runtime.narrator.paused) {
      runtime.state = 'paused';
      setStatus(runtime, 'Paused.');
      updateDock(runtime);
    }
    return;
  }
  if (runtime.state === 'paused') {
    runtime.narrator.resume();
    if (!runtime.narrator.paused) {
      runtime.state = 'playing';
      setStatus(runtime, 'Listening.');
      updateDock(runtime);
    }
    return;
  }
  if (runtime.state === 'preparing') {
    stopPlayback(runtime);
    return;
  }
  const visibleControls=runtime.currentRoot?.querySelector?.('[data-read-aloud]');
  if(visibleControls)runtime.previewConfig=sourceConfig(visibleControls,runtime.currentRoot);
  const previewIsDifferent = runtime.previewConfig && runtime.previewConfig.key !== runtime.source?.key;
  const config = previewIsDifferent || runtime.previewConfig?.key === runtime.source?.key
    ? runtime.previewConfig : runtime.source?.config || runtime.previewConfig;
  if (config) void startSource(runtime, config);
}

function cancelUtterance(runtime) {
  clearTimeout(runtime.startTimer);
  runtime.startTimer = null;
  runtime.generation += 1;
  settleInitialBuffer(runtime);
  clearLookahead(runtime);
  runtime.initialBufferPending = false;
  runtime.utteranceDispatched = false;
  runtime.chapterRoutes.clear();
  runtime.narrator.cancel();
  runtime.loadingNext = null;
  runtime.loadError = null;
  return runtime.generation;
}

function stopPlayback(runtime) {
  if (!runtime.source && !runtime.narrator.current) return;
  cancelUtterance(runtime);
  runtime.state = 'stopped';
  runtime.queue = [];
  runtime.index = 0;
  runtime.chunkIndex = 0;
  runtime.currentVerseVoice = null;
  runtime.source = null;
  runtime.position = null;
  runtime.samples = null;
  runtime.peaks = null;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  updateDock(runtime);
}

function normalizeNext(next, fallbackBookIndex) {
  if (!next || typeof next !== 'object') return null;
  const bookIndex = numberOrNull(next.book_index ?? next.bookIndex ?? next.book ?? fallbackBookIndex);
  const chapter = numberOrNull(next.chapter);
  return bookIndex === null || !chapter ? null : {bookIndex, chapter};
}

function normalizeVerses(payload, config, {chapterMode = false} = {}) {
  if (!payload || !Array.isArray(payload.verses)) throw new Error('The passage did not contain Scripture verses.');
  const parsed = parseReference(config.reference);
  const book = String(payload.book || config.book || parsed?.book || '');
  const bookIndex = numberOrNull(payload.book_index ?? payload.bookIndex ?? config.bookIndex);
  const chapter = Number(payload.chapter ?? config.chapter ?? parsed?.chapter);
  const route = String(payload.route || config.route || (book && chapter ? routeFor(book, chapter) : ''));
  const records = payload.verses.map(item => ({
    book,
    bookIndex,
    chapter: Number(item?.chapter ?? chapter),
    verse: Number(item?.verse),
    route: String(item?.route || route),
    text: typeof item?.text === 'string' ? item.text.trim() : '',
  })).filter(item => item.text && Number.isInteger(item.verse) && item.verse > 0
    && Number.isInteger(item.chapter) && item.chapter > 0);
  if (!records.length) throw new Error('The passage did not contain readable Scripture verses.');
  return {records, next: chapterMode ? normalizeNext(payload.next, bookIndex) : null,
    book, bookIndex, chapter, route};
}

function apiRequest(uri) {
  if (typeof window.rustBibleApi === 'function') return Promise.resolve(window.rustBibleApi(uri));
  return fetch(uri, {cache: 'no-store'}).then(response => {
    if (!response.ok) throw new Error(`Bible audio request returned HTTP ${response.status}.`);
    return response.json();
  });
}

function textFromVerseNode(node) {
  const clone = node.cloneNode(true);
  clone.querySelectorAll('sup').forEach(number => number.remove());
  return clone.textContent.trim();
}

function fallbackChapterFromDOM(root, config, chapter) {
  const nodes = [...(root?.querySelectorAll?.('[data-reader-route] .scripture .v') || [])];
  const reader = root?.querySelector?.('[data-reader]');
  const bookIndex = numberOrNull(reader?.dataset.bookIndex ?? config.bookIndex);
  const book = reader?.dataset.bookName || config.book || parseReference(config.reference)?.book || '';
  const route = reader?.dataset.readerRoute || routeFor(book, chapter);
  const records = nodes.map(node => {
    const position = positionForVerse(node);
    return {
      book: position?.book || book,
      bookIndex: position?.bookIndex ?? bookIndex,
      chapter: position?.chapter ?? chapter,
      verse: position?.verse ?? Number(node.dataset.verse),
      route: position?.route || routeFor(position?.book || book, position?.chapter ?? chapter),
      text: textFromVerseNode(node),
    };
  }).filter(record => record.chapter === chapter && record.text && Number.isInteger(record.verse)
    && (bookIndex === null || record.bookIndex === bookIndex));
  if (!records.length) return null;
  return {records, next: null, book, bookIndex, chapter, route};
}

function fallbackBookChapter(config, chapter) {
  try {
    if (Number(config.root?.querySelector?.('[data-book-page]')?.dataset.bookIndex) !== config.bookIndex) return null;
    const raw = config.root?.querySelector?.('[data-book-scripture]')?.textContent || '[]';
    const all = JSON.parse(raw);
    const verses = Array.isArray(all) ? all.filter(item => Number(item.chapter) === chapter) : [];
    if (!verses.length) return null;
    const book = config.book;
    const chapterCount = Math.max(...all.map(item => Number(item.chapter) || 0));
    return {
      records: verses.map(item => ({book, bookIndex: config.bookIndex, chapter,
        verse: Number(item.verse), route: routeFor(book, chapter), text: String(item.text || '').trim()})).filter(item => item.text),
      next: chapter < chapterCount ? {bookIndex: config.bookIndex, chapter: chapter + 1} : null,
      book, bookIndex: config.bookIndex, chapter, route: routeFor(book, chapter),
    };
  } catch { return null; }
}

function fallbackRange(root, config) {
  const parsed = parseReference(config.reference);
  if (!parsed) return null;
  const nodes = [...(root?.querySelectorAll?.('.scripture .v') || [])].filter(node => {
    const position = positionForVerse(node);
    if (position && (position.book !== parsed.book || position.chapter !== parsed.chapter)) return false;
    const verse = Number(node.dataset.verse);
    if (!Number.isInteger(verse) || verse < parsed.verse || (parsed.lastVerse && verse > parsed.lastVerse)) return false;
    const candidate = parseReference(node.dataset.reference || node.closest('[data-reference]')?.dataset.reference || '');
    return !candidate || (candidate.book.toLowerCase() === parsed.book.toLowerCase() && candidate.chapter === parsed.chapter);
  });
  if (!nodes.length) return null;
  const first = positionForVerse(nodes[0]);
  const reader = root?.querySelector?.('[data-reader]');
  const bookIndex = first?.bookIndex ?? numberOrNull(reader?.dataset.bookIndex) ?? numberOrNull(controlsBookIndex(root));
  return {book: parsed.book, bookIndex, chapter: parsed.chapter,
    route: first?.route || routeFor(parsed.book, parsed.chapter), next: null,
    records: nodes.map(node => {
      const position = positionForVerse(node);
      return {book: position?.book || parsed.book, bookIndex: position?.bookIndex ?? bookIndex,
        chapter: position?.chapter || parsed.chapter, verse: Number(node.dataset.verse),
        route: position?.route || routeFor(parsed.book, parsed.chapter), text: textFromVerseNode(node)};
    }).filter(record => record.text)};
}

function controlsBookIndex(root) {
  const controls = root?.querySelector?.('[data-read-aloud]');
  return controls?.dataset.bookIndex;
}

async function loadChapter(config, bookIndex, chapter, generation) {
  const uri = `/api/audio-chapter?book=${bookIndex}&chapter=${chapter}`;
  try {
    const payload = await apiRequest(uri);
    if (generation !== config.runtime.generation) return null;
    return normalizeVerses(payload, {...config, bookIndex, chapter}, {chapterMode: true});
  } catch (error) {
    if (generation !== config.runtime.generation) return null;
    const fallback = config.kind === 'book'
      ? fallbackBookChapter(config, chapter)
      : fallbackChapterFromDOM(config.root, config, chapter);
    if (fallback) return fallback;
    throw error;
  }
}

function currentReaderPosition(root, reader, reference) {
  const bookIndex = numberOrNull(reader?.dataset.bookIndex);
  const route = reader?.dataset.readerRoute || '';
  const positionId = reader?.dataset.readerPosition;
  const selected = positionId
    ? [...(root?.querySelectorAll?.('[data-reader-route] .scripture .v') || [])].find(node => node.id === positionId)
    : null;
  const selectedPosition = selected ? positionForVerse(selected) : null;
  if (selectedPosition) {
    return {bookIndex: selectedPosition.bookIndex, chapter: selectedPosition.chapter, verse: selectedPosition.verse,
      book: selectedPosition.book, route: selectedPosition.route};
  }
  const saved = readProgress();
  if (saved && (saved.route === route || saved.entryRoute === route)) {
    return {bookIndex: saved.bookIndex ?? bookIndex, chapter: saved.chapter, verse: saved.verse || 1,
      book: saved.book, route: saved.route};
  }
  const parsed = parseReference(reference);
  return {bookIndex, chapter: Number(reader?.dataset.chapter || parsed?.chapter || 1),
    verse: parsed?.verse || 1, book: reader?.dataset.bookName || parsed?.book || '', route};
}

function sourceConfig(controls, root) {
  const readerMode = controls.hasAttribute('data-reader-audio');
  const bookMode = controls.hasAttribute('data-book-audio');
  const kind = readerMode ? 'reader' : bookMode ? 'book' : 'range';
  const reader = root.querySelector('[data-reader]');
  const reference = String(controls.dataset.reference || reader?.dataset.bookName || 'Scripture').trim();
  const parsed = parseReference(reference);
  const saved = readProgress();
  let bookIndex = numberOrNull(controls.dataset.bookIndex ?? reader?.dataset.bookIndex);
  let chapter = Number(reader?.dataset.chapter || parsed?.chapter || 1);
  let verse = parsed?.verse || 1;
  let book = reader?.dataset.bookName || parsed?.book || reference;
  let route = reader?.dataset.readerRoute || '';
  if (readerMode) {
    const selected = currentReaderPosition(root, reader, reference);
    bookIndex = selected.bookIndex ?? bookIndex;
    chapter = selected.chapter;
    verse = selected.verse;
    book = selected.book || book;
    route = selected.route || route;
  } else if (bookMode) {
    book = reference;
    if (saved && saved.bookIndex === bookIndex) { chapter = saved.chapter; verse = saved.verse || 1; }
    route = routeFor(book, chapter);
  } else if (parsed) {
    book = parsed.book;
    chapter = parsed.chapter;
    verse = parsed.verse;
    route = routeFor(book, chapter);
  }
  const locationURL = routeURL();
  let returnRoute = currentRoute();
  if (bookMode) {
    returnRoute = routeFor(book, chapter);
    if (locationURL.searchParams.get('context') === 'journey') returnRoute += '?context=journey';
  }
  const key = `${kind}|${bookIndex ?? ''}|${chapter}|${verse}|${reference}|${route}`;
  return {kind, readerMode, bookMode, reference, book, bookIndex, chapter, verse, route, readerRoute: route,
    returnHref: returnRoute, key, root,
    rangeEnd: parsed?.lastVerse || null};
}

function sourceControls(controls, root, runtime) {
  const play = controls.querySelector('[data-audio-action="play"]');
  if (!play) return;
  const bookMode = controls.hasAttribute('data-book-audio');
  controls.classList.add('audio-source-only');
  play.dataset.audioSourceButton = '';
  const initialConfig = sourceConfig(controls, root);
  const defaultInDock = initialConfig.readerMode || Boolean(root.querySelector('[data-today]'));
  runtime.previewConfig = defaultInDock ? initialConfig : null;
  play.dataset.audioSourceKey = initialConfig.key;
  play.dataset.audioSourceKind = initialConfig.kind;
  play.dataset.audioSourceBookIndex = String(initialConfig.bookIndex ?? '');
  play.dataset.audioSourceChapter = String(initialConfig.chapter);
  const text = bookMode ? `Listen to ${controls.dataset.reference || 'this book'}` : 'Listen to this passage';
  play.dataset.audioSourceDefaultContext = String(defaultInDock);
  play.dataset.audioSourceIdleLabel = text;
  play.dataset.audioSourceSwitchLabel = bookMode
    ? `Listen to ${controls.dataset.reference || 'this book'} instead` : 'Listen to this passage instead';
  const icon = play.querySelector('svg')?.cloneNode(true);
  const label = document.createElement('span');
  label.textContent = text;
  play.replaceChildren(...(icon ? [icon, label] : [label]));
  if (!bookMode) controls.replaceChildren(play);
  else controls.querySelector('.book-playback-settings')?.setAttribute('hidden', '');
  play.classList.add('audio-source-action');
  if (!SOURCE_BUTTONS.has(play)) {
    SOURCE_BUTTONS.add(play);
    play.addEventListener('click', event => {
      event.preventDefault();
      const config = sourceConfig(controls, root);
      void startSource(runtime, config);
    });
  }
  runtime.currentRoot = root;
  updateSourceButton(runtime);
}

function canContinueBook(runtime, next) {
  if (!next) return false;
  if (!runtime.source?.bookMode) return true;
  return next.bookIndex === runtime.source.bookIndex;
}

async function appendNextChapter(runtime, generation) {
  if (runtime.loadError) return false;
  if (runtime.loadingNext) return runtime.loadingNext;
  const next = runtime.source?.next;
  if (!canContinueBook(runtime, next)) return false;
  const pending = (async () => {
    try {
      const config = {...runtime.source.config, runtime, root: runtime.currentRoot};
      const loaded = await loadChapter(config, next.bookIndex, next.chapter, generation);
      if (generation !== runtime.generation || !loaded) return false;
      const existing = new Set(runtime.queue.map(item => `${item.bookIndex}:${item.chapter}:${item.verse}`));
      const additional = loaded.records.filter(item => !existing.has(`${item.bookIndex}:${item.chapter}:${item.verse}`));
      if (!additional.length) {
        runtime.loadError = new Error('The next chapter did not add any readable verses.');
        return false;
      }
      runtime.queue.push(...additional);
      runtime.source.next = loaded.next;
      runtime.source.lastLoadedRoute = loaded.route;
      prefetchNextChapter(runtime);
      runtime.loadError = null;
      return true;
    } catch (error) {
      if (generation === runtime.generation) runtime.loadError = error;
      return false;
    } finally {
      if (runtime.loadingNext === pending) runtime.loadingNext = null;
    }
  })();
  runtime.loadingNext = pending;
  return pending;
}

function readingStatus(record) {
  return `Listening · ${record.book} ${record.chapter}:${record.verse}`;
}

function segmentId(runtime, record, chunkIndex) {
  return `${runtime.generation}:${record.bookIndex ?? 'x'}:${record.chapter}:${record.verse}:${chunkIndex}`;
}

function voiceFor(runtime, uri) {
  return runtime.narrator.getVoices().find(item => item.voiceURI === uri) || null;
}

function chapterRouteKey(record, voiceURI) {
  return `${record.bookIndex}:${record.chapter}:${voiceURI}`;
}

function chapterRoutesFor(runtime, record, voiceURI, generation, priority='active') {
  const currentChapterPrefix = `${record.bookIndex}:${record.chapter}:`;
  for (const key of runtime.chapterRoutes.keys()) {
    if (!key.startsWith(currentChapterPrefix)) runtime.chapterRoutes.delete(key);
  }
  const key = chapterRouteKey(record, voiceURI);
  const existing = runtime.chapterRoutes.get(key);
  if (existing?.status === 'ready') return Promise.resolve(existing.routes);
  if (existing?.status === 'pending') return existing.promise;
  if (existing?.status === 'failed') return Promise.reject(existing.error);
  const records = runtime.queue.filter(item => item.bookIndex === record.bookIndex && item.chapter === record.chapter);
  const entry = {status:'pending',routes:null,promise:null};
  runtime.chapterRoutes.set(key, entry);
  entry.promise = runtime.narrator.prepareVerses(records, voiceURI, {priority}).then(routes => {
    if (generation !== runtime.generation || runtime.chapterRoutes.get(key) !== entry) return null;
    entry.status = 'ready';
    entry.routes = routes;
    return routes;
  }).catch(error => {
    if (runtime.chapterRoutes.get(key) === entry) {
      entry.status = 'failed';
      entry.error = error;
    }
    throw error;
  });
  return entry.promise;
}

function prefetchNextChapter(runtime) {
  const next = runtime.source?.next;
  if (!next) return;
  const voice = runtime.currentVerseVoice || runtime.voice;
  void runtime.narrator.prefetchChapter(next.bookIndex, next.chapter, voice).catch(() => {});
}

function failPlayback(runtime, error, generation) {
  if (generation !== runtime.generation) return;
  clearTimeout(runtime.startTimer);
  runtime.startTimer = null;
  settleInitialBuffer(runtime);
  clearLookahead(runtime);
  runtime.state = 'error';
  runtime.narrator.cancel();
  setStatus(runtime, `Audio could not play: ${error?.message || error?.error || 'unknown error'}. Try again or choose the other natural voice.`);
  updateDock(runtime);
}

function settleInitialBuffer(runtime, buffer = runtime.initialBuffer) {
  if (!buffer || buffer.settled) return;
  buffer.settled = true;
  clearTimeout(buffer.timer);
  if (runtime.initialBuffer === buffer) runtime.initialBuffer = null;
  buffer.resolve();
}

function createInitialBuffer(runtime, generation) {
  let resolve;
  const buffer = {
    generation,
    promise: new Promise(done => { resolve = done; }),
    resolve: () => resolve?.(),
    settled: false,
    timer: null,
  };
  runtime.initialBuffer = buffer;
  return buffer;
}

function clearLookahead(runtime) {
  runtime.lookaheadEpoch += 1;
  runtime.lookahead.clear();
  runtime.lookaheadSamples = 0;
  runtime.lookaheadWork = null;
  runtime.lookaheadRefresh = false;
}

function dropLookahead(runtime, id, expected) {
  const entry = runtime.lookahead.get(id);
  if (!entry || (expected && entry !== expected)) return;
  if (entry.status === 'ready') runtime.lookaheadSamples = Math.max(0, runtime.lookaheadSamples - entry.sampleCount);
  runtime.lookahead.delete(id);
}

function failLookahead(runtime, id, entry) {
  if (runtime.lookahead.get(id) !== entry) return;
  entry.status = 'failed';
  entry.sampleCount = 0;
}

async function upcomingSpeechSegments(runtime, generation) {
  const segments = [];
  let index = runtime.index;
  let chunkIndex = runtime.chunkIndex + 1;
  let loadedBoundary = false;

  while (segments.length < LOOKAHEAD_SEGMENT_COUNT) {
    if (generation !== runtime.generation) return [];
    const record = runtime.queue[index];
    if (!record) {
      if (loadedBoundary || !runtime.source?.continuous || !runtime.source.next) break;
      loadedBoundary = true;
      if (!await appendNextChapter(runtime, generation)) break;
      continue;
    }

    const voiceURI = index === runtime.index ? runtime.currentVerseVoice || runtime.voice : runtime.voice;
    const voice = voiceFor(runtime, voiceURI);
    const routes = runtime.chapterRoutes.get(chapterRouteKey(record, voiceURI));
    const route = routes?.status === 'ready' ? routes.routes.get(Number(record.verse)) : null;
    if (!route) break;
    if (route.kind === 'hosted') {
      if (index > runtime.index || chunkIndex === 0) {
        segments.push({text:record.text,voice,local:false,route,
          segmentId:segmentId(runtime,record,0)});
      }
      index += 1;
      chunkIndex = 0;
      continue;
    }
    const chunks = splitSpeech(record.text);
    while (chunkIndex < chunks.length && segments.length < LOOKAHEAD_SEGMENT_COUNT) {
      const text = chunks[chunkIndex];
      if (text.trim()) segments.push({
        text,
        voice,
        local: true,
        segmentId: segmentId(runtime, record, chunkIndex),
      });
      chunkIndex += 1;
    }
    if (chunkIndex >= chunks.length) {
      index += 1;
      chunkIndex = 0;
    } else {
      break;
    }
  }

  return segments;
}

function maintainLookahead(runtime, generation, initialBuffer = null) {
  if (generation !== runtime.generation) return Promise.resolve();
  const epoch = runtime.lookaheadEpoch;
  const cursor = `${runtime.index}:${runtime.chunkIndex}`;
  const currentWork = runtime.lookaheadWork;
  if (currentWork && currentWork.generation === generation && currentWork.epoch === epoch
    && currentWork.cursor === cursor) {
    if (initialBuffer) currentWork.initialBuffers.add(initialBuffer);
    if (runtime.state === 'playing') runtime.lookaheadRefresh = true;
    return currentWork.promise;
  }

  const workRecord = {generation, epoch, cursor, promise: null, initialBuffers: new Set()};
  if (initialBuffer) workRecord.initialBuffers.add(initialBuffer);
  const work = (async () => {
    try {
      const candidates = await upcomingSpeechSegments(runtime, generation);
      if (generation !== runtime.generation || epoch !== runtime.lookaheadEpoch) return;

      const candidateIds = new Set(candidates.map(candidate => candidate.segmentId));
      for (const id of runtime.lookahead.keys()) {
        if (!candidateIds.has(id)) dropLookahead(runtime, id);
      }

      for (const candidate of candidates) {
        if (generation !== runtime.generation || epoch !== runtime.lookaheadEpoch) break;
        if (`${runtime.index}:${runtime.chunkIndex}` !== cursor) {
          runtime.lookaheadRefresh = true;
          break;
        }
        if (!candidate.local) continue;

        const existing = runtime.lookahead.get(candidate.segmentId);
        if (existing?.status === 'ready') continue;
        if (existing?.status === 'failed') break;
        if (existing?.status === 'pending') {
          if (!await existing.promise) break;
          continue;
        }

        const activeSamples = runtime.samples instanceof Float32Array ? runtime.samples.length : 0;
        const availableSamples = Math.max(0, LOOKAHEAD_SAMPLE_LIMIT - activeSamples - runtime.lookaheadSamples);
        if (!availableSamples) break;

        const entry = {status: 'pending', sampleCount: 0, promise: null};
        runtime.lookahead.set(candidate.segmentId, entry);
        const request = runtime.narrator.prefetch(candidate.text, candidate.voice, {
          segmentId: candidate.segmentId,
          maxCacheSamples: availableSamples,
        });
        if (!request) {
          failLookahead(runtime, candidate.segmentId, entry);
          break;
        }

        entry.promise = Promise.resolve(request).then(result => {
          if (generation !== runtime.generation || epoch !== runtime.lookaheadEpoch
            || runtime.lookahead.get(candidate.segmentId) !== entry) return false;
          const sampleCount = result?.samples?.length || 0;
          const currentSamples = runtime.samples instanceof Float32Array ? runtime.samples.length : 0;
          if (!sampleCount || sampleCount > availableSamples
            || currentSamples + runtime.lookaheadSamples + sampleCount > LOOKAHEAD_SAMPLE_LIMIT) {
            failLookahead(runtime, candidate.segmentId, entry);
            return false;
          }
          entry.status = 'ready';
          entry.sampleCount = sampleCount;
          runtime.lookaheadSamples += sampleCount;
          return true;
        }).catch(() => {
          failLookahead(runtime, candidate.segmentId, entry);
          return false;
        });

        if (!await entry.promise) break;
      }
    } catch {
      // Lookahead is opportunistic: the current segment can still play.
    } finally {
      const ownsCurrentWork = runtime.lookaheadWork === workRecord;
      if (ownsCurrentWork) runtime.lookaheadWork = null;
      for (const buffer of workRecord.initialBuffers) settleInitialBuffer(runtime, buffer);
      if (ownsCurrentWork && epoch === runtime.lookaheadEpoch && runtime.lookaheadRefresh
        && generation === runtime.generation && runtime.state === 'playing') {
        runtime.lookaheadRefresh = false;
        queueMicrotask(() => { void maintainLookahead(runtime, generation); });
      }
    }
  })();

  workRecord.promise = work;
  runtime.lookaheadWork = workRecord;
  return work;
}

function finishPlayback(runtime) {
  if (runtime.narrator.mode === 'hosted' && !runtime.narrator.paused) runtime.narrator.pause({notify:false});
  runtime.state = 'finished';
  setStatus(runtime, 'Finished this passage. Use Play to listen again or return to the reader.');
  updateDock(runtime);
}

async function advance(runtime, generation) {
  if (generation !== runtime.generation) return;
  const previous = runtime.queue[runtime.index];
  runtime.index += 1;
  runtime.chunkIndex = 0;
  runtime.currentVerseVoice = null;
  const nextRecord = runtime.queue[runtime.index];
  if (previous && nextRecord
    && (nextRecord.bookIndex !== previous.bookIndex || nextRecord.chapter !== previous.chapter)) runtime.chapterRoutes.clear();
  if (runtime.narrator.mode === 'hosted'
    && (!nextRecord || nextRecord.bookIndex !== previous?.bookIndex || nextRecord.chapter !== previous?.chapter)) {
    runtime.narrator.pause({notify:false});
  }
  if (runtime.index < runtime.queue.length) {
    speakCurrent(runtime, generation);
    return;
  }
  if (runtime.source?.continuous && runtime.source.next) {
    runtime.state = 'preparing';
    setStatus(runtime, `Loading ${runtime.source.next.bookName || runtime.source.config.book} ${runtime.source.next.chapter}…`);
    updateDock(runtime);
    if (await appendNextChapter(runtime, generation)) {
      if (generation === runtime.generation) speakCurrent(runtime, generation);
      return;
    }
    if (runtime.loadError) {
      runtime.state = 'error';
      setStatus(runtime, 'The next chapter could not load. Your place is saved; use Listen from the reader to try again.');
      updateDock(runtime);
      return;
    }
  }
  finishPlayback(runtime);
}

function speakCurrent(runtime, generation) {
  if (generation !== runtime.generation) return;
  const record = runtime.queue[runtime.index];
  if (!record) { finishPlayback(runtime); return; }
  if (runtime.chunkIndex === 0 && !runtime.currentVerseVoice) runtime.currentVerseVoice = runtime.voice;
  const voiceURI = runtime.currentVerseVoice || runtime.voice;
  const voice = voiceFor(runtime, voiceURI);
  const key = chapterRouteKey(record, voiceURI);
  let routeEntry = runtime.chapterRoutes.get(key);
  let route = routeEntry?.status === 'ready' ? routeEntry.routes.get(Number(record.verse)) : null;
  if (route?.kind === 'hosted' && !runtime.narrator.hasHostedRoute(route)) {
    runtime.chapterRoutes.delete(key);
    routeEntry = null;
    route = null;
  }
  if (!route) {
    if (runtime.state === 'playing' && runtime.narrator.mode === 'hosted') runtime.narrator.pause({notify:false});
    runtime.samples = null;
    runtime.peaks = null;
    runtime.sampleRate = 0;
    runtime.waveDuration = 0;
    runtime.narrator.pending = true;
    runtime.state = 'preparing';
    setStatus(runtime, `Checking hosted narration · ${record.book} ${record.chapter}:${record.verse}…`);
    updateDock(runtime);
    void chapterRoutesFor(runtime, record, voiceURI, generation, 'active').then(routes => {
      if (generation !== runtime.generation || !routes) return;
      if (runtime.currentVerseVoice !== voiceURI) { speakCurrent(runtime, generation); return; }
      speakCurrent(runtime, generation);
    }).catch(error => {
      if (runtime.currentVerseVoice !== voiceURI) { speakCurrent(runtime, generation); return; }
      failPlayback(runtime, error, generation);
    });
    return;
  }
  const hosted = route.kind === 'hosted';
  const chunks = hosted ? [record.text] : splitSpeech(record.text);
  if (hosted) runtime.chunkIndex = 0;
  if (runtime.chunkIndex >= chunks.length) {
    void advance(runtime, generation);
    return;
  }
  const chunkIndex = runtime.chunkIndex;
  if (!chunks[chunkIndex].trim()) {
    runtime.chunkIndex += 1;
    speakCurrent(runtime,generation);
    return;
  }
  runtime.samples = null;
  runtime.peaks = null;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  const initialBuffer = runtime.initialBufferPending && !hosted
    ? createInitialBuffer(runtime, generation) : null;
  if (initialBuffer) runtime.initialBufferPending = false;
  const utterance = {text: chunks[chunkIndex], lang: 'en-US', rate: runtime.rate, voice, route,
    stopAtCueEnd: hosted && !runtime.source?.continuous && runtime.index === runtime.queue.length - 1,
    segmentId:segmentId(runtime,record,chunkIndex), segmentIndex:chunkIndex, segmentCount:chunks.length};
  if (initialBuffer) {
    utterance.beforePlay = initialBuffer.promise;
    utterance.onbuffer = () => {
      if (generation !== runtime.generation || initialBuffer.generation !== generation) {
        settleInitialBuffer(runtime, initialBuffer);
        return;
      }
      setStatus(runtime, `Preparing local natural voice · buffering up to 3 segments · ${record.book} ${record.chapter}:${record.verse}…`);
      updateDock(runtime);
      initialBuffer.timer = setTimeout(() => settleInitialBuffer(runtime, initialBuffer), INITIAL_BUFFER_TIMEOUT_MS);
      void maintainLookahead(runtime, generation, initialBuffer);
    };
  }
  runtime.narrator.current = utterance;
  utterance.onstart = () => {
    if (generation !== runtime.generation) return;
    clearTimeout(runtime.startTimer);
    runtime.startTimer = null;
    runtime.state = 'playing';
    setStatus(runtime, readingStatus(record));
    if (chunkIndex === 0) publishPosition(runtime, record);
    dropLookahead(runtime, utterance.segmentId);
    maintainLookahead(runtime,generation);
    prefetchNextChapter(runtime);
    updateDock(runtime);
  };
  utterance.onend = () => {
    if (generation !== runtime.generation) return;
    runtime.utteranceDispatched = false;
    if (!hosted && chunkIndex + 1 < chunks.length) {
      runtime.chunkIndex += 1;
      speakCurrent(runtime, generation);
    } else {
      void advance(runtime, generation);
    }
  };
  utterance.onpause = () => {
    if (generation !== runtime.generation) return;
    runtime.state = 'paused';
    setStatus(runtime, `Paused · ${record.book} ${record.chapter}:${record.verse}`);
    updateDock(runtime);
  };
  utterance.onresume = () => {
    if (generation !== runtime.generation) return;
    runtime.state = 'playing';
    setStatus(runtime, readingStatus(record));
    maintainLookahead(runtime,generation);
    updateDock(runtime);
  };
  utterance.onerror = event => {
    settleInitialBuffer(runtime, initialBuffer);
    failPlayback(runtime, event, generation);
  };
  runtime.state = 'preparing';
  const label = hosted ? 'Preparing hosted chapter audio…'
    : route.fallbackFromHosted ? 'This chapter is not published yet · preparing local narration…'
      : 'Preparing the local natural voice…';
  setStatus(runtime, `${label} ${record.book} ${record.chapter}:${record.verse}`);
  updateDock(runtime);
  runtime.utteranceDispatched = true;
  runtime.narrator.speak(utterance);
}

function setSource(runtime, config, loaded, generation) {
  runtime.queue = loaded.records;
  runtime.index = Math.max(0, runtime.queue.findIndex(item => item.verse >= config.verse));
  if (runtime.index < 0) runtime.index = 0;
  runtime.chunkIndex = 0;
  const first = runtime.queue[runtime.index];
  const readerRoute = first?.route || loaded.route || config.route;
  const {root: _sourceRoot, runtime: _sourceRuntime, ...persistedConfig} = config;
  runtime.source = {
    key: config.key,
    kind: config.kind,
    readerMode: config.readerMode,
    bookMode: config.bookMode,
    bookIndex: loaded.bookIndex ?? config.bookIndex,
    book: loaded.book || config.book,
    reference: config.reference,
    route: readerRoute,
    readerRoute: routeURL(config.returnHref).pathname,
    returnHref: config.returnHref,
    config: persistedConfig,
    next: loaded.next,
    nextChapterName: null,
    continuous: config.readerMode || config.bookMode,
    lastLoadedRoute: loaded.route,
  };
  runtime.state = 'preparing';
  runtime.position = null;
  runtime.samples = null;
  runtime.peaks = null;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  setStatus(runtime, `Preparing ${first?.book || config.book || config.reference}…`);
  updateDock(runtime);
  prefetchNextChapter(runtime);
  if (generation === runtime.generation) speakCurrent(runtime, generation);
}

async function startSource(runtime, inputConfig) {
  if (!runtime.narrator.supported) {
    runtime.state = 'error';
    runtime.source = {reference: inputConfig.reference, returnHref: inputConfig.returnHref, key: inputConfig.key};
    updateDock(runtime);
    setStatus(runtime, 'Read-aloud is unavailable in this browser.');
    return;
  }
  const generation = cancelUtterance(runtime);
  runtime.source = {reference: inputConfig.reference, returnHref: inputConfig.returnHref, key: inputConfig.key,
    readerMode: inputConfig.readerMode, bookMode: inputConfig.bookMode, readerRoute: routeURL(inputConfig.returnHref).pathname};
  runtime.state = 'preparing';
  runtime.position = null;
  runtime.samples = null;
  runtime.peaks = null;
  runtime.initialBufferPending = true;
  runtime.currentVerseVoice = null;
  runtime.utteranceDispatched = false;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  runtime.narrator.unlock();
  setStatus(runtime, `Preparing ${inputConfig.reference || inputConfig.book || 'Scripture'}…`);
  updateDock(runtime);
  const config = {...inputConfig, runtime};
  try {
    let loaded;
    if (config.kind === 'range') {
      try {
        const payload = await apiRequest(`/api/verse?ref=${encodeURIComponent(config.reference)}`);
        if (generation !== runtime.generation) return;
        loaded = normalizeVerses(payload, config);
      } catch (error) {
        if (generation !== runtime.generation) return;
        loaded = fallbackRange(config.root, config);
        if (!loaded) throw error;
      }
      loaded.records = loaded.records.filter(item => item.verse >= config.verse
        && (!config.rangeEnd || item.verse <= config.rangeEnd));
      if (!loaded.records.length) throw new Error('No verses matched this passage.');
    } else {
      loaded = await loadChapter(config, config.bookIndex, config.chapter, generation);
      if (generation !== runtime.generation || !loaded) return;
    }
    if (generation !== runtime.generation) return;
    if (!Number.isInteger(loaded.bookIndex) || loaded.bookIndex < 0) {
      const visible = currentReaderPosition(config.root, config.root?.querySelector?.('[data-reader]'), config.reference);
      if (Number.isInteger(visible.bookIndex)) {
        loaded.bookIndex = visible.bookIndex;
        loaded.records.forEach(record => { record.bookIndex = visible.bookIndex; });
      }
    }
    if (config.kind === 'reader' && loaded.records.length) {
      loaded.records = loaded.records.filter(item => item.verse >= config.verse || item.chapter > config.chapter);
    }
    runtime.source.next = loaded.next;
    setSource(runtime, config, loaded, generation);
  } catch (error) {
    if (generation !== runtime.generation) return;
    runtime.state = 'error';
    setStatus(runtime, error.message || 'Scripture could not be loaded for audio.');
    updateDock(runtime);
  }
}

export function initExperience(root = document) {
  const runtime = getRuntime();
  runtime.currentRoot = root;
  runtime.previewConfig = null;
  const reader = root.querySelector?.('[data-reader]');
  if (runtime.source?.readerMode && ACTIVE_STATES.has(runtime.state) && reader && runtime.position
    && routeURL(reader.dataset.readerRoute).pathname === routeURL(runtime.position.route).pathname
    && routeURL().pathname === routeURL(reader.dataset.readerRoute).pathname) {
    runtime.source.readerRoute = routeURL(reader.dataset.readerRoute).pathname;
    const current = routeURL();
    if (current.searchParams.has('context')) runtime.source.returnHref = currentRoute();
  }
  const controls = root.querySelector?.('[data-read-aloud]');
  if (controls) sourceControls(controls, root, runtime);
  if (runtime.position) {preserveContextHref(runtime.source,runtime.position);followVerse(runtime, runtime.position);}
  updateDock(runtime);
  return window.biblePlayback;
}

export function disposeExperience() {
  // Reader routes are frequently remounted by the Rust/WASM shell. The
  // body-level narrator and its queue intentionally outlive those fragments.
  return window.biblePlayback;
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => initExperience(), {once: true});
else initExperience();

export { parseReference, normalizeNext, normalizeVerses, splitSpeech, formatTime };
