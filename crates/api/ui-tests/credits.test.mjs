// The credits of a track (ADR 0064 item 8).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { usesAlbumCredits, withTrackCredits, withAlbumCredits } from '../ui/credits.js';

const album = [
  { _cid: 'a', key: 0, name: 'Ann', role: 'vocalist', group: 'music', img: null, href: 'https://example.com/ann' },
  { _cid: 'b', key: 1, name: 'Bob', role: null, group: null, img: 'https://example.com/bob.jpg', href: null },
];

test('withTrackCredits copies the album persons with no _cid and no key, and keeps the album list', () => {
  const before = JSON.stringify(album);
  const item = withTrackCredits({ title: 'Song', persons: [] }, album);
  assert.equal(item.title, 'Song');
  assert.equal(item.persons.length, 2);
  assert.deepEqual(item.persons[0], { name: 'Ann', role: 'vocalist', group: 'music', img: null, href: 'https://example.com/ann' });
  assert.deepEqual(item.persons[1], { name: 'Bob', role: null, group: null, img: 'https://example.com/bob.jpg', href: null });
  assert.ok(item.persons.every((p) => !('_cid' in p) && !('key' in p)));
  assert.equal(JSON.stringify(album), before);
  assert.equal(usesAlbumCredits(item), false);
});

test('withAlbumCredits gives an empty list and usesAlbumCredits then gives true', () => {
  const item = withAlbumCredits({ title: 'Song', persons: [{ name: 'Ann' }] });
  assert.deepEqual(item.persons, []);
  assert.equal(usesAlbumCredits(item), true);
});

test('usesAlbumCredits gives false for an item with one person', () => {
  assert.equal(usesAlbumCredits({ persons: [{ name: 'Ann' }] }), false);
});
