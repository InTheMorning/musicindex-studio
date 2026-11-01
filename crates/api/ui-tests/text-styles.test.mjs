// The text styles (musicindex ADR 0029 R-SCALE-1 and R-SCALE-2, ADR 0024
// R-TYPE-1). Each text size is a text token. A token has the web size of
// tokens.json by default and the touch size with a coarse pointer. On
// 2026-11-01 styles.css had 58 free sizes and no iOS sizes with Touch.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const tokens = JSON.parse(readFileSync(new URL('../ui/assets/tokens.json', import.meta.url), 'utf8'));
const owner = 'musicindex ADR 0029 R-SCALE-1';
const coarse = css.slice(css.search(/@media\s*\(pointer:\s*coarse\)/));

// Each value of a custom property in a CSS text.
function values(text, name) {
  return [...text.matchAll(new RegExp(`(?<![\\w-])${name}\\s*:\\s*([^;}]+)`, 'g'))].map((m) => m[1].trim());
}

test('each text token has the web size, and the touch size with a coarse pointer', () => {
  for (const style of Object.keys(tokens.type.web)) {
    const name = `--text-${style}`;
    const web = `${tokens.type.web[style][0]}px`;
    const touch = `${tokens.type.touch[style][0]}px`;
    assert.equal(values(css, name)[0], web, `${owner}: the first ${name} must be ${web}, the web size of tokens.json. Put it in :root.`);
    assert.equal(values(coarse, name)[0], touch, `${owner}: ${name} must be ${touch} in @media (pointer: coarse), the iOS size of tokens.json.`);
  }
});

test('no text has a free size', () => {
  const allowed = /^(var\(--text-[a-z0-9-]+\)|[0-9.]+em|inherit|clamp\(.*\))$/;
  for (const value of values(css, 'font-size')) {
    assert.match(value, allowed, `${owner}: "font-size: ${value}" is a free size. Use var(--text-<style>) with the style of the text, for example var(--text-callout). A size in em follows the text around it.`);
  }
});
