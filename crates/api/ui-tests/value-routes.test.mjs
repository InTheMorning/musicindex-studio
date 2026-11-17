// The sizes of the value routes (musicindex ADR 0082 R-ROUTES-7). Each size
// is one custom property in styles.css with the name of its token, and its
// value agrees with tokens.json.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
const { shape } = JSON.parse(readFileSync(new URL('../ui/assets/tokens.json', import.meta.url), 'utf8'));
const owner = 'musicindex ADR 0082 R-ROUTES-7';
const fix = 'Copy tokens.json from musicindex-assets to crates/api/ui/assets/, and change the custom property in styles.css to its value.';

const want = {
  '--radius-value-panel': `${shape.radius['value-panel']}px`,
  '--share-stack-width': `${shape['share-stack'].width}px`,
  '--share-stack-height': `${shape['share-stack'].height}px`,
  '--share-stack-end-radius': `${shape['share-stack']['end-radius']}px`,
  '--share-stack-gap': `${shape['share-stack'].gap}px`,
  '--share-stack-minimum-segment': `${shape['share-stack']['minimum-segment']}px`,
  '--share-stack-fee-stripe': `${shape['share-stack']['fee-stripe']}px`,
  '--share-meter-width': `${shape['share-meter'].width}px`,
  '--share-meter-height': `${shape['share-meter'].height}px`,
  '--share-meter-radius': `${shape['share-meter'].radius}px`,
  '--value-columns-share-rem': `${shape['value-columns']['share-rem']}rem`,
  '--value-columns-address-maximum-rem': `${shape['value-columns']['address-maximum-rem']}rem`,
};

test('each size of the value routes has the value of tokens.json', () => {
  for (const [name, value] of Object.entries(want)) {
    const found = [...css.matchAll(new RegExp(`(?<![\\w-])${name}\\s*:\\s*([^;}]+)`, 'g'))].map((m) => m[1].trim());
    assert.deepEqual(found, [value], `${owner}: ${name} differs from tokens.json. ${fix}`);
  }
});
