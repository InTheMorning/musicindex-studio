import test from 'node:test';
import assert from 'node:assert/strict';
import { readdir, readFile } from 'node:fs/promises';
import { undoNoticeText, makeDeleteUndo, DELETE_UNDO_ACTIONS } from '../ui/undonotice.js';

// A fake route function. It records each call and fails on request.
function fakeApi(failOn = null) {
  const calls = [];
  const api = async (path, options = {}) => {
    calls.push(`${options.method || 'GET'} ${path}`);
    if (failOn && path === failOn) throw new Error('The undo record is gone.');
    return { undoable: true };
  };
  return { api, calls };
}

test('ADR 0228 item 3: the status texts of a delete', () => {
  assert.equal(undoNoticeText.personDeleted('Ann'), 'Ann is deleted.');
  assert.equal(undoNoticeText.feedRemoved('Album'), 'Album is removed from your collection.');
});

test('ADR 0228 items 3 and 4: after a delete the bar model has the text and an enabled Undo', async () => {
  const { api, calls } = fakeApi();
  let done = 0;
  const control = makeDeleteUndo(api, { done: () => { done += 1; } });
  assert.deepEqual(control.model(), { text: '', error: false, canUndo: false, canRedo: false });
  control.record('Ann is deleted.', async () => calls.push('DELETE again'));
  assert.deepEqual(control.model(), { text: 'Ann is deleted.', error: false, canUndo: true, canRedo: false });
  await control.undo();
  assert.deepEqual(calls, ['POST /undo']);
  assert.equal(done, 1);
  assert.equal(control.model().canUndo, false);
  assert.equal(control.model().canRedo, true);
  await control.redo();
  assert.deepEqual(calls, ['POST /undo', 'DELETE again']);
  assert.equal(control.model().canUndo, true);
  assert.equal(control.model().text, 'Ann is deleted.');
});

test('ADR 0228 item 4: a new delete replaces the step and no timer sends DELETE /undo', async () => {
  const { api, calls } = fakeApi();
  const control = makeDeleteUndo(api);
  control.record('One is deleted.', async () => {});
  control.record('Two is deleted.', async () => {});
  assert.equal(control.model().text, 'Two is deleted.');
  await control.undo();
  assert.equal(control.model().canUndo, false);
  assert.deepEqual(calls, ['POST /undo']);
  control.clear();
  assert.equal(control.model().canRedo, false);
});

test('ADR 0228 item 5: a failed undo shows the error of the core and removes the step', async () => {
  const { api } = fakeApi('/undo');
  let done = 0;
  const control = makeDeleteUndo(api, { done: () => { done += 1; } });
  control.record('Ann is deleted.', async () => {});
  await control.undo();
  assert.deepEqual(control.model(), { text: 'The undo record is gone.', error: true, canUndo: false, canRedo: false });
  assert.equal(done, 0);
  control.record('Bo is deleted.', async () => {});
  assert.equal(control.model().error, false);
});

test('ADR 0228 item 1: no module renders the undo notice', async () => {
  const files = (await readdir(new URL('../ui/', import.meta.url))).filter((name) => name.endsWith('.js') || name === 'styles.css');
  for (const name of files) {
    const text = await readFile(new URL(`../ui/${name}`, import.meta.url), 'utf8');
    assert.doesNotMatch(text, /undo-notice/, `ADR 0228 item 1: ${name} renders the bottom notice. Show the delete step in the bar with DeleteUndoBar.`);
  }
});

test('ADR 0228 items 3 to 5: each call of useDeleteUndo() names an action of the control', async () => {
  const control = makeDeleteUndo(async () => {});
  for (const name of DELETE_UNDO_ACTIONS) {
    assert.equal(typeof control[name], 'function', `makeDeleteUndo gives no ${name}.`);
  }
  const files = (await readdir(new URL('../ui/', import.meta.url))).filter((name) => name.endsWith('.js'));
  for (const name of files) {
    const text = await readFile(new URL(`../ui/${name}`, import.meta.url), 'utf8');
    for (const match of text.matchAll(/deleteUndo\.(\w+)/g)) {
      assert.ok(match[1] === 'model' || DELETE_UNDO_ACTIONS.includes(match[1]),
        `ADR 0228 items 3 to 5: ${name} calls deleteUndo.${match[1]}, and useDeleteUndo() does not give it. Add the action to DELETE_UNDO_ACTIONS in crates/api/ui/undonotice.js.`);
    }
  }
});
