// The home button and the title button of the narrow shell (musicindex ADR
// 0080 R-TITLE-1, ADR 0206 item 11).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { showsRootButtons, rootNav } from '../ui/shell.js';

const fix = 'Fix the function in crates/api/ui/shell.js.';
const nav = (views, index) => ({ entries: views.map((v) => (typeof v === 'string' ? { name: v } : v)), index });

test('the two root buttons show in a narrow window on each page other than the root page', () => {
  assert.equal(showsRootButtons(nav(['root', 'releases'], 1), true), true,
    `musicindex ADR 0080 R-TITLE-1 item 1: a page other than the root page has the home button and the title button. ${fix}`);
  assert.equal(showsRootButtons(nav(['root', 'releases'], 0), true), false,
    `musicindex ADR 0080 R-TITLE-1 item 9: the root page has no home button and no title button. ${fix}`);
  assert.equal(showsRootButtons(nav(['releases'], 0), false), false,
    `musicindex ADR 0080 R-TITLE-1 item 11: a wide window has no home button and no title button. ${fix}`);
  assert.equal(showsRootButtons(null, true), false);
});

test('the root action goes back to the root entry and keeps the entries after it', () => {
  const start = nav(['root', 'releases', { name: 'outline', id: 7 }, { name: 'feed', id: 7, page: 'tracks' }], 3);
  const next = rootNav(start);
  assert.equal(next.index, 0, `musicindex ADR 0080 R-TITLE-1 item 4: the action goes to the root entry. ${fix}`);
  assert.equal(next.entries[0].name, 'root');
  assert.equal(next.entries.length, 4,
    `musicindex ADR 0080 R-TITLE-1 item 4: the stack keeps the pages after the root entry for Forward. ${fix}`);
  assert.equal(start.index - next.index, 3, 'musicindex ADR 0080 R-TITLE-1 item 4: n is the index of the current entry.');
  assert.equal(start.index, 3, `rootNav must not change its argument. ${fix}`);
});

test('the root action on the root page does not move', () => {
  const start = nav(['root', 'releases'], 0);
  assert.equal(rootNav(start).index, 0);
});

// The order of the bar is in the template of `Bar`, and no Node test renders
// it. This check reads the source: the home button comes before the back
// control, and the title button comes after Forward (R-SHELL-2 item 6).
test('the home button is the first control of the bar, and the title button follows Forward', () => {
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = source.indexOf('export function Bar(');
  const body = source.slice(start, source.indexOf('\n}\n', start));
  const home = body.indexOf('class="bar-home"');
  const back = body.indexOf('aria-label="Back"');
  const forward = body.indexOf('aria-label="Forward"');
  const title = body.indexOf('class="bar-title-button"');
  const order = 'musicindex ADR 0080 R-SHELL-2 item 6: the order is the home button, Back, Forward, then the title button. '
    + 'Fix the template of Bar in crates/api/ui/app.js.';
  assert.ok(home >= 0 && back >= 0 && forward >= 0 && title >= 0, order);
  assert.ok(home < back && back < forward && forward < title, order);
  assert.match(body, /class="bar-home"[^>]*aria-label="Destinations"/,
    'musicindex ADR 0080 R-TITLE-1 item 7: the accessible name of the home button is "Destinations". Fix Bar in crates/api/ui/app.js.');
});
