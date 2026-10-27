// The cover rule (ADR 0060 items 2 to 6, ADR 0042). Each test checks one
// rule of the channel cover.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { coverOf, coverForm, coverConflict, withCover } from '../ui/cover.js';

const A = 'https://m.example/a.jpg';
const B = 'https://m.example/b.jpg';
const rss = (url) => ({ url, title: 'T', link: null, description: null });

const NONE = { title: 'T', image: null, rss_image: null };
const RSS_ONLY = { title: 'T', image: null, rss_image: rss(A) };
const ITUNES_ONLY = { title: 'T', image: A, rss_image: null };
const BOTH = { title: 'T', image: A, rss_image: rss(A) };
const CONFLICT = { title: 'T', image: A, rss_image: rss(B) };

test('coverForm names the four forms', () => {
  assert.equal(coverForm(NONE), 'none');
  assert.equal(coverForm(RSS_ONLY), 'rss-only');
  assert.equal(coverForm(ITUNES_ONLY), 'itunes-only');
  assert.equal(coverForm(BOTH), 'both');
  assert.equal(coverForm({ title: 'T' }), 'none');
});

test('coverOf gives the itunes:image first, then the RSS image, then an empty text', () => {
  assert.equal(coverOf(NONE), '');
  assert.equal(coverOf(RSS_ONLY), A);
  assert.equal(coverOf(ITUNES_ONLY), A);
  assert.equal(coverOf(CONFLICT), A);
  assert.equal(coverOf({ image: null, rss_image: rss(null) }), '');
});

test('coverConflict gives the RSS address only when the two addresses are different', () => {
  assert.equal(coverConflict(NONE), null);
  assert.equal(coverConflict(RSS_ONLY), null);
  assert.equal(coverConflict(ITUNES_ONLY), null);
  assert.equal(coverConflict(BOTH), null);
  assert.equal(coverConflict(CONFLICT), B);
});

test('withCover keeps the form of a feed with only the RSS image', () => {
  const next = withCover(RSS_ONLY, B);
  assert.equal(next.image, null);
  assert.deepEqual(next.rss_image, rss(B));
  assert.equal(RSS_ONLY.rss_image.url, A);
});

test('withCover keeps the form of a feed with only itunes:image', () => {
  const next = withCover(ITUNES_ONLY, B);
  assert.equal(next.image, B);
  assert.equal(next.rss_image, null);
});

test('withCover puts the address in both tags of a feed with both', () => {
  const next = withCover(CONFLICT, B);
  assert.equal(next.image, B);
  assert.equal(next.rss_image.url, B);
  assert.equal(next.rss_image.title, 'T');
  assert.equal(coverConflict(next), null);
});

test('withCover sets itunes:image in a feed with neither tag', () => {
  const next = withCover(NONE, A);
  assert.equal(next.image, A);
  assert.equal(next.rss_image, null);
});

test('withCover with an empty address removes image and the full rss_image', () => {
  for (const model of [NONE, RSS_ONLY, ITUNES_ONLY, BOTH, CONFLICT]) {
    for (const empty of ['', null]) {
      const next = withCover(model, empty);
      assert.equal(next.image, null);
      assert.equal(next.rss_image, null);
      assert.equal(next.title, 'T');
    }
  }
});
