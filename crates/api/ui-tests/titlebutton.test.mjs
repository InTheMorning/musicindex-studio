// The title button of the narrow shell (musicindex ADR 0079 R-TITLE-1, ADR 0206
// item 11).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { showsTitleButton, rootNav } from '../ui/shell.js';

const fix = 'Fix the function in crates/api/ui/shell.js.';
const nav = (views, index) => ({ entries: views.map((v) => (typeof v === 'string' ? { name: v } : v)), index });

test('the title button shows in a narrow window on each page other than the root page', () => {
  assert.equal(showsTitleButton(nav(['root', 'releases'], 1), true), true,
    `musicindex ADR 0079 R-TITLE-1 item 1: a page other than the root page has the title button. ${fix}`);
  assert.equal(showsTitleButton(nav(['root', 'releases'], 0), true), false,
    `musicindex ADR 0079 R-TITLE-1 item 9: the root page has no title button. ${fix}`);
  assert.equal(showsTitleButton(nav(['releases'], 0), false), false,
    `musicindex ADR 0079 R-TITLE-1 item 11: a wide window has no title button. ${fix}`);
  assert.equal(showsTitleButton(null, true), false);
});

test('the root action goes back to the root entry and keeps the entries after it', () => {
  const start = nav(['root', 'releases', { name: 'outline', id: 7 }, { name: 'feed', id: 7, page: 'tracks' }], 3);
  const next = rootNav(start);
  assert.equal(next.index, 0, `musicindex ADR 0079 R-TITLE-1 item 4: the action goes to the root entry. ${fix}`);
  assert.equal(next.entries[0].name, 'root');
  assert.equal(next.entries.length, 4,
    `musicindex ADR 0079 R-TITLE-1 item 4: the stack keeps the pages after the root entry for Forward. ${fix}`);
  assert.equal(start.index - next.index, 3, 'musicindex ADR 0079 R-TITLE-1 item 4: n is the index of the current entry.');
  assert.equal(start.index, 3, `rootNav must not change its argument. ${fix}`);
});

test('the root action on the root page does not move', () => {
  const start = nav(['root', 'releases'], 0);
  assert.equal(rootNav(start).index, 0);
});
