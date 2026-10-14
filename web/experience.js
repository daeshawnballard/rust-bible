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
    const synth = window.speechSynthesis;
    const play = controls.querySelector('[data-audio-action="play"]');
    const pause = controls.querySelector('[data-audio-action="pause"]');
    const stopButton = controls.querySelector('[data-audio-action="stop"]');
    const rate = controls.querySelector('[data-audio-rate]');
    const voice = controls.querySelector('[data-audio-voice]');
    const status = controls.querySelector('[data-audio-status]');
    const nodes = [...root.querySelectorAll('.scripture .v')];
    const verses = nodes.map(node => {
      const clone = node.cloneNode(true);
      clone.querySelectorAll('sup').forEach(number => number.remove());
      return clone.textContent.trim();
    });
    let generation = 0, current = null, timer = null, index = 0;
    const setState = (state, message) => {
      controls.dataset.audioState = state;
      status.textContent = message;
      pause.disabled = !['reading', 'paused'].includes(state);
      stopButton.disabled = !['reading', 'paused', 'starting'].includes(state);
      play.textContent = state === 'paused' ? 'Resume' : 'Listen to Scripture';
      pause.textContent = state === 'paused' ? 'Paused' : 'Pause';
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
          setState('finished', 'Finished reading this passage.');
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
          nodes[verseIndex].classList.add('is-speaking');
          const number = nodes[verseIndex].dataset.verse || nodes[verseIndex].querySelector('sup')?.textContent || verseIndex + 1;
          setState('reading', `Reading verse ${number} · ${verseIndex + 1} of ${verses.length}.`);
          console.info('Bible read-aloud: speech started', {verse: number, voice: utterance.voice?.name || 'device default'});
        };
        utterance.onend = () => {
          if (run === generation) readVerse(run, verseIndex, chunkIndex + 1, chunks);
        };
        utterance.onpause = () => {
          if (run === generation) setState('paused', 'Paused. Press Resume to continue.');
        };
        utterance.onresume = () => {
          if (run === generation) setState('reading', `Reading verse ${nodes[index].dataset.verse || index + 1} · ${index + 1} of ${verses.length}.`);
        };
        utterance.onerror = event => {
          if (run !== generation) return;
          stop(`Speech could not play (${event.error}). Choose another voice or browser.`);
          controls.dataset.audioState = 'error';
        };
        timer = setTimeout(() => {
          if (run === generation && controls.dataset.audioState === 'starting') {
            stop('The device speech service did not start. Try another voice or browser.');
            controls.dataset.audioState = 'error';
          }
        }, 7000);
        synth.speak(utterance);
      };
      on(play, 'click', () => {
        if (controls.dataset.audioState === 'paused' && current) {
          synth.resume();
          if (!synth.paused) setState('reading', `Reading verse ${nodes[index].dataset.verse || index + 1} · ${index + 1} of ${verses.length}.`);
          return;
        }
        stop('Starting the device voice…');
        if (!verses.length) { setState('error', 'There is no Scripture passage to read here.'); return; }
        setState('starting', 'Starting the device voice…');
        readVerse(generation, 0);
      });
      on(pause, 'click', () => {
        synth.pause();
        if (synth.paused) setState('paused', 'Paused. Press Resume to continue.');
      });
      on(stopButton, 'click', () => stop());
      on(rate, 'change', () => {
        try { localStorage.setItem('bible.rate', rate.value); } catch {}
        stop('Speed changed. Press Listen to restart this passage.');
      });
      on(voice, 'change', () => {
        try { localStorage.setItem('bible.voice', voice.value); } catch {}
        stop('Voice changed. Press Listen to restart this passage.');
      });
      on(window, 'pagehide', () => stop());
    }
  }
  dispose = () => { stop(); listeners.forEach(remove => remove()); };
}

// Native pages load this module directly; WASM pages call init after each render.
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => initExperience(), {once:true});
else initExperience();
