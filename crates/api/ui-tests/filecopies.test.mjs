// The copy rows of the Files page (ADR 0180 items 13 to 17).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
const copyRow = editor.slice(editor.indexOf('function CopyRow('), editor.indexOf('function FileEntry('));
const fileEntry = editor.slice(editor.indexOf('function FileEntry('), editor.indexOf('function AddFileAgain('));

test('the copy in use has no action and each other copy has two actions', () => {
  assert.match(copyRow, /if \(copy\.in_use\)[\s\S]*Used by the feed/);
  const inUse = copyRow.slice(copyRow.indexOf('if (copy.in_use)'), copyRow.indexOf('return html', copyRow.indexOf('if (copy.in_use)')));
  assert.doesNotMatch(inUse, /<button/);
  const otherCopy = copyRow.slice(copyRow.indexOf('return html', copyRow.indexOf('if (copy.in_use)')));
  assert.match(otherCopy, /Use This Copy/);
  assert.match(otherCopy, />Delete<\/button>/);
  assert.doesNotMatch(copyRow, /Move to /);
});

test('a pending copy can become the copy in use and has Delete', () => {
  const pending = fileEntry.slice(fileEntry.indexOf('file.pending_copies.map'));
  assert.match(pending, /Use This Copy/);
  assert.match(pending, />Delete<\/button>/);
  assert.doesNotMatch(fileEntry, /File host of this file/);
  assert.doesNotMatch(fileEntry, /op: 'move'/);
});
