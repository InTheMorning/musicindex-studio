// The cover rule (ADR 0060 items 2 to 6, ADR 0042). Each test checks one
// rule of the channel cover.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { coverOf, coverForm, coverConflict, withSavedCover, coverNote } from '../ui/cover.js';

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

test('withSavedCover gives the saved cover when the local cover is the sent cover', () => {
  const sent = { title: 'T', image: B, rss_image: rss(A) };
  const saved = { title: 'T', image: null, rss_image: rss(B) };
  const next = withSavedCover({ ...sent, title: 'Changed' }, sent, saved);
  assert.equal(next.image, null);
  assert.deepEqual(next.rss_image, rss(B));
  assert.equal(next.title, 'Changed');
});

test('withSavedCover keeps the local cover when the user changed it during the save', () => {
  const sent = { title: 'T', image: B, rss_image: null };
  const local = { title: 'T', image: A, rss_image: null };
  const saved = { title: 'T', image: B, rss_image: null };
  assert.equal(withSavedCover(local, sent, saved), local);
});

test('a cover above the limit of the image route gives its reason', () => {
  assert.equal(coverNote('too_large'), 'The cover is larger than 10 MB, so Studio does not show it.', 'ADR 0090 item 3: give the reason for the code too_large. Use coverNote of crates/api/ui/cover.js.');
  assert.equal(coverNote('not_image'), null);
  assert.equal(coverNote(undefined), null);
});
