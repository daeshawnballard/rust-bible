import { saveProgress, readProgress, saveBookProgress } from './progress.js';
import { LocalNarrator, DEFAULT_NARRATOR } from './local-voice.js';

let dispose = () => {};

// Scripture comes from Rust's KJV renderer. The speech layer never reads
// explanatory notes, navigation, or markup as though they were Scripture.
export function initExperience(root = document) {
  dispose();
  const listeners = [];
  const on = (target, event, handler) => {
    target?.addEventListener(event, handler);
    if (target) listeners.push(() => target.removeEventListener(event, handler));
  };
  const controls = root.querySelector('[data-read-aloud]');
  let stop = () => {};
  if (controls) {
    const compact = controls.hasAttribute('data-audio-compact');
    const rawCompactScope = controls.dataset.audioScope?.trim();
    const compactScope = rawCompactScope
      ? (/^(?:this|the)\s/i.test(rawCompactScope) ? rawCompactScope : `this ${rawCompactScope}`)
      : 'Scripture';
    const bookMode = controls.hasAttribute('data-book-audio');
    const readerMode = controls.hasAttribute('data-reader-audio');
    const follow = controls.querySelector('[data-audio-follow]');
    const bookName = controls.dataset.reference || 'this book';
    const synth = new LocalNarrator({onProgress: progress => {
      if(controls.dataset.audioState !== 'starting' || !current)return;
      const total=Number(progress.total)||0,loaded=Number(progress.loaded)||0;
      status.textContent=total>0 ? `Downloading natural voice · ${Math.round(loaded/total*100)}% (${Math.ceil(loaded/1048576)} / ${Math.ceil(total/1048576)} MB)` : progress.message || 'Preparing the free natural voice…';
    }});
    const play = controls.querySelector('[data-audio-action="play"]');
    const pause = controls.querySelector('[data-audio-action="pause"]');
    const stopButton = controls.querySelector('[data-audio-action="stop"]');
    const rate = controls.querySelector('[data-audio-rate]');
    const voice = controls.querySelector('[data-audio-voice]');
    const status = controls.querySelector('[data-audio-status]');
    const audioLabel = play?.querySelector('[data-audio-label]');
    const progressNotice = controls.querySelector('[data-progress-notice]');
    const updateProgressNotice = persisted => {
      if (progressNotice) progressNotice.textContent = persisted
        ? 'Verse saved on this device. Clear it on Today.'
        : 'Browser storage is unavailable. Your verse cannot be remembered after you leave.';
    };
    if (readerMode) updateProgressNotice(root.querySelector('[data-reader]')?.dataset.positionSaved === 'true');
    const playbackSettings = controls.querySelector('.book-playback-settings');
    const primaryScripture = root.querySelector('[data-reader-primary-scripture]');
    const collectNodes = () => [...(readerMode ? root.querySelector('[data-reader]') : primaryScripture || root).querySelectorAll(readerMode || !primaryScripture ? '.scripture .v' : '.v')];
    let nodes = collectNodes();
    let scripture;
    if (bookMode) {
      try {
        const payload = controls.querySelector('[data-book-scripture]')?.textContent || '[]';
        const parsed = JSON.parse(payload);
        scripture = Array.isArray(parsed) ? parsed
          .filter(item => item && typeof item.text === 'string' && item.text.trim())
          .map(item => ({text: item.text.trim(), chapter: item.chapter, verse: item.verse})) : [];
      } catch {
        scripture = [];
      }
    } else {
      scripture = nodes.map(node => {
        const clone = node.cloneNode(true);
        clone.querySelectorAll('sup').forEach(number => number.remove());
        return {text: clone.textContent.trim(), reference: node.dataset.reference};
      });
    }
    let verses = scripture.map(item => item.text);
    let generation = 0, current = null, timer = null, index = 0;
    const verseNumber = verseIndex => bookMode
      ? scripture[verseIndex]?.verse ?? verseIndex + 1
      : nodes[verseIndex].dataset.verse || nodes[verseIndex].querySelector('sup')?.textContent || verseIndex + 1;
    const location = verseIndex => bookMode
      ? `${bookName} ${scripture[verseIndex]?.chapter ?? '?'}:${verseNumber(verseIndex)}`
      : readerMode ? `${scripture[verseIndex]?.reference}:${verseNumber(verseIndex)}` : `verse ${verseNumber(verseIndex)}`;
    const readingMessage = verseIndex => (bookMode || readerMode)
      ? `Reading ${location(verseIndex)}${readerMode ? '' : ` · ${verseIndex + 1} of ${verses.length}.`}`
      : `Reading verse ${verseNumber(verseIndex)} · ${verseIndex + 1} of ${verses.length}.`;
    const followVerse = verseIndex => {
      if (!readerMode || !follow?.checked || root.querySelector('[data-atlas-page]')) return;
      const node = nodes[verseIndex];
      const rect = node?.getBoundingClientRect();
      const toolbar = root.querySelector('.reader-toolbar')?.getBoundingClientRect();
      if (rect && (rect.top < Math.max(20, toolbar?.bottom || 0) + 12 || rect.bottom > window.innerHeight - 24)) {
        node.scrollIntoView({block: 'center', behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth'});
      }
    };
    const extendReading = async (run, quiet = false) => {
      const marker = root.querySelector('[data-reader-next]');
      if (!readerMode || !marker) return false;
      const uri = marker.dataset.readerNext;
      if(!quiet)setState('starting', 'Continuing into the next chapter…');
      const result = typeof window.rustBibleApi === 'function'
        ? window.rustBibleApi(uri)
        : await fetch(uri, {cache: 'no-store'}).then(response => {
          if (!response.ok) throw new Error('Chapter could not load.');
          return response.json();
        });
      if (run !== generation || !marker.isConnected) return false;
      const template = document.createElement('template');
      template.innerHTML = result.html;
      const previousCount = nodes.length;
      marker.replaceWith(template.content);
      nodes = collectNodes();
      scripture = nodes.map(node => ({text: node.querySelector('.verse-text').textContent.trim(), reference: node.dataset.reference}));
      verses = scripture.map(item => item.text);
      return nodes.length > previousCount;
    };
    const revealPlaybackSettings = () => {
      if (bookMode && playbackSettings) playbackSettings.hidden = false;
    };
    const setState = (state, message) => {
      controls.dataset.audioState = state;
      status.textContent = message;
      if (pause) pause.disabled = !['reading', 'paused'].includes(state);
      if (stopButton) stopButton.disabled = !['reading', 'paused', 'starting'].includes(state);
      if (bookMode) {
        if (audioLabel) audioLabel.textContent = state === 'reading' ? `Pause ${bookName}` : state === 'paused' ? `Resume ${bookName}` : `Listen to ${bookName}`;
        play.setAttribute('aria-label', state === 'reading' ? `Pause ${bookName}` : state === 'paused' ? `Resume ${bookName}` : `Listen to ${bookName}`);
        if(state==='starting'){if(audioLabel)audioLabel.textContent='Cancel loading';play.setAttribute('aria-label','Cancel loading');}
        play.disabled = state === 'unavailable';
        if (['error', 'unavailable'].includes(state)) revealPlaybackSettings();
      } else if (compact) {
        const label = state==='starting' ? 'Cancel loading' : readerMode ? (state === 'reading' ? 'Pause reading' : state === 'paused' ? 'Resume reading' : 'Listen from here')
          : state === 'reading' ? `Pause ${compactScope}` : state === 'paused' ? `Resume ${compactScope}` : `Listen to ${compactScope}`;
        play.setAttribute('aria-label', label);
        if(state==='starting'){if(audioLabel)audioLabel.textContent='Cancel loading';play.setAttribute('aria-label','Cancel loading');}
        play.disabled = state === 'unavailable';
        if (audioLabel) audioLabel.textContent = label;
        controls.querySelector('.player-title').textContent = (controls.dataset.audioScope || state==='starting') ? label : state === 'paused' ? `${compactScope[0].toUpperCase()}${compactScope.slice(1)} paused` : state === 'reading' ? `Reading ${compactScope}` : `Listen to ${compactScope}`;
      } else {
        play.textContent = state === 'paused' ? 'Resume' : 'Listen to Scripture';
        pause.textContent = state === 'paused' ? 'Paused' : 'Pause';
      }
    };
    const selectedBookIndex = () => {
      const saved = readProgress();
      if (saved?.bookIndex !== Number(controls.dataset.bookIndex)) return 0;
      const selected = scripture.findIndex(item => item.chapter === saved.chapter && item.verse === (saved.verse || 1));
      return selected < 0 ? 0 : selected;
    };
    const selectedReaderIndex = () => {
      const id = root.querySelector('[data-reader]')?.dataset.readerPosition;
      const selected = nodes.findIndex(node => node.id === id);
      return selected < 0 ? 0 : selected;
    };
    const voiceNotice = () => voice.value.startsWith('kokoro:') ? ' · Natural voice · first download about 100 MB.' : ' · Device voice.';
    const readyMessage = () => (readerMode && nodes.length
      ? `Ready at ${location(selectedReaderIndex())}` : 'Ready. Scripture only.') + voiceNotice();
    on(root, 'bible:position', event => {
      if (!readerMode) return;
      if (!['starting', 'reading', 'paused'].includes(controls.dataset.audioState)) {
        setState('idle', `Ready at ${event.detail.record.book} ${event.detail.record.chapter}:${event.detail.record.verse}${voiceNotice()}`);
      }
      controls.dataset.positionSaved = String(event.detail.persisted);
      updateProgressNotice(event.detail.persisted);
    });
    const clearHighlight = () => nodes.forEach(node => node.classList.remove('is-speaking'));
    stop = (message = readerMode ? readyMessage() : 'Stopped.', explicitStart = false) => {
      const ownsSpeech = current !== null;
      generation++;
      clearTimeout(timer);
      current = null;
      // Device speech services can share a queue across tabs. Disposing an
      // idle book/reader must not cancel an utterance owned by another page.
      if (ownsSpeech || explicitStart) {
        synth?.cancel();
        // cancel() can leave the device queue paused. Release that state so a
        // later Listen click can start a new utterance after Pause -> Stop.
        if (synth?.paused) synth.resume();
      }
      clearHighlight();
      setState('idle', message);
    };
    if (!synth.supported) {
      [play, pause, stopButton, rate, voice].forEach(control => { if (control) control.disabled = true; });
      setState('unavailable', 'Read-aloud is unavailable in this browser. Try a browser with audio playback support.');
    } else {
      let populatedVoices = false;
      const populateVoices = () => {
        let stored = '';
        try { stored = localStorage.getItem('bible.narrator') || ''; } catch {}
        const selected = populatedVoices ? voice.value : stored || DEFAULT_NARRATOR;
        populatedVoices = true;
        const options = synth.getVoices().filter(item => /^en\b/i.test(item.lang));
        voice.replaceChildren();
        for (const item of options) voice.add(new Option(`${item.name} (${item.lang})`, item.voiceURI));
        voice.add(new Option('Device default · no download','device:default'));
        voice.value = [...voice.options].some(item => item.value === selected) ? selected : DEFAULT_NARRATOR;
      };
      try { rate.value = localStorage.getItem('bible.rate') || '1'; } catch {}
      populateVoices();
      on(synth, 'voiceschanged', populateVoices);
      setState('idle', readyMessage());
      const readVerse = async (run, verseIndex, chunkIndex = 0, chunks = null) => {
        if (run !== generation) return;
        if (verseIndex >= verses.length) {
          try {
            if (await extendReading(run)) { readVerse(run, verseIndex); return; }
          } catch {
            if (run === generation) { stop('The next chapter could not load. Press Listen to try again.'); controls.dataset.audioState = 'error'; }
            return;
          }
          if (run !== generation) return;
          current = null;
          clearHighlight();
          setState('finished', bookMode ? `Finished reading ${bookName}.` : readerMode ? 'Finished reading the final chapter.' : 'Finished reading this passage.');
          return;
        }
        index = verseIndex;
        // Keep utterances short for engines that stall on long text.
        chunks ||= verses[verseIndex].match(/.{1,220}(?:\s|$)|\S{1,220}/g) || [verses[verseIndex]];
        if (chunkIndex >= chunks.length) { readVerse(run, verseIndex + 1); return; }
        const utterance = {text:chunks[chunkIndex].trim()};
        current = utterance;
        utterance.lang = 'en-US';
        utterance.rate = Number(rate.value) || 1;
        utterance.voice = synth.getVoices().find(item => item.voiceURI === voice.value) || null;
        utterance.onstart = () => {
          if (run !== generation) return;
          clearTimeout(timer);
          clearHighlight();
          nodes[verseIndex]?.classList.add('is-speaking');
          if (readerMode) saveProgress(nodes[verseIndex], 'listening');
          if (bookMode) saveBookProgress(controls, scripture[verseIndex].chapter, scripture[verseIndex].verse);
          followVerse(verseIndex);
          const number = verseNumber(verseIndex);
          setState('reading', readingMessage(verseIndex));
          console.info('Bible read-aloud: speech started', {verse: bookMode ? location(verseIndex) : number, voice: utterance.voice?.name || 'device default'});
          const prefetchNext = () => {
            const nextChunk=chunks[chunkIndex+1] || verses[verseIndex+1]?.match(/.{1,220}(?:\s|$)|\S{1,220}/g)?.[0];
            synth.prefetch(nextChunk?.trim(),utterance.voice);
          };
          prefetchNext();
          if(readerMode && synth.isLocalVoice(utterance.voice) && verseIndex===verses.length-1 && !chunks[chunkIndex+1]) {
            extendReading(run,true).then(()=>{if(run===generation)prefetchNext();}).catch(()=>{});
          }
        };
        utterance.onend = () => {
          if (run === generation) readVerse(run, verseIndex, chunkIndex + 1, chunks);
        };
        utterance.onpause = () => {
          if (run === generation) setState('paused', bookMode
            ? `Paused at ${location(verseIndex)}. Press Resume to continue.`
            : 'Paused. Press Resume to continue.');
        };
        utterance.onresume = () => {
          if (run === generation) { setState('reading', readingMessage(index)); followVerse(index); }
        };
        utterance.onerror = event => {
          if (run !== generation) return;
          stop(`Speech could not play${bookMode ? ` at ${location(verseIndex)}` : ''}: ${event.error}. Choose Device default in Playback options, or try Listen again.`);
          controls.dataset.audioState = 'error';
          revealPlaybackSettings();
        };
        if(!synth.isLocalVoice(utterance.voice))timer = setTimeout(() => {
          if (run === generation && controls.dataset.audioState === 'starting') {
            stop(`The device speech service did not start${bookMode ? ` for ${location(index)}` : ''}. Try another voice or browser.`);
            controls.dataset.audioState = 'error';
            revealPlaybackSettings();
          }
        }, 7000);
        if(synth.isLocalVoice(utterance.voice))setState('starting', `Preparing natural voice · ${location(verseIndex)}…`);
        synth.speak(utterance);
      };
      on(play, 'click', () => {
        revealPlaybackSettings();
        if(controls.dataset.audioState==='starting'){stop('Loading canceled. Press Listen to try again.');return;}
        if ((compact || bookMode) && controls.dataset.audioState === 'reading') {
          synth.pause();
          if (synth.paused) setState('paused', bookMode
            ? `Paused at ${location(index)}. Press Resume to continue.`
            : 'Paused. Press Resume to continue.');
          return;
        }
        if (controls.dataset.audioState === 'paused' && current) {
          synth.resume();
          if (!synth.paused) { setState('reading', readingMessage(index)); followVerse(index); }
          return;
        }
        stop('Preparing the selected voice…', true);
        synth.unlock();
        if (!verses.length) { setState('error', bookMode ? `There is no Scripture text available for ${bookName}.` : 'There is no Scripture passage to read here.'); return; }
        setState('starting', bookMode ? `Starting ${bookName}…` : 'Preparing the selected voice…');
        readVerse(generation, readerMode ? selectedReaderIndex() : bookMode ? selectedBookIndex() : 0);
      });
      on(pause, 'click', () => {
        synth.pause();
        if (synth.paused) setState('paused', bookMode
          ? `Paused at ${location(index)}. Press Resume to continue.`
          : 'Paused. Press Resume to continue.');
      });
      on(stopButton, 'click', () => stop());
      on(follow, 'change', () => { if (controls.dataset.audioState === 'reading') followVerse(index); });
      on(rate, 'change', () => {
        try { localStorage.setItem('bible.rate', rate.value); } catch {}
        stop(readerMode ? readyMessage() : bookMode ? `Speed changed. Press Listen to restart ${bookName}.` : 'Speed changed. Press Listen to restart this passage.');
      });
      on(voice, 'change', () => {
        try { localStorage.setItem('bible.narrator', voice.value); } catch {}
        stop(readerMode ? readyMessage() : bookMode ? `Voice changed. Press Listen to restart ${bookName}.` : 'Voice changed. Press Listen to restart this passage.');
      });
      on(window, 'pagehide', () => stop());
    }
    listeners.push(()=>synth.destroy());
  }
  dispose = () => { stop(); listeners.forEach(remove => remove()); };
}

// Native pages load this module directly; WASM pages call init after each render.
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => initExperience(), {once:true});
else initExperience();
