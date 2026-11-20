// The check for a change from outside Studio (ADR 0150).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { outsideMark, outsideNotice, readOnlyHostNotice, indexNotice, releaseMusicIndexLink, moveNotice, writeNotice, nextNoticeLoad, LATE_NOTICE_LOAD, eachLimited, OUTSIDE_LIMIT, panelSet, stateMark } from '../ui/outsidecheck.js';
import { pageSequence, tierError } from '../ui/pages.js';
import { INDEX_READ_AGAIN_MS } from '../ui/releasemove.js';

test('A tile has a mark for a change and a move, and none for an index difference (ADR 0150 items 2, 3 and 5)', () => {
  assert.equal(outsideMark({ state: 'changed' }).text, 'Changed outside Studio');
  assert.equal(outsideMark({ state: 'moved' }).text, 'Moved outside Studio');
  assert.equal(outsideMark({ state: 'moved' }).tone, 'warning');
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

test('The release page links to Music Index only when it holds the feed (ADR 0220 items 1 to 3)', () => {
  const feed = { podcast_guid: 'a b' };
  assert.deepEqual(releaseMusicIndexLink(feed, { held: true }), {
    href: 'https://musicindex.org/#feed=a%20b',
    text: 'Open on musicindex.org',
  });
  assert.equal(releaseMusicIndexLink(feed, { held: false }), null);
  assert.equal(releaseMusicIndexLink(feed, { held: null }), null);
  assert.equal(releaseMusicIndexLink({ podcast_guid: null }, { held: true }), null);
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
  assert.equal(notice.lines[0].action, undefined, 'ADR 0191 item 5: a waiting line has no button. Correct moveNotice in crates/api/ui/outsidecheck.js.');
  assert.equal(notice.lines[1].action, 'Send the Podping Again', 'ADR 0191 item 4: a sent line can send the podping again. Correct moveNotice in crates/api/ui/outsidecheck.js.');
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

test('ADR 0228 items 8 and 9: each write state has Dismiss and the confirmation has no podping line', () => {
  const check = { own_url: 'https://x/feed.xml', last_write: {
    written_at: 100, index_read: false,
    announcements: [{ url: 'https://x/feed.xml', state: 'sent', finished_at: 110 }],
  } };
  const waiting = writeNotice(check, { reads: 1, nowSeconds: 120, confirmedHere: false });
  const noReply = writeNotice(check, { reads: 15, nowSeconds: 120, confirmedHere: false });
  const confirmed = writeNotice({ ...check, last_write: { ...check.last_write, index_read: true } }, { reads: 2, nowSeconds: 120, confirmedHere: true });
  for (const notice of [waiting, noReply, confirmed]) {
    assert.equal(notice.dismiss, 'Dismiss', 'ADR 0228 item 8: each state has Dismiss. Fix: set dismiss in writeNotice.');
  }
  assert.deepEqual(confirmed.lines, [], 'ADR 0228 item 9: the confirmation has no podping line. Fix: return no lines.');
  assert.equal(confirmed.action, undefined, 'ADR 0228 item 9: the confirmation has no Send the Podping Again. Fix: set no action.');
  assert.equal(noReply.lines[0].action, 'Send the Podping Again', 'ADR 0191 item 4: a sent line keeps its button. Fix: keep podpingLines.');
});

test('ADR 0175 item 1: a write notice gives its states and suppresses inapplicable notices', () => {
  const check = { own_url: 'https://x/feed.xml', last_write: {
    written_at: 100, index_read: false,
    announcements: [{ url: 'https://x/feed.xml', state: 'waiting', send_at: 110 }],
  } };
  const waiting = writeNotice(check, { reads: 1, nowSeconds: 120, confirmedHere: false });
  assert.match(waiting.strong, /^You wrote this feed at /, 'ADR 0175 item 1: show the waiting write text. Fix: use the required local-time text.');
  assert.equal(waiting.lines[0].state, 'waiting', 'ADR 0175 item 1: show the podping line. Fix: reuse the announcement state.');
  const confirmed = writeNotice({ ...check, last_write: { ...check.last_write, index_read: true } }, { reads: 2, nowSeconds: 120, confirmedHere: true });
  assert.match(confirmed.strong, /^Music Index has your write of /, 'ADR 0175 item 1: confirm a read after waiting. Fix: return the confirmation text.');
  const noReply = writeNotice(check, { reads: 15, nowSeconds: 120, confirmedHere: false });
  assert.match(noReply.strong, /^Music Index has not read your write of /, 'ADR 0175 item 1: show no reply at the read limit. Fix: return the no-reply state.');
  assert.deepEqual([waiting.kind, confirmed.kind, noReply.kind], ['waiting', 'confirmed', 'no_reply'], 'ADR 0175 item 1: each state has its kind. Fix: set kind in writeNotice.');
  assert.equal(noReply.action, 'Check Again', 'ADR 0175 item 1: offer Check Again. Fix: set the notice action.');
  assert.equal(writeNotice(null, { reads: 0, nowSeconds: 120, confirmedHere: false }), null, 'ADR 0175 item 1: no check gives no notice. Fix: return null.');
  assert.equal(writeNotice({ last_write: null }, { reads: 0, nowSeconds: 120, confirmedHere: false }), null, 'ADR 0175 item 1: no write gives no notice. Fix: return null.');
  assert.equal(writeNotice({ last_write: { ...check.last_write, index_read: null, announcements: [] } }, { reads: 0, nowSeconds: 120, confirmedHere: false }), null, 'ADR 0175 item 1: no answer and no waiting podping gives no notice. Fix: return null.');
  assert.equal(writeNotice({ index_is_move_source: true, last_move: {}, last_write: check.last_write }, { reads: 1, nowSeconds: 120, confirmedHere: false }), null, 'ADR 0175 item 1: a move suppresses the write notice. Fix: return null.');
  assert.equal(writeNotice({ ...check, last_write: { ...check.last_write, index_read: true } }, { reads: 2, nowSeconds: 120, confirmedHere: false }), null, 'ADR 0175 item 1: a first-load confirmation stays hidden. Fix: require confirmedHere.');
  assert.match(writeNotice(check, { reads: 1, nowSeconds: 401, confirmedHere: false }).strong, /^Music Index has not read/, 'ADR 0175 item 1: an old write shows no reply. Fix: use the five-minute limit.');
});

test('ADR 0175 item 1: after a sent podping, a feed that Music Index does not hold yet keeps the notice and the reads', () => {
  // rc.27 check finding 7: the index held the test album only 172 seconds
  // after the podping. Before that time, the route gave index_read null.
  const notHeld = { own_url: 'https://x/feed.xml', last_write: {
    written_at: 100, index_read: null,
    announcements: [{ url: 'https://x/feed.xml', state: 'sent', send_at: 130, finished_at: 131 }],
  } };
  const notice = writeNotice(notHeld, { reads: 2, nowSeconds: 140, confirmedHere: false });
  assert.equal(notice?.kind, 'waiting', 'ADR 0175 item 1: the notice stays until Music Index has read the write. Fix: treat index_read null after a podping as not read.');
  assert.equal(nextNoticeLoad(notHeld, 140, 2), INDEX_READ_AGAIN_MS, 'ADR 0175 item 1: the reads go on until Music Index has read the write. Fix: treat index_read null after a podping as not read.');
  assert.equal(writeNotice(notHeld, { reads: 15, nowSeconds: 140, confirmedHere: false })?.kind, 'no_reply', 'ADR 0175 item 1: the read limit gives Check Again. Fix: treat index_read null after a podping as not read.');
  assert.equal(nextNoticeLoad({ last_write: { ...notHeld.last_write, announcements: [] } }, 140, 2), null, 'ADR 0175 item 1: a write with no podping gives no reads. Fix: read again only after a podping.');
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

test('ADR 0175 item 1: a write check uses the move read limit and delay', () => {
  assert.equal(nextNoticeLoad({ last_write: { index_read: false } }, 100, 0), 20000, 'ADR 0175 item 1: read again after 20 seconds. Fix: use INDEX_READ_AGAIN_MS.');
  assert.equal(nextNoticeLoad({ last_write: { index_read: false } }, 100, 15), null, 'ADR 0175 item 1: stop after 15 reads. Fix: use INDEX_READ_AGAIN_MAX.');
  assert.equal(nextNoticeLoad({ last_write: { index_read: false, written_at: 100 } }, 401, 1), null, 'ADR 0175 item 1: an old write gets one request on page load. Fix: do not schedule another read after five minutes.');
  assert.equal(nextNoticeLoad({ last_write: { index_read: true } }, 100, 0), null, 'ADR 0175 item 1: a read write needs no poll. Fix: return null.');
  const old = { last_write: { index_read: false, written_at: 100, announcements: [] } };
  assert.equal(nextNoticeLoad(old, 401, 0, 400), 20000, 'ADR 0175 item 1: Check Again starts the reads again for an old write. Fix: count the age from the later of written_at and since.');
  assert.equal(writeNotice(old, { reads: 1, nowSeconds: 401, confirmedHere: false, since: 400 }).kind, 'waiting', 'ADR 0175 item 1: after Check Again an old write waits again. Fix: count the age from the later of written_at and since.');
  const writeWaits = { last_write: { index_read: false, written_at: 100, announcements: [{ state: 'waiting', send_at: 110 }] } };
  assert.equal(nextNoticeLoad(writeWaits, 100, 15), 15000, 'ADR 0175 item 1: a waiting podping loads again after the read limit. Fix: apply the read limit only to the 20-second reads.');
  const source = readFileSync(new URL('../ui/releasemove.js', import.meta.url), 'utf8');
  const both = { last_move: { announcements: [{ state: 'not_sent' }] }, last_write: { index_read: null, written_at: 100, announcements: [{ state: 'waiting', send_at: 110 }] } };
  assert.equal(nextNoticeLoad(both, 100, 1), 15000, 'ADR 0175 item 1: a waiting podping of a write loads again when the feed has an older move. Fix: read the announcements of last_move and of last_write.');
  const own = readFileSync(new URL('../ui/outsidecheck.js', import.meta.url), 'utf8');
  assert.doesNotMatch(own, /reads >= \d/, 'ADR 0175 item 1: one read-limit definition. Fix: use INDEX_READ_AGAIN_MAX from releasemove.js.');
  assert.equal((source.match(/export const INDEX_READ_AGAIN_MS\s*=/g) || []).length, 1, 'ADR 0175 item 1: one delay definition. Fix: keep the shared constant in releasemove.js.');
  assert.equal((source.match(/export const INDEX_READ_AGAIN_MAX\s*=/g) || []).length, 1, 'ADR 0175 item 1: one read-limit definition. Fix: keep the shared constant in releasemove.js.');
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
  assert.equal(stateMark(feeds, results).tone, 'warning');
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

test('A single-feed mark shows outside changes and hides an up-to-date feed (ADR 0165 item 2)', () => {
  assert.deepEqual(
    stateMark([feedOf(1)], { 1: read('changed') }),
    { kind: 'outside', text: 'Changed outside Studio', icon: 'cloud-alert', tone: 'warning' },
  );
  assert.equal(stateMark([feedOf(1)], { 1: read() }), null);
});

test('The open feed outline uses its single-feed state mark (ADR 0165 items 2 and 6)', () => {
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = source.indexOf('function Outline');
  const end = source.indexOf('function LevelPage', start);
  const outline = source.slice(start, end);
  const fix = 'ADR 0165 items 2 and 6: the open feed outline shows its state mark. Fix Outline in crates/api/ui/app.js.';
  assert.match(outline, /useStateMark\(feeds\.filter\(\(item\) => item\.id === feed\.id\)\)/, fix);
  assert.match(outline, /<\$\{StateMark\} mark=\$\{feedMark\} \/>/, fix);
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

test('ADR 0211: outside warnings use the Warning tone and the move actions', () => {
  assert.equal(outsideMark({ state: 'changed' }).tone, 'warning', 'ADR 0211 item 20: outside changes use Warning. Fix outsideMark in crates/api/ui/outsidecheck.js.');
  assert.equal(outsideMark({ state: 'moved' }).tone, 'warning', 'ADR 0211 item 20: outside moves use Warning. Fix outsideMark in crates/api/ui/outsidecheck.js.');
  const notice = outsideNotice({ state: 'moved', new_url: 'https://new.example/a.xml', new_host: 'New host', connected: true });
  assert.deepEqual([notice.strong, notice.text, notice.host, notice.after], ['A different Studio moved this feed.', 'New address', 'New host', 'The old address has a pointer file.'], 'ADR 0211 item 6: give the warning text and new host. Fix outsideNotice in crates/api/ui/outsidecheck.js.');
  assert.deepEqual([notice.follow, notice.difference, notice.keep], [true, true, true], 'ADR 0211 items 7 and 8: give each button in order. Fix outsideNotice in crates/api/ui/outsidecheck.js.');
  assert.equal(outsideNotice({ state: 'moved', new_url: 'https://x/a.xml', new_host: 'New Host', connected: true }).back, true,
    'ADR 0211 item 17: Move It Back shows when a connected file host gives the new address. Fix outsideNotice in crates/api/ui/outsidecheck.js.');
  assert.equal(outsideNotice({ state: 'moved', new_url: 'https://x/a.xml', new_host: null, connected: true }).back, false,
    'ADR 0211 item 18: with no connected file host for the new address, a return cannot write its pointer file. Fix outsideNotice in crates/api/ui/outsidecheck.js.');
  assert.equal(outsideNotice({ state: 'changed' }).tone, 'warning', 'ADR 0211 item 22: a change notice uses Warning. Fix outsideNotice in crates/api/ui/outsidecheck.js.');
  assert.equal(stateMark([feedOf(1)], { 1: read('changed') }).tone, 'warning', 'ADR 0211 item 21: the panel mark uses Warning. Fix STATE_MARKS in crates/api/ui/outsidecheck.js.');
});

test('ADR 0211: read-only host notice gives its text and action', () => {
  const fix = 'Fix readOnlyHostNotice in crates/api/ui/outsidecheck.js.';
  assert.deepEqual(readOnlyHostNotice(feedOf(1, { followed_move: true })),
    { text: 'This Studio is not connected to the file host of this address. Connect it to edit the feed here.', button: 'Connect a File Host…', use: false },
    `ADR 0211 item 15: a read-only feed that followed a move has the notice. ${fix}`);
  assert.deepEqual(readOnlyHostNotice(feedOf(1, { host_for_address: true })),
    { text: 'A connected file host gives the address of this feed. Use it to edit the feed here.', button: 'Use This File Host', use: true },
    `ADR 0211 item 16: a connected file host for the address gives Use This File Host. ${fix}`);
  assert.equal(readOnlyHostNotice(feedOf(1)), null,
    `ADR 0211 item 15: a different read-only feed has no notice. ${fix}`);
  assert.equal(readOnlyHostNotice(feedOf(1, { connector_id: 3 })), null);
});

test('ADR 0211: the feed notice follows a move and removes retired wording', () => {
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(source, /outside\/follow/);
  assert.doesNotMatch(source, /moved outside Studio/i);
});
