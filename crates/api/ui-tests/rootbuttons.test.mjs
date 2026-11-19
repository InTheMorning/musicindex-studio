// The home button and the title button of the narrow shell (musicindex ADR
// 0080 R-TITLE-1, ADR 0206 item 11).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { showsRootButtons, rootNav, backNav, canBack, canForward } from '../ui/shell.js';

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

test('the root action adds the root page after the current entry and removes the entries after it', () => {
  // rc.27 check finding 10: Forward from the root page opened the artist page.
  const start = nav(['root', 'releases', { name: 'feed', id: 7 }, { name: 'outline', id: 7 }, 'people'], 3);
  const next = rootNav(start);
  assert.equal(next.index, 4, `musicindex ADR 0080 R-TITLE-1 item 4: the root page is a new entry after the current entry, at its index + 1. ${fix}`);
  assert.equal(next.entries[4].name, 'root', `musicindex ADR 0080 R-TITLE-1 item 4: the new entry is the root page. ${fix}`);
  assert.equal(next.entries.length, 5,
    `musicindex ADR 0080 R-TITLE-1 item 4: the entries after the current entry go, so Forward has no page. ${fix}`);
  assert.equal(canForward(next), false, `musicindex ADR 0080 R-TITLE-1 item 4: Forward has no page after the action. ${fix}`);
  assert.deepEqual(backNav(next).entries[backNav(next).index], { name: 'outline', id: 7 },
    `musicindex ADR 0080 R-TITLE-1 item 4: Back on the new root page opens the page that the person left. ${fix}`);
  assert.equal(start.index, 3, `rootNav must not change its argument. ${fix}`);
});

test('the root action on the root page does not move', () => {
  const start = nav(['root', 'releases', 'root'], 2);
  assert.equal(rootNav(start), start);
});

test('a root page at a later entry has no back control', () => {
  assert.equal(canBack(nav(['root', 'releases', 'root'], 2)), false,
    `musicindex ADR 0080 R-SHELL-2: the root page has no back control, also at a later entry. Fix canBack in crates/api/ui/shell.js.`);
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
