// The collections of the desktop app (ADR 0149).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { collectionMark, collectionRow, collectionAction, collectionSheet } from '../ui/collectionlist.js';
import { leftPanel } from '../ui/shell.js';
import { pageSequence, tierError } from '../ui/pages.js';

const main = { folder: 'main', name: 'Main collection', feeds: 14, main: true, open: false };
const test4 = { folder: 'test', name: 'Test', feeds: 1, main: false, open: true };

test('The mark shows when the list has more than one collection (ADR 0176 item 12)', () => {
  assert.equal(collectionMark([main, test4]), 'Collection: Test');
  assert.equal(collectionMark([{ ...main, open: true }, { ...test4, open: false }]), 'Collection: Main collection');
  assert.equal(collectionMark([{ ...main, open: true }]), null);
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

test('A closed row has Open, Rename, Replace and Delete (ADR 0176 item 2)', () => {
  assert.deepEqual(collectionRow(main), {
    name: 'Main collection', line: '14 feeds', tag: null,
    canOpen: true, canRename: true, canReplace: true, canDelete: true,
  });
});

test('The open row has Rename and the tag Open now, and no Replace and no Delete (ADR 0176 item 3)', () => {
  assert.deepEqual(collectionRow(test4), {
    name: 'Test', line: '1 feed', tag: 'Open now',
    canOpen: false, canRename: true, canReplace: false, canDelete: false,
  });
});

test('The sheets of replace and delete give the name, the count and the parts that go (ADR 0176 items 6 and 7)', () => {
  for (const kind of ['replace', 'delete']) {
    const sheet = collectionAction(kind, main);
    assert.match(sheet.title, /Main collection/);
    assert.match(sheet.lines[0], /14 feeds/);
    const text = sheet.lines.join(' ');
    for (const part of ['feeds', 'drafts', 'versions', 'people', 'vault']) assert.match(text, new RegExp(part));
    assert.match(text, /Nothing changes on a file host/);
    assert.equal(sheet.exportFirst, true);
    assert.equal(sheet.destructive, true);
  }
  assert.equal(collectionAction('replace', main).file, true);
  assert.equal(collectionAction('delete', main).file, false);
  const rename = collectionAction('rename', main);
  assert.equal(rename.start, 'Main collection');
  assert.equal(rename.exportFirst, false);
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

test('Move Studio shows only in a shell with no list of collections (ADR 0176 items 10 and 11)', () => {
  const ids = (options) => pageSequence('studio-settings', options).map((s) => s.id);
  assert.ok(!ids({ server: false, collections: true }).includes('move'));
  const bare = pageSequence('studio-settings', { server: false, collections: false });
  assert.ok(bare.some((s) => s.id === 'move') && !bare.some((s) => s.id === 'collections'));
  assert.equal(tierError('studio-settings', bare), null);
});
