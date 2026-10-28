import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pageSequence, pageNames, tierError, TIERS } from '../ui/pages.js';

const FEED_FIELDS = ['title', 'author', 'nostr', 'artwork', 'description', 'link', 'listing', 'explicit'];
test('pageSequence gives a new copy at each call', () => {
  const a = pageSequence('album');
  const b = pageSequence('album');
  assert.notEqual(a, b, 'ADR 0071 item 5: pageSequence must give a new copy. Fix the function in crates/api/ui/pages.js.');
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
    'ADR 0068 item 4: the field Language belongs to the page Feed settings. Remove it from ChannelFields in crates/api/ui/editor.js.',
  );
});

test('the page publisher gives the sections in the sequence of ADR 0075', () => {
  assert.deepEqual(
    pageSequence('publisher').map((s) => s.id),
    ['channel', 'releases', 'details', 'role', 'feed'],
    'ADR 0075 item 1: the publisher page must give channel, releases, details, role and feed. Fix the page publisher in crates/api/ui/pages.js.',
  );
  assert.deepEqual(pageSequence('publisher')[2].rows, ['recipients', 'credits']);
});

for (const server of [true, false]) {
  for (const page of pageNames()) {
    test(`the sections of the page ${page} have tiers that do not go down (server ${server})`, () => {
      const sections = pageSequence(page, { server });
      for (const { id, tier } of sections) {
        assert.ok(
          TIERS.includes(tier),
          `ADR 0071 item 1: on the page ${page}, the section ${id} has no tier of ADR 0071 item 1. Give the section a tier in crates/api/ui/pages.js.`,
        );
      }
      assert.equal(tierError(page, sections), null, tierError(page, sections) ?? '');
    });
  }
}

test('tierError gives a message for a list with a lower tier after a higher tier', () => {
  const message = tierError('album', [
    { id: 'feed', tier: 'setup' },
    { id: 'tracks', tier: 'content' },
  ]);
  assert.equal(
    message,
    'ADR 0071 item 6: on the page album, the section tracks (content) comes after a section of the tier setup. Give the section its tier of ADR 0071 item 1, and put it after each section of a lower tier in crates/api/ui/pages.js.',
  );
});

test('tierError gives a message that names ADR 0071 item 1 for an unknown tier', () => {
  const message = tierError('album', [{ id: 'tracks', tier: 'urgent' }]);
  assert.ok(message && message.includes('ADR 0071 item 1'));
  assert.equal(tierError('album', [{ id: 'a', tier: 'setup' }, { id: 'b', tier: 'setup' }]), null);
});

test('studio-settings has account only on a server', () => {
  const ids = (server) => pageSequence('studio-settings', { server }).map((s) => s.id);
  assert.deepEqual(ids(true), ['account', 'apps', 'announcements', 'nostr', 'library']);
  assert.deepEqual(ids(false), ['apps', 'announcements', 'nostr', 'library']);
});

test('hosts starts with cards', () => {
  assert.equal(pageSequence('hosts')[0].id, 'cards');
});

test('nostr follows author on the three feed pages', () => {
  for (const name of ['album', 'publisher', 'playlist']) {
    const fields = pageSequence(name)[0].fields;
    assert.equal(fields.indexOf('nostr'), fields.indexOf('author') + 1,
      'ADR 0074 item 19: the row nostr must follow author. Fix FEED_FIELDS in crates/api/ui/pages.js.');
  }
});
