// The check for a change from outside Studio (ADR 0150).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { outsideMark, outsideNotice, indexNotice, eachLimited, OUTSIDE_LIMIT, panelSet, stateMark } from '../ui/outsidecheck.js';
import { pageSequence, tierError } from '../ui/pages.js';

test('A tile has a mark for a change and a move, and none for an index difference (ADR 0150 items 2, 3 and 5)', () => {
  assert.equal(outsideMark({ state: 'changed' }).text, 'Changed outside Studio');
  assert.equal(outsideMark({ state: 'moved' }).text, 'Moved outside Studio');
  assert.equal(outsideMark({ state: 'moved' }).tone, 'error');
  assert.equal(outsideMark({ state: null, index_url: 'https://x/a.xml' }), null);
  assert.equal(outsideMark(null), null);
});

test('The notice of a move names the address and has Keep This File Host (ADR 0150 items 3 and 4)', () => {
  const notice = outsideNotice({ state: 'moved', new_url: 'https://x/a.xml', connected: true });
  assert.equal(notice.address, 'https://x/a.xml');
  assert.equal(notice.keep, true);
  assert.equal(notice.difference, true);
  assert.equal(outsideNotice({ state: 'moved', new_url: 'https://x/a.xml', connected: false }).difference, false);
  assert.equal(outsideNotice({ state: 'changed' }).again, true);
  assert.equal(outsideNotice({ state: null }), null);
});

test('The index notice gives both addresses (ADR 0150 item 5)', () => {
  const notice = indexNotice({ index_url: 'https://a/x.xml', own_url: 'https://b/x.xml' });
  assert.deepEqual([notice.index, notice.own], ['https://a/x.xml', 'https://b/x.xml']);
  assert.equal(indexNotice({ index_url: null }), null);
});

test('eachLimited runs at most the limit and continues after a failure (ADR 0150 item 11)', async () => {
  assert.equal(OUTSIDE_LIMIT, 4);
  let running = 0;
  let most = 0;
  const done = [];
  await eachLimited([1, 2, 3, 4, 5, 6, 7, 8, 9], OUTSIDE_LIMIT, async (n) => {
    running += 1;
    most = Math.max(most, running);
    await new Promise((resolve) => { setTimeout(resolve, 1); });
    running -= 1;
    if (n === 2) throw new Error('failed');
    done.push(n);
  });
  assert.equal(most, 4);
  assert.equal(done.length, 8);
});

test('The setting is a check box, and the section has a tier (ADR 0150 item 10, ADR 0071)', () => {
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = source.indexOf('function OutsidePanel');
  assert.ok(start > 0);
  assert.match(source.slice(start, start + 900), /type="checkbox"/);
  for (const server of [true, false]) {
    const sections = pageSequence('studio-settings', { server });
    assert.ok(sections.some((s) => s.id === 'outside'));
    assert.equal(tierError('studio-settings', sections), null);
  }
});

const feedOf = (id, extra = {}) => ({ id, draft_state: 'unchanged', release_ids: [], publisher_ids: [], ...extra });
const read = (state = null) => ({ read: true, state });

test('The mark of a set ranks outside, pending and unchecked (ADR 0165 item 2)', () => {
  const feeds = [feedOf(1), feedOf(2, { draft_state: 'new' }), feedOf(3)];
  const results = { 1: read(), 2: read(), 3: read('changed') };
  assert.equal(stateMark(feeds, results).kind, 'outside');
  assert.equal(stateMark(feeds, results).text, 'Changed outside Studio');
  assert.equal(stateMark(feeds, results).icon, 'cloud-alert');
  assert.equal(stateMark(feeds, results).tone, 'error');
  assert.equal(stateMark(feeds, { ...results, 1: read('moved') }).text, 'Moved outside Studio');
  const pending = stateMark(feeds, { 1: read(), 2: read(), 3: read() });
  assert.deepEqual([pending.kind, pending.icon, pending.tone, pending.text], ['pending', 'pencil', 'accent', 'Changes not written']);
  assert.equal(stateMark([feedOf(1, { draft_state: 'changed' })], {}).kind, 'pending');
  const unchecked = stateMark([feedOf(1), feedOf(2)], { 1: read() });
  assert.deepEqual([unchecked.kind, unchecked.icon, unchecked.tone, unchecked.text], ['unchecked', 'circle-dashed', 'muted', 'Not checked']);
  assert.equal(stateMark([feedOf(1)], { 1: { read: false, state: null } }).kind, 'unchecked');
  assert.equal(stateMark([feedOf(1, { draft_state: 'none' })], { 1: read() }), null);
  assert.equal(stateMark([], {}), null);
});

test('A row covers the feed and each release, and a group covers its feeds (ADR 0165 item 3)', () => {
  const feeds = [
    feedOf(1, { release_ids: [2, 3], roles: ['artist'] }),
    feedOf(2, { publisher_ids: [1] }),
    feedOf(3, { publisher_ids: [1] }),
    feedOf(4, { release_ids: [] }),
  ];
  assert.deepEqual(panelSet(feeds, 1).map((f) => f.id), [1, 2, 3]);
  assert.deepEqual(panelSet(feeds, 99), []);
  assert.ok(Array.isArray(panelSet(feeds, 'none')));
  assert.ok(Array.isArray(panelSet(feeds, 'other')));
});

test('The style of a mark sets the color of the icon, not only of the row (ADR 0165 item 2)', () => {
  const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
  for (const [tone, token] of [['error', '--error'], ['accent', '--accent'], ['muted', '--muted']]) {
    assert.ok(css.includes(`.row .state-mark.${tone} .icon { color: var(${token}); }`), tone);
  }
});
