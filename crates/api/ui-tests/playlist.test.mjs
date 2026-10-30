// The playlist models (ADR 0053, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  pickKey, pickFromIndex, remoteItemFromPick, addTrack, hasTrack,
  removeTrack, trackRow, tracksSummary,
} from '../ui/playlist.js';

const a = { _cid: 1, feed_guid: 'f1', item_guid: 'i1', feed_url: 'https://x.example/a.xml', medium: 'music', title: 'One' };
const b = { _cid: 2, feed_guid: 'f1', item_guid: 'i2', medium: 'music' };
const c = { _cid: 3, feed_guid: 'f2', item_guid: 'i1', medium: 'music', title: 'Three' };

test('the pick key joins the feed GUID and the item GUID', () => {
  assert.equal(pickKey(a), 'f1 i1');
  assert.equal(pickKey({ feed_guid: 'f2', item_guid: 'i9', title: 'x' }), 'f2 i9');
});

test('a pick from the index uses the track GUID as the item GUID', () => {
  const feed = { feed_guid: 'f1', feed_url: 'https://x.example/a.xml', title: 'Album', tracks: [] };
  assert.deepEqual(pickFromIndex(feed, { track_guid: 'aa11', title: 'Track One' }), {
    feed_guid: 'f1', item_guid: 'aa11', feed_url: 'https://x.example/a.xml', title: 'Track One',
  });
});

test('a remote item from a pick has no entry key and omits missing fields', () => {
  assert.deepEqual(remoteItemFromPick({ feed_guid: 'f1', item_guid: 'i1', feed_url: 'u', title: 'T' }), {
    feed_guid: 'f1', item_guid: 'i1', feed_url: 'u', medium: 'music', title: 'T',
  });
  const bare = remoteItemFromPick({ feed_guid: 'f1', item_guid: 'i1', feed_url: null, title: '' });
  assert.deepEqual(bare, { feed_guid: 'f1', item_guid: 'i1', medium: 'music' });
  assert.equal('_cid' in bare, false);
});

test('add puts a new track last and ignores a track with the same key', () => {
  const list = [a, b];
  const added = addTrack(list, c);
  assert.deepEqual(added.map((x) => x._cid), [1, 2, 3]);
  assert.equal(list.length, 2);
  assert.equal(addTrack(list, { ...a, _cid: 9 }), list);
});

test('has track compares the pick keys', () => {
  assert.equal(hasTrack([a, b], { feed_guid: 'f1', item_guid: 'i2', title: 'x' }), true);
  assert.equal(hasTrack([a, b], { feed_guid: 'f2', item_guid: 'i2', title: 'x' }), false);
});

test('remove drops one entry and keeps the others', () => {
  const list = [a, b, c];
  assert.deepEqual(removeTrack(list, 1).map((x) => x._cid), [1, 3]);
  assert.equal(list.length, 3);
});

test('a track row gives the title and the GUIDs', () => {
  assert.deepEqual(trackRow(a), { title: 'One', sub: 'f1 · i1' });
  assert.deepEqual(trackRow(b), { title: 'Untitled track', sub: 'f1 · i2' });
});

test('the summary counts the tracks', () => {
  assert.equal(tracksSummary([]), 'No tracks');
  assert.equal(tracksSummary([a]), '1 track');
  assert.equal(tracksSummary([a, b]), '2 tracks');
});
