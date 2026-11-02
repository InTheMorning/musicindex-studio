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

// ---- What changes (task 077c) ----
import { readFileSync } from 'node:fs';
import { fieldRows, guardRow, changeRow, KEY_WORDS, pathWord, inSentence } from '../ui/review.js';

const KEY = 'npub1' + 'q'.repeat(58);
const credit = (role) => ({ op: 'changed', path: 'npub', entry: role, old: null, new: KEY });
const profiles = new Map([[KEY, { npub: KEY, name: 'City Radio' }]]);

test('six changes of one key give a summary, two rows and a collapsed line', () => {
  const roles = ['musician', 'audio engineer', 'songwriter', 'composer', 'guitarist', 'keyboardist'];
  const [group] = fieldRows([{ kind: 'credits', name: 'City Radio', changes: roles.map(credit) }], profiles);
  assert.equal(group.label, 'Credits · City Radio');
  assert.equal(group.summary, '6 credits get the Nostr key City Radio');
  assert.equal(group.rows.length, 2);
  assert.equal(group.rows[0].label, 'musician');
  assert.match(group.rows[0].text, /Nostr key: none → City Radio \(/);
  assert.equal(group.more.summary, '4 more credits: songwriter, composer, guitarist, keyboardist');
  assert.equal(group.more.rows.length, 4);
});

test('a title, an added entry and a removed entry have a line', () => {
  const title = changeRow({ kind: 'field', name: 'title' }, { op: 'changed', path: '', old: 'A', new: 'B' });
  assert.deepEqual(title, { label: 'Title', text: 'A → B', warning: null });
  const live = changeRow({ kind: 'live', name: 'Show' }, { op: 'changed', path: 'item.title', entry: null, old: 'A', new: 'B' });
  assert.equal(live.label, 'Title');
  assert.equal(pathWord('item.title'), 'Title');
  const add = changeRow({ kind: 'credits', name: 'X' }, { op: 'added', path: '', entry: 'mixer', old: null, new: { name: 'X' } });
  assert.deepEqual([add.label, add.text], ['mixer', 'Added']);
  const gone = changeRow({ kind: 'track', name: 'T', position: 1 }, { op: 'removed', path: 'persons', entry: null, old: { name: 'Zed' }, new: null });
  assert.deepEqual([gone.label, gone.text], ['Credits', 'Removed: Zed']);
});

test('a key line shows the profile, the short key or the warning', () => {
  const group = { kind: 'credits', name: 'HeyCitizen' };
  const withProfile = changeRow(group, credit('musician'), profiles);
  assert.match(withProfile.text, /City Radio \(npub1/);
  assert.equal(withProfile.warning, 'The profile name is not "HeyCitizen". Check this key.');
  const without = changeRow(group, credit('musician'), null);
  assert.doesNotMatch(without.text, /City Radio/);
  assert.equal(without.warning, null);
  assert.equal(changeRow({ ...group, name: 'City Radio' }, credit('musician'), profiles).warning, null);
});

test('a track credit key shows the warning from the name of the credit', () => {
  const track = { kind: 'track', name: 'Song', position: 1 };
  const change = { ...credit('musician'), path: 'persons.npub', name: 'HeyCitizen' };
  assert.equal(changeRow(track, change, profiles).warning, 'The profile name is not "HeyCitizen". Check this key.');
  assert.equal(changeRow(track, change, null).warning, null);
  assert.equal(changeRow(track, { ...change, name: undefined }, profiles).warning, null);
});

test('the guard row counts the uncovered changes', () => {
  assert.equal(guardRow(0), null);
  assert.equal(guardRow(1).title, '1 more change in the XML');
  assert.match(guardRow(1).text, /does not show it/);
  assert.equal(guardRow(3).title, '3 more changes in the XML');
  assert.match(guardRow(3).text, /does not show them/);
});

test('the word table has a word for each key of the feed model', () => {
  const source = readFileSync(new URL('../../core/src/feed.rs', import.meta.url), 'utf8');
  const keys = new Set();
  for (const block of source.matchAll(/pub struct (\w+)[^{]*\{([\s\S]*?)\n\}/g)) {
    for (const m of block[2].matchAll(/^\s*pub (\w+):/gm)) keys.add(m[1]);
  }
  assert.ok(keys.size > 40);
  const missing = [...keys].filter((k) => !(k in KEY_WORDS));
  assert.deepEqual(missing, []);
});

test('a word in a summary line keeps sentence case', () => {
  assert.equal(inSentence('Nostr key'), 'Nostr key');
  assert.equal(inSentence('Picture'), 'picture');
  assert.equal(inSentence('Information address'), 'information address');
  const change = (entry) => ({ op: 'changed', path: 'img', entry, old: null, new: 'https://a.example/p.jpg' });
  const [group] = fieldRows([{ kind: 'credits', name: 'Ann', changes: ['a', 'b', 'c'].map(change) }], null);
  assert.equal(group.summary, '3 credits get the picture https://a.example/p.jpg');
});
