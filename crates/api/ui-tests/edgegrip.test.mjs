// The edge grip of musicindex ADR 0071 (R-GRIP-1 to R-GRIP-3) in the narrow
// shell of Studio (ADR 0206 item 11).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { showsEdgeGrip } from '../ui/shell.js';

const read = (name) => readFileSync(new URL(`../ui/${name}`, import.meta.url), 'utf8');
const fix = 'Fix showsEdgeGrip in crates/api/ui/shell.js.';
const nav = (names, index) => ({ entries: names.map((name) => ({ name })), index });

test('the grip shows only when an entry of Studio is below the current entry (R-GRIP-2)', () => {
  const start = nav(['root', 'releases'], 1);
  assert.equal(showsEdgeGrip(start, true, true, true), true,
    `musicindex ADR 0071 R-GRIP-2 item 3: a page above the root page has the grip. ${fix}`);
  assert.equal(showsEdgeGrip(nav(['root', 'releases', 'outline', 'feed'], 3), true, true, true), true);
  assert.equal(showsEdgeGrip(nav(['root', 'releases'], 0), true, true, true), false,
    `musicindex ADR 0071 R-GRIP-2 item 4: the root page has no grip. ${fix}`);
  assert.equal(showsEdgeGrip(nav(['releases'], 0), true, true, true), false,
    `musicindex ADR 0071 R-GRIP-2 item 3: with no entry below, no grip shows. ${fix}`);
  assert.equal(showsEdgeGrip(start, true, false, true), false,
    `musicindex ADR 0071 R-GRIP-2 item 1: the Desktop profile has no grip. ${fix}`);
  assert.equal(showsEdgeGrip(start, false, true, true), false,
    `musicindex ADR 0071 R-GRIP-2 item 2: a wide window has no grip. ${fix}`);
  assert.equal(showsEdgeGrip(start, true, true, false), false,
    `musicindex ADR 0071 R-GRIP-2 item 3: when the browser history does not agree with the state, no grip shows. ${fix}`);
  assert.equal(showsEdgeGrip(null, true, true, true), false);
});

test('the grip has the values of the token shape.edge-grip and is not a control (R-GRIP-1, R-GRIP-3)', () => {
  const grip = JSON.parse(read('assets/tokens.json')).shape['edge-grip'];
  assert.deepEqual(grip, { 'width-px': 5, 'height-px': 48, 'inset-px': 4, radius: 'pill', fill: 'color.text-muted', opacity: 0.8 },
    'musicindex ADR 0071 R-GRIP-3: copy tokens.json of musicindex-assets to crates/api/ui/assets/tokens.json.');
  const css = read('styles.css');
  const start = css.indexOf('.edge-grip {');
  assert.ok(start >= 0, 'musicindex ADR 0071 R-GRIP-1: add the rule .edge-grip to crates/api/ui/styles.css.');
  const rule = css.slice(start, css.indexOf('}', start));
  const has = (text, item) => assert.ok(rule.includes(text),
    `musicindex ADR 0071 R-GRIP-1 item ${item}: the rule .edge-grip needs "${text}". Fix crates/api/ui/styles.css.`);
  has(`width: ${grip['width-px']}px;`, 1);
  has(`height: ${grip['height-px']}px;`, 1);
  has(`left: ${grip['inset-px']}px;`, 2);
  has('position: fixed;', 2);
  has('background: var(--muted);', 3);
  has(`opacity: ${grip.opacity};`, 3);
  has('pointer-events: none;', 4);
  assert.ok(!/animation|transition/.test(rule),
    'musicindex ADR 0071 R-GRIP-1 item 5: the grip has no animation. Fix crates/api/ui/styles.css.');
  const app = read('app.js');
  assert.ok(app.includes('showsEdgeGrip(nav, narrow, coarse, mirrored.current) && html`<div class="edge-grip" aria-hidden="true"></div>`'),
    'musicindex ADR 0071 R-GRIP-1 item 4: the grip is one element with aria-hidden, and showsEdgeGrip decides if it shows. Fix the shell in crates/api/ui/app.js.');
});
