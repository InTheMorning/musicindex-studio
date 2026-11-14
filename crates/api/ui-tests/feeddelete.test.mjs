import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DELETE_CHOICES, deleteButtonEnabled, deleteWrite, retireSteps, operationNote } from '../ui/feeddelete.js';
import { readFileSync } from 'node:fs';

test('the first sheet orders its choices and marks only the last choice destructive', () => {
  assert.deepEqual(DELETE_CHOICES.map(({ title, destructive }) => [title, destructive]), [
    ['Move It to a Different File Host', false],
    ['Retire It', false],
    ['Delete the File Only', true],
  ]);
});

const state = (extra = {}) => ({
  ask: 'written', removal: { answer: 'not_yet' }, removal_done: false, delete: 'not_done', ...extra,
});

test('retire steps show the state of each step', () => {
  const steps = retireSteps(state({ removal: { answer: 'removed' }, removal_done: true, delete: 'done' }));
  assert.deepEqual(steps.map(({ done }) => done), [true, true, true]);
  assert.equal(steps[1].text, 'Music Index does not hold this feed.');
  assert.equal(steps[2].button, null);
});

test('step three offers Delete Anyway before removal and Delete the File after removal', () => {
  assert.equal(retireSteps(state())[2].button, 'Delete Anyway');
  assert.equal(retireSteps(state({ removal: { answer: 'removed' }, removal_done: true }))[2].button, 'Delete the File');
});

test('step three offers Delete Anyway when Music Index holds a replaced GUID', () => {
  const result = retireSteps(state({ removal: { answer: 'held_with_other_guid', new_guid: 'new-guid' } }));
  assert.equal(result[1].text, 'Music Index holds this feed with a different GUID.');
  assert.equal(result[2].button, 'Delete Anyway');
});

test('the final button waits for an exact, nonempty confirmation', () => {
  assert.equal(deleteButtonEnabled('Album', 'Album'), true);
  assert.equal(deleteButtonEnabled('Album', 'album'), false);
  assert.equal(deleteButtonEnabled('Album', 'Album '), false);
  assert.equal(deleteButtonEnabled('', ''), false);
  assert.equal(deleteButtonEnabled(null, ''), false);
});

test('operationNote marks only the row of the feed file', () => {
  assert.equal(operationNote({ kind: 'delete', deletes_feed_file: true }), 'This write deletes the feed file.');
  assert.equal(operationNote({ kind: 'delete', deletes_feed_file: false }), 'Studio deletes it after the feed is written.');
  assert.equal(operationNote({ kind: 'upload', deletes_feed_file: false }), 'The feed uses this file.');
});

test('ADR 0198 names and counts all delete operations', () => {
  assert.deepEqual(deleteWrite({ operations: [{ kind: 'delete', deletes_feed_file: true }] }), {
    label: 'Delete the Feed File', destructive: true, cancel: 'Cancel the Delete',
  }, 'ADR 0198 item 1: name one delete. Count the delete operations.');
  assert.equal(deleteWrite({ operations: [
    { kind: 'delete', deletes_feed_file: true },
    { kind: 'delete', deletes_feed_file: false },
    { kind: 'delete', deletes_feed_file: false },
  ] }).label, 'Delete 3 Files',
  'ADR 0198 item 1: count every delete operation when one marks the feed file.');
  assert.equal(deleteWrite({ operations: [{ kind: 'delete', deletes_feed_file: false }] }), null,
    'ADR 0198 item 4: keep the usual review without a feed-file delete.');
});

test('ADR 0198 keeps the destructive review button first and guards the write', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const deleteStart = editor.indexOf('${deletion && diff && hasWork && !addressesBlocked && !choosing && html`<button class="button destructive"');
  const cancel = editor.indexOf('deletion.cancel : \'Discard the Draft\'');
  const usual = editor.indexOf('${!deletion && diff && hasWork && !addressesBlocked && !choosing && html`<button class="button primary"');
  assert.ok(deleteStart >= 0 && cancel > deleteStart,
    'ADR 0198 items 2 and 3: put the guarded destructive delete before Cancel the Delete.');
  assert.ok(usual > cancel && editor.includes("'Write to the File Host'"),
    'ADR 0198 item 4: keep Write to the File Host in the guarded primary path.');
});

test('ADR 0198 hides actions that a deleted feed cannot use', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes("retire.delete !== 'done' && html`<button class=\"button\" type=\"button\" disabled=${action.busy} onClick=${stopRetire}>Stop Retiring</button>"),
    'ADR 0198 item 6: hide Stop Retiring after the delete.');
  assert.ok(editor.includes('!feed.host_deleted && candidates.length > 0'),
    'ADR 0198 item 7: hide the linked page picker after the delete.');
  assert.ok(editor.includes('!feed.host_deleted && candidates.length === 0'),
    'ADR 0198 item 7: hide the linked page hint after the delete.');
  const picker = editor.indexOf('${!feed.host_deleted && candidates.length > 0 && html`<div class="field">');
  const link = editor.indexOf('>Link to This Page</button>', picker);
  assert.ok(picker >= 0 && link > picker,
    'ADR 0198 item 7: put Link to This Page inside the host_deleted guard.');
});

test('ADR 0198 reads the retire state again after a write', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes('}, [retiring, feed.id, written]);') && editor.includes('feed.last_version') && editor.includes('feed.host_deleted}`'),
    'ADR 0198 item 6: the Retire block must read its state again after a write. Add the version of the feed and host_deleted to the dependencies of the retire state effect in FeedDelete.');
  assert.ok(editor.includes('if (request === retireRequest.current) setRetire(state);'),
    'ADR 0198 item 6: a late answer must not show the state before the write. Set the retire state only from the answer of the last request.');
  assert.ok(editor.includes('onClick=${loadRetire}>Check Again</button>') && !editor.includes('setRetire(await api('),
    'ADR 0198 item 6: Check Again must use loadRetire. Do not set the retire state directly from an API answer.');
});
