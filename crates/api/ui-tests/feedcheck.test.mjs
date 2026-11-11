import test from 'node:test';
import assert from 'node:assert/strict';
import {
  tabLabel, checkCount, checkRows, unreadableRows, repairLabel, hasShowGrant, checkTexts,
  CHECK_LEAD, CHECK_NONE, CHECK_GRANT, CHECK_READ_ONLY, CHECK_UNREADABLE,
} from '../ui/feedcheck.js';

const finding = (kind, repair, extra = {}) => ({
  kind, level: 'notice', title: `T ${kind}`, reason: `R ${kind}`, entry: null, repair, url: null, ...extra,
});
const check = (findings, extra = {}) => ({ revision: 'r1', read_only: false, read: true, unreadable: [], findings, ...extra });

test('the tab label has the count, and no count for no finding', () => {
  const owner = 'ADR 0174 item 17: check tabLabel in feedcheck.js.';
  assert.equal(tabLabel(5), 'Check 5', owner);
  assert.equal(tabLabel(1), 'Check 1', owner);
  assert.equal(tabLabel(0), 'Check', owner);
  assert.equal(tabLabel(undefined), 'Check', owner);
  assert.equal(checkCount(null), 0, owner);
  assert.equal(checkCount(check([finding('no_language', 'select_a_language', { level: 'question' })])), 1, owner);
});

test('the model gives one row for each finding with its level and its button', () => {
  const owner = 'ADR 0174 items 7 and 19: check checkRows in feedcheck.js.';
  const rows = checkRows(check([
    finding('no_self_link', 'add_the_link'),
    finding('http_file', 'use_the_https_address', { entry: 'g-1', url: 'http://a.example/x.mp3' }),
    finding('no_file_size', 'read_the_size', { entry: 'g-1' }),
    finding('no_duration', 'open_the_track', { entry: 'g-2' }),
    finding('no_language', 'select_a_language', { level: 'question' }),
    finding('move_not_confirmed', null),
  ]));
  assert.deepEqual(rows.map((r) => r.button), [
    'Add the Link', 'Use the https Address', 'Read the Size', 'Open the Track', 'Select a Language', null,
  ], owner);
  assert.deepEqual(rows.map((r) => r.level), ['Notice', 'Notice', 'Notice', 'Notice', 'Question', 'Notice'], owner);
  assert.equal(rows[0].title, 'T no_self_link', owner);
  assert.equal(rows[0].reason, 'R no_self_link', owner);
  assert.deepEqual(rows[1].body, { kind: 'http_file', entry: 'g-1', url: 'http://a.example/x.mp3' }, owner);
  assert.equal(rows[1].action, 'repair', owner);
  assert.equal(rows[3].action, 'open-track', owner);
  assert.equal(rows[3].track, 'g-2', owner);
  assert.equal(rows[3].body, null, owner);
  assert.equal(rows[4].action, 'open-language', owner);
  assert.equal(rows[5].action, null, owner);
});

test('a read-only feed gives rows with no button', () => {
  const owner = 'ADR 0174 item 20: check checkRows in feedcheck.js.';
  const rows = checkRows(check([finding('no_self_link', null), finding('no_language', null, { level: 'question' })], { read_only: true }));
  assert.equal(rows.length, 2, owner);
  assert.deepEqual(rows.map((r) => r.button), [null, null], owner);
  assert.deepEqual(rows.map((r) => r.action), [null, null], owner);
});

test('Open the Track has no button for a track with no GUID', () => {
  const rows = checkRows(check([finding('no_duration', 'open_the_track')]));
  assert.equal(rows[0].button, null, 'ADR 0174 item 7: check checkRows in feedcheck.js.');
});

test('the repair labels follow the core names', () => {
  assert.equal(repairLabel('use_the_public_address'), 'Use the Public Address');
  assert.equal(repairLabel(null), null);
  assert.equal(repairLabel('other'), null);
});

test('each address that a read did not reach gives a notice', () => {
  const rows = unreadableRows(check([], { unreadable: ['https://a.example/x.mp3', 'https://b.example/y.mp3'] }));
  assert.deepEqual(rows.map((r) => r.title), ['https://a.example/x.mp3', 'https://b.example/y.mp3'], 'ADR 0174 item 16');
  assert.equal(rows[0].reason, CHECK_UNREADABLE);
  assert.equal(CHECK_UNREADABLE, 'Studio could not read this address.');
  assert.deepEqual(unreadableRows(null), []);
});

test('the texts of the page follow items 19 to 21', () => {
  const owner = 'ADR 0174 items 19 to 21: check checkTexts in feedcheck.js.';
  const rows = check([finding('no_self_link', 'add_the_link')]);
  assert.deepEqual(checkTexts(rows), { lead: CHECK_LEAD, readOnly: null, grant: null, none: null, reading: null }, owner);
  assert.equal(checkTexts(check([])).none, CHECK_NONE, owner);
  assert.equal(checkTexts(check([])).lead, null, owner);
  assert.equal(checkTexts(rows, { grant: true }).grant, CHECK_GRANT, owner);
  assert.equal(checkTexts(check([finding('no_self_link', null)], { read_only: true })).readOnly, CHECK_READ_ONLY, owner);
  assert.equal(checkTexts(check([finding('no_self_link', null)], { read_only: true })).lead, null, owner);
  assert.equal(checkTexts(check([], { read_only: true }), { grant: true }).grant, null, owner);
  assert.equal(checkTexts(null).none, null, owner);
  assert.notEqual(checkTexts(null).reading, null, owner);
  assert.equal(CHECK_LEAD, 'Studio does not change the feed until you select a button. Each change goes to the draft, and you review it before the write.');
  assert.equal(CHECK_NONE, 'Studio found no problem in this feed.');
});

test('a show grant is active until it ends', () => {
  const live = [{ item: { guid: 'live-1' }, start: '2026-11-10T10:00:00Z', end: '2026-11-10T11:00:00Z' }];
  const now = Date.parse('2026-11-10T12:00:00Z') / 1000;
  const owner = 'ADR 0054 item 14: check hasShowGrant in feedcheck.js.';
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: null }], live, now), true, owner);
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: 5 }], live, now), false, owner);
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: null }], live, now + 3 * 86400), false, owner);
  assert.equal(hasShowGrant([], live, now), false, owner);
  assert.equal(hasShowGrant(null, live, now), false, owner);
});
