// The moves of a position menu (ADR 0081).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { moveTo, moveAmong } from '../ui/order.js';

const list = ['a', 'b', 'c', 'd', 'e'];

test('moveTo moves an entry to the start in one step', () => {
  assert.deepEqual(moveTo(list, 4, 0), ['e', 'a', 'b', 'c', 'd']);
});

test('moveTo moves an entry to the end, and the entries between move up', () => {
  assert.deepEqual(moveTo(list, 1, 4), ['a', 'c', 'd', 'e', 'b']);
});

test('moveTo makes a new list and keeps the entries', () => {
  const entries = [{ _cid: 1 }, { _cid: 2 }, { _cid: 3 }];
  const moved = moveTo(entries, 0, 2);
  assert.deepEqual(moved.map((x) => x._cid), [2, 3, 1]);
  assert.equal(moved[2], entries[0]);
  assert.deepEqual(entries.map((x) => x._cid), [1, 2, 3]);
});

test('moveTo gives the same list for the same position or a position out of the list', () => {
  assert.equal(moveTo(list, 2, 2), list);
  assert.equal(moveTo(list, -1, 2), list);
  assert.equal(moveTo(list, 2, 5), list);
  assert.equal(moveTo(list, 1.5, 2), list);
});

test('moveAmong keeps the index of each entry that is not in the slots', () => {
  // The slots 0, 2 and 4 hold releases. The entries 1 and 3 are other items.
  const items = ['r1', 'x', 'r2', 'y', 'r3'];
  assert.deepEqual(moveAmong(items, [0, 2, 4], 2, 0), ['r3', 'x', 'r1', 'y', 'r2']);
});

test('moveAmong gives the same list when nothing moves', () => {
  const items = ['r1', 'x', 'r2'];
  assert.equal(moveAmong(items, [0, 2], 1, 1), items);
  assert.equal(moveAmong(items, [0, 2], 0, 9), items);
});
