// The model of the field list (ADR 0154, musicindex ADR 0052 R-QUERY-7).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { listRows, namePrefix, applyRow, listKey, NAME_SOURCE } from '../ui/fieldlist.js';

const fieldAnswer = {
  rows: [{ row: 'field', token: 'kind:', values: ['album', 'ep', 'single'], values_text: 'album, ep, single', example: 'kind:single', description: 'The release kind', insert: 'kind:' }],
  edit: { start: 6, end: 8 },
  names: null,
  error: null,
};

test('the rows of the core give a label, a line and the text to put in', () => {
  assert.deepEqual(listRows(fieldAnswer), [{ label: 'kind:', line: 'album, ep, single · kind:single', insert: 'kind:' }]);
  const value = { rows: [{ row: 'value', field: 'kind:', value: 'single', description: 'One track', insert: 'single' }], edit: { start: 5, end: 6 }, names: null, error: null };
  assert.deepEqual(listRows(value), [{ label: 'single', line: 'One track', insert: 'single' }]);
  assert.deepEqual(listRows(null), []);
});

test('only artist: with a typed prefix gets name rows, after the rows of the core', () => {
  const artist = { rows: [], edit: { start: 7, end: 9 }, names: { field: 'artist:', prefix: 'th' }, error: null };
  assert.equal(namePrefix(artist), 'th');
  const names = [{ name: 'The Ben', insert: '"The Ben"' }];
  assert.deepEqual(listRows(artist, names), [{ label: 'The Ben', line: NAME_SOURCE, insert: '"The Ben"' }]);
  assert.equal(namePrefix({ ...artist, names: { field: 'artist:', prefix: '' } }), null);
  const release = { ...artist, names: { field: 'release:', prefix: 'ab' } };
  assert.equal(namePrefix(release), null);
  assert.deepEqual(listRows(release, names), []);
});

test('a row replaces the range that the core gave, and the cursor follows it', () => {
  assert.deepEqual(applyRow('Björk ki', { start: 6, end: 8 }, 'kind:'), { text: 'Björk kind:', cursor: 11 });
  // UTF-16 indexes: the note before the range is two code units.
  assert.deepEqual(applyRow('🎵 artist:th x', { start: 10, end: 12 }, '"The Ben"'), { text: '🎵 artist:"The Ben" x', cursor: 19 });
  assert.equal(applyRow('abc', null, 'kind:'), null);
});

// ADR 0154 items 7 and 8: no row is selected at the start, Up and Down move,
// Tab or Enter puts the selected row in, Escape closes, and Enter with no
// selected row keeps the action of the field.
test('the keys of the list', () => {
  const start = { open: true, selected: null };
  assert.deepEqual(listKey(start, 'Enter', 3), { open: true, selected: null, act: 'pass' });
  assert.deepEqual(listKey(start, 'Tab', 3).act, 'pass');
  assert.deepEqual(listKey(start, 'ArrowDown', 3), { open: true, selected: 0, act: 'move' });
  assert.deepEqual(listKey({ open: true, selected: 2 }, 'ArrowDown', 3).selected, 2);
  assert.deepEqual(listKey(start, 'ArrowUp', 3).selected, 2);
  assert.equal(listKey({ open: true, selected: 0 }, 'ArrowUp', 3).selected, null);
  assert.deepEqual(listKey({ open: true, selected: 1 }, 'Enter', 3), { open: true, selected: null, act: 'insert' });
  assert.equal(listKey({ open: true, selected: 1 }, 'Tab', 3).act, 'insert');
  assert.deepEqual(listKey({ open: true, selected: 1 }, 'Escape', 3), { open: false, selected: null, act: 'close' });
  assert.equal(listKey({ open: true, selected: 1 }, 'a', 3).act, 'pass');
  // With no list, each key keeps its action.
  for (const key of ['ArrowDown', 'Enter', 'Escape']) {
    assert.equal(listKey({ open: false, selected: null }, key, 3).act, 'pass', key);
    assert.equal(listKey({ open: true, selected: null }, key, 0).act, 'pass', key);
  }
});
