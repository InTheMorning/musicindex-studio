// The collections of the desktop app (ADR 0149).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { collectionMark, collectionRow, collectionSectionActions, collectionAction, collectionSheet, switchCollectionRows } from '../ui/collectionlist.js';
import { pageSequence, tierError } from '../ui/pages.js';

const main = { folder: 'main', name: 'Main collection', feeds: 14, main: true, open: false };
const test4 = { folder: 'test', name: 'Test', feeds: 1, main: false, open: true };

test('ADR 0241 item 5: switchCollectionRows gives one action row for each collection', () => {
  const rows = switchCollectionRows([main, test4]);
  assert.deepEqual(rows, [
    { folder: 'main', name: 'Main collection', open: false, status: null, action: 'open' },
    { folder: 'test', name: 'Test', open: true, status: 'Open now', action: null },
  ], 'ADR 0241 item 5: fix switchCollectionRows in crates/api/ui/collectionlist.js.');
});

test('wallet and collection rows use the row menu control (ADR 0207 items 53 to 56, ADR 0183 item 1)', () => {
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const people = readFileSync(new URL('../ui/peoplepages.js', import.meta.url), 'utf8');
  const addressRow = readFileSync(new URL('../ui/addressrow.js', import.meta.url), 'utf8');
  assert.match(app, /<\$\{MenuControl\} row=\$\{true\} Icon=\$\{Icon\} buttonLabel="Actions" accessibleName=\$\{`Actions for \$\{collection\.name\}`\}/,
    'ADR 0183 item 1: pass row to MenuControl in the collection rows of crates/api/ui/app.js.');
  assert.match(addressRow, /<\$\{MenuControl\} row=\$\{true\} Icon=\$\{Icon\}/,
    'ADR 0207 item 53: pass row to MenuControl in the shared address row of crates/api/ui/addressrow.js.');
  const ui = new URL('../ui/', import.meta.url);
  for (const name of readdirSync(ui).filter((file) => file.endsWith('.js'))) {
    const source = readFileSync(new URL(name, ui), 'utf8');
    assert.ok(!/<button\b[^>]*>\s*…\s*<\/button>/.test(source),
      `ADR 0207 item 56: do not use the ellipsis character as a button label in ${name}. Use the ellipsis icon with an accessible name.`);
  }
});

test('The mark shows when the list has more than one collection (ADR 0176 item 12)', () => {
  assert.equal(collectionMark([main, test4]), 'Collection: Test');
  assert.equal(collectionMark([{ ...main, open: true }, { ...test4, open: false }]), 'Collection: Main collection');
  assert.equal(collectionMark([{ ...main, open: true }]), null);
  assert.equal(collectionMark(null), null);
});

test('The open row has the Actions and Current badge of ADR 0183 items 2 and 6', () => {
  assert.deepEqual(collectionRow(test4), {
    name: 'Test', line: '1 feed', badge: 'Current',
    actions: [
      { label: 'Rename…', action: 'rename' },
      { label: 'Copy…', action: 'copy' },
      { label: 'Export…', action: 'export' },
    ],
  });
  assert.equal(collectionRow(test4).tag, undefined);
});

test('A closed row has the Actions of ADR 0183 item 3', () => {
  assert.deepEqual(collectionRow(main), {
    name: 'Main collection', line: '14 feeds', badge: null,
    actions: [
      { label: 'Open', action: 'open' },
      { label: 'Rename…', action: 'rename' },
      { label: 'Export…', action: 'export' },
      { label: 'Replace with a File…', action: 'replace' },
      { label: 'Delete…', action: 'delete', separatorBefore: true, destructive: true },
    ],
  });
});

test('The section actions below the list are New and Import only (ADR 0183 item 5)', () => {
  assert.deepEqual(collectionSectionActions(), [
    { label: 'New Collection…', action: 'new' },
    { label: 'Import an Account File…', action: 'import' },
  ]);
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
