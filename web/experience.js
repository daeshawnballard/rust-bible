import { readProgress, positionForVerse } from './progress.js';
import { LocalNarrator, DEFAULT_NARRATOR } from './local-voice.js';

const RUNTIME_KEY = Symbol.for('rust-bible.global-audio.v1');
const RATE_KEY = 'bible.rate';
const VOICE_KEY = 'bible.narrator';
const FOLLOW_KEY = 'bible.audio.follow';
const VOLUME_KEY = 'bible.audio.volume';
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
  return `${window.location.pathname}${window.location.search}${window.location.hash}`;
}

function appHref(route) {
  return window.location.hash.startsWith('#/') ? `#${route}` : route;
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

function splitSpeech(text) {
  return String(text || '').match(/.{1,220}(?:\s|$)|\S{1,220}/g)?.map(part => part.trim()).filter(Boolean) || [];
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
    <label class="audio-dock-rate" aria-label="Playback speed">
      <select data-dock-rate aria-label="Playback speed">
        <option value="0.75">0.75×</option><option value="1">1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option><option value="2">2×</option>
      </select>
    </label>
    <details class="audio-dock-options">
      <summary aria-label="More playback options">•••</summary>
      <div class="audio-dock-menu">
        <label class="audio-dock-menu-row">Voice<select data-dock-voice aria-label="Narrator voice"></select></label>
        <label class="audio-dock-menu-row audio-dock-follow"><input type="checkbox" data-dock-follow> Follow the spoken verse</label>
        <label class="audio-dock-menu-row">Natural voice volume<input data-dock-volume type="range" min="0" max="1" step="0.01" aria-label="Natural voice volume"></label>
        <button class="audio-dock-stop" data-dock-stop type="button">Stop reading</button>
        <small>Free natural voices are generated on this device. Device voice volume follows your system settings.</small>
      </div>
    </details>`;
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
    follow: dock.querySelector('[data-dock-follow]'),
    volume: dock.querySelector('[data-dock-volume]'),
    stop: dock.querySelector('[data-dock-stop]'),
  };

  runtime.ui.rate.value = String(runtime.rate);
  runtime.ui.follow.checked = runtime.follow;
  runtime.ui.volume.value = String(runtime.volume);
  populateVoices(runtime);

  runtime.ui.toggle.addEventListener('click', () => togglePlayback(runtime));
  runtime.ui.stop.addEventListener('click', () => stopPlayback(runtime));
  runtime.ui.rate.addEventListener('change', () => {
    runtime.rate = Number(runtime.ui.rate.value) || 1;
    writePreference(RATE_KEY, runtime.rate);
    runtime.narrator.setRate(runtime.rate);
    if (ACTIVE_STATES.has(runtime.state)) {
      setStatus(runtime, 'Playback speed updated. The current generated audio continues at the selected speed.');
    }
  });
  runtime.ui.voice.addEventListener('change', () => {
    runtime.voice = runtime.ui.voice.value || DEFAULT_NARRATOR;
    writePreference(VOICE_KEY, runtime.voice);
    if (ACTIVE_STATES.has(runtime.state)) {
      setStatus(runtime, 'Voice selected. It will be used for the next verse.');
    }
  });
  runtime.ui.follow.addEventListener('change', () => {
    runtime.follow = runtime.ui.follow.checked;
    runtime.followWasSaved = true;
    writePreference(FOLLOW_KEY, runtime.follow);
    if (runtime.follow && runtime.position) followVerse(runtime, runtime.position);
  });
  runtime.ui.volume.addEventListener('input', () => {
    runtime.volume = Number(runtime.ui.volume.value);
    writePreference(VOLUME_KEY, runtime.volume);
    runtime.narrator.setVolume(runtime.volume);
  });
  runtime.ui.seek.addEventListener('input', () => {
    if (runtime.narrator.seek(Number(runtime.ui.seek.value))) updateTimeline(runtime);
  });

  const media = runtime.narrator.audio;
  for (const event of ['timeupdate', 'durationchange', 'loadedmetadata', 'play', 'pause', 'ended']) {
    media.addEventListener(event, () => updateTimeline(runtime));
  }
  runtime.narrator.addEventListener('audiobuffer', event => {
    runtime.samples = event.detail.samples;
    runtime.sampleRate = event.detail.sampleRate;
    runtime.waveDuration = event.detail.duration;
    drawWaveform(runtime);
    updateTimeline(runtime);
  });
  runtime.narrator.addEventListener('voiceschanged', () => populateVoices(runtime));
  window.addEventListener('resize', () => drawWaveform(runtime), {passive: true});
  updateDock(runtime);
  return dock;
}

function getRuntime() {
  if (window[RUNTIME_KEY]) return window[RUNTIME_KEY];
  const runtime = {
    narrator: null, dock: null, ui: null, source: null, previewConfig: null, queue: [], index: 0, chunkIndex: 0,
    generation: 0, state: 'idle', position: null, samples: null, sampleRate: 0,
    waveDuration: 0, rate: Number(readPreference(RATE_KEY, '1')) || 1,
    voice: readPreference(VOICE_KEY, DEFAULT_NARRATOR),
    follow: readPreference(FOLLOW_KEY, null) === null ? true : readPreference(FOLLOW_KEY, 'true') === 'true',
    followWasSaved: readPreference(FOLLOW_KEY, null) !== null,
    volume: Math.max(0, Math.min(1, Number(readPreference(VOLUME_KEY, '1')) || 0)),
    currentRoot: document, startTimer: null, loadingNext: null, loadError: null,
  };
  runtime.narrator = new LocalNarrator({onProgress: progress => {
    if (runtime.state !== 'preparing') return;
    const total = Number(progress.total) || 0;
    const loaded = Number(progress.loaded) || 0;
    const message = total > 0
      ? `Downloading the free natural voice · ${Math.round(loaded / total * 100)}% (${Math.ceil(loaded / 1048576)} / ${Math.ceil(total / 1048576)} MB)`
      : progress.message || 'Preparing the free natural voice…';
    setStatus(runtime, message);
  }});
  runtime.narrator.setVolume(runtime.volume);
  window[RUNTIME_KEY] = runtime;
  window.biblePlayback = Object.freeze({
    active: () => ACTIVE_STATES.has(runtime.state),
    position: () => runtime.position ? {...runtime.position} : null,
    stop: () => stopPlayback(runtime),
    returnHref: () => runtime.source?.returnHref || currentRoute(),
    setFollow: value => {
      runtime.follow = Boolean(value);
      runtime.followWasSaved = true;
      if (runtime.ui) runtime.ui.follow.checked = runtime.follow;
      writePreference(FOLLOW_KEY, runtime.follow);
    },
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
  const voices = runtime.narrator.getVoices().filter(voice => /^en\b/i.test(voice.lang));
  runtime.ui.voice.replaceChildren();
  for (const voice of voices) {
    const option = document.createElement('option');
    option.value = voice.voiceURI;
    option.textContent = `${voice.name} (${voice.lang})`;
    runtime.ui.voice.append(option);
  }
  const device = document.createElement('option');
  device.value = 'device:default';
  device.textContent = 'Device default · no download';
  runtime.ui.voice.append(device);
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
    runtime.ui.follow.checked = runtime.follow;
    runtime.ui.rate.value = String(runtime.rate);
    runtime.ui.volume.value = String(runtime.volume);
    if (previewOnly) runtime.ui.status.textContent = 'Ready to listen · the voice prepares after Play';
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
  runtime.ui.stop.disabled = !['preparing', 'playing', 'paused'].includes(runtime.state);
  updateTimeline(runtime);
  drawWaveform(runtime);
  updateSourceButton(runtime);
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
    ? `${formatTime(currentTime)} of ${formatTime(duration)}` : 'Seeking is available for generated natural voice audio');
  runtime.ui.volume.disabled = media.mode === 'device';
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
  if (!(samples instanceof Float32Array) || !samples.length) return;
  const bars = Math.max(24, Math.min(112, Math.floor(rect.width / 5)));
  const gap = Math.max(1, Math.round(2 * scale));
  const barWidth = Math.max(1, Math.floor(width / bars) - gap);
  const progress = runtime.narrator.seekable && runtime.narrator.duration > 0
    ? runtime.narrator.currentTime / runtime.narrator.duration : 0;
  const middle = height / 2;
  for (let index = 0; index < bars; index++) {
    const start = Math.floor(index * samples.length / bars);
    const end = Math.max(start + 1, Math.floor((index + 1) * samples.length / bars));
    let peak = 0;
    for (let sample = start; sample < end; sample++) peak = Math.max(peak, Math.abs(samples[sample]));
    const barHeight = Math.max(Math.round(2 * scale), Math.round(peak * height * 0.86));
    const x = Math.floor(index * width / bars);
    context.fillStyle = index / bars <= progress ? '#e4bd65' : '#6d6b64';
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
  return [...document.querySelectorAll('[data-reader-route] .scripture .v')].find(node => {
    const position = positionForVerse(node);
    return position && position.bookIndex === record.bookIndex
      && position.chapter === record.chapter && position.verse === record.verse;
  }) || null;
}

function followVerse(runtime, record) {
  if (!runtime.follow || !isAtCurrentReader(runtime)) return;
  const node = findVisibleVerse(record);
  if (!node) return;
  document.querySelectorAll('.is-speaking').forEach(active => active.classList.remove('is-speaking'));
  node.classList.add('is-speaking');
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
  controls.hidden = (defaultInDock && !activeElsewhere) || sameSource;
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
  const previewIsDifferent = runtime.previewConfig && runtime.previewConfig.key !== runtime.source?.key;
  const config = previewIsDifferent || runtime.previewConfig?.key === runtime.source?.key
    ? runtime.previewConfig : runtime.source?.config || runtime.previewConfig;
  if (config) void startSource(runtime, config);
}

function cancelUtterance(runtime) {
  clearTimeout(runtime.startTimer);
  runtime.startTimer = null;
  runtime.generation += 1;
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
  runtime.source = null;
  runtime.position = null;
  runtime.samples = null;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  document.querySelectorAll('.is-speaking').forEach(node => node.classList.remove('is-speaking'));
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
    returnHref: returnRoute, key, root, followDefault: controls.querySelector('[data-audio-follow]')?.checked !== false,
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
  play.dataset.audioFollowDefault = String(initialConfig.followDefault);
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
      config.followDefault = play.dataset.audioFollowDefault === 'true';
      if (!runtime.followWasSaved) {
        runtime.follow = config.followDefault;
        runtime.ui.follow.checked = runtime.follow;
        runtime.followWasSaved = true;
        writePreference(FOLLOW_KEY, runtime.follow);
      }
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
      if (!additional.length) return false;
      runtime.queue.push(...additional);
      runtime.source.next = loaded.next;
      runtime.source.lastLoadedRoute = loaded.route;
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

function prefetchUpcoming(runtime, generation) {
  if (!runtime.source?.continuous || runtime.index !== runtime.queue.length - 1) return;
  void appendNextChapter(runtime, generation);
}

function readingStatus(record) {
  return `Listening · ${record.book} ${record.chapter}:${record.verse}`;
}

function finishPlayback(runtime) {
  runtime.state = 'finished';
  setStatus(runtime, 'Finished this passage. Use Play to listen again or return to the reader.');
  updateDock(runtime);
}

async function advance(runtime, generation) {
  if (generation !== runtime.generation) return;
  runtime.index += 1;
  runtime.chunkIndex = 0;
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
  const chunks = splitSpeech(record.text);
  if (runtime.chunkIndex >= chunks.length) {
    void advance(runtime, generation);
    return;
  }
  const chunkIndex = runtime.chunkIndex;
  runtime.samples = null;
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  const voice = runtime.narrator.getVoices().find(item => item.voiceURI === runtime.voice) || null;
  const utterance = {text: chunks[chunkIndex], lang: 'en-US', rate: runtime.rate, voice};
  runtime.narrator.current = utterance;
  utterance.onstart = () => {
    if (generation !== runtime.generation) return;
    clearTimeout(runtime.startTimer);
    runtime.startTimer = null;
    runtime.state = 'playing';
    setStatus(runtime, readingStatus(record));
    if (chunkIndex === 0) publishPosition(runtime, record);
    prefetchUpcoming(runtime, generation);
    const nextChunk = chunks[chunkIndex + 1] || runtime.queue[runtime.index + 1]?.text;
    runtime.narrator.prefetch(nextChunk, voice);
    updateDock(runtime);
  };
  utterance.onend = () => {
    if (generation !== runtime.generation) return;
    if (chunkIndex + 1 < chunks.length) {
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
    updateDock(runtime);
  };
  utterance.onerror = event => {
    if (generation !== runtime.generation) return;
    clearTimeout(runtime.startTimer);
    runtime.startTimer = null;
    runtime.state = 'error';
    runtime.narrator.cancel();
    setStatus(runtime, `Audio could not play: ${event.error || 'unknown error'}. Choose another voice or try again.`);
    updateDock(runtime);
  };
  runtime.state = 'preparing';
  const label = runtime.narrator.isLocalVoice(voice) ? 'Preparing the free natural voice…' : 'Starting the device voice…';
  setStatus(runtime, `${label} ${record.book} ${record.chapter}:${record.verse}`);
  updateDock(runtime);
  if (!runtime.narrator.isLocalVoice(voice)) {
    runtime.startTimer = setTimeout(() => {
      if (generation !== runtime.generation || runtime.state !== 'preparing') return;
      runtime.state = 'error';
      runtime.narrator.cancel();
      setStatus(runtime, 'The device speech service did not start. Choose another voice or try again.');
      updateDock(runtime);
    }, 7000);
  }
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
  runtime.sampleRate = 0;
  runtime.waveDuration = 0;
  setStatus(runtime, `Preparing ${first?.book || config.book || config.reference}…`);
  updateDock(runtime);
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
