import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pageSequence, settingsTabs, personTabs, pageNames, tierError, TIERS } from '../ui/pages.js';

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
    'ADR 0206 item 33: the field Language belongs to the page Feed settings. Remove it from ChannelFields in crates/api/ui/editor.js.',
  );
});

test('the page publisher gives the sections in the sequence of ADR 0075', () => {
  assert.deepEqual(
    pageSequence('publisher').map((s) => s.id),
    ['role', 'channel', 'releases', 'members', 'feed'],
    'ADR 0075 item 1, ADR 0118 item 8 and ADR 0239 item 7: the publisher page must give role, channel, releases, members and feed. Fix the page publisher in crates/api/ui/pages.js.',
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

test('the release kind field follows Title in the first tab', () => {
  const owner = 'ADR 0213 item 13: put Kind between Title and Artist on the first tab of a release. Fix pages.js.';
  assert.deepEqual(pageSequence('album', { release: true })[0].fields.slice(0, 3), ['title', 'kind', 'author'], owner);
  assert.ok(!pageSequence('album', { release: false })[0].fields.includes('kind'), owner);
});

test('Settings has no release kind section', () => {
  assert.ok(!pageSequence('settings', { album: true, release: true }).some((section) => section.id === 'kind'),
    'ADR 0213 item 13: remove Kind from Settings. Fix pages.js.');
});

test('the release kind hint says that the value saves at once', () => {
  const text = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(text.includes('Saves at once. The feed file has no tag for the kind.'),
    'ADR 0213 item 16: add the save hint to the Kind menu in crates/api/ui/editor.js.');
  assert.ok(!text.includes('Studio shows this kind. The feed file has no tag for it.'),
    'ADR 0213 item 16: remove the old Kind hint from crates/api/ui/editor.js.');
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
  assert.deepEqual(ids(true), ['scale', 'lock', 'account', 'apps', 'nostr', 'outside', 'version', 'library']);
  // ADR 0176 items 10 and 11: the desktop app has "Collections", and a shell with no list has "Move Studio".
  assert.deepEqual(ids(false), ['scale', 'lock', 'window', 'apps', 'nostr', 'outside', 'version', 'collections', 'library']);
  const desktop = pageSequence('studio-settings', { server: false, collections: false });
  assert.equal(desktop.find((s) => s.id === 'move').tier, 'setup');
  assert.equal(tierError('studio-settings', desktop), null);
});

test('a server that holds no assets shows files on this server in the tier limits (ADR 0105 item 17)', () => {
  const sections = pageSequence('studio-settings', { server: true, holdAssets: false });
  assert.deepEqual(sections.map((s) => s.id), ['scale', 'lock', 'account', 'apps', 'nostr', 'outside', 'version', 'files']);
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
    ['scale', 'lock', 'window', 'outside', 'version'], ['nostr'], ['apps'], ['collections', 'library'],
  ]);
  const server = settingsTabs({ server: true, holdAssets: true });
  assert.deepEqual(server.map(({ label }) => label), ['General', 'Account', 'Nostr', 'Apps', 'Data']);
  assert.deepEqual(server.map(({ sections }) => sections.map(({ id }) => id)), [
    ['scale', 'lock', 'outside', 'version'], ['account'], ['nostr'], ['apps'], ['library'],
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

test('the person page has more with the tier content before payment (ADR 0182 item 10)', () => {
  const seq = pageSequence('person');
  const more = seq.findIndex((s) => s.id === 'more');
  assert.ok(more >= 0 && seq[more].tier === 'content');
  assert.equal(seq[more + 1].id, 'payment',
    'ADR 0182 item 10: put more before payment. Fix the person page in crates/api/ui/pages.js.');
  assert.equal(tierError('person', seq), null);
});

test('the person page follows ADR 0182 item 10', () => {
  const seq = pageSequence('person');
  assert.deepEqual(seq.map((s) => s.id), ['profile', 'member', 'credits', 'where', 'check', 'more', 'payment', 'support', 'change', 'delete'],
    'ADR 0182 item 10: set the section sequence. Fix the person page in crates/api/ui/pages.js.');
  assert.deepEqual(seq.map((s) => s.tier), ['identity', 'identity', 'identity', 'identity', 'identity', 'content', 'content', 'details', 'setup', 'limits'],
    'ADR 0182 item 10: set each section tier. Fix the person page in crates/api/ui/pages.js.');
  assert.equal(tierError('person', seq), null);
});

test('Person tabs group the sections by ADR 0226 items 9 to 12', () => {
  const tabs = personTabs();
  assert.deepEqual(tabs.map(({ id, label }) => [id, label]), [
    ['profile', 'Profile'], ['feeds', 'Feeds'], ['payment', 'Payment'], ['settings', 'Settings'],
  ]);
  assert.deepEqual(tabs.map(({ sections }) => sections.map(({ id }) => id)), [
    ['member', 'check'], ['credits', 'where', 'more'], ['payment', 'support'], ['change', 'delete'],
  ]);
  for (const tab of tabs) assert.equal(tierError(`person ${tab.label}`, tab.sections), null);
});

test('a tab change of Settings or of one person page does not ask the leave question', async () => {
  const { isTabChange } = await import('../ui/shell.js');
  const fix = 'ADR 0166 item 10 and ADR 0226 item 12: a tab change keeps the draft. Fix isTabChange in crates/api/ui/shell.js.';
  assert.equal(isTabChange({ name: 'person', id: 4 }, { name: 'person', id: 4, tab: 'payment' }), true, fix);
  assert.equal(isTabChange({ name: 'settings', tab: 'general' }, { name: 'settings', tab: 'data' }), true, fix);
  assert.equal(isTabChange({ name: 'person', id: 4, tab: 'feeds' }, { name: 'person', id: 5, tab: 'payment' }), false,
    `ADR 0116 item 8: a different person is a leave. ${fix}`);
  assert.equal(isTabChange({ name: 'person', id: 4, tab: 'feeds' }, { name: 'people' }), false, fix);
  assert.equal(isTabChange({ name: 'person', id: 4, tab: 'feeds' }, undefined), false, fix);
});
