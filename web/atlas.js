import { createTerrainMap } from './terrain.js';
import { readProgress, saveProgress, positionForVerse } from './progress.js';

const mounted = new WeakMap();
const mapPath = record => `/explore?book=${record.bookIndex ?? encodeURIComponent(record.book)}&chapter=${record.chapter}&verse=${record.verse || 1}`;
const href = path => typeof window.rustBibleApi === 'function' ? `#${path}` : path;
const navigate = path => { window.location.href = href(path); };
const reduced = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const isReading = root => ['starting','reading','paused'].includes(root.querySelector('[data-reader-audio]')?.dataset.audioState);

export function resolveMapRoute(route) {
  const [path,query=''] = route.split('?');
  if (path !== '/explore' || new URLSearchParams(query).has('book') || new URLSearchParams(query).has('scene')) return route;
  const saved=readProgress();
  return saved ? mapPath(saved) : route;
}

export function disposeAtlas(root=document) { mounted.get(root)?.(); mounted.delete(root); }

export function initAtlas(root=document) {
  disposeAtlas(root);
  const abort=new AbortController();
  const on=(target,event,fn,options={})=>target?.addEventListener(event,fn,{...options,signal:abort.signal});
  let disposed=false, view=null, state=null, following=true, threeD=true, request=0, autoScrollUntil=0, scrollTimer;
  const page=root.querySelector('[data-atlas-page]');
  const syncLinks=(record=readProgress())=>{
    root.querySelectorAll('[data-atlas-entry], [data-atlas-reader-link]').forEach(link=>{
      if(record) link.href=href(mapPath(record));
    });
  };
  syncLinks();
  const cleanup=()=>{disposed=true;request++;clearTimeout(scrollTimer);abort.abort();view?.destroy();};
  mounted.set(root,cleanup);
  if(!page){on(root,'bible:position',event=>syncLinks(event.detail.record));return;}
  try {state=JSON.parse(page.querySelector('[data-atlas-state]').textContent);} catch {return;}
  const panel=page.querySelector('[data-atlas-scroll]');
  const mapContainer=page.querySelector('[data-atlas-map]');
  const followButton=page.querySelector('[data-atlas-action=follow]');
  const title=page.querySelector('[data-atlas-title]');
  const status=page.querySelector('[data-atlas-map-message]');
  const retry=page.querySelector('[data-atlas-action=retry]');
  const api=uri=>typeof window.rustBibleApi==='function' ? Promise.resolve(window.rustBibleApi(uri)) : fetch(uri,{cache:'no-store'}).then(r=>{if(!r.ok)throw new Error('The geographic context could not load.');return r.json();});
  const currentLabel=()=>`${state.book_name} ${state.chapter}:${state.verse}`;
  const showTab=name=>{
    page.querySelectorAll('[data-atlas-tab]').forEach(tab=>{tab.setAttribute('aria-selected',String(tab.dataset.atlasTab===name));tab.tabIndex=tab.dataset.atlasTab===name ? 0 : -1;});
    panel.hidden=name!=='scripture';page.querySelector('[data-atlas-context]').hidden=name!=='context';
  };
  const setFollow=value=>{
    following=value;followButton.setAttribute('aria-pressed',String(value));followButton.textContent=value?'Follow passage':'Return to passage';
    page.querySelector('[data-atlas-follow-status]').textContent=`${value?'Following':'Exploring ·'} ${currentLabel()}`;
  };
  const scrollToNode=(node,animate=true)=>{
    if(!node || panel.hidden)return;
    autoScrollUntil=Date.now()+1000;
    const top=node.getBoundingClientRect().top-panel.getBoundingClientRect().top+panel.scrollTop-Math.min(130,panel.clientHeight*.22);
    panel.scrollTo({top:Math.max(0,top),behavior:animate&&!reduced()?'smooth':'instant'});
  };
  const findVerse=(book,chapter,verse)=>[...panel.querySelectorAll('.scripture .v')].find(node=>{
    const p=positionForVerse(node);return p?.bookIndex===book && p.chapter===chapter && p.verse===verse;
  });
  const chooseVerse=(book,chapter,verse)=>{
    const node=findVerse(book,chapter,verse);
    if(!node){navigate(`/explore?book=${book}&chapter=${chapter}&verse=${verse}`);return;}
    if(isReading(root))root.querySelector('[data-audio-action=stop]')?.click();
    showTab('scripture');saveProgress(node);scrollToNode(node);node.focus({preventScroll:true});
  };
  const sourceLink=(url,label)=>{
    const link=document.createElement('a');link.textContent=label;
    if(typeof url==='string' && url.startsWith('https://')){link.href=url;link.target='_blank';link.rel='noopener noreferrer';}
    return link;
  };
  const showPlace=place=>{
    page.dataset.panelOpen='true';page.querySelector('[data-atlas-action=panel]').setAttribute('aria-expanded','true');
    showTab('context');page.querySelector('[data-atlas-place-name]').textContent=place.name;
    page.querySelector('[data-atlas-explanation]').textContent=place.note || 'This is a geographic place referenced by the text. Its mention does not, by itself, establish where the surrounding action happens.';
    const detail=page.querySelector('[data-atlas-place-detail]');detail.replaceChildren();
    const confidence=document.createElement('p');confidence.className='atlas-confidence';confidence.textContent=`Identification: ${place.confidence || 'approximate'}`;detail.append(confidence);
    detail.append(sourceLink(place.source?.startsWith('https://')?place.source:place.url,'Location source ↗'));
    const verse=place.verse || place.verses?.[0];
    if(verse && state.scope==='chapter'){
      const button=document.createElement('button');button.type='button';button.textContent=`Read ${state.book_name} ${state.chapter}:${verse}`;
      button.addEventListener('click',()=>chooseVerse(state.book,state.chapter,verse),{signal:abort.signal});detail.append(button);
    }
  };
  const drawLists=()=>{
    const list=page.querySelector('[data-atlas-place-list]');list.replaceChildren();
    for(const place of state.places){const button=document.createElement('button');button.type='button';button.textContent=place.name;button.addEventListener('click',()=>showPlace(place),{signal:abort.signal});list.append(button);}
    page.querySelector('[data-atlas-place-help]').textContent=state.scope==='book'?'This passage has no identified place. These are references elsewhere in this book.':'Place references are distinct from the story setting. Some identifications have multiple candidates.';
    const stops=page.querySelector('[data-atlas-stops]');stops.replaceChildren();
    if(state.stops.length){for(const stop of state.stops){const button=document.createElement('button');button.type='button';button.textContent=stop.label||stop.name;button.dataset.stop=stop.id;
      if(state.setting?.id===stop.id)button.setAttribute('aria-current','step');
      button.addEventListener('click',()=>chooseVerse(state.book,stop.chapter,stop.verse),{signal:abort.signal});stops.append(button);}}
    else {const label=document.createElement('span');label.textContent=state.scope==='none'?'No geographic setting assigned':`${state.places.length} identified ${state.places.length===1?'place':'places'} · ${state.scope} context`;stops.append(label);}
  };
  const apply=(next,{scroll=false,force=false}={})=>{
    const changed=state.book!==next.book || state.chapter!==next.chapter || state.setting?.id!==next.setting?.id;
    state=next;page.dataset.atlasBook=String(state.book);page.dataset.atlasChapter=String(state.chapter);page.dataset.atlasVerse=String(state.verse);page.dataset.atlasSetting=state.setting?.id||'';
    title.textContent=state.title;page.querySelector('[data-atlas-reference]').textContent=`${state.book_name} ${state.chapter}`;
    page.querySelector('[data-atlas-current]').textContent=currentLabel();
    const progress=page.querySelector('[data-atlas-progress]');progress.max=state.verse_count;progress.value=state.verse;
    const book=page.querySelector('[data-atlas-book]');book.value=String(state.book);
    const chapters=page.querySelector('[data-atlas-chapter]');
    if(chapters.options.length!==state.chapter_count){chapters.replaceChildren();for(let n=1;n<=state.chapter_count;n++){const option=document.createElement('option');option.value=String(n);option.textContent=`Chapter ${n}`;chapters.append(option);}}
    chapters.value=String(state.chapter);
    page.querySelectorAll('[data-atlas-back]').forEach(link=>link.href=href(state.reader_path));
    page.querySelector('[data-atlas-route-caption]').textContent=state.stops.length?'Schematic journey · exact route unknown':'Approximate place identifications';
    page.querySelector('[data-atlas-explanation]').textContent=state.setting?.note || state.message;
    page.querySelector('[data-atlas-place-name]').textContent=state.setting?.name || 'Places in this passage';
    const currentDetail=page.querySelector('[data-atlas-place-detail]');currentDetail.replaceChildren();if(state.setting){const confidence=document.createElement('p');confidence.className='atlas-confidence';confidence.textContent=`Identification: ${state.setting.confidence}`;currentDetail.append(confidence,sourceLink(state.setting.url,'Location source ↗'));}
    panel.querySelectorAll('.is-map-current').forEach(node=>node.classList.remove('is-map-current'));
    const node=findVerse(state.book,state.chapter,state.verse);node?.classList.add('is-map-current');
    setFollow(following);drawLists();view?.update(state,{follow:following,force,animate:!reduced()});
    if(state.scope==='none' || !state.setting){page.querySelector('[data-atlas-follow-status]').title=state.message;}
    if(!state.setting)page.querySelector('[data-atlas-map-message]').dataset.contextMessage=state.message;
    if(page.dataset.mapState==='ready'){status.hidden=!!state.setting;status.textContent=state.message;}
    if(scroll)scrollToNode(node);
    if(typeof window.rustBibleApi==='function')window.history.replaceState(null,'',`#${mapPath({bookIndex:state.book,chapter:state.chapter,verse:state.verse})}`);
    return changed;
  };
  const updatePosition=async record=>{
    const id=++request;
    try {const next=await api(`/api/atlas?book=${record.bookIndex ?? encodeURIComponent(record.book)}&chapter=${record.chapter}&verse=${record.verse||1}`);
      if(disposed || id!==request)return;
      apply(next,{scroll:record.source==='listening' && !!root.querySelector('[data-audio-follow]')?.checked});
    }catch(error){if(!disposed){status.hidden=false;status.textContent=error.message;}}
  };
  on(root,'bible:position',event=>{syncLinks(event.detail.record);void updatePosition(event.detail.record);});
  on(page,'click',event=>{
    const tab=event.target.closest('[data-atlas-tab]');if(tab){showTab(tab.dataset.atlasTab);return;}
    const action=event.target.closest('[data-atlas-action]')?.dataset.atlasAction;
    if(action==='follow'){setFollow(true);view?.update(state,{follow:true,force:true,animate:!reduced()});}
    if(action==='overview'){setFollow(false);view?.fit();}
    if(action==='zoom-in')view?.zoom(1);
    if(action==='zoom-out')view?.zoom(-1);
    if(action==='north')view?.resetNorth();
    if(action==='terrain'){threeD=!threeD;event.target.closest('button').setAttribute('aria-pressed',String(threeD));event.target.closest('button').textContent=threeD?'3D':'2D';view?.set3D(threeD);}
    if(action==='panel'){const open=page.dataset.panelOpen!=='false';page.dataset.panelOpen=String(!open);event.target.closest('button').setAttribute('aria-expanded',String(!open));}
    if(action==='retry')void mountMap();
    const verse=event.target.closest('.scripture .v');if(verse && panel.contains(verse)){const p=positionForVerse(verse);if(p)chooseVerse(p.bookIndex,p.chapter,p.verse);}
  });
  on(page,'keydown',event=>{
    const tab=event.target.closest('[data-atlas-tab]');
    if(tab && ['ArrowLeft','ArrowRight'].includes(event.key)){event.preventDefault();const name=tab.dataset.atlasTab==='scripture'?'context':'scripture';showTab(name);page.querySelector(`[data-atlas-tab=${name}]`).focus();}
    const verse=event.target.closest('.scripture .v');if(verse && (event.key==='Enter'||event.key===' ')){event.preventDefault();const p=positionForVerse(verse);if(p)chooseVerse(p.bookIndex,p.chapter,p.verse);}
  });
  on(page.querySelector('[data-atlas-book]'),'change',event=>navigate(`/explore?book=${event.target.value}&chapter=1&verse=1`));
  on(page.querySelector('[data-atlas-chapter]'),'change',event=>navigate(`/explore?book=${state.book}&chapter=${event.target.value}&verse=1`));
  on(window,'resize',()=>{autoScrollUntil=Date.now()+1200;clearTimeout(scrollTimer);requestAnimationFrame(()=>{if(disposed)return;scrollToNode(findVerse(state.book,state.chapter,state.verse),false);view?.update(state,{follow:following,force:following,animate:false});});});
  on(panel,'scroll',()=>{clearTimeout(scrollTimer);if(Date.now()<autoScrollUntil || isReading(root))return;scrollTimer=setTimeout(()=>{
    const bounds=panel.getBoundingClientRect();const node=[...panel.querySelectorAll('.scripture .v')].find(node=>{const r=node.getBoundingClientRect();return r.bottom>bounds.top+35 && r.top<bounds.bottom;});if(node)saveProgress(node);
  },180);},{passive:true});
  const mountMap=async()=>{
    view?.destroy();view=null;status.hidden=false;status.textContent='Loading geographic terrain…';retry.hidden=true;
    try {const next=await createTerrainMap(mapContainer,{onInteract:()=>setFollow(false),onPlaceSelect:showPlace,onStatus:result=>{
      if(disposed)return;page.dataset.mapState=result.state;
      status.hidden=result.state==='ready' && !!state.setting;
      status.textContent=result.state==='ready' && !state.setting?state.message:result.message;
      retry.hidden=!['error','degraded'].includes(result.state);
    }});
      if(disposed){next.destroy();return;}view=next;view.set3D(threeD);view.update(state,{follow:following,force:true,animate:false});
    }catch(error){if(!disposed){status.hidden=false;status.textContent='Terrain could not load. Scripture and place references remain available.';retry.hidden=false;page.dataset.mapState='error';}}
  };
  apply(state);requestAnimationFrame(()=>scrollToNode(findVerse(state.book,state.chapter,state.verse),false));document.fonts?.ready.then(()=>{if(!disposed && !isReading(root))scrollToNode(findVerse(state.book,state.chapter,state.verse),false);});void mountMap();
}

if(!document.getElementById('app')){
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',()=>initAtlas(),{once:true});else initAtlas();
}
