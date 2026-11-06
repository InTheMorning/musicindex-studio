// The UI scale (ADR 0134): the levels of v4vmm, with a mouse only.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { UI_SCALES, DEFAULT_SCALE, normalizeScale } from '../ui/uiscale.js';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
const owner = 'ADR 0134: each UI scale level has the factor of v4vmm in styles.css, in a block with no coarse pointer.';

test('a level that is not known gives the default', () => {
  assert.equal(normalizeScale('large'), 'large');
  assert.equal(normalizeScale('huge'), DEFAULT_SCALE);
  assert.equal(normalizeScale(null), DEFAULT_SCALE);
});

test('each level has the factor of v4vmm, and touch has no factor', () => {
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
  const rest = css.replace(/--window-(height|width): calc\(100(dvh|vw) \/ var\(--ui-zoom\)\);/g, '').replace(/\/\*[\s\S]*?\*\//g, '');
  assert.doesNotMatch(rest, /[0-9](dvh|svh|lvh|vh|vw)\b/, 'ADR 0134: a zoom also scales dvh and vw, so the shell does not fit the window. Use var(--window-height) or var(--window-width).');
});
