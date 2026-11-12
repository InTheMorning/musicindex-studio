// The check for a change from outside Studio (ADR 0150).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { outsideMark, outsideNotice, indexNotice, moveNotice, nextNoticeLoad, LATE_NOTICE_LOAD, eachLimited, OUTSIDE_LIMIT, panelSet, stateMark } from '../ui/outsidecheck.js';
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

test('The Studio move notice gives each podping state and its action (ADR 0184 items 3 to 7)', () => {
  const notice = moveNotice({ index_is_move_source: true, own_url: 'https://new.example/a.xml', last_move: {
    moved_at: 100,
    announcements: [
      { url: 'https://new.example/a.xml', state: 'waiting', send_at: 110 },
      { url: 'https://old.example/a.xml', state: 'sent', finished_at: 120 },
      { url: 'https://new.example/a.xml', state: 'failed', finished_at: 130, error: 'timeout' },
      { url: 'https://old.example/a.xml', state: 'not_sent' },
    ],
  } });
  assert.equal(notice.tone, 'plain');
  assert.match(notice.strong, /^You moved this feed at /);
  const expectedTime = (seconds) => new Date(seconds * 1000).toLocaleString();
  for (const [index, state, tag, time] of [
    [0, 'waiting', 'Waiting', 110],
    [1, 'sent', 'Sent', 120],
    [2, 'failed', 'Failed', 130],
    [3, 'not_sent', 'Not sent', null],
  ]) {
    assert.equal(notice.lines[index].state, state, 'ADR 0189 item 7: each podping line gives its state.');
    assert.equal(notice.lines[index].tag, tag, 'ADR 0189 item 7: each podping line gives its status tag.');
    assert.equal(notice.lines[index].time, time === null ? null : expectedTime(time), 'ADR 0189 item 7: each podping line gives its time in the tag.');
  }
  assert.equal(notice.lines[0].text, '', 'ADR 0189 item 7: the waiting line text does not repeat its state or time.');
  assert.equal(notice.lines[1].text, '', 'ADR 0189 item 7: the sent line text does not repeat its state or time.');
  assert.equal(notice.lines[2].text, 'timeout', 'ADR 0189 item 7: a failed line gives its error text.');
  assert.equal(notice.lines[2].action, 'Send the Podping Again');
  assert.equal(notice.lines[3].text, '');
  assert.equal(notice.lines[3].action, 'Send the Podping');
  assert.equal(notice.lines[0].action, undefined);
  assert.equal(notice.lines[1].action, undefined);
  assert.equal(moveNotice({ index_is_move_source: false, index_url: 'https://old.example/a.xml' }), null);
  assert.ok(indexNotice({ index_url: 'https://old.example/a.xml', own_url: 'https://new.example/a.xml' }));
});

test('A Studio move notice has no send button with no public address (ADR 0184 item 11)', () => {
  const notice = moveNotice({ index_is_move_source: true, own_url: null, last_move: {
    moved_at: 100,
    announcements: [
      { url: 'https://new.example/a.xml', state: 'failed', finished_at: 130, error: null },
      { url: 'https://old.example/a.xml', state: 'not_sent' },
    ],
  } });
  assert.equal(notice.lines[0].state, 'failed', 'ADR 0189 item 7: a failed line gives its state.');
  assert.equal(notice.lines[0].tag, 'Failed', 'ADR 0189 item 7: a failed line gives its status tag.');
  assert.equal(notice.lines[0].time, new Date(130000).toLocaleString(), 'ADR 0189 item 7: a failed line gives its time in the tag.');
  assert.equal(notice.lines[0].text, '', 'ADR 0189 item 7: a failed line with no error has no sentence text.');
  assert.equal(notice.lines[0].action, undefined);
  assert.equal(notice.lines[1].action, undefined);
});

test('ADR 0189 item 6: a waiting podping loads again five seconds after its send time', () => {
  const check = { last_move: { announcements: [
    { state: 'waiting', send_at: 110 },
    { state: 'waiting', send_at: 108 },
    { state: 'sent', send_at: 100 },
  ] } };
  assert.equal(nextNoticeLoad(check, 100), 13000);
  assert.equal(nextNoticeLoad(check, 120), LATE_NOTICE_LOAD, 'ADR 0189 item 6: a late podping loads each 30 seconds. Fix: give LATE_NOTICE_LOAD after the send time.');
});

test('ADR 0189 item 6: sent, failed, not-sent, and absent notices have no next load', () => {
  for (const state of ['sent', 'failed', 'not_sent']) {
    assert.equal(nextNoticeLoad({ last_move: { announcements: [{ state, send_at: 110 }] } }, 100), null);
  }
  assert.equal(nextNoticeLoad({ last_move: { announcements: [] } }, 100), null);
  assert.equal(nextNoticeLoad(null, 100), null);
  assert.equal(nextNoticeLoad({ announcements: [{ state: 'waiting', send_at: 110 }] }, 100), null);
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
    assert.ok(css.includes(`.state-mark.${tone} .icon { color: var(${token}); }`), tone);
  }
});
