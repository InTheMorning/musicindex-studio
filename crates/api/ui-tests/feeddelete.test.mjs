import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DELETE_CHOICES, deleteButtonEnabled, retireSteps, operationNote } from '../ui/feeddelete.js';

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
