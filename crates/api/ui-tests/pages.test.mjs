import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pageSequence, settingsTabs, pageNames, tierError, TIERS } from '../ui/pages.js';

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
    ['role', 'channel', 'releases', 'members', 'feed'],
    'ADR 0075 item 1, ADR 0118 item 8 and ADR 0135 item 7: the publisher page must give role, channel, releases, members and feed. Fix the page publisher in crates/api/ui/pages.js.',
  );
});

test('the page settings gives op3 after locked for an album only', () => {
  const ids = (options) => pageSequence('settings', options).map((s) => s.id);
  const album = ids({ album: true });
  assert.equal(
    album.indexOf('op3'),
    album.indexOf('locked') + 1,
    'ADR 0076 item 5: the section op3 must come after locked on the page settings of an album. Fix the page settings in crates/api/ui/pages.js.',
  );
  assert.ok(!ids({ album: false }).includes('op3'), 'ADR 0076 item 5: only an album shows op3. Fix the page settings in crates/api/ui/pages.js.');
  assert.ok(!ids().includes('op3'));
  assert.equal(tierError('settings', pageSequence('settings', { album: true })), null);
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
  assert.deepEqual(ids(true), ['scale', 'lock', 'account', 'apps', 'nostr', 'outside', 'library']);
  // ADR 0176 items 10 and 11: the desktop app has "Collections", and a shell with no list has "Move Studio".
  assert.deepEqual(ids(false), ['scale', 'lock', 'window', 'apps', 'nostr', 'outside', 'collections', 'library']);
  const desktop = pageSequence('studio-settings', { server: false, collections: false });
  assert.equal(desktop.find((s) => s.id === 'move').tier, 'setup');
  assert.equal(tierError('studio-settings', desktop), null);
});

test('a server that holds no assets shows files on this server in the tier limits (ADR 0105 item 17)', () => {
  const sections = pageSequence('studio-settings', { server: true, holdAssets: false });
  assert.deepEqual(sections.map((s) => s.id), ['scale', 'lock', 'account', 'apps', 'nostr', 'outside', 'files']);
  assert.equal(sections.at(-1).tier, 'limits');
  assert.equal(tierError('studio-settings', sections), null);
  const on = pageSequence('studio-settings', { server: true, holdAssets: true }).map((s) => s.id);
  assert.ok(on.includes('library') && !on.includes('files'));
  const desktop = pageSequence('studio-settings', { server: false, holdAssets: false }).map((s) => s.id);
  assert.ok(desktop.includes('library') && !desktop.includes('files'));
});

test('Settings tabs give the labels and sections of ADR 0166 item 1', () => {
  const desktop = settingsTabs({ server: false });
  assert.deepEqual(desktop.map(({ label }) => label), ['General', 'Nostr', 'Apps', 'Data']);
  assert.deepEqual(desktop.map(({ sections }) => sections.map(({ id }) => id)), [
    ['scale', 'lock', 'window', 'outside'], ['nostr'], ['apps'], ['collections', 'library'],
  ]);
  const server = settingsTabs({ server: true, holdAssets: true });
  assert.deepEqual(server.map(({ label }) => label), ['General', 'Account', 'Nostr', 'Apps', 'Data']);
  assert.deepEqual(server.map(({ sections }) => sections.map(({ id }) => id)), [
    ['scale', 'lock', 'outside'], ['account'], ['nostr'], ['apps'], ['library'],
  ]);
  for (const tabs of [desktop, server, settingsTabs({ server: true, holdAssets: false })]) {
    for (const tab of tabs) assert.equal(tierError(`studio-settings ${tab.label}`, tab.sections), null);
  }
  assert.deepEqual(settingsTabs({ server: true, holdAssets: false }).at(-1).sections.map(({ id }) => id), ['files']);
});

test('hosts starts with cards', () => {
  assert.equal(pageSequence('hosts')[0].id, 'cards');
});

test('nostr follows author on the three feed pages', () => {
  for (const name of ['album', 'publisher', 'playlist']) {
    const fields = pageSequence(name).find((s) => s.fields).fields;
    assert.equal(fields.indexOf('nostr'), fields.indexOf('author') + 1,
      'ADR 0074 item 19: the row nostr must follow author. Fix FEED_FIELDS in crates/api/ui/pages.js.');
  }
});

test('the person page has more with the tier content before payment (ADR 0103 item 11)', () => {
  const seq = pageSequence('person');
  const more = seq.findIndex((s) => s.id === 'more');
  assert.ok(more >= 0 && seq[more].tier === 'content');
  assert.equal(seq[more + 1].id, 'payment',
    'ADR 0103 item 11: more comes before payment. Fix the page person in crates/api/ui/pages.js.');
  assert.equal(tierError('person', seq), null);
});

test('the person page follows the sequence of ADR 0111 item 1 (task 075b)', () => {
  const seq = pageSequence('person');
  assert.deepEqual(seq.map((s) => s.id), ['profile', 'where', 'check', 'more', 'payment', 'credits', 'support', 'member', 'change', 'delete'],
    'ADR 0111 item 1 and ADR 0156 item 1: fix the page person in crates/api/ui/pages.js.');
  assert.equal(tierError('person', seq), null);
});
