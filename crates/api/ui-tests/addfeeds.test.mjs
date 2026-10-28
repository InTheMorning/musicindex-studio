// The rules of the Add feeds page and of the form that makes a feed (ADR 0073).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { addChoices, makeFields, feedFileStart, MAKE_TYPES } from '../ui/addfeeds.js';

test('feedFileStart gives the file name from the title', () => {
  assert.equal(feedFileStart('Pixel Palace'), 'pixel-palace.xml');
  assert.equal(feedFileStart('Ça va — Live!'), 'ca-va-live.xml');
  assert.equal(feedFileStart('2024: Best Of'), '2024-best-of.xml');
});

test('feedFileStart gives an empty path when the title has no usable character', () => {
  assert.equal(feedFileStart('  '), '');
  assert.equal(feedFileStart(''), '');
  assert.equal(feedFileStart('!!!'), '');
});

test('addChoices gives two groups with all rows when a host exists', () => {
  const groups = addChoices({ hasHost: true });
  assert.deepEqual(groups.map((g) => [g.id, g.label]), [['make', 'Make a new feed'], ['have', 'Add a feed you have']]);
  assert.deepEqual(groups[0].rows.map((r) => r.id), ['album', 'publisher', 'show', 'playlist']);
  assert.deepEqual(groups[1].rows.map((r) => r.id), ['find', 'folder', 'msp']);
  assert.equal(groups[0].connect, undefined);
  for (const row of groups.flatMap((g) => g.rows)) {
    assert.ok(row.title && row.text, `the row ${row.id} needs a title and a text`);
  }
});

test('addChoices leaves out the make rows and the folder row when no host exists', () => {
  const groups = addChoices({ hasHost: false });
  assert.deepEqual(groups[0].rows, []);
  assert.equal(groups[0].connect, true);
  assert.deepEqual(groups[1].rows.map((r) => r.id), ['find', 'msp']);
});

test('makeFields gives the fields of each type', () => {
  assert.deepEqual(makeFields('album'), { medium: 'music', titleLabel: 'Album title', authorLabel: 'Artist', role: false, button: 'Make the release', busyLabel: 'Making the release…' });
  assert.deepEqual(makeFields('publisher'), { medium: 'publisher', titleLabel: 'Name', authorLabel: null, role: true, button: 'Make the feed', busyLabel: 'Making the feed…' });
  assert.deepEqual(makeFields('show'), { medium: 'music', titleLabel: 'Show feed title', authorLabel: 'Artist', role: false, button: 'Make the show feed', busyLabel: 'Making the show feed…' });
  assert.deepEqual(makeFields('playlist'), { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the playlist', busyLabel: 'Making the playlist…' });
});

test('MAKE_TYPES lists the four types that makeFields knows', () => {
  assert.deepEqual(MAKE_TYPES.map(([id]) => id), ['album', 'publisher', 'show', 'playlist']);
  for (const [id] of MAKE_TYPES) assert.ok(makeFields(id).button);
});
