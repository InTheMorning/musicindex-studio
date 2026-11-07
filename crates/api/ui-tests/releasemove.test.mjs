// The words of a move of a release (ADR 0148).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { outcomeText, operationLine } from '../ui/recipients.js';
import { changeTitle, changeRow, afterDiscard } from '../ui/people.js';
import { planRows, stayRows, deleteLabel, waitingView, readyNotice, partMoveNotice } from '../ui/releasemove.js';

const hosts = [{ id: 1, name: 'Old Host' }, { id: 2, name: 'New Host' }];
const empty = { count: 0, bytes: 0 };
const plan = {
  connector_id: 2,
  feed: { moves: true, from_connector_id: 1, stays: null },
  audio: { count: 12, bytes: 94_000_000 },
  images: { count: 1, bytes: 6_000_000 },
  other: empty,
  download: { count: 2, bytes: 10 },
  on_target: 0,
  from_connector_ids: [1],
  stays: [{ name: 'https://x/c.jpg', reason: 'not_in_library' }],
};

test('planRows names the feed file and each part that moves (ADR 0148 item 2)', () => {
  const rows = planRows(plan, hosts);
  assert.deepEqual(rows[0], { label: 'Feed file', value: 'Old Host → New Host' });
  assert.equal(rows[1].label, '12 audio files');
  assert.match(rows[1].value, /^Old Host → New Host · /);
  assert.equal(rows[2].label, '1 image');
  assert.deepEqual(rows[3], { label: '2 files not in the library', value: 'Studio downloads them first' });
});

test('planRows says that a Blossom host keeps no feed file (ADR 0148 item 3)', () => {
  const rows = planRows({ ...plan, feed: { moves: false, from_connector_id: 1, stays: 'blossom' }, audio: empty, images: empty, download: empty }, hosts);
  assert.deepEqual(rows, [{ label: 'Feed file', value: 'Stays. New Host cannot keep a feed file.' }]);
});

test('stayRows gives the cause of each file that does not move', () => {
  assert.deepEqual(stayRows(plan), [{ name: 'https://x/c.jpg', text: 'Not in the library, so it does not move.' }]);
});

test('deleteLabel names the one file host, or no name for many', () => {
  assert.equal(deleteLabel(plan, hosts), 'Delete the files on Old Host when the move is complete');
  assert.equal(deleteLabel({ ...plan, from_connector_ids: [1, 3] }, hosts), 'Delete the files on the file hosts from before when the move is complete');
});

const move = { records: [1, 2], connector_id: 1, to_connector_id: 2, wait_days: 30, days_passed: 12, index: 'follows', ready: false };

test('waitingView gives the three lines (ADR 0148 item 8)', () => {
  const view = waitingView(move, hosts);
  assert.equal(view.title, 'Move to New Host');
  assert.equal(view.summary, 'Waiting. 2 files stay on Old Host until the checks pass.');
  assert.deepEqual(view.rows.map((r) => r.value), ['✓ All 2 are on New Host', '✓ Follows the new feed address', '12 of 30 days']);
  assert.equal(waitingView({ ...move, index: 'unknown' }, hosts).rows[1].value, 'No answer. The days decide.');
  assert.equal(waitingView({ ...move, index: 'different' }, hosts).rows[1].value, 'Gives a different feed address');
});

test('readyNotice names the count and the file host (ADR 0148 item 10)', () => {
  assert.deepEqual(readyNotice(move, hosts), {
    strong: 'The move to New Host is complete.',
    text: '2 old files are ready to delete from Old Host.',
  });
});

test('partMoveNotice names the count of the files that stay (ADR 0148 item 15)', () => {
  assert.deepEqual(partMoveNotice(25, 2, 1, hosts), {
    strong: 'The feed moves to New Host. 25 files stay on Old Host.',
    text: 'The feed still works while Old Host keeps these files.',
  });
});

const moveAll = { kind: 'move_releases', from_name: 'Old Host', to_name: 'New Host' };

test('a move change set has its own words (ADR 0148 item 17)', () => {
  assert.equal(changeTitle(moveAll), 'Move All Releases');
  assert.match(operationLine(moveAll, ''), /^Move the releases from Old Host to New Host\./);
  assert.equal(outcomeText({ outcome: 'changed', channel: true, items: 3 }, moveAll).text, 'Moves · Feed file · 3 files');
  assert.equal(outcomeText({ outcome: 'not_changed', cause: { cause: 'has_feed_move' } }, moveAll).text,
    'Not changed · The draft already moves the feed file. Write or discard that draft first.');
  assert.equal(changeRow({ operation: moveAll, feeds: [] }).title, 'Move All Releases · Old Host to New Host');
  assert.deepEqual(afterDiscard({ operation: moveAll }), { name: 'hosts' });
});
