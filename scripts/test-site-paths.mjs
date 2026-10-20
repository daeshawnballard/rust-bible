import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { mountHtml, mountApiHtml } from '../web/site-paths.js';

const base = '/projects/bibleinrust/';
const html = `<a href="#/book/Genesis/18">Read</a><img src="/art/journey/scene.webp"><div style="background:url('/art/covers/genesis.webp')"></div><a href="https://github.com/daeshawnballard/rust-bible">Source</a>`;
assert.equal(mountHtml(html, '/'), html);
const mounted = mountHtml(html, base);
assert(mounted.includes('href="#/book/Genesis/18"'));
assert(mounted.includes('src="/projects/bibleinrust/art/journey/scene.webp"'));
assert(mounted.includes("url('/projects/bibleinrust/art/covers/genesis.webp')"));
assert(mounted.includes('href="https://github.com/daeshawnballard/rust-bible"'));
assert.equal(mountHtml('<img src="//example.test/image.webp">', base), '<img src="//example.test/image.webp">');
const scripture = 'And God said, Let there be light: and there was light.';
const api = { text: scripture, html, nested: [{ html }] };
const result = mountApiHtml(api, base);
assert.equal(result.text, scripture);
assert.equal(result.html, mounted);
assert.equal(result.nested[0].html, mounted);
assert.equal(api.html, html);
for (const file of ['web/style.css', 'web/today.css']) {
  assert(!/url\(['"]?\/(?!\/)/.test(await readFile(file, 'utf8')), `${file} has a root-only resource URL`);
}
assert((await readFile('scripts/build-wasm.sh', 'utf8')).includes('cp web/site-paths.js dist/'));
console.log('PASS: project-path artwork, nested API HTML, root compatibility, unchanged Scripture/routes, CSS paths and build inclusion');
