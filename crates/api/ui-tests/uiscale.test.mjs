// The UI scale (ADR 0134): the levels of HitIt, with a mouse only.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { UI_SCALES, DEFAULT_SCALE, normalizeScale, windowUnitZoom } from '../ui/uiscale.js';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
const owner = 'ADR 0134: each UI scale level has the factor of HitIt in styles.css, in a block with no coarse pointer.';

test('a level that is not known gives the default', () => {
  assert.equal(normalizeScale('large'), 'large');
  assert.equal(normalizeScale('huge'), DEFAULT_SCALE);
  assert.equal(normalizeScale(null), DEFAULT_SCALE);
});

test('each level has the factor of HitIt, and touch has no factor', () => {
  const factors = { 'x-small': '0.85', small: '0.92', large: '1.12', 'x-large': '1.25' };
  assert.deepEqual(UI_SCALES.map(([id]) => id), ['x-small', 'small', 'medium', 'large', 'x-large'], owner);
  const start = css.indexOf('@media not (pointer: coarse) {\n  /* ADR 0134');
  assert.ok(start >= 0, owner);
  const block = css.slice(start, css.indexOf('\n}', start));
  for (const [level, factor] of Object.entries(factors)) {
    assert.ok(block.includes(`:root[data-ui-scale="${level}"] { zoom: ${factor}; --ui-zoom: ${factor}; }`), `${owner} Level ${level}.`);
  }
});

test('only the window tokens use a unit of the window', () => {
  const rest = css.replace(/--window-(height|width): calc\(100(dvh|vw) \/ var\(--window-unit-zoom\)\);/g, '').replace(/\/\*[\s\S]*?\*\//g, '');
  assert.doesNotMatch(rest, /[0-9](dvh|svh|lvh|vh|vw)\b/, 'ADR 0134: some engines scale dvh and vw by the zoom, so the shell does not fit the window. Use var(--window-height) or var(--window-width).');
});

test('the measured boxes give the zoom of a unit of the window', () => {
  // Chromium and WebKitGTK 2.54 scale 100dvh by the zoom: 963.2 px in a window of 860 px.
  assert.equal(windowUnitZoom(963.2, 860, 1.12), 1.12);
  assert.equal(windowUnitZoom(731, 860, 0.85), 0.85);
  // An engine that does not scale the unit gives the height of the window.
  assert.equal(windowUnitZoom(860, 860, 1.12), 1);
  assert.equal(windowUnitZoom(860.4, 860, 0.92), 1);
  // A box with no size keeps the zoom of the page.
  assert.equal(windowUnitZoom(0, 0, 1.25), 1.25);
});
