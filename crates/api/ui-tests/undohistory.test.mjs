import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { makeHistory, undoKey, focusedField, keysOf, restoreKeys, carryCids, carryCidsByContent } from '../ui/undohistory.js';

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

const titlesAndKeys = (list) => list.map((e) => [e.title, e.key]);

test('ADR 0022 item 5: an undo after a move gives each entry the key of the server draft', () => {
  // The draft had A, B, C. The move put C first, and the save gave the new keys.
  const before = { remote_items: [{ _cid: 1, key: 0, title: 'A' }, { _cid: 2, key: 1, title: 'B' }, { _cid: 3, key: 2, title: 'C' }] };
  const saved = { remote_items: [{ _cid: 3, key: 0, title: 'C' }, { _cid: 1, key: 1, title: 'A' }, { _cid: 2, key: 2, title: 'B' }] };
  const restored = restoreKeys(before, keysOf(saved), new Set());
  assert.deepEqual(titlesAndKeys(restored.remote_items), [['A', 1], ['B', 2], ['C', 0]],
    'An undo step must send the keys of the server draft, or the core copies the unmodeled attributes of a different entry.');
  assert.equal(before.remote_items[0].key, 0, 'The step in the history does not change.');
});

test('ADR 0022 item 5: an undo of a saved removal gives the restored entry no key, also in a nested list', () => {
  const before = { items: [{ _cid: 1, key: 0, title: 'One', persons: [{ _cid: 5, key: 0 }, { _cid: 6, key: 1 }] }, { _cid: 2, key: 1, title: 'Two' }] };
  const saved = { items: [{ _cid: 1, key: 0, title: 'One', persons: [{ _cid: 6, key: 0 }] }] };
  const restored = restoreKeys(before, keysOf(saved), new Set([2, 5]));
  assert.equal(restored.items[0].key, 0);
  assert.equal('key' in restored.items[1], false, 'The server draft has no element for a removed entry.');
  assert.equal('key' in restored.items[0].persons[0], false);
  assert.equal(restored.items[0].persons[1].key, 0);
});

test('ADR 0022 item 5: an entry with an unknown client ID keeps its own key', () => {
  const step = { items: [{ _cid: 9, key: 1, title: 'Two' }] };
  assert.equal(restoreKeys(step, new Map(), new Set()).items[0].key, 1);
});

test('ADR 0227 item 7: a load of the same server draft keeps the client IDs, so an earlier step matches', () => {
  const earlier = { items: [{ _cid: 1, key: 0, title: 'One', persons: [{ _cid: 5, key: 0 }] }, { _cid: 2, key: 1, title: 'Two' }] };
  const loaded = { items: [{ _cid: 70, key: 0, title: 'One', persons: [{ _cid: 75, key: 0 }] }, { _cid: 71, key: 1, title: 'Two' }, { _cid: 72, title: 'New' }] };
  const carried = carryCids(loaded, earlier);
  assert.deepEqual(carried.items.map((e) => e._cid), [1, 2, 72]);
  assert.equal(carried.items[0].persons[0]._cid, 5);
  assert.equal(carried.items[1].title, 'Two', 'The loaded values stay.');
});

test('a new revision carries the client IDs of entries with the same content', () => {
  // The earlier server draft after a saved move: C, A, B. A route then
  // changed one field of B, and a new revision loaded with new client IDs.
  const earlier = { hidden: false, items: [
    { _cid: 'c', key: 0, title: 'C' },
    { _cid: 'a', key: 1, title: 'A', tracks: [{ _cid: 't', key: 0, n: 1 }] },
    { _cid: 'b', key: 2, title: 'B' },
    { _cid: 'd1', key: 3, title: 'Same' },
    { _cid: 'd2', key: 4, title: 'Same' },
  ] };
  const loaded = { hidden: true, items: [
    { _cid: 'n1', key: 0, title: 'C' },
    { _cid: 'n2', key: 1, title: 'A', tracks: [{ _cid: 'n3', key: 0, n: 1 }] },
    { _cid: 'n4', key: 2, title: 'B', hidden: true },
    { _cid: 'n5', key: 3, title: 'Same' },
    { _cid: 'n6', key: 4, title: 'Same' },
  ] };
  const carried = carryCidsByContent(loaded, earlier);
  assert.deepEqual(carried.items.map((entry) => entry._cid), ['c', 'a', 'n4', 'n5', 'n6']);
  assert.equal(carried.items[1].tracks[0]._cid, 't');
  // An undo step from before the move gets the keys of the new revision.
  const step = { hidden: false, items: [
    { _cid: 'a', key: 0, title: 'A', tracks: [{ _cid: 't', key: 0, n: 1 }] },
    { _cid: 'b', key: 1, title: 'B' },
    { _cid: 'c', key: 2, title: 'C' },
  ] };
  const restored = restoreKeys(step, keysOf(carried), new Set());
  assert.deepEqual(restored.items.map((entry) => entry.key), [1, 1, 0]);
});
