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
    const bookMode = controls.hasAttribute('data-book-audio');
    const bookName = controls.dataset.reference || 'this book';
    const synth = window.speechSynthesis;
    const play = controls.querySelector('[data-audio-action="play"]');
    const pause = controls.querySelector('[data-audio-action="pause"]');
    const stopButton = controls.querySelector('[data-audio-action="stop"]');
    const rate = controls.querySelector('[data-audio-rate]');
    const voice = controls.querySelector('[data-audio-voice]');
    const status = controls.querySelector('[data-audio-status]');
    const audioLabel = play?.querySelector('[data-audio-label]');
    const playbackSettings = controls.querySelector('.book-playback-settings');
    const nodes = [...root.querySelectorAll('.scripture .v')];
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
        return {text: clone.textContent.trim()};
      });
    }
    const verses = scripture.map(item => item.text);
    let generation = 0, current = null, timer = null, index = 0;
    const verseNumber = verseIndex => bookMode
      ? scripture[verseIndex]?.verse ?? verseIndex + 1
      : nodes[verseIndex].dataset.verse || nodes[verseIndex].querySelector('sup')?.textContent || verseIndex + 1;
    const location = verseIndex => bookMode
      ? `${bookName} ${scripture[verseIndex]?.chapter ?? '?'}:${verseNumber(verseIndex)}`
      : `verse ${verseNumber(verseIndex)}`;
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
        play.disabled = state === 'starting' || state === 'unavailable';
        if (['error', 'unavailable'].includes(state)) revealPlaybackSettings();
      } else if (compact) {
        play.setAttribute('aria-label', state === 'reading' ? 'Pause Scripture' : state === 'paused' ? 'Resume Scripture' : 'Listen to Scripture');
        play.disabled = state === 'starting' || state === 'unavailable';
        controls.querySelector('.player-title').textContent = state === 'paused' ? 'Scripture paused' : state === 'reading' ? 'Reading Scripture' : 'Listen to Scripture';
      } else {
        play.textContent = state === 'paused' ? 'Resume' : 'Listen to Scripture';
        pause.textContent = state === 'paused' ? 'Paused' : 'Pause';
      }
    };
    const clearHighlight = () => nodes.forEach(node => node.classList.remove('is-speaking'));
    stop = (message = 'Stopped.') => {
      generation++;
      clearTimeout(timer);
      current = null;
      synth?.cancel();
      clearHighlight();
      setState('idle', message);
    };
    if (!synth || !('SpeechSynthesisUtterance' in window)) {
      [play, pause, stopButton, rate, voice].forEach(control => { if (control) control.disabled = true; });
      setState('unavailable', 'Read-aloud is unavailable in this browser. Try a browser with device speech support.');
    } else {
      const populateVoices = () => {
        let stored = '';
        try { stored = localStorage.getItem('bible.voice') || ''; } catch {}
        const selected = voice.value || stored;
        const options = synth.getVoices().filter(item => /^en\b/i.test(item.lang));
        voice.replaceChildren(new Option('Device default', ''));
        for (const item of options) voice.add(new Option(`${item.name} (${item.lang})`, item.voiceURI));
        if ([...voice.options].some(item => item.value === selected)) voice.value = selected;
      };
      try { rate.value = localStorage.getItem('bible.rate') || '1'; } catch {}
      populateVoices();
      on(synth, 'voiceschanged', populateVoices);
      setState('idle', 'Ready. Reads the KJV text only.');
      const readVerse = (run, verseIndex, chunkIndex = 0, chunks = null) => {
        if (run !== generation) return;
        if (verseIndex >= verses.length) {
          current = null;
          clearHighlight();
          setState('finished', bookMode ? `Finished reading ${bookName}.` : 'Finished reading this passage.');
          return;
        }
        index = verseIndex;
        // Keep utterances short for engines that stall on long text.
        chunks ||= verses[verseIndex].match(/.{1,220}(?:\s|$)|\S{1,220}/g) || [verses[verseIndex]];
        if (chunkIndex >= chunks.length) { readVerse(run, verseIndex + 1); return; }
        const utterance = new SpeechSynthesisUtterance(chunks[chunkIndex].trim());
        current = utterance;
        utterance.lang = 'en-US';
        utterance.rate = Number(rate.value) || 1;
        utterance.voice = synth.getVoices().find(item => item.voiceURI === voice.value) || null;
        utterance.onstart = () => {
          if (run !== generation) return;
          clearTimeout(timer);
          clearHighlight();
          nodes[verseIndex]?.classList.add('is-speaking');
          const number = verseNumber(verseIndex);
          setState('reading', bookMode
            ? `Reading ${location(verseIndex)} · ${verseIndex + 1} of ${verses.length}.`
            : `Reading verse ${number} · ${verseIndex + 1} of ${verses.length}.`);
          console.info('Bible read-aloud: speech started', {verse: bookMode ? location(verseIndex) : number, voice: utterance.voice?.name || 'device default'});
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
          if (run === generation) setState('reading', bookMode
            ? `Reading ${location(index)} · ${index + 1} of ${verses.length}.`
            : `Reading verse ${nodes[index].dataset.verse || index + 1} · ${index + 1} of ${verses.length}.`);
        };
        utterance.onerror = event => {
          if (run !== generation) return;
          stop(`Speech could not play${bookMode ? ` at ${location(verseIndex)}` : ''} (${event.error}). Choose another voice or browser.`);
          controls.dataset.audioState = 'error';
          revealPlaybackSettings();
        };
        timer = setTimeout(() => {
          if (run === generation && controls.dataset.audioState === 'starting') {
            stop(`The device speech service did not start${bookMode ? ` for ${location(index)}` : ''}. Try another voice or browser.`);
            controls.dataset.audioState = 'error';
            revealPlaybackSettings();
          }
        }, 7000);
        synth.speak(utterance);
      };
      on(play, 'click', () => {
        revealPlaybackSettings();
        if ((compact || bookMode) && controls.dataset.audioState === 'reading') {
          synth.pause();
          if (synth.paused) setState('paused', bookMode
            ? `Paused at ${location(index)}. Press Resume to continue.`
            : 'Paused. Press Resume to continue.');
          return;
        }
        if (controls.dataset.audioState === 'paused' && current) {
          synth.resume();
          if (!synth.paused) setState('reading', bookMode
            ? `Reading ${location(index)} · ${index + 1} of ${verses.length}.`
            : `Reading verse ${nodes[index].dataset.verse || index + 1} · ${index + 1} of ${verses.length}.`);
          return;
        }
        stop('Starting the device voice…');
        if (!verses.length) { setState('error', bookMode ? `There is no Scripture text available for ${bookName}.` : 'There is no Scripture passage to read here.'); return; }
        setState('starting', bookMode ? `Starting ${bookName}…` : 'Starting the device voice…');
        readVerse(generation, 0);
      });
      on(pause, 'click', () => {
        synth.pause();
        if (synth.paused) setState('paused', bookMode
          ? `Paused at ${location(index)}. Press Resume to continue.`
          : 'Paused. Press Resume to continue.');
      });
      on(stopButton, 'click', () => stop());
      on(rate, 'change', () => {
        try { localStorage.setItem('bible.rate', rate.value); } catch {}
        stop(bookMode ? `Speed changed. Press Listen to restart ${bookName}.` : 'Speed changed. Press Listen to restart this passage.');
      });
      on(voice, 'change', () => {
        try { localStorage.setItem('bible.voice', voice.value); } catch {}
        stop(bookMode ? `Voice changed. Press Listen to restart ${bookName}.` : 'Voice changed. Press Listen to restart this passage.');
      });
      on(window, 'pagehide', () => stop());
    }
  }
  dispose = () => { stop(); listeners.forEach(remove => remove()); };
}

// Native pages load this module directly; WASM pages call init after each render.
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => initExperience(), {once:true});
else initExperience();
