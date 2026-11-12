// The look of a check box (musicindex ADR 0063).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({ selector: m[1].trim(), body: m[2] }));
const selectsBox = (selector) => /checkbox|\.check\b|\.pick-box|\.use-row|\.choice\s+input/.test(selector);
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
