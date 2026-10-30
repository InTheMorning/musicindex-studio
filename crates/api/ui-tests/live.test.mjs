// The live item models (ADR 0054, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  liveKey, isLiveItem, canRecord, hasRecording, statusLabel, nextAction, showSummary, liveProblems,
  grantLimit, grantActive, grantSummary, grantWriteText,
  hostUsesSigner, grantBoxState,
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
  assert.deepEqual(nextAction('pending'), { label: 'Go live', status: 'live' });
  assert.deepEqual(nextAction('live'), { label: 'End the show', status: 'ended' });
  assert.equal(nextAction('ended'), null);
  assert.equal(nextAction('soon'), null);
});

test('the summary line gives the status and the start time', () => {
  assert.equal(showSummary(live, '10/26/2026, 8:00 PM'), 'Planned · starts 10/26/2026, 8:00 PM');
  assert.equal(showSummary({ ...live, status: 'live' }, ''), 'Live now');
});

test('the problems have the texts of the core', () => {
  assert.deepEqual(liveProblems(live), []);
  assert.deepEqual(liveProblems({ ...live, content_links: [] }), ['This show has no content link.']);
  assert.deepEqual(
    liveProblems({ ...live, start: ' ', status: 'soon', content_links: [] }),
    [
      'This show has no start time.',
      'This show has no content link.',
      'This show has a status that is not pending, live or ended.',
    ],
  );
  assert.deepEqual(liveProblems({ ...live, start: null }), ['This show has no start time.']);
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
  assert.equal(grantSummary(open, live, now, stamp),
    `Connected apps can start and end this show until ${stamp(startSeconds + day)}.`);
  assert.equal(grantSummary({ ...open, ended_at: 77 }, live, now, stamp), 'The grant ended at t77.');
  assert.equal(grantSummary(open, live, startSeconds + day + 1, stamp),
    `The grant ended at ${stamp(startSeconds + day)}.`);
  assert.equal(grantSummary(open, { ...live, start: null }, now, stamp),
    'Connected apps can start and end this show.');
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
