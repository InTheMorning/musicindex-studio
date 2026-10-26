// The live item models (ADR 0054, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  liveKey, isLiveItem, statusLabel, nextAction, showSummary, liveProblems,
  startToIso, isoToLocalInput,
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

test('the start time converts to ISO text and back', () => {
  assert.equal(startToIso(''), null);
  assert.equal(startToIso('not a time'), null);
  const iso = startToIso('2026-11-01T20:30');
  assert.match(iso, /^2026-1[01]-\d\dT\d\d:\d\d:00\.000Z$/);
  assert.equal(isoToLocalInput(iso), '2026-11-01T20:30');
  assert.equal(isoToLocalInput('x'), '');
  assert.equal(isoToLocalInput(null), '');
});
