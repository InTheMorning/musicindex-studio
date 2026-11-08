import test from 'node:test';
import assert from 'node:assert/strict';
import { withPersonKey, shortKey, keySummary, keyRowView, feedKey, withFeedKey, sourceText } from '../ui/nostrkey.js';

const KEY = 'npub1g2nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxck';

test('shortKey keeps the first 8 and the last 4 characters', () => {
  assert.equal(shortKey(KEY), 'npub1g2n…xxck');
  assert.equal(shortKey(''), '');
  assert.equal(shortKey(null), '');
});

test('keyRowView gives the short key and no art before a profile answer (ADR 0147)', () => {
  assert.deepEqual(keyRowView('', KEY, null), { summary: shortKey(KEY), art: null });
  assert.deepEqual(keyRowView('', KEY, new Map()), { summary: shortKey(KEY), art: null });
  assert.deepEqual(keyRowView('', null, null), { summary: 'Not set', art: null });
});

test('keyRowView takes the name and the picture from the profile (ADR 0147)', () => {
  const profiles = new Map([[KEY, { npub: KEY, name: ' Mara Tide ', picture: 'https://x/p.jpg' }]]);
  assert.deepEqual(keyRowView('', KEY, profiles), {
    summary: `Mara Tide · ${shortKey(KEY)}`,
    art: { name: 'Mara Tide', img: 'https://x/p.jpg' },
  });
});

test('keyRowView keeps the name of the feed in the place of the profile name (ADR 0147)', () => {
  const profiles = new Map([[KEY, { npub: KEY, name: 'mara', picture: null }]]);
  assert.deepEqual(keyRowView('Mara Tide', KEY, profiles), {
    summary: `Mara Tide · ${shortKey(KEY)}`,
    art: { name: 'Mara Tide', img: null },
  });
});

test('keySummary gives the name and the short key, or one of them', () => {
  const short = shortKey(KEY);
  assert.equal(keySummary('Matt Finlay', KEY), `Matt Finlay · ${short}`);
  assert.equal(keySummary('', KEY), short);
  assert.equal(keySummary('  ', KEY), short);
  assert.equal(keySummary('Matt Finlay', null), 'Not set');
  assert.equal(keySummary('', ''), 'Not set');
});

test('feedKey gives the value of the first npub entry', () => {
  assert.equal(feedKey({ txts: [] }), null);
  assert.equal(feedKey({}), null);
  assert.equal(feedKey({ txts: [{ purpose: 'other', value: 'x' }, { purpose: 'npub', value: KEY }] }), KEY);
});

test('withFeedKey sets, adds and removes the entry', () => {
  const model = { txts: [{ purpose: 'other', value: 'x' }, { purpose: 'npub', value: 'old' }] };
  const before = JSON.stringify(model);
  assert.deepEqual(withFeedKey(model, KEY).txts, [{ purpose: 'other', value: 'x' }, { purpose: 'npub', value: KEY }]);
  assert.deepEqual(withFeedKey({ txts: [] }, KEY).txts, [{ purpose: 'npub', value: KEY }]);
  assert.deepEqual(withFeedKey({}, KEY).txts, [{ purpose: 'npub', value: KEY }]);
  assert.deepEqual(withFeedKey(model, null).txts, [{ purpose: 'other', value: 'x' }]);
  assert.deepEqual(withFeedKey({ txts: [] }, null).txts, []);
  assert.equal(JSON.stringify(model), before);
});

test('sourceText names each source', () => {
  assert.equal(sourceText({ kind: 'collection' }), 'In your feeds');
  assert.equal(sourceText({ kind: 'music_index', feeds: 1 }), 'In 1 Music Index feed');
  assert.equal(sourceText({ kind: 'music_index', feeds: 3 }), 'In 3 Music Index feeds');
  assert.equal(sourceText({ kind: 'relay' }), 'Nostr search');
});

test('withPersonKey sets the key and the picked values only', () => {
  const person = { name: 'Ann', img: null, npub: null };
  assert.deepEqual(withPersonKey(person, KEY), { name: 'Ann', img: null, npub: KEY });
  assert.deepEqual(withPersonKey(person, KEY, { name: 'Ann B', picture: 'https://p.example/a.png' }),
    { name: 'Ann B', img: 'https://p.example/a.png', npub: KEY });
  assert.deepEqual(withPersonKey(person, KEY, { name: null, picture: null }), { name: 'Ann', img: null, npub: KEY });
  assert.equal(withPersonKey({ name: 'Ann', npub: KEY }, null).npub, null);
  assert.equal(person.npub, null);
});
