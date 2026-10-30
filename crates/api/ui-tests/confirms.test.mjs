// The confirmation sheets (task 057, musicindex ADR 0023 R-CONFIRM-1 and
// R-CASE-1). Each test reads the UI modules.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { CONFIRMS } from '../ui/confirms.js';

const uiDir = new URL('../ui/', import.meta.url);

test('no UI module uses the browser box confirm()', () => {
  const names = readdirSync(uiDir).filter((name) => name.endsWith('.js'));
  for (const name of names) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    assert.ok(!/(?<![A-Za-z0-9_$.])confirm\(/.test(text),
      `musicindex ADR 0023 R-CONFIRM-1: ${name} uses the browser box confirm(). Use askConfirm of app.js with an entry of confirms.js.`);
  }
});

const SMALL_WORDS = new Set(['a', 'an', 'the', 'from', 'of', 'to', 'in', 'on', 'and', 'or', 'for']);

function titleCase(text) {
  return text.split(' ').every((word, i) => i === 0 || !SMALL_WORDS.has(word)
    ? /^[A-Z]/.test(word) : /^[a-z]+$/.test(word));
}

test('each entry gives a title, a text, an action and a role', () => {
  for (const [name, entry] of Object.entries(CONFIRMS)) {
    const { title, text, action, role } = entry('X', 'track');
    assert.ok(title.endsWith('?'), `musicindex ADR 0023 R-CONFIRM-1: the title of ${name} must end with "?".`);
    assert.ok(text.length > 0, `musicindex ADR 0023 R-CONFIRM-1: ${name} needs a text.`);
    assert.ok(titleCase(action),
      `musicindex ADR 0023 R-CASE-1: the action of ${name} ("${action}") must be in title case.`);
    assert.ok(role === 'destructive' || role === 'primary',
      `ADR 0037: the role of ${name} must be destructive or primary.`);
  }
});
