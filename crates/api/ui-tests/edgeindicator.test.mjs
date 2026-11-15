// The edge indicator of musicindex ADR 0071 (R-EDGE-1 to R-EDGE-3) in the narrow
// shell of Studio (ADR 0206 item 11).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { showsEdgeIndicator } from '../ui/shell.js';

const read = (name) => readFileSync(new URL(`../ui/${name}`, import.meta.url), 'utf8');
const fix = 'Fix showsEdgeIndicator in crates/api/ui/shell.js.';
const nav = (names, index) => ({ entries: names.map((name) => ({ name })), index });

test('the indicator shows only when an entry of Studio is below the current entry (R-EDGE-2)', () => {
  const start = nav(['root', 'releases'], 1);
  assert.equal(showsEdgeIndicator(start, true, true, true), true,
    `musicindex ADR 0071 R-EDGE-2 item 3: a page above the root page has the indicator. ${fix}`);
  assert.equal(showsEdgeIndicator(nav(['root', 'releases', 'outline', 'feed'], 3), true, true, true), true);
  assert.equal(showsEdgeIndicator(nav(['root', 'releases'], 0), true, true, true), false,
    `musicindex ADR 0071 R-EDGE-2 item 4: the root page has no indicator. ${fix}`);
  assert.equal(showsEdgeIndicator(nav(['releases'], 0), true, true, true), false,
    `musicindex ADR 0071 R-EDGE-2 item 3: with no entry below, no indicator shows. ${fix}`);
  assert.equal(showsEdgeIndicator(start, true, false, true), false,
    `musicindex ADR 0071 R-EDGE-2 item 1: the Desktop profile has no indicator. ${fix}`);
  assert.equal(showsEdgeIndicator(start, false, true, true), false,
    `musicindex ADR 0071 R-EDGE-2 item 2: a wide window has no indicator. ${fix}`);
  assert.equal(showsEdgeIndicator(start, true, true, false), false,
    `musicindex ADR 0071 R-EDGE-2 item 3: when the browser history does not agree with the state, no indicator shows. ${fix}`);
  assert.equal(showsEdgeIndicator(null, true, true, true), false);
});

test('the indicator has the values of the token shape.edge-indicator and is not a control (R-EDGE-1, R-EDGE-3)', () => {
  const indicator = JSON.parse(read('assets/tokens.json')).shape['edge-indicator'];
  assert.deepEqual(indicator, { 'width-px': 5, 'height-px': 48, 'inset-px': 4, radius: 'pill', fill: 'color.text-muted', opacity: 0.8 },
    'musicindex ADR 0071 R-EDGE-3: copy tokens.json of musicindex-assets to crates/api/ui/assets/tokens.json.');
  const css = read('styles.css');
  const start = css.indexOf('.edge-indicator {');
  assert.ok(start >= 0, 'musicindex ADR 0071 R-EDGE-1: add the rule .edge-indicator to crates/api/ui/styles.css.');
  const rule = css.slice(start, css.indexOf('}', start));
  const has = (text, item) => assert.ok(rule.includes(text),
    `musicindex ADR 0071 R-EDGE-1 item ${item}: the rule .edge-indicator needs "${text}". Fix crates/api/ui/styles.css.`);
  has(`width: ${indicator['width-px']}px;`, 1);
  has(`height: ${indicator['height-px']}px;`, 1);
  has(`left: ${indicator['inset-px']}px;`, 2);
  has('position: fixed;', 2);
  has('background: var(--muted);', 3);
  has(`opacity: ${indicator.opacity};`, 3);
  has('pointer-events: none;', 4);
  assert.ok(!/animation|transition/.test(rule),
    'musicindex ADR 0071 R-EDGE-1 item 5: the indicator has no animation. Fix crates/api/ui/styles.css.');
  const app = read('app.js');
  assert.ok(app.includes('showsEdgeIndicator(nav, narrow, coarse, mirrored.current) && html`<div class="edge-indicator" aria-hidden="true"></div>`'),
    'musicindex ADR 0071 R-EDGE-1 item 4: the indicator is one element with aria-hidden, and showsEdgeIndicator decides if it shows. Fix the shell in crates/api/ui/app.js.');
});
