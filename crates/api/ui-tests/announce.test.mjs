// The announcement models (ADR 0042). Each test checks one text of task 031d.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { announceSkipLine, announceStatus, announceHint, isWaiting, ANNOUNCE_FOOTER_LABEL } from '../ui/announce.js';
import { localTime } from '../ui/outsidecheck.js';

test('the hint of the announcement choice gives the send time after a wait of Music Index (ADR 0190 items 6 and 7)', () => {
  const owner = 'ADR 0190 items 6 and 7. Correct announceHint in crates/api/ui/announce.js.';
  const until = 1_800_000_000;
  assert.equal(announceHint(true, { state: 'waits', until, too_far: false }),
    `Studio sends it at ${localTime(until + 30)}, after the wait of Music Index.`, owner);
  for (const wait of [null, { state: 'follows' }, { state: 'unknown' }, { state: 'waits', until, too_far: true }]) {
    assert.equal(announceHint(true, wait), 'Studio sends it 30 seconds after the write.', owner);
  }
  assert.equal(announceHint(false, { state: 'waits', until, too_far: false }),
    'No index learns about this write until it reads the feed by itself.', owner);
});

const at = (seconds) => `T${seconds}`;
const entry = (fields) => ({
  id: 1, feed_id: 1, url: 'https://a.example/feed.xml', podping: false, state: 'waiting',
  attempts: 0, created_at: 100, next_at: 130, finished_at: null, last_error: null, ...fields,
});

test('the footer of the review sheet gives the label and the cause of no podping (ADR 0205 item 19)', () => {
  const fix = 'ADR 0205 item 19. Fix announceSkipLine in crates/api/ui/announce.js.';
  assert.equal(ANNOUNCE_FOOTER_LABEL, 'Send a podping', 'ADR 0205 item 19: use the footer label.');
  assert.equal(announceSkipLine({ announcements: [], announce_skip: 'no_address' }), 'No podping: this feed has no public address.', fix);
  assert.equal(announceSkipLine({ announcements: [], announce_skip: 'feed_deleted' }), 'No podping: this write deletes the feed file.', fix);
  for (const diff of [{ announcements: [], announce_skip: 'off' }, { announcements: [], announce_skip: null }, {}, null]) {
    assert.equal(announceSkipLine(diff), null, `The check box gives the choice, so no line repeats it. ${fix}`);
  }
});

test('announceStatus with no entry', () => {
  assert.equal(announceStatus([], at), 'No announcement yet.');
});

test('announceStatus for a waiting or sending entry with no attempt', () => {
  const text = 'Waiting. Studio sends it at T130.';
  assert.equal(announceStatus([entry({})], at), text);
  assert.equal(announceStatus([entry({ state: 'sending' })], at), text);
});

test('announceStatus for a waiting entry after an attempt', () => {
  assert.equal(
    announceStatus([entry({ attempts: 2, next_at: 400, last_error: 'timeout' })], at),
    'Waiting. Studio tries again at T400. Last error: timeout');
});

test('announceStatus for sent and failed entries', () => {
  assert.equal(announceStatus([entry({ state: 'sent', finished_at: 140 })], at), 'Sent at T140.');
  assert.equal(
    announceStatus([entry({ state: 'failed', finished_at: 150, last_error: 'HTTP 500' })], at),
    'Failed at T150: HTTP 500');
});

test('announceStatus gives each address of the last write on its own line', () => {
  const entries = [
    entry({ id: 3, url: 'https://b.example/f.xml', state: 'sent', finished_at: 140 }),
    entry({ id: 2, url: 'https://a.example/f.xml', state: 'failed', finished_at: 141, last_error: 'no' }),
    entry({ id: 1, url: 'https://a.example/f.xml', state: 'sent', finished_at: 50, created_at: 40 }),
  ];
  assert.equal(announceStatus(entries, at),
    'https://b.example/f.xml: Sent at T140.\nhttps://a.example/f.xml: Failed at T141: no');
});

test('isWaiting is true for the states waiting and sending only', () => {
  assert.equal(isWaiting([]), false);
  assert.equal(isWaiting([entry({ state: 'waiting' })]), true);
  assert.equal(isWaiting([entry({ state: 'sending' })]), true);
  assert.equal(isWaiting([entry({ state: 'sent' })]), false);
  assert.equal(isWaiting([entry({ state: 'failed' })]), false);
  assert.equal(isWaiting([entry({ state: 'sent' }), entry({ state: 'waiting' })]), false);
});
