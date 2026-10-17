import { readProgress, saveProgress, positionForVerse } from './progress.js';
import { createTerrainMap } from './terrain.js';

const mounted = new WeakMap();
const DISPLAY_KEY = 'rust-bible:reader-display:v1';
const routeURL = () => new URL(location.hash.startsWith('#/') ? location.hash.slice(1) : location.pathname + location.search + location.hash, location.origin);
const requestContext = async (uri) => typeof window.rustBibleApi === 'function' ? window.rustBibleApi(uri) : fetch(uri).then(async response => {const value = await response.json(); if (!response.ok) throw new Error(value.error); return value;});

export function resolveLegacyReaderRoute(route) {
  const url = new URL(route, location.origin);
  if ((url.pathname === '/explore' || url.pathname === '/explore/studies') && !url.searchParams.has('book') && !url.searchParams.has('scene')) {
    const saved = readProgress();
    return `${saved?.route || '/book/Genesis/1'}?resume=${saved?.verse || 1}&context=journey#v${saved?.verse || 1}`;
  }
  return route;
}

export function disposeReaderContext(root = document) { mounted.get(root)?.(); mounted.delete(root); }

export function initReaderContext(root = document) {
  disposeReaderContext(root);
  const reader = root.querySelector('.reader-page[data-reader]');
  const panel = reader?.querySelector('[data-reader-context]');
  if (!reader || !panel) return;
  const body = panel.querySelector('[data-context-body]');
  const status = panel.querySelector('[data-context-status]');
  let active = true, generation = 0, terrain = null, mapState = null, currentValue = null;
  let mode = ['journey', 'illustration'].includes(routeURL().searchParams.get('context')) ? routeURL().searchParams.get('context') : 'passage';
  let part = routeURL().searchParams.get('part');
  let scene = routeURL().searchParams.get('scene');
  let artView = routeURL().searchParams.get('art') === 'scene' ? 'scene' : routeURL().searchParams.get('art') === 'detail' || part ? 'detail' : 'scene';
  let lastHTML = '', follow = true;
  const node = reader.dataset.readerPosition ? document.getElementById(reader.dataset.readerPosition) : null;
  let position = positionForVerse(node) || {bookIndex:Number(reader.dataset.bookIndex), chapter:Number(reader.dataset.chapter), verse:1, route:reader.dataset.readerRoute};
  const abort = new AbortController();
  const { signal } = abort;
  // A tall companion scrolls until its bottom is reachable, then stays beside
  // the passage. No nested scrolling area or unreachable sticky-card actions.
  const sizePanel=()=>{panel.style.top=`${Math.min(20,innerHeight-panel.offsetHeight-20)}px`;};
  const panelObserver=new ResizeObserver(sizePanel); panelObserver.observe(panel);
  window.addEventListener('resize',sizePanel,{signal});

  const writeURL = () => {
    const url = routeURL();
    for (const key of ['context','scene','part','art']) url.searchParams.delete(key);
    if (mode !== 'passage') url.searchParams.set('context', mode);
    if (mode === 'illustration' && scene) url.searchParams.set('scene', scene);
    if (mode === 'illustration' && part) url.searchParams.set('part', part);
    if (mode === 'illustration') url.searchParams.set('art',artView);
    const path = `${url.pathname}${url.search}${url.hash}`;
    history.replaceState(history.state, '', location.hash.startsWith('#/') ? `#${path}` : path);
  };
  const applyDisplay = (value) => {
    reader.dataset.textSize = ['standard','large','larger'].includes(value?.size) ? value.size : 'standard';
    reader.dataset.lineSpacing = value?.spacing === 'spacious' ? 'spacious' : 'comfortable';
    reader.querySelector('[data-reader-size]').value = reader.dataset.textSize;
    reader.querySelector('[data-reader-spacing]').value = reader.dataset.lineSpacing;
  };
  try { applyDisplay(JSON.parse(localStorage.getItem(DISPLAY_KEY))); } catch { applyDisplay(null); }
  const stopMap = () => { terrain?.destroy(); terrain = null; mapState = null; if(body.querySelector('[data-context-map]')) lastHTML=''; };
  const updateButtons = () => {
    reader.dataset.contextMode = mode;
    reader.dataset.artView = artView;
    reader.querySelectorAll('[data-context-art-view]').forEach(button=>button.setAttribute('aria-pressed',String(button.dataset.contextArtView===artView)));
    panel.hidden = mode === 'passage';
    reader.querySelectorAll('.reader-modes [data-reader-mode]').forEach(button => {
      button.setAttribute('aria-pressed', String(button.dataset.readerMode === mode));
      if (button.dataset.readerMode === 'illustration' && currentValue) {button.classList.toggle('has-study', !!currentValue.relevant_scene); button.hidden=!currentValue.relevant_scene && mode!=='illustration';}
      if (button.dataset.readerMode === 'journey' && currentValue) {
        button.classList.toggle('has-journey', !!currentValue.journey_available);
        button.title = currentValue.journey_available ? 'Illustrated Journey available in this chapter' : 'View this passage’s setting';
      }
    });
  };
  async function refresh({focusPart = null} = {}) {
    updateButtons();
    const request = ++generation;
    if (mode === 'passage') {
      stopMap();
      try {const value=await requestContext(`/api/reader-context?book=${position.bookIndex}&chapter=${position.chapter}&verse=${position.verse}&mode=availability`); if(active && request===generation) {currentValue=value; updateButtons();}}catch{}
      return;
    }
    status.textContent = 'Following the passage…';
    const params = new URLSearchParams({book:position.bookIndex, chapter:position.chapter, verse:position.verse, mode});
    if (part) params.set('part', part);
    try {
      const value = await requestContext(`/api/reader-context?${params}`);
      if (!active || request !== generation) return;
      const sceneChanged = scene && value.scene && scene !== value.scene;
      if (sceneChanged && part) { part = null; scene = value.scene; return refresh(); }
      currentValue = value;
      scene = value.scene || null;
      panel.querySelector('[data-context-title]').textContent = value.title;
      panel.querySelector('[data-context-reference]').textContent = value.reference;
      const sameGeo = value.kind === 'geography' && mapState && mapState.book === value.book && mapState.chapter === value.chapter;
      if (sameGeo) {
        mapState = value.map;
        terrain?.update(value.map, {follow, animate:true});
        body.querySelector('[data-context-place-title]').textContent = value.map.setting?.name || value.map.title;
        body.querySelector('[data-context-place-note]').textContent = value.map.setting?.note || value.map.message;
      } else if (lastHTML !== value.html) {
        stopMap(); body.innerHTML = value.html; lastHTML = value.html;
        if (value.kind === 'illustration') {
          const tools = document.createElement('div'); tools.className = 'reader-context-follow';
          const views=document.createElement('div'); views.className='reader-context-art-views'; views.setAttribute('role','group'); views.setAttribute('aria-label','Illustration view');
          for(const [view,label] of [['scene','Scene'],['detail','Detail']]) {const button=document.createElement('button');button.type='button';button.dataset.contextArtView=view;button.textContent=label;views.append(button);}
          tools.append(views);
          const button = document.createElement('button'); button.type='button'; button.dataset.contextFollow='';
          button.textContent = part ? 'Follow passage' : 'Following passage';
          button.setAttribute('aria-pressed',String(!part)); tools.append(button);
          const label=document.createElement('label'); label.textContent='Detail ';
          const select=document.createElement('select'); select.dataset.contextPartSelect=''; select.setAttribute('aria-label','Illustrated detail');
          for(const hotspot of body.querySelectorAll('[data-context-part]')) {const option=document.createElement('option'); option.value=hotspot.dataset.contextPart; option.textContent=hotspot.getAttribute('aria-label').replace(/^Select /,''); option.selected=hotspot.getAttribute('aria-pressed')==='true'; select.append(option);}
          label.append(select); tools.append(label); body.prepend(tools);
        }
        if (value.kind === 'geography') {
          mapState = value.map;
          const mapHost=body.querySelector('[data-context-map]');
          const controller = await createTerrainMap(mapHost, {
            onInteract() { follow = false; const button=body.querySelector('[data-context-map-action="follow"]'); if(button) button.textContent='Return to passage'; },
            onPlaceSelect(place) { const heading=body.querySelector('[data-context-place-title]'); if(heading) heading.textContent=place.name; const note=body.querySelector('[data-context-place-note]'); if(note) note.textContent=`${place.confidence || 'Approximate'} identification. Use a journey-stop link to read its passage.`; },
            onStatus(event) { const node=body.querySelector('[data-context-map-status]'); if(node) node.textContent=event.message || 'Modern terrain · Approximate locations'; const retry=body.querySelector('[data-context-map-action="retry"]'); if(retry) retry.hidden=!['error','degraded'].includes(event.state); }
          });
          if (!active || !mapHost.isConnected || mapHost!==body.querySelector('[data-context-map]')) {controller.destroy(); return;}
          terrain = controller; terrain.set3D(true); terrain.update(mapState, {follow, animate:false, force:true});
        }
      }
      const followButton=body.querySelector('[data-context-follow]');
      if(followButton) {followButton.textContent=part ? 'Follow passage' : 'Following passage'; followButton.setAttribute('aria-pressed',String(!part));}
      if (focusPart) {
        const target=artView==='detail' ? body.querySelector('.reader-illustration-copy h3') : body.querySelector(`[data-context-part="${CSS.escape(focusPart)}"]`);
        if(target) {if(artView==='detail') target.tabIndex=-1;target.focus({preventScroll:true});}
      }
      status.textContent = value.kind === 'illustration' ? (part ? 'Browsing a detail. Your reading position has not moved.' : value.source_match ? 'Details follow the passage. Explanations are separate from Scripture.' : 'Illustration for this chapter. The selected item’s source is shown below.') : 'Journey follows your reading and listening position.';
      updateButtons(); writeURL();
    } catch (error) {
      if (!active || request !== generation) return;
      if (part) {part = null; return refresh();}
      status.textContent = 'The companion could not load. Scripture and audio remain available.';
      console.warn('Reader companion:', error.message);
    }
  }
  function findVerse(book, chapter, verse) {
    return [...reader.querySelectorAll('.scripture .v')].find(node => {
      const value = positionForVerse(node);
      return value && value.route === `/book/${book}/${chapter}` && value.verse === Number(verse);
    });
  }
  root.addEventListener('click', event => {
    const target = event.target.closest('[data-reader-mode],[data-context-part],[data-context-follow],[data-context-read],[data-context-map-action],[data-context-art-view]');
    if (!target || !reader.contains(target)) return;
    if (target.hasAttribute('data-context-read')) {
      const verse = findVerse(target.dataset.contextBook, target.dataset.contextChapter, target.dataset.contextVerse);
      if (!verse) {
        // An intentional source jump opens the text even on a narrow screen.
        const sourceURL=new URL(target.getAttribute('href').replace(/^#(?=\/)/,''),location.origin);
        sourceURL.searchParams.set('focus','passage');
        const href=`${sourceURL.pathname}${sourceURL.search}${sourceURL.hash}`;
        target.setAttribute('href',location.hash.startsWith('#/')?`#${href}`:href);
        return;
      }
      event.preventDefault(); event.stopImmediatePropagation();
      const targetURL = new URL(target.getAttribute('href').replace(/^#(?=\/)/,''),location.origin);
      for (const key of ['context','scene','part']) if(targetURL.searchParams.has(key)) {
        if(key==='context') mode=targetURL.searchParams.get(key);
        if(key==='part') part=targetURL.searchParams.get(key);
        if(key==='scene') scene=targetURL.searchParams.get(key);
      }
      const url=routeURL(); url.searchParams.set('resume',target.dataset.contextVerse); url.hash=`v${target.dataset.contextVerse}`;
      history.replaceState(history.state,'',location.hash.startsWith('#/')?`#${url.pathname}${url.search}${url.hash}`:`${url.pathname}${url.search}${url.hash}`);
      const player=reader.querySelector('[data-reader-audio]');
      if(['starting','reading','paused'].includes(player?.dataset.audioState)) player.querySelector('[data-audio-action="stop"]')?.click();
      saveProgress(verse); verse.classList.add('is-context-current');
      reader.querySelectorAll('.is-context-current').forEach(node=>{if(node!==verse) node.classList.remove('is-context-current');});
      verse.scrollIntoView({block:'center',behavior:matchMedia('(prefers-reduced-motion:reduce)').matches?'instant':'smooth'});
      verse.tabIndex=-1; verse.focus({preventScroll:true}); refresh(); return;
    }
    event.preventDefault(); event.stopImmediatePropagation();
    if (target.dataset.contextArtView) {artView=target.dataset.contextArtView;updateButtons();writeURL();return;}
    if (target.dataset.readerMode) {mode=target.dataset.readerMode; writeURL(); refresh(); return;}
    if (target.dataset.contextPart) {part=target.dataset.contextPart;artView='detail'; refresh({focusPart:part}); return;}
    if (target.hasAttribute('data-context-follow')) {part=null; refresh(); return;}
    const action=target.dataset.contextMapAction;
    if(action==='zoom-in') terrain?.zoom(1);
    if(action==='zoom-out') terrain?.zoom(-1);
    if(action==='overview') {follow=false; terrain?.fit();}
    if(action==='terrain') {const enabled=target.getAttribute('aria-pressed')!=='true';target.setAttribute('aria-pressed',String(enabled));terrain?.set3D(enabled);}
    if(action==='retry') {stopMap();lastHTML='';refresh();}
    if(action==='follow') {follow=true; terrain?.update(mapState,{follow:true,force:true}); target.textContent='Follow passage';}
  }, {capture:true,signal});
  reader.addEventListener('change', event => {
    if(event.target.matches('[data-context-part-select]')) {part=event.target.value;artView='detail';refresh({focusPart:part});return;}
    if(!event.target.matches('[data-reader-size],[data-reader-spacing]')) return;
    const value={size:reader.querySelector('[data-reader-size]').value,spacing:reader.querySelector('[data-reader-spacing]').value};
    applyDisplay(value); try {localStorage.setItem(DISPLAY_KEY,JSON.stringify(value));}catch{}
  },{signal});
  reader.addEventListener('bible:position', event => {
    const next=event.detail?.record;
    if(!next || next.bookIndex===position.bookIndex && next.chapter===position.chapter && next.verse===position.verse) return;
    if(next.bookIndex!==position.bookIndex || next.chapter!==position.chapter) part=null;
    position=next; refresh();
  },{signal});
  const dispose=()=>{active=false;generation++;abort.abort();panelObserver.disconnect();stopMap();};
  mounted.set(root,dispose); refresh();
}

if (!document.getElementById('app')) {
  const run=()=>initReaderContext(document);
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',run,{once:true}); else run();
}
