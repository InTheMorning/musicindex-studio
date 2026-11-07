// The collections of the desktop app (ADR 0149).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { collectionMark, collectionRow, collectionSheet } from '../ui/collectionlist.js';
import { leftPanel } from '../ui/shell.js';
import { pageSequence, tierError } from '../ui/pages.js';

const main = { folder: 'main', name: 'Main collection', feeds: 14, main: true, open: false };
const test4 = { folder: 'test', name: 'Test', feeds: 1, main: false, open: true };

test('The mark shows only when the open collection is not Main (ADR 0149 item 17)', () => {
  assert.equal(collectionMark([main, test4]), 'Collection: Test');
  assert.equal(collectionMark([{ ...main, open: true }, { ...test4, open: false }]), null);
  assert.equal(collectionMark(null), null);
});

test('The left panel gives the mark after Settings, and it opens Settings (ADR 0149 item 17)', () => {
  const base = { view: { name: 'releases' }, feeds: [], model: null, vaultOpen: true };
  const ids = (panel) => panel.bottom.map((e) => e.id);
  assert.deepEqual(ids(leftPanel(base)), ['people', 'hosts', 'settings', 'lock']);
  const panel = leftPanel({ ...base, collection: 'Collection: Test' });
  assert.deepEqual(ids(panel), ['people', 'hosts', 'settings', 'collection', 'lock']);
  const row = panel.bottom.find((e) => e.id === 'collection');
  assert.equal(row.label, 'Collection: Test');
  assert.equal(row.current, false);
});

test('A row of the open collection and of Main has no Delete (ADR 0149 item 12)', () => {
  assert.deepEqual(collectionRow(main), { name: 'Main collection', line: '14 feeds', canOpen: true, canDelete: false });
  assert.deepEqual(collectionRow(test4), { name: 'Test', line: '1 feed', canOpen: false, canDelete: false });
  assert.equal(collectionRow({ ...test4, open: false }).canDelete, true);
});

test('The sheet of a copy gives its size and the warning (ADR 0149 item 9)', () => {
  const sheet = collectionSheet('copy', 'Production', '12 MB');
  assert.equal(sheet.title, 'Copy Production');
  assert.match(sheet.lines[0], /12 MB/);
  assert.match(sheet.warning.strong, /same file hosts/);
  assert.equal(collectionSheet('new', 'Production').warning, null);
  assert.equal(collectionSheet('import', 'Production').warning, null);
});

test('The section has a tier, and a server has none (ADR 0149 item 5, ADR 0071)', () => {
  const desktop = pageSequence('studio-settings', { server: false });
  assert.ok(desktop.some((s) => s.id === 'collections'));
  assert.equal(tierError('studio-settings', desktop), null);
  assert.ok(!pageSequence('studio-settings', { server: true }).some((s) => s.id === 'collections'));
});
