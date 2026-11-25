import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { makeHistory, undoKey, focusedField } from '../ui/undohistory.js';

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
  assert.ok(toolbar.indexOf('<${UndoButtons}') < toolbar.indexOf('draft-status'));
  const buttons = editor.slice(editor.indexOf('export function UndoButtons'), editor.indexOf('// The page takes undo keys'));
  assert.ok(buttons.includes('aria-label="Undo"'));
  assert.ok(buttons.includes('aria-label="Redo"'));
  const app = await readFile(new URL('../ui/app.js', import.meta.url), 'utf8');
  const settings = app.slice(app.indexOf('function SettingsPage'), app.indexOf('// The mark of a setting'));
  assert.ok(settings.indexOf('<${UndoButtons}') < settings.indexOf('draft-status'));
  const people = await readFile(new URL('../ui/peoplepages.js', import.meta.url), 'utf8');
  const person = people.slice(people.indexOf('export function PersonPage'), people.indexOf('function NicknameSheet'));
  assert.ok(person.indexOf('<${UndoButtons}') < person.indexOf('draft-status'));
});

test('ADR 0227 item 6: an edit in the same field after an undo starts a new step', () => {
  const history = makeHistory();
  history.record('a0', 'a', 0);
  assert.equal(history.undo('a1'), 'a0');
  history.record('a0', 'a', 100);
  assert.equal(history.undo('a2'), 'a0');
});

test('ADR 0227 item 4: the focused text field gives the key of the step', () => {
  assert.equal(focusedField({ tagName: 'INPUT', id: 'person-name' }), 'person-name');
  assert.equal(focusedField({ tagName: 'TEXTAREA', id: '', name: 'bio' }), 'bio');
  assert.equal(focusedField({ tagName: 'SELECT', id: 'kind' }), null);
  assert.equal(focusedField(null), null);
});

test('ADR 0228 items 2 and 7: an operation step undoes and redoes through its own functions', async () => {
  const { isOperationStep } = await import('../ui/undohistory.js');
  const sent = [];
  const history = makeHistory();
  history.record('values-0', 'title', 0);
  history.recordOperation({ undo: async () => sent.push('kind before'), redo: async () => sent.push('kind after') });
  assert.equal(history.canUndo, true);
  const step = history.undo('values-1');
  assert.equal(isOperationStep(step), true);
  await history.perform(step, 'undo');
  assert.deepEqual(sent, ['kind before']);
  assert.equal(history.canRedo, true);
  const again = history.redo('values-1');
  assert.equal(again, step);
  await history.perform(again, 'redo');
  assert.deepEqual(sent, ['kind before', 'kind after']);
  // The step before is a state step, and it keeps its order.
  history.undo('values-1');
  assert.equal(history.undo('values-1'), 'values-0');
});

test('ADR 0228 item 5: a failed operation step leaves the history', async () => {
  const history = makeHistory();
  history.recordOperation({ undo: async () => { throw new Error('No.'); }, redo: async () => {} });
  const step = history.undo(null);
  await assert.rejects(history.perform(step, 'undo'), /No\./);
  assert.equal(history.canUndo, false);
  assert.equal(history.canRedo, false);
});

test('ADR 0254 item 37: a Catalog operation has Undo and no Redo', async () => {
  const history = makeHistory();
  let calls = 0;
  history.recordOperation({ undo: async () => { calls += 1; } });
  const step = history.undo(null);
  await history.perform(step, 'undo');
  assert.equal(calls, 1);
  assert.equal(history.canUndo, false);
  assert.equal(history.canRedo, false);
  assert.equal(history.redo(null), null);
});

test('ADR 0254 item 37: edits around a Catalog operation keep their Redo rules', async () => {
  const history = makeHistory();
  history.record('before', null, 0);
  history.recordOperation({ undo: async () => {} });
  history.record('after Catalog', null, 1);
  assert.equal(history.undo('after edit'), 'after Catalog');
  assert.equal(history.canRedo, true);
  await history.perform(history.undo(null), 'undo');
  assert.equal(history.canRedo, true);
  assert.equal(history.redo('after Catalog'), 'after edit');
  assert.equal(history.undo('after edit'), 'after Catalog');
  assert.equal(history.undo('after Catalog'), 'before');
  assert.equal(history.redo('before'), 'after Catalog');
  history.record('new edit', null, 2);
  assert.equal(history.canRedo, false);
});

test('ADR 0254 item 37: notice Undo removes the same step from the page history', async () => {
  const page = makeHistory();
  const catalog = makeHistory();
  let calls = 0;
  let pageStep;
  let catalogStep;
  const operation = { undo: async () => {
    calls += 1;
    page.forgetOperation(pageStep);
    catalog.forgetOperation(catalogStep);
  } };
  page.record('before edit', null, 0);
  pageStep = page.recordOperation(operation);
  catalogStep = catalog.recordOperation(operation);
  await catalog.perform(catalog.undo(null), 'undo');
  assert.equal(calls, 1);
  assert.equal(catalog.canUndo, false);
  assert.equal(page.canUndo, true);
  assert.equal(page.undo('after edit'), 'before edit');
  assert.equal(page.canRedo, true);
  assert.equal(page.redo('before edit'), 'after edit');
  assert.equal(calls, 1);
});
