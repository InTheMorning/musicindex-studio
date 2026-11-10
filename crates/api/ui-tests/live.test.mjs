// The live item models (ADR 0054, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  liveKey, isLiveItem, canRecord, hasRecording, statusLabel, nextAction, showSummary,
  grantLimit, grantActive, grantSummary, grantWriteText,
  hostUsesSigner, grantBoxState,
  OTHER_TYPE, typeIsSelected, typeMenuState, filledType, checkLine, checkTypeWarning,
} from '../ui/live.js';

const link = { href: 'https://example.com/show', label: 'Watch' };
const live = {
  _cid: 4, status: 'pending', start: '2026-11-01T10:00:00Z', end: null,
  item: { guid: 'g-1', title: 'Show' }, live_value: null, content_links: [link],
};

test('the key of a live item is its GUID, or its client ID with no GUID', () => {
  assert.equal(liveKey(live), 'g-1');
  assert.equal(liveKey({ ...live, item: { title: 'x' } }), 'cid-4');
  assert.equal(isLiveItem(live, 'cid-4'), true);
  assert.equal(isLiveItem(live, 'g-1'), true);
  assert.equal(isLiveItem(live, 'g-2'), false);
});

test('the status labels', () => {
  assert.equal(statusLabel('pending'), 'Planned');
  assert.equal(statusLabel('live'), 'Live now');
  assert.equal(statusLabel('ended'), 'Ended');
});

test('the next action follows the status', () => {
  assert.deepEqual(nextAction('pending'), { label: 'Go Live', status: 'live' });
  assert.deepEqual(nextAction('live'), { label: 'End the Show', status: 'ended' });
  assert.equal(nextAction('ended'), null);
  assert.equal(nextAction('soon'), null);
});

test('the summary line gives the status and the start time', () => {
  assert.equal(showSummary(live, '10/26/2026, 8:00 PM'), 'Planned · starts 10/26/2026, 8:00 PM');
  assert.equal(showSummary({ ...live, status: 'live' }, ''), 'Live now');
});

test('only an ended show can have a recording', () => {
  assert.equal(canRecord({ ...live, status: 'ended' }), true);
  assert.equal(canRecord(live), false);
  assert.equal(canRecord({ ...live, status: 'live' }), false);
});

test('a recording is an enclosure with an audio type', () => {
  const enclosure = (media_type) => ({ ...live, item: { ...live.item, enclosure: { url: 'https://example.com/a', length: '1', media_type } } });
  assert.equal(hasRecording(live), false);
  assert.equal(hasRecording(enclosure('application/x-mpegURL')), false);
  assert.equal(hasRecording(enclosure('audio/mpeg')), true);
  assert.equal(hasRecording({ ...live, item: { ...live.item, enclosure: { url: null, length: null, media_type: 'audio/mpeg' } } }), false);
});

// ---- Show grants ----

const stamp = (s) => `t${s}`;
const day = 24 * 60 * 60;
const startSeconds = Date.parse('2026-11-01T10:00:00Z') / 1000;

test('the limit of a grant is 24 hours after the end, or after the start with no end', () => {
  assert.equal(grantLimit(live), startSeconds + day);
  assert.equal(grantLimit({ ...live, end: '' }), startSeconds + day);
  assert.equal(grantLimit({ ...live, end: '2026-11-01T12:00:00Z' }), startSeconds + 2 * 3600 + day);
  assert.equal(grantLimit({ ...live, start: '2021-09-26T07:30:00.000-0600', end: null }),
    Date.parse('2021-09-26T13:30:00Z') / 1000 + day);
});

test('the limit of a grant is null when a time cannot be read', () => {
  assert.equal(grantLimit({ ...live, start: null }), null);
  assert.equal(grantLimit({ ...live, start: 'soon' }), null);
  assert.equal(grantLimit({ ...live, start: '2026-11-01T10:00:00' }), null);
  assert.equal(grantLimit({ ...live, end: 'later' }), null);
});

test('the summary of a grant covers each case', () => {
  const open = { id: 1, item_guid: 'g-1', created_at: 5, ended_at: null, writes: [] };
  const now = startSeconds;
  assert.equal(grantSummary(null, live, now, stamp), 'No grant.');
  assert.equal(grantSummary(open, live, now, stamp, 'Live Publisher'),
    `Live Publisher can start and end this show until ${stamp(startSeconds + day)}.`);
  assert.equal(grantSummary(open, live, now, stamp),
    `The app can start and end this show until ${stamp(startSeconds + day)}.`);
  assert.equal(grantSummary({ ...open, ended_at: 77 }, live, now, stamp), 'The grant ended at t77.');
  assert.equal(grantSummary(open, live, startSeconds + day + 1, stamp),
    `The grant ended at ${stamp(startSeconds + day)}.`);
  assert.equal(grantSummary(open, { ...live, start: null }, now, stamp, 'Live Publisher'),
    'Live Publisher can start and end this show.');
  assert.equal(grantActive(open, live, now), true);
  assert.equal(grantActive(open, live, startSeconds + day), false);
  assert.equal(grantActive({ ...open, ended_at: 1 }, live, now), false);
  assert.equal(grantActive(null, live, now), false);
});

test('the text of a granted write names the app, the change and the time', () => {
  const write = (change) => ({ app_name: 'Show tool', change, written_at: 9 });
  assert.equal(grantWriteText(write('go_live'), stamp), 'Show tool · Went live · t9');
  assert.equal(grantWriteText(write('end'), stamp), 'Show tool · Ended the show · t9');
  assert.equal(grantWriteText(write('set_live_value'), stamp), 'Show tool · Set the live value · t9');
  assert.equal(grantWriteText(write('remove_live_value'), stamp), 'Show tool · Removed the live value · t9');
});

