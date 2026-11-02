// The lines of a review (ADR 0114 items 8 to 12).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { shownLines, runs, commonIndent, moreLabel, stripIndent, normalizeView } from '../ui/review.js';

const part = (text, changed = false) => ({ text, changed });
const same = (text) => ({ kind: 'same', parts: [part(text)], pair: null });
const removed = (parts, pair = null) => ({ kind: 'removed', parts, pair });
const added = (parts) => ({ kind: 'added', parts });

test('a pair with only added parts is one row with the sign ±', () => {
  const lines = [
    removed([part('<a')], 1),
    added([part('<a'), part(' x="1"', true)]),
  ];
  const rows = shownLines(lines, 'simplified');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].sign, '±');
  assert.equal(rows[0].kind, 'add');
  assert.equal(shownLines(lines, 'detailed').length, 2);
});

test('a pair with a removed and an added part is two rows', () => {
  const lines = [
    removed([part('<t>'), part('Old', true)], 1),
    added([part('<t>'), part('New', true)]),
  ];
  const rows = shownLines(lines, 'simplified');
  assert.deepEqual(rows.map((r) => r.sign), ['−', '+']);
});

test('a pair with only removed parts is one row from the removed line', () => {
  const lines = [removed([part('<a'), part(' x="1"', true)], 1), added([part('<a')])];
  const rows = shownLines(lines, 'simplified');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].kind, 'remove');
  assert.equal(rows[0].sign, '±');
});

test('runs keeps three rows of context and counts each hidden run', () => {
  const rows = [];
  for (let i = 0; i < 20; i += 1) rows.push({ kind: i === 10 ? 'add' : 'same', sign: ' ', parts: [part(`l${i}`)] });
  const segments = runs(rows, false);
  const hidden = segments.filter((s) => s.hidden);
  assert.deepEqual(hidden.map((s) => s.count), [7, 6]);
  assert.equal(segments.filter((s) => !s.hidden).length, 7);
  assert.equal(runs(rows, true).some((s) => s.hidden), false);
});

test('commonIndent gives the shared indent and ignores an empty row', () => {
  const row = (text) => ({ parts: [part(text)] });
  assert.equal(commonIndent([row('    <a>'), row(''), row('        <b>'), row('\t\t<c>')]), 2);
  assert.equal(commonIndent([]), 0);
  assert.deepEqual(stripIndent([part('    <a', false), part(' x', true)], 7), [part('x', true)]);
});

test('moreLabel and normalizeView', () => {
  assert.equal(moreLabel(1), 'Show 1 more line');
  assert.equal(moreLabel(12), 'Show 12 more lines');
  assert.equal(normalizeView('x'), 'simplified');
  assert.equal(normalizeView('detailed'), 'detailed');
});
