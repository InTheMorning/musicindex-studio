import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pageSequence } from '../ui/pages.js';

const FEED_FIELDS = ['title', 'author', 'artwork', 'description', 'link', 'listing', 'explicit'];
const FEED = { id: 'feed', rows: ['settings', 'facts'] };

const EXPECTED = {
  album: [
    { id: 'album', fields: FEED_FIELDS },
    { id: 'tracks' },
    { id: 'details', rows: ['recipients', 'credits'] },
    { id: 'publishers' },
    FEED,
  ],
  publisher: [{ id: 'channel', fields: FEED_FIELDS }, { id: 'releases' }, { id: 'role' }, FEED],
  playlist: [{ id: 'channel', fields: FEED_FIELDS }, FEED],
  settings: [{ id: 'language' }, { id: 'keywords' }, { id: 'owner' }, { id: 'locked' }, { id: 'editors' }],
  track: [
    { id: 'track', fields: ['title', 'audio', 'duration', 'alternates', 'artwork', 'description'] },
    { id: 'credits' },
    { id: 'lyrics' },
    { id: 'payments' },
    { id: 'more', rows: ['pub_date', 'explicit', 'identity'] },
  ],
};

for (const [name, list] of Object.entries(EXPECTED)) {
  test(`pageSequence gives the sequence of the ${name} page`, () => {
    assert.deepEqual(
      pageSequence(name),
      list,
      `ADR 0068: the sequence of the ${name} page differs. Change the list in crates/api/ui/pages.js and the ADR together.`,
    );
  });
}

test('pageSequence gives a new copy at each call', () => {
  const a = pageSequence('album');
  const b = pageSequence('album');
  assert.notEqual(a, b, 'ADR 0068: pageSequence must give a new copy. Change the list in crates/api/ui/pages.js and the ADR together.');
  a[0].fields.push('extra');
  assert.equal(pageSequence('album')[0].fields.length, FEED_FIELDS.length);
});

test('ChannelFields in editor.js has no field Language', () => {
  const text = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const start = text.indexOf('function ChannelFields');
  const end = text.indexOf('\n}\n', start);
  assert.ok(start > 0 && end > start);
  assert.ok(
    !text.slice(start, end).includes('Language'),
    'ADR 0068: the field Language belongs to the page Feed settings. Change the list in crates/api/ui/pages.js and the ADR together.',
  );
});
