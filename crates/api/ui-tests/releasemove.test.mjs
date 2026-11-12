// The words of a move of a release (ADR 0148).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { outcomeText, operationLine } from '../ui/recipients.js';
import { changeTitle, changeRow, afterDiscard } from '../ui/people.js';
import { MOVE_SCOPES, fileHostRows, planRows, stayRows, deleteLabel, waitingView, readyNotice, partMoveNotice, existsRefusalView, addDisabled, replacesCopyNote, replaceWarning, REPLACE_LABEL } from '../ui/releasemove.js';

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

test('the Files page gives two text rows for the file hosts and no select menu', () => {
  const feed = { connector_id: 1, public_url: 'https://old.example/album.xml' };
  const rows = fileHostRows(feed, { files_connector_id: null, feed_move: null }, hosts);
  assert.deepEqual(rows, [
    { label: 'Feed file', host: 'Old Host', address: 'https://old.example/album.xml' },
    { label: 'Media files', host: 'Old Host', address: null },
  ]);
  for (const row of rows) assert.deepEqual(Object.keys(row), ['label', 'host', 'address']);
  const moved = fileHostRows(feed, { files_connector_id: 2, feed_move: { connector_id: 2 } }, hosts);
  assert.equal(moved[0].host, 'New Host');
  assert.equal(moved[1].host, 'New Host');
});

test('What moves has the three values of ADR 0180, and the first is the start value', () => {
  assert.deepEqual(MOVE_SCOPES.map((scope) => scope.label), [
    'Feed and media files', 'Feed file only', 'Media files only']);
  assert.deepEqual(MOVE_SCOPES.map((scope) => scope.value), ['both', 'feed', 'files']);
});

test('planRows names the new files that go to the target at the write (ADR 0177 item 4)', () => {
  const rows = planRows({ ...plan, new_files: 2 }, hosts);
  assert.ok(rows.some((r) => r.label === '2 new files' && r.value.endsWith('at the write')), JSON.stringify(rows));
  assert.ok(!planRows(plan, hosts).some((r) => r.label.includes('new file')));
});

test('planRows gives no count of zero file hosts when no known host has the files', () => {
  const rows = planRows({ ...plan, from_connector_ids: [] }, hosts);
  assert.ok(rows.every((r) => !r.value.startsWith('0 file hosts')), JSON.stringify(rows));
  assert.ok(rows.some((r) => r.value.startsWith('Their addresses →')), JSON.stringify(rows));
});

const exists = {
  code: 'file_exists',
  message: 'a file exists at TestAlbum/test.xml on the new file host already',
  details: { path: 'TestAlbum/test.xml', connector_id: 2, host: 'New Host', title: 'Other Album', guid: 'g-9', size: 2048 },
};

test('the refusal of a different file at the feed path gives "Replace the File" and no add (ADR 0178 items 5 and 8)', () => {
  const view = existsRefusalView(exists);
  assert.equal(view.replace, 'Replace the File');
  assert.equal(REPLACE_LABEL, 'Replace the File');
  assert.match(view.text, /^New Host has a file at TestAlbum\/test\.xml/);
  assert.deepEqual(view.facts.map((row) => row.label), ['File', 'Title', 'GUID', 'Size']);
  assert.equal(view.facts[1].value, 'Other Album');
  assert.equal(addDisabled(plan, false, exists), true, 'ADR 0178 item 8: no add while the refusal shows');
  assert.equal(addDisabled(plan, false, null), false);
  assert.equal(addDisabled(plan, false, { code: 'conflict', message: 'x' }), false);
  assert.equal(addDisabled(plan, true, null), true);
});

test('a file that is not a feed has no GUID row, and another error has no refusal view', () => {
  const view = existsRefusalView({ ...exists, details: { ...exists.details, title: null, guid: null } });
  assert.deepEqual(view.facts.map((row) => row.label), ['File', 'Title', 'Size']);
  assert.equal(view.facts[1].value, 'Not a feed that Studio can read');
  assert.equal(existsRefusalView({ code: 'file_exists', message: 'x' }), null);
  assert.equal(existsRefusalView({ code: 'not_found', details: exists.details }), null);
  assert.equal(existsRefusalView(null), null);
});

test('Review rows of a move that replaces a copy or a different file (ADR 0178 items 3 and 6)', () => {
  assert.equal(replacesCopyNote({ replaces_copy: true }), 'Replaces an old copy of this feed');
  assert.equal(replacesCopyNote({ replaces_copy: false }), null);
  const titled = replaceWarning({ path: 'a.xml', replaces_file: { title: 'Other Album', guid: 'g', size: 1 } });
  assert.match(titled.text, /"Other Album"/);
  const plain = replaceWarning({ path: 'a.xml', replaces_file: { title: null, guid: null, size: 1 } });
  assert.match(plain.text, /a\.xml/);
  assert.equal(replaceWarning({ path: 'a.xml', replaces_file: null }), null);
});
