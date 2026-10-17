import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
// The production browser module has no dependencies or build-time transforms.
Object.defineProperty(globalThis, 'localStorage', {value: undefined, writable: true, configurable: true});
const source = await readFile(new URL('../web/progress.js', import.meta.url), 'utf8');
const {canonicalProgress, progressHref, resumeRoute, readProgress} = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
const legacy = {route:'/book/Psalms/23',book:'Psalms',chapter:23,bookIndex:18};
assert.equal(canonicalProgress(legacy).verse, undefined);
assert.equal(progressHref(canonicalProgress(legacy)), '/book/Psalms/23');
const verse = canonicalProgress({...legacy,verse:5,source:'listening'});
assert.equal(progressHref(verse), '/book/Psalms/23?resume=5#v5');
assert.equal(resumeRoute('/book/Psalms/23', verse), progressHref(verse));
assert.equal(resumeRoute('/book/Psalms/23?resume=2#v2', verse), '/book/Psalms/23?resume=2#v2');
assert.equal(resumeRoute('/book/Psalms/23?resume=2#v2', verse, {reload:true}), progressHref(verse));
assert.equal(resumeRoute('/book/Psalms/23?context=journey', verse), '/book/Psalms/23?context=journey&resume=5#v5');
assert.equal(resumeRoute('/book/Psalms/23?context=journey&resume=2#v2', verse), '/book/Psalms/23?context=journey&resume=2#v2');
assert.equal(resumeRoute('/book/Psalms/23?context=journey&resume=2#v2', verse, {reload:true}), '/book/Psalms/23?context=journey&resume=5#v5');
assert.equal(resumeRoute('/book/Psalms/23?v=1#v1', verse), '/book/Psalms/23?v=1#v1');
assert.equal(resumeRoute('/book/John/3', verse), '/book/John/3');
const advanced = canonicalProgress({...verse,route:'/book/Psalms/24',chapter:24,entryRoute:legacy.route});
assert.equal(resumeRoute(legacy.route, advanced), '/book/Psalms/24?resume=5#v5');
for (const invalid of [null, {}, {...legacy,route:'//example.com'}, {...legacy,route:'/book/Psalms/23?x=1'}, {...legacy,book:'<script>'}, {...legacy,chapter:24}, {...legacy,verse:0}, {...legacy,verse:177}, {...legacy,verse:2.5}, {...legacy,bookIndex:66}]) assert.equal(canonicalProgress(invalid), null);
assert.equal(canonicalProgress({route:'/book/1-Corinthians/13',book:'1 Corinthians',chapter:13,verse:7}).verse,7);
assert.equal(canonicalProgress({route:'/book/Song-of-Solomon/2',book:'Song of Solomon',chapter:2,verse:1}).verse,1);
assert.equal(resumeRoute('/book/Psalms/23?context=illustration&art=scene', verse), '/book/Psalms/23?context=illustration&art=scene&resume=5#v5');
assert.equal(resumeRoute('/book/Psalms/23?context=journey&focus=passage', verse), '/book/Psalms/23?context=journey&focus=passage&resume=5#v5');
// Browsing remains safe when persistence is unavailable or corrupt.
assert.equal(readProgress(), null); // localStorage absent in Node.
globalThis.localStorage = { getItem() { return '{broken'; } };
assert.equal(readProgress(), null);
globalThis.localStorage = { getItem() { throw new Error('Storage blocked'); } };
assert.equal(readProgress(), null);
console.log('PASS: legacy migration, exact resume, explicit navigation, continuation route, unsafe/corrupt records and unavailable storage');
