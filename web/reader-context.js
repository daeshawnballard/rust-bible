import { readProgress, saveProgress, positionForVerse } from './progress.js';
import { createTerrainMap } from './terrain.js';

const mounted = new WeakMap();
const DISPLAY_KEY = 'rust-bible:reader-display:v1';
const routeURL = () => new URL(location.hash.startsWith('#/') ? location.hash.slice(1) : location.pathname + location.search + location.hash, location.origin);
const request = async uri => typeof window.rustBibleApi === 'function' ? window.rustBibleApi(uri) : fetch(uri,{cache:'no-store'}).then(async r=>{const value=await r.json();if(!r.ok)throw new Error(value.error);return value;});
export function resolveLegacyReaderRoute(route) {
  const u=new URL(route,location.origin);
  if(['/explore','/explore/studies'].includes(u.pathname)&&!u.searchParams.has('book')&&!u.searchParams.has('scene')){
    const saved=readProgress();return (saved?.route||'/book/Genesis/1')+'?resume='+(saved?.verse||1)+'&context=journey#v'+(saved?.verse||1);
  }
  return route;
}
export function disposeReaderContext(root=document){mounted.get(root)?.();mounted.delete(root);}
export function initReaderContext(root=document) {
  disposeReaderContext(root);
  const reader=root.querySelector('.reader-page[data-reader]');
  const panel=reader?.querySelector('[data-reader-context]');
  if(!reader||!panel)return;
  const abort=new AbortController(), signal=abort.signal;
  const on=(target,event,fn,options={})=>target?.addEventListener(event,fn,{...options,signal});
  const content=reader.querySelector('.reader-content');
  const heading=reader.querySelector('.reader-heading');
  const toolbar=reader.querySelector('.reader-toolbar');
  const body=panel.querySelector('[data-context-body]');
  const status=panel.querySelector('[data-context-status]');
  let active=true, generation=0, terrain=null, lastArt='', value=null, inspected=null;
  let mode=['journey','illustration'].includes(routeURL().searchParams.get('context'))?routeURL().searchParams.get('context'):'passage';
  let tab=routeURL().searchParams.get('entry')?'codex':'scripture';
  let part=routeURL().searchParams.get('part'), entry=routeURL().searchParams.get('entry'), scene=routeURL().searchParams.get('scene');
  const start=reader.querySelector('#'+CSS.escape(reader.dataset.readerPosition||'v1'));
  let position=positionForVerse(start)||{book:reader.dataset.bookName,bookIndex:Number(reader.dataset.bookIndex),chapter:Number(reader.dataset.chapter),verse:1,route:reader.dataset.readerRoute};
  const workspace=reader.querySelector('.reader-workspace');
  const frame=document.createElement('section');frame.className='passage-window';frame.setAttribute('aria-label','Passage window');
  workspace.before(frame);frame.append(heading,toolbar,content,panel);workspace.remove();
  const canvas=document.createElement('div');canvas.className='reader-canvas';canvas.hidden=true;canvas.setAttribute('aria-label','Illustrated passage journey');
  const world=document.createElement('div');world.className='journey-world';
  const caption=document.createElement('p');caption.className='journey-caption';
  canvas.append(world,caption);reader.prepend(canvas);
  const selectors=toolbar.querySelector('.reader-selectors');
  selectors.append(toolbar.querySelector('.reader-display'));
  const locationMenu=document.createElement('details');locationMenu.className='reader-location';
  const locationSummary=document.createElement('summary');locationSummary.setAttribute('aria-label','Choose book and chapter');
  locationSummary.append(heading.querySelector('h1'));
  locationMenu.append(locationSummary,selectors);heading.prepend(locationMenu);
  const modes=toolbar.querySelector('.reader-modes');
  heading.append(modes);
  const codexButton=document.createElement('button');codexButton.type='button';codexButton.dataset.windowTab='codex';codexButton.className='reader-codex-button';codexButton.textContent='Codex';heading.append(codexButton);
  const tabs=document.createElement('nav');tabs.className='passage-tabs';tabs.setAttribute('aria-label','Passage window content');
  for(const key of ['scripture','context','codex']){
    const b=document.createElement('button');b.type='button';b.dataset.windowTab=key;b.textContent=key==='scripture'?'Scripture':key==='context'?'Context':'Codex';tabs.append(b);
  }
  toolbar.after(tabs);
  const footer=document.createElement('footer');footer.className='passage-window-footer';
  const follow=document.createElement('button');follow.type='button';follow.dataset.windowFollow='';follow.textContent='Follow passage';
  const expand=document.createElement('button');expand.type='button';expand.dataset.readerMode='passage';expand.textContent='Read';expand.title='Expand Scripture';
  expand.setAttribute('aria-label','Expand Scripture into Read view');
  footer.append(follow,expand);frame.append(footer);
  const viewStatus=document.createElement('p');viewStatus.className='sr-only';viewStatus.setAttribute('role','status');
  frame.append(viewStatus);
  panel.querySelector('.reader-context-heading').hidden=true;
  status.classList.add('sr-only');
  const show=()=>{
    reader.dataset.contextMode=mode;
    const view=mode==='passage'?'Read':'Journey';
    reader.setAttribute('aria-label','Bible passage reader — '+view+' view');
    if(viewStatus.textContent!==view+' view')viewStatus.textContent=view+' view';
    reader.dataset.windowTab=tab;
    document.body.classList.toggle('has-scene',mode!=='passage');
    document.body.classList.add('has-reader');
    canvas.hidden=mode==='passage';
    content.hidden=tab!=='scripture';
    panel.hidden=tab==='scripture';
    tabs.querySelectorAll('button').forEach(b=>{b.setAttribute('aria-pressed',String(b.dataset.windowTab===tab));if(b.dataset.windowTab==='context')b.hidden=mode==='passage';});
    modes.querySelectorAll('[data-reader-mode]').forEach(b=>{
      const target=b.dataset.readerMode;
      b.removeAttribute('aria-pressed');
      const hasContext=value&&(value.journey_available||value.relevant_scene||value.geographic_available||value.kind==='geography');
      b.hidden=target==='passage'||target==='journey'&&(mode!=='passage'||!hasContext)||target==='illustration';
      if(target==='journey'){b.textContent='Journey';b.setAttribute('aria-label','Open Journey view');}
    });
    expand.hidden=mode==='passage';
    follow.hidden=mode==='passage';
    follow.textContent=inspected?'Follow passage':'Following passage';
    follow.setAttribute('aria-pressed',String(!inspected));
    const u=routeURL();for(const key of ['context','part','scene','entry'])u.searchParams.delete(key);
    if(mode!=='passage')u.searchParams.set('context',mode);
    if(mode==='illustration'&&part)u.searchParams.set('part',part);
    if(mode==='illustration'&&scene)u.searchParams.set('scene',scene);
    if(tab==='codex'&&entry)u.searchParams.set('entry',entry);
    const path=u.pathname+u.search+u.hash;
    history.replaceState(history.state,'',location.hash.startsWith('#/')?'#'+path:path);
  };
  const applyDisplay=value=>{
    reader.dataset.textSize=['standard','large','larger'].includes(value?.size)?value.size:'standard';
    reader.dataset.lineSpacing=value?.spacing==='spacious'?'spacious':'comfortable';
    reader.querySelector('[data-reader-size]').value=reader.dataset.textSize;
    reader.querySelector('[data-reader-spacing]').value=reader.dataset.lineSpacing;
  };
  try{applyDisplay(JSON.parse(localStorage.getItem(DISPLAY_KEY)));}catch{applyDisplay(null);}
  const selectVerse=(record,scroll=false)=>{
    const found=[...reader.querySelectorAll('.scripture .v')].find(n=>{const p=positionForVerse(n);return p?.route===record.route&&p.verse===record.verse;});
    if(!found)return;
    reader.querySelectorAll('.is-context-current').forEach(n=>n.classList.toggle('is-context-current',n===found));
    found.classList.add('is-context-current');
    if(scroll&&tab==='scripture')found.scrollIntoView({block:'center',behavior:matchMedia('(prefers-reduced-motion:reduce)').matches?'instant':'smooth'});
  };
  let pan=0,worldWidth=0,worldHeight=0,manualPan=false;
  const sizeScene=()=>{
    const figure=world.querySelector('figure'), img=figure?.querySelector('img');
    if(!figure||!img)return;
    const w=canvas.clientWidth,h=canvas.clientHeight;
    const scale=Math.max(w/(img.naturalWidth||1536),h/(img.naturalHeight||1024));
    worldWidth=(img.naturalWidth||1536)*scale;worldHeight=(img.naturalHeight||1024)*scale;
    world.style.width=worldWidth+'px';world.style.height=worldHeight+'px';
    world.style.top=(h-worldHeight)/2+'px';
    const selected=figure.querySelector('[aria-current],[aria-pressed="true"]');
    if(!manualPan){
      const x=selected?parseFloat(selected.style.left)/100:.3;
      const desktop=innerWidth>=900;
      const target=w*(desktop ? 0.4 : 0.45);
      const initial=(w-worldWidth)/2;
      const scenePoint=initial+x*worldWidth;
      pan=scenePoint>w*(desktop ? 0.65 : 0.78)||scenePoint<44?target-x*worldWidth:initial;
      pan=Math.max(w-worldWidth-(desktop?w*.28:0),Math.min(0,pan));
    }
    world.style.left=pan+'px';
  };
  let dragging=null,pointerFocus=false;
  on(world,'pointerdown',()=>{pointerFocus=true;},{capture:true});
  on(window,'keydown',()=>{pointerFocus=false;},{capture:true});
  on(canvas,'pointerdown',e=>{if(e.target.closest('a,button'))return;dragging={x:e.clientX,pan};canvas.setPointerCapture(e.pointerId);});
  on(canvas,'pointermove',e=>{if(!dragging)return;manualPan=true;pan=Math.max(canvas.clientWidth-worldWidth-(innerWidth>=900?innerWidth*.28:0),Math.min(0,dragging.pan+e.clientX-dragging.x));world.style.left=pan+'px';});
  on(world,'focusin',e=>{
    const pin=e.target.closest('a,button');if(!pin||!worldWidth||pointerFocus)return;
    const r=pin.getBoundingClientRect(),right=innerWidth>=900?frame.getBoundingClientRect().left-20:canvas.clientWidth-12;
    if(r.left<12||r.right>right){manualPan=true;pan+=right*.5-(r.left+r.width*.5);pan=Math.max(canvas.clientWidth-worldWidth-(innerWidth>=900?innerWidth*.28:0),Math.min(0,pan));world.style.left=pan+'px';}
  });
  on(canvas,'pointerup',()=>{dragging=null;});
  on(canvas,'pointercancel',()=>{dragging=null;});
  on(window,'resize',sizeScene);
  const stopMap=()=>{terrain?.destroy();terrain=null;};
  async function loadCodex(){
    const p=new URLSearchParams({book:position.bookIndex,chapter:position.chapter,verse:position.verse});if(entry)p.set('entry',entry);
    const result=await request('/api/codex?'+p);
    if(active&&tab==='codex'){body.innerHTML=result.html;show();}
  }
  async function refresh({scroll=false}={}){
    show();selectVerse(position,scroll);
    const run=++generation;
    if(tab==='codex'){try{await loadCodex();}catch{body.textContent='The Codex could not load. Try again.';}}
    const reference=inspected||position;
    const kind=mode==='passage'?'availability':mode;
    const params=new URLSearchParams({book:reference.bookIndex,chapter:reference.chapter,verse:reference.verse,mode:kind});
    if(part)params.set('part',part);
    try{
      const result=await request('/api/reader-context?'+params);
      if(!active||run!==generation)return;
      value=result;
      const available=result.journey_available||result.relevant_scene||result.geographic_available||result.kind==='geography';
      const journeyButton=modes.querySelector('[data-reader-mode="journey"]');
      journeyButton.hidden=mode!=='passage'||!available;
      if(mode==='passage')return;
      scene=result.scene||scene;
      if(result.kind==='empty'){
        stopMap();world.replaceChildren();lastArt='';mode='passage';inspected=null;
        if(tab==='context')tab='scripture';show();selectVerse(position,true);return;
      }
      const template=document.createElement('template');template.innerHTML=result.html||'';
      const figure=template.content.querySelector('.story-journey-art,.reader-illustration-scene');
      const sourceCaption=template.content.querySelector('.story-journey-caption');
      if(figure){
        const artKey=figure.outerHTML;
        if(lastArt!==artKey){
          stopMap();world.replaceChildren(figure);lastArt=artKey;manualPan=false;
          const img=figure.querySelector('img');
          img.loading='eager';img.draggable=false;img.addEventListener('load',sizeScene,{once:true,signal});
          img.addEventListener('error',()=>{caption.textContent='Illustration could not load. Scripture remains available.';},{once:true,signal});
          sizeScene();
        }
        caption.textContent=sourceCaption?.textContent||'Illustrated reconstruction · appearance and placement are illustrative';
      }else if(result.kind==='geography'){
        const map=template.content.querySelector('[data-context-map]');
        if(lastArt!=='geo:'+reference.bookIndex+':'+reference.chapter){
          stopMap();world.replaceChildren(map);world.style.cssText='';map.classList.add('immersive-terrain');
          lastArt='geo:'+reference.bookIndex+':'+reference.chapter;
          terrain=await createTerrainMap(map,{onInteract(){manualPan=true;},onPlaceSelect(place){if(tab==='context'){body.querySelector('h3')?.replaceChildren(document.createTextNode(place.name));}},onStatus(e){caption.textContent=e.message||'Modern terrain · approximate locations';}});
          if(!active){terrain.destroy();return;}terrain.set3D(true);
        }
        terrain?.update(result.map,{follow:!inspected&&!manualPan,animate:true,force:true});
        caption.textContent='Modern terrain · approximate locations and schematic journey';
      }else{
        stopMap();world.replaceChildren();lastArt='';canvas.classList.add('is-unassigned');
      }
      if(tab==='context'){
        const detail=template.content.querySelector('[data-context-detail]');
        body.replaceChildren();
        if(detail)body.append(detail);
        else{
          const copy=template.content.querySelector('.reader-context-copy,.story-journey-current,.reader-context-empty');
          if(copy)body.append(copy);
          const explanatory=template.content.querySelector('.story-journey-details');
          if(explanatory){explanatory.open=true;body.append(explanatory);}
          const stops=template.content.querySelector('.reader-geo-stops');if(stops)body.append(stops);
          const sources=template.content.querySelector('.reader-context-sources');if(sources)body.append(sources);
        }
        if(inspected){
          const jump=document.createElement('a');jump.className='reader-context-read';jump.dataset.contextRead='';
          jump.href=(typeof window.rustBibleApi==='function'?'#':'')+inspected.route+'?resume='+inspected.verse+'&context='+mode+'#v'+inspected.verse;
          jump.dataset.contextBook=inspected.book; jump.dataset.contextChapter=inspected.chapter;jump.dataset.contextVerse=inspected.verse;
          jump.textContent='Read '+inspected.book+' '+inspected.chapter+':'+inspected.verse+' →';body.append(jump);
        }
      }
      status.textContent=inspected?'Browsing context. Your reading position has not moved.':'Following '+position.book+' '+position.chapter+':'+position.verse;
      show();
    }catch(error){if(active&&run===generation){status.textContent='Context could not load. Scripture and audio remain available.';if(tab==='context')body.textContent=status.textContent;}}
  }
  const inspectLink=target=>{
    const ch=Number(target.dataset.contextChapter),v=Number(target.dataset.contextVerse);
    inspected={book:target.dataset.contextBook,bookIndex:position.bookIndex,chapter:ch,verse:v,route:'/book/'+target.dataset.contextBook.replaceAll(' ','-')+'/'+ch};
    part=null;tab='context';refresh();
  };
  on(reader,'click',event=>{
    const target=event.target.closest('[data-reader-mode],button[data-window-tab],[data-window-follow],[data-context-part],[data-context-read],[data-codex-entry],[data-codex-read]');
    if(!target)return;
    if(target.hasAttribute('data-codex-read')){
      window.biblePlayback?.stop();return;
    }
    if(target.hasAttribute('data-context-read')){
      if(canvas.contains(target)){event.preventDefault();event.stopImmediatePropagation();inspectLink(target);return;}
      const chapter=Number(target.dataset.contextChapter),verse=Number(target.dataset.contextVerse);
      const record={book:target.dataset.contextBook,chapter,verse,route:'/book/'+target.dataset.contextBook.replaceAll(' ','-')+'/'+chapter};
      const found=[...reader.querySelectorAll('.scripture .v')].find(n=>{const p=positionForVerse(n);return p?.route===record.route&&p.verse===verse;});
      window.biblePlayback?.stop();
      if(found){event.preventDefault();event.stopImmediatePropagation();inspected=null;tab='scripture';saveProgress(found);position=positionForVerse(found);refresh({scroll:true});}return;
    }
    event.preventDefault();event.stopImmediatePropagation();
    if(target.dataset.readerMode){
      mode=target.dataset.readerMode;
      if(mode==='journey'&&!value?.journey_available&&value?.relevant_scene)mode='illustration';
      tab='scripture';inspected=null;part=null;manualPan=false;refresh({scroll:true});return;
    }
    if(target.dataset.windowTab){tab=target.dataset.windowTab;if(tab==='scripture'){inspected=null;part=null;}refresh({scroll:tab==='scripture'});return;}
    if(target.hasAttribute('data-window-follow')){inspected=null;part=null;entry=null;manualPan=false;tab='scripture';refresh({scroll:true});return;}
    if(target.dataset.contextPart){part=target.dataset.contextPart;tab='context';inspected={...position};refresh();return;}
    if(target.dataset.codexEntry){entry=target.dataset.codexEntry;tab='codex';loadCodex();show();}
  },{capture:true});
  on(reader,'change',event=>{
    if(!event.target.matches('[data-reader-size],[data-reader-spacing]'))return;
    const settings={size:reader.querySelector('[data-reader-size]').value,spacing:reader.querySelector('[data-reader-spacing]').value};
    applyDisplay(settings);try{localStorage.setItem(DISPLAY_KEY,JSON.stringify(settings));}catch{}
  });
  on(reader,'bible:position',event=>{
    const p=event.detail?.record;if(!p)return;
    position=p;
    const u=routeURL();
    if(p.source!=='listening'&&p.route!==reader.dataset.readerRoute&&document.getElementById('app')){
      u.pathname=p.route;u.searchParams.set('resume',p.verse);u.hash='v'+p.verse;
      location.hash='#'+u.pathname+u.search+u.hash;return;
    }
    if(p.source!=='listening'||p.route===reader.dataset.readerRoute){
      u.pathname=p.route;u.searchParams.set('resume',p.verse);u.hash='v'+p.verse;
      history.replaceState(history.state,'',location.hash.startsWith('#/')?'#'+u.pathname+u.search+u.hash:u.pathname+u.search+u.hash);
    }
    locationSummary.querySelector('h1').textContent=p.book+' '+p.chapter;
    const chapters=selectors.querySelector('[data-reader-chapter]');
    const base=p.route.substring(0,p.route.lastIndexOf('/')+1);
    const chapterOption=[...chapters.options].find(o=>o.value.replace(/^#/,'')===p.route);
    if(chapterOption)chapters.value=chapterOption.value;
    selectVerse(p);if(!inspected)refresh();
  });
  // The inset uses the same Scripture nodes. Scrolling it is real reading input,
  // and the shared journey tracker records the visible verse.
  on(content,'scroll',()=>reader.dispatchEvent(new CustomEvent('bible:panel-scroll',{bubbles:true})),{passive:true});
  const cleanup=()=>{active=false;generation++;abort.abort();stopMap();document.body.classList.remove('has-scene','has-reader');};
  mounted.set(root,cleanup);refresh({scroll:mode!=='passage'});
  document.fonts.ready.then(()=>{if(active&&mode!=='passage'&&tab==='scripture'&&!inspected){selectVerse(position,true);sizeScene();}});
}
if(!document.getElementById('app')){
  const run=()=>initReaderContext(document);
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',run,{once:true});else run();
}
