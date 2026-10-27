// The type dot and the entity colors (musicindex ADR 0032 R-DOT-1 and
// R-DOT-2). styles.css keeps the values of its copy of tokens.json.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { liveDot } from '../ui/live.js';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8');
const tokens = JSON.parse(readFileSync(new URL('../ui/assets/tokens.json', import.meta.url), 'utf8'));
const kinds = Object.keys(tokens.entity.dark);
const owner = 'musicindex ADR 0032 R-DOT-2';
const fix = 'Copy tokens.json from musicindex-assets to crates/api/ui/assets/, and change styles.css to its values.';

// Each value of a custom property in styles.css, in lower case.
function values(name) {
  const found = [...css.matchAll(new RegExp(`(?<![\\w-])${name}\\s*:\\s*([^;}]+)`, 'g'))];
  return new Set(found.map((match) => match[1].trim().toLowerCase()));
}

// The declarations of the rule with this exact selector.
function rule(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const match = css.match(new RegExp(`(?:^|\\n)${escaped}\\s*\\{([^}]*)\\}`));
  return match ? match[1] : null;
}

test('each entity color has the dark and the light value of tokens.json', () => {
  for (const kind of kinds) {
    const want = new Set([tokens.entity.dark[kind], tokens.entity.light[kind]].map((v) => v.toLowerCase()));
    assert.deepEqual(values(`--c-${kind}`), want, `${owner}: --c-${kind} differs from tokens.json. ${fix}`);
  }
});

test('the dot and its ring have the dimensions of tokens.json', () => {
  const dot = tokens.shape['type-dot'];
  assert.deepEqual(values('--dot-size'), new Set([`${dot['size-px']}px`]), `${owner}: --dot-size differs from tokens.json. ${fix}`);
  assert.deepEqual(values('--dot-ring'), new Set([`${dot['hollow-ring-px']}px`]), `${owner}: --dot-ring differs from tokens.json. ${fix}`);
  const base = rule('.dot');
  assert.ok(base, `${owner}: styles.css has no .dot rule. Add it with the width and the height var(--dot-size).`);
  assert.match(base, /width:\s*var\(--dot-size\)/, `${owner}: .dot must use var(--dot-size) for its width.`);
  assert.match(base, /height:\s*var\(--dot-size\)/, `${owner}: .dot must use var(--dot-size) for its height.`);
  assert.match(rule('.dot.hollow') ?? '', /var\(--dot-ring\)/, `${owner}: .dot.hollow must use var(--dot-ring) for its ring.`);
});

test('each entity color has a dot class', () => {
  for (const kind of kinds) {
    assert.match(rule(`.dot.${kind}`) ?? '', new RegExp(`--dot:\\s*var\\(--c-${kind}\\)`),
      `musicindex ADR 0032 R-DOT-1 item 2: add ".dot.${kind} { --dot: var(--c-${kind}); }" to styles.css.`);
  }
});

test('a live show is red only when it is live now', () => {
  assert.equal(liveDot('live'), 'live');
  assert.equal(liveDot('pending'), 'feed');
  assert.equal(liveDot('ended'), 'feed');
  assert.equal(liveDot(undefined), 'feed');
});
