import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const source = await readFile(new URL('../web/local-voice.js', import.meta.url), 'utf8');
const {MAX_SPEECH_SEGMENT_LENGTH,splitSpeech} = await import(
  `data:text/javascript;base64,${Buffer.from(source).toString('base64')}`,
);

const inputs = [
  '',
  'A short verse.',
  '  Leading and trailing space stays attached.  ',
  `${'In the beginning God created the heaven and the earth. '.repeat(18)}And it was so.`,
  `${'word'.repeat(65)} end`,
  `${'a'.repeat(219)}🧭 then the next word`,
  `He said, “${'Do not be afraid; '.repeat(22)}I am with you.”`,
];

for (const input of inputs) {
  const segments = splitSpeech(input);
  assert.equal(segments.join(''), input, 'segment plan must preserve every source character');
  assert(segments.every(segment => segment.length <= MAX_SPEECH_SEGMENT_LENGTH),
    'each segment must stay within the bounded inference size');
}

assert.deepEqual(splitSpeech(''), []);
const sentencePlan = splitSpeech('One sentence. '.repeat(22));
assert(sentencePlan.length > 1, 'long text should be split');
assert(sentencePlan[0].endsWith(' '), 'sentence boundary whitespace must remain in the plan');
console.log('PASS: bounded lossless speech segments, preserved whitespace, sentence breaks, and surrogate pairs');
