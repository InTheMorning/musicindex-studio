// addressParts gives each web address of a text as its own part, so the UI can
// let only the address break inside a word (musicindex ADR 0065 R-FIT-3 item 3).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { addressParts } from '../ui/breaks.js';

test('an address in a sentence is its own part, with no full stop', () => {
  assert.deepEqual(addressParts('Music Index holds https://old.example.org/rows.xml. The public address is https://a.example/feed.xml.'), [
    { text: 'Music Index holds ', address: false },
    { text: 'https://old.example.org/rows.xml', address: true },
    { text: '. The public address is ', address: false },
    { text: 'https://a.example/feed.xml', address: true },
    { text: '.', address: false },
  ]);
});

test('a text with no address is one plain part', () => {
  assert.deepEqual(addressParts('The https address gives the same file.'), [
    { text: 'The https address gives the same file.', address: false },
  ]);
});

test('an empty value gives no part', () => {
  assert.deepEqual(addressParts(undefined), []);
  assert.deepEqual(addressParts(''), []);
});
