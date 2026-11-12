// The look of a check box (musicindex ADR 0063).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({ selector: m[1].trim(), body: m[2] }));
// A selector that only excludes check boxes, with :not(:where(...)), does not select one.
const selectsBox = (selector) => /checkbox|\.check\b|\.pick-box|\.use-row|\.choice\s+input/.test(selector.replace(/:not\(:where\(\[type="checkbox"\]\)\)/g, ''));
const boxRules = rules.filter((r) => selectsBox(r.selector));
const mainRule = rules.find((r) => r.selector === 'input[type="checkbox"]');

test('no rule that selects a check box has accent-color', () => {
  for (const r of boxRules) {
    assert.ok(!/accent-color/.test(r.body),
      `musicindex ADR 0063 R-CHECK-4: "${r.selector}" sets accent-color. Delete it and draw the box in the rule input[type="checkbox"] in crates/api/ui/styles.css.`);
  }
});

test('the check box rule has the values of R-CHECK-1', () => {
  assert.ok(mainRule, 'musicindex ADR 0063 R-CHECK-4: styles.css has no rule input[type="checkbox"]. Add it with appearance: none.');
  assert.match(mainRule.body, /appearance:\s*none/, 'musicindex ADR 0063 R-CHECK-4: the box needs appearance: none so that the app draws it.');
  assert.match(mainRule.body, /width:\s*18px/, 'musicindex ADR 0063 R-CHECK-1 item 1: set width: 18px in the rule input[type="checkbox"].');
  assert.match(mainRule.body, /height:\s*18px/, 'musicindex ADR 0063 R-CHECK-1 item 1: set height: 18px in the rule input[type="checkbox"].');
  assert.match(mainRule.body, /border-radius:\s*4px/, 'musicindex ADR 0063 R-CHECK-1 item 2: set border-radius: 4px in the rule input[type="checkbox"].');
  assert.match(mainRule.body, /border:\s*1px solid/, 'musicindex ADR 0063 R-CHECK-1 item 3: set border: 1px solid var(--check-border) in the rule input[type="checkbox"].');
});

test('no other rule gives a check box its own size', () => {
  for (const r of boxRules) {
    if (r === mainRule || !/input/.test(r.selector) || /::(before|after)/.test(r.selector)) continue;
    assert.ok(!/(^|[;\s])(width|height)\s*:/.test(r.body),
      `musicindex ADR 0063 R-CHECK-4: "${r.selector}" sets the size of a check box. Delete it: every check box has the look of input[type="checkbox"].`);
  }
});

// R-CHECK-1 item 3: the border of a box has a contrast of 3:1 on each surface of its theme.
// The file styles.css has the values. The test composites a translucent color on the page color of its theme.
const declarations = (selector) => {
  const vars = {};
  for (const m of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    if (m[1].trim() !== selector) continue;
    for (const d of m[2].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) vars[d[1]] = d[2].trim();
  }
  return vars;
};
const themes = {
  dark: declarations(':root'),
  light: { ...declarations(':root'), ...declarations(':root[data-theme="light"]') },
};
const resolve = (theme, value) => {
  let v = value;
  for (let i = 0; i < 8; i++) {
    const m = v.match(/^var\((--[\w-]+)\)$/);
    if (!m) break;
    v = themes[theme][m[1]];
  }
  return v;
};
const parseColor = (text) => {
  const hex = text.match(/^#([0-9a-f]{6})$/i);
  if (hex) return { r: parseInt(hex[1].slice(0, 2), 16), g: parseInt(hex[1].slice(2, 4), 16), b: parseInt(hex[1].slice(4, 6), 16), a: 1 };
  const rgba = text.match(/^rgba?\(\s*(\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\s*\)$/);
  if (rgba) return { r: +rgba[1], g: +rgba[2], b: +rgba[3], a: rgba[4] === undefined ? 1 : +rgba[4] };
  throw new Error(`checkbox.test.mjs cannot read the color "${text}".`);
};
const over = (top, under) => ({
  r: top.r * top.a + under.r * (1 - top.a),
  g: top.g * top.a + under.g * (1 - top.a),
  b: top.b * top.a + under.b * (1 - top.a),
  a: 1,
});
const luminance = (c) => {
  const lin = (v) => { const s = v / 255; return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4; };
  return 0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b);
};
const contrast = (a, b) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};
// The surfaces that carry a box: body (--level-1), .group and dialog.sheet (--level-2), an open entry (--level-3).
const surfaces = ['--level-1', '--level-2', '--level-3'];

for (const theme of ['dark', 'light']) {
  for (const surface of surfaces) {
    test(`the check box border has a contrast of 3:1 on ${surface} in the ${theme} theme`, () => {
      const page = parseColor(resolve(theme, themes[theme]['--level-1']));
      const border = over(parseColor(resolve(theme, themes[theme]['--check-border'])), page);
      const ground = over(parseColor(resolve(theme, themes[theme][surface])), page);
      const ratio = contrast(border, ground);
      assert.ok(ratio >= 3,
        `musicindex ADR 0063 R-CHECK-1 item 3: --check-border has a contrast of ${ratio.toFixed(2)}:1 on ${surface} in the ${theme} theme. It needs 3:1. Change --check-border in crates/api/ui/styles.css, or the surface, so that the ratio is 3:1 or more.`);
    });
  }
}

// A rule for each input of a field must not reach a check box. The rule
// ".field input" gave the boxes of "Hide from apps" and "Explicit" a height
// of 44 px (2026-11-11).
test('no general input rule changes the size or the fill of a check box', () => {
  const shape = /(^|[;\s])(width|height|min-height|min-width|padding|background)\s*:/;
  for (const r of rules) {
    if (!shape.test(r.body)) continue;
    for (const part of r.selector.split(',').map((x) => x.trim())) {
      const last = part.split(/\s+|>|\+|~/).filter(Boolean).pop() || '';
      if (!/^input\b/.test(last) || /^input\[type=/.test(last) || /^input\./.test(last)) continue;
      assert.ok(last.includes(':not(:where([type="checkbox"]))'),
        `musicindex ADR 0063 R-CHECK-1: "${part}" sets the size or the fill of each input, also of a check box. Add :not(:where([type="checkbox"])) to the input of that selector in crates/api/ui/styles.css. :where keeps the specificity of the touch rule.`);
    }
  }
});
