// The key model for lists and trees (ADR 0172, musicindex ADR 0061).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createListModel, gridLines, listKey } from '../ui/listkeys.js';

const rows = [
  { kind: 'page' },
  { kind: 'tree', parent: null, expanded: true },
  { kind: 'tree', parent: 1 },
  { kind: 'expandable' },
];

function press(model, key) {
  return listKey(model, key);
}

test('Up and Down move between rows and stop at each end', () => {
  let model = createListModel(rows);
  assert.equal(press(model, 'ArrowUp').model.current, 0);
  assert.equal(press(model, 'ArrowDown').model.current, 1);
  model = createListModel(rows, 2);
  assert.equal(press(model, 'ArrowDown').model.current, 3);
  assert.equal(press(model, 'ArrowDown').model.current, 3);
});

test('Home and End select the first and last rows', () => {
  const model = createListModel(rows, 2);
  assert.equal(press(model, 'Home').model.current, 0);
  assert.equal(press(model, 'End').model.current, rows.length - 1);
});

test('movement skips rows hidden by a closed tree folder', () => {
  const model = createListModel([
    { kind: 'tree', expandable: true },
    { kind: 'page', parent: 0, visible: false },
    { kind: 'page' },
  ]);
  assert.equal(press(model, 'ArrowDown').model.current, 2);
  assert.equal(press(model, 'End').model.current, 2);
  assert.equal(press(model, 'Home').model.current, 0);
});

test('Right opens a closed row and Left closes an open row', () => {
  const closed = createListModel([{ kind: 'expandable' }]);
  const opened = press(closed, 'ArrowRight');
  assert.deepEqual(opened.action, { type: 'toggle', row: 0, expanded: true });
  assert.equal(opened.model.rows[0].expanded, true);
  const shut = press(opened.model, 'ArrowLeft');
  assert.deepEqual(shut.action, { type: 'toggle', row: 0, expanded: false });
  assert.equal(shut.model.rows[0].expanded, false);
});

test('Left on a closed child row selects its parent', () => {
  const model = createListModel(rows, 2);
  assert.equal(press(model, 'ArrowLeft').model.current, 1);
  const leaf = createListModel([
    { kind: 'tree', expandable: true, expanded: true },
    { kind: 'page', parent: 0 },
  ], 1);
  assert.equal(press(leaf, 'ArrowLeft').model.current, 0);
});

test('Enter opens a page or toggles an expandable row', () => {
  const page = press(createListModel(rows), 'Enter');
  assert.deepEqual(page.action, { type: 'page', row: 0 });
  const expandable = press(createListModel(rows, 3), 'Enter');
  assert.deepEqual(expandable.action, { type: 'toggle', row: 3, expanded: true });
  assert.equal(expandable.model.rows[3].expanded, true);
  const folder = press(createListModel([{ kind: 'tree', expandable: true }]), 'Enter');
  assert.deepEqual(folder.action, { type: 'toggle', row: 0, expanded: true });
});

test('the model keeps its current row when focus leaves the list', () => {
  const model = createListModel(rows, 2);
  const afterTab = press(model, 'Tab');
  assert.equal(afterTab.model.current, 2);
  assert.equal(press(afterTab.model, 'ArrowDown').model.current, 3);
});

// musicindex ADR 0078 R-GRIDKEY-1: 10 tiles in 4 columns.
const tiles = Array.from({ length: 10 }, () => ({ kind: 'page' }));
const tileLines = [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9]];
const gridVectors = [
  ['ArrowDown', 0, 4],
  ['ArrowDown', 5, 9],
  ['ArrowDown', 8, 8],
  ['ArrowDown', 6, 9],
  ['ArrowUp', 2, 2],
  ['ArrowUp', 6, 2],
  ['ArrowRight', 3, 4],
  ['ArrowLeft', 4, 3],
  ['ArrowRight', 9, 9],
  ['Home', 5, 0],
  ['End', 5, 9],
];

for (const [key, from, to] of gridVectors) {
  test(`musicindex ADR 0078 R-GRIDKEY-1: ${key} from tile ${from} goes to tile ${to}`, () => {
    const model = createListModel(tiles, from, { mode: 'grid', lines: tileLines });
    assert.equal(press(model, key).model.current, to);
  });
}

test('gridLines divides tiles into lines from the top of each tile', () => {
  const tops = [0, 0, 0, 0, 120, 120.4, 120, 120, 240, 240];
  assert.deepEqual(gridLines(tops), tileLines);
  // Groups of different sizes: a heading between the groups moves the next line down.
  assert.deepEqual(gridLines([0, 0, 0, 180, 300, 300]), [[0, 1, 2], [3], [4, 5]]);
});

test('an empty list has no current row and no key action', () => {
  const model = createListModel([]);
  assert.equal(model.current, null);
  assert.deepEqual(press(model, 'ArrowDown').action, { type: 'none' });
});
