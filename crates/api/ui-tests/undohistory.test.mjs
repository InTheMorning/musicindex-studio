import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { makeHistory, undoKey } from '../ui/undohistory.js';

test('ADR 0227 item 4: field edits group and undo restores the first value', () => {
  const history = makeHistory();
  history.record('title-0', 'title', 0);
  history.record('title-1', 'title', 500);
  assert.equal(history.undo('title-2'), 'title-0');
  assert.equal(history.redo('title-0'), 'title-2');
});

test('ADR 0227 item 4: a different field or a pause starts a step', () => {
  const byField = makeHistory();
  byField.record('a0', 'a', 0);
  byField.record('b0', 'b', 1);
  assert.equal(byField.undo('b1'), 'b0');
  assert.equal(byField.undo('a1'), 'a0');
  const byTime = makeHistory();
  byTime.record('a0', 'a', 0);
  byTime.record('a1', 'a', 1001);
  assert.equal(byTime.undo('a2'), 'a1');
});

test('ADR 0227 items 6 and 11: a new record clears redo and retains 100 steps', () => {
  const history = makeHistory();
  history.record(0, 'f', 0);
  assert.equal(history.undo(1), 0);
  history.record(1, 'g', 2);
  assert.equal(history.redo(2), null);
  for (let i = 0; i < 110; i += 1) history.record(i, null, i + 3);
  for (let i = 0; i < 100; i += 1) assert.notEqual(history.undo(-1), null);
  assert.equal(history.undo(-1), null);
  history.clear();
  assert.equal(history.canUndo, false);
  assert.equal(history.canRedo, false);
});

test('ADR 0227 item 3: undo keys reject Alt and recognize Ctrl and Cmd forms', () => {
  const key = (values) => undoKey({ key: 'z', ctrlKey: false, metaKey: false, shiftKey: false, altKey: false, ...values });
  assert.equal(key({ ctrlKey: true }), 'undo');
  assert.equal(key({ metaKey: true }), 'undo');
  assert.equal(key({ ctrlKey: true, shiftKey: true }), 'redo');
  assert.equal(key({ metaKey: true, shiftKey: true }), 'redo');
  assert.equal(undoKey({ key: 'y', ctrlKey: true }), 'redo');
  assert.equal(key({ ctrlKey: true, altKey: true }), null);
  assert.equal(key({ key: 'x', ctrlKey: true }), null);
  assert.equal(key({ key: 'z' }), null);
});

test('ADR 0227 item 2: the toolbar puts undo and redo before draft status', async () => {
  const editor = await readFile(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const toolbar = editor.slice(editor.indexOf('export function DraftToolbar'), editor.indexOf('// ---- The editor pages'));
  assert.ok(toolbar.indexOf('aria-label="Undo"') < toolbar.indexOf('draft-status'));
  assert.ok(toolbar.indexOf('aria-label="Redo"') < toolbar.indexOf('draft-status'));
});

test('ADR 0227 item 6: an edit in the same field after an undo starts a new step', () => {
  const history = makeHistory();
  history.record('a0', 'a', 0);
  assert.equal(history.undo('a1'), 'a0');
  history.record('a0', 'a', 100);
  assert.equal(history.undo('a2'), 'a0');
});