test('a refused write gives its cause (ADR 0110 item 43, screen 11)', () => {
  const refused = (change, cause) => ({ app_name: 'Live Publisher', change, written_at: 9, cause });
  assert.equal(grantWriteText(refused('go_live', 'draft_with_changes'), stamp),
    'Live Publisher · Not started: the feed had a draft with changes · t9');
  assert.equal(grantWriteText(refused('end', 'grant_ended'), stamp),
    'Live Publisher · Not ended: the grant had ended · t9');
  assert.equal(grantWriteText(refused('set_live_value', 'live_value_invalid'), stamp),
    'Live Publisher · Live value not set: Studio refused the live value · t9');
  assert.equal(grantWriteText(refused('remove_live_value', 'vault_not_open'), stamp),
    'Live Publisher · Live value not removed: the grant vault did not open · t9');
  assert.equal(grantWriteText({ ...refused('go_live', null) }, stamp), 'Live Publisher · Went live · t9');
});

test('the grant box is disabled for a host with My signer, with the reason', () => {
  const state = grantBoxState(true);
  assert.equal(state.disabled, true);
  assert.equal(
    state.note,
    'Connected apps cannot write to a file host with My signer, because no person is present to sign.',
  );
  assert.deepEqual(grantBoxState(false), { disabled: false, note: null });
});

test('an nsite or Blossom host uses My signer unless its key is in the vault', () => {
  const row = (kind, config) => ({ id: 1, kind, config: JSON.stringify(config) });
  assert.equal(hostUsesSigner(row('nsite', { signing: 'signer' })), true);
  assert.equal(hostUsesSigner(row('blossom', { signing: 'signer' })), true);
  assert.equal(hostUsesSigner(row('nsite', { signing: 'vault' })), false);
  assert.equal(hostUsesSigner(row('sftp', { signing: 'signer' })), false);
  assert.equal(hostUsesSigner(row('s3', {})), false);
  assert.equal(hostUsesSigner({ kind: 'nsite', config: 'not json' }), false);
  assert.equal(hostUsesSigner(undefined), false);
});

// ---- The stream type and the stream check (ADR 0162) ----

const MENU = [
  { label: 'HLS', media_type: 'application/x-mpegURL' },
  { label: 'MP3', media_type: 'audio/mpeg' },
  { label: 'DASH', media_type: 'application/dash+xml' },
];

test('a type that is not blank at page load counts as selected', () => {
  assert.equal(typeIsSelected('audio/mpeg'), true);
  assert.equal(typeIsSelected(''), false);
  assert.equal(typeIsSelected('  '), false);
  assert.equal(typeIsSelected(null), false);
});

test('the page fills the type until the user selects one', () => {
  // Not selected: the suggestion of each new address replaces the type.
  let type = null;
  type = filledType(type, false, 'application/x-mpegURL');
  assert.equal(type, 'application/x-mpegURL');
  type = filledType(type, false, 'audio/mpeg');
  assert.equal(type, 'audio/mpeg');
  // An address with no known extension gives no suggestion: the type stays.
  assert.equal(filledType(type, false, null), 'audio/mpeg');
  // The type that a check gives fills too.
  assert.equal(filledType(type, false, 'audio/aac'), 'audio/aac');
  // Selected: nothing replaces the type.
  assert.equal(filledType('audio/ogg', true, 'application/x-mpegURL'), 'audio/ogg');
  assert.equal(filledType(null, true, 'audio/mpeg'), null);
});

test('the menu shows a menu type, and a type that is not in it shows as Other with its text', () => {
  assert.deepEqual(typeMenuState('audio/mpeg', MENU), { value: 'audio/mpeg', showText: false, text: 'audio/mpeg' });
  assert.equal(typeMenuState('application/x-mpegurl', MENU).value, 'application/x-mpegURL');
  assert.deepEqual(typeMenuState('audio/flac', MENU), { value: OTHER_TYPE, showText: true, text: 'audio/flac' });
  assert.deepEqual(typeMenuState(null, MENU), { value: '', showText: false, text: '' });
  // After the user picks Other, the text field shows, also with no text.
  assert.deepEqual(typeMenuState(null, MENU, true), { value: OTHER_TYPE, showText: true, text: '' });
  assert.equal(typeMenuState('audio/mpeg', MENU, true).value, OTHER_TYPE);
});

test('the result line of a check gives the type and the time, or the cause', () => {
  assert.deepEqual(
    checkLine({ live: true, server_type: 'audio/mpeg', cause: null }, '14:05'),
    { warning: false, text: 'Live · audio/mpeg · checked 14:05' },
  );
  assert.deepEqual(
    checkLine({ live: false, server_type: null, cause: 'The stream server answered with status 404.' }, '14:05'),
    { warning: true, text: 'The stream server answered with status 404.' },
  );
  assert.equal(checkLine(null, '14:05'), null);
});

test('a stream check warns about a selected type that does not agree (ADR 0162 item 9)', () => {
  const result = {
    live: true, server_type: 'audio/mpeg', fill_type: 'audio/mpeg', fill_label: 'MP3',
    type_warning: 'The stream server sends audio/mpeg, but the stream type is HLS.',
  };
  assert.deepEqual(checkTypeWarning(result, true), {
    text: 'The stream server sends audio/mpeg, but the stream type is HLS.',
    fillType: 'audio/mpeg', fillLabel: 'MP3',
  });
  assert.equal(checkTypeWarning(result, false), null);
  assert.equal(checkTypeWarning({ ...result, type_warning: null }, true), null);
});
