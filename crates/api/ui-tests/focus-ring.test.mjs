// The focus ring gap on a control with the accent fill (musicindex ADR 0212
// R-FOCUS-2 item 6). In the Dark theme the focus color and the accent color
// are the same, so a ring with no gap on a primary button does not show.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
const tokens = JSON.parse(readFileSync(new URL('../ui/assets/tokens.json', import.meta.url), 'utf8'));
const owner = 'musicindex ADR 0212 R-FOCUS-2 item 6';
const fix = 'Copy tokens.json from musicindex-assets to crates/api/ui/assets/, and change styles.css to its values.';

test('the gap of the focus ring has the value of tokens.json', () => {
  const want = `${tokens['focus-ring'].studio['filled-offset']}px`;
  const found = [...css.matchAll(/(?<![\w-])--focus-filled-offset\s*:\s*([^;}]+)/g)].map((m) => m[1].trim());
  assert.deepEqual(found, [want], `${owner}: --focus-filled-offset differs from tokens.json. ${fix}`);
});

test('each control with the accent fill uses the gap', () => {
  for (const selector of ['.button.primary:focus-visible', 'input[type="checkbox"]:focus-visible']) {
    const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const match = css.match(new RegExp(`(?:^|\\n)${escaped}\\s*\\{([^}]*)\\}`));
    assert.ok(match && /outline-offset:\s*var\(--focus-filled-offset\)/.test(match[1]),
      `${owner}: ${selector} must have outline-offset: var(--focus-filled-offset). Without the gap, the ring has the color of the fill.`);
  }
});
