// The text size of a field with Touch (musicindex ADR 0024 R-TYPE-1, Body on
// Touch). iOS Safari zooms the page when a field with text under 16 px gets
// the focus, and the page stays zoomed. On 2026-10-31 the sheets of the
// Recipients page had fields of 15 px on a phone (task 062).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../ui/styles.css', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
const owner = 'musicindex ADR 0024 R-TYPE-1 (task 062)';
const last = ':root :is(input, select, textarea)';

// Each innermost rule: its selector text and its declarations, in file order.
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({ selector: m[1].trim(), body: m[2] }));

// The specificity of one selector as [ids, classes, elements]. :where()
// counts nothing, and :is() and :not() count their most specific argument.
function specificity(selector) {
  let text = selector.replace(/:where\((?:[^()]|\([^()]*\))*\)/g, '');
  const extra = [0, 0, 0];
  text = text.replace(/:(?:is|not)\(((?:[^()]|\([^()]*\))*)\)/g, (_, args) => {
    const best = split(args).map((a) => specificity(a.trim())).sort(compare).pop();
    best.forEach((n, i) => { extra[i] += n; });
    return '';
  });
  text = text.replace(/::[\w-]+(\([^)]*\))?/g, ' ');
  const ids = (text.match(/#[\w-]+/g) || []).length;
  const classes = (text.match(/\.[\w-]+|\[[^\]]*\]|:[\w-]+(\([^)]*\))?/g) || []).length;
  const elements = (text.replace(/[.#:][\w-]+(\([^)]*\))?|\[[^\]]*\]/g, ' ').match(/(?:^|[\s>+~])[a-z][\w-]*/gi) || []).length;
  return [ids + extra[0], classes + extra[1], elements + extra[2]];
}

// The selectors of a list, split at each comma outside parentheses.
function split(list) {
  const parts = [];
  let depth = 0;
  let start = 0;
  [...list].forEach((c, i) => {
    if (c === '(') depth += 1;
    if (c === ')') depth -= 1;
    if (c === ',' && depth === 0) { parts.push(list.slice(start, i).trim()); start = i + 1; }
  });
  parts.push(list.slice(start).trim());
  return parts;
}

function compare(a, b) {
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return 0;
}

const setsFont = (body) => /(?:^|[;\s])font(?:-size)?\s*:/.test(body);
const isField = (selector) => /(?:^|[\s>+~(,])(?:input|select|textarea)(?![\w-])/.test(selector.split(/[\s>+~]+/).pop());

test('the last rule gives each field 16 px or more with a coarse pointer', () => {
  const index = rules.findIndex((r) => r.selector === last);
  assert.ok(index >= 0, `${owner}: styles.css has no rule "${last}". Add it in @media (pointer: coarse) at the end of the file, with font-size: var(--text-body).`);
  // The size is a pixel value, or a text token with its Touch value.
  const value = (rules[index].body.match(/font-size:\s*([^;]+)/) || [])[1] || '';
  const token = value.match(/^var\((--text-[a-z0-9-]+)\)$/);
  const touch = token && css.match(new RegExp(`@media\\s*\\(pointer:\\s*coarse\\)[^@]*?${token[1]}:\\s*(\\d+)px`));
  const size = token ? touch && touch[1] : (value.match(/^(\d+(?:\.\d+)?)px$/) || [])[1];
  assert.ok(size && Number(size) >= 16, `${owner}: "${last}" must give 16px or more with Touch. iOS zooms the page for a smaller field. Write font-size: var(--text-body).`);
  const before = css.slice(0, css.indexOf(last));
  assert.match(before, /@media\s*\(pointer:\s*coarse\)\s*\{\s*$/, `${owner}: "${last}" must be the only rule in @media (pointer: coarse). Put it in its own block.`);
  assert.equal(index, rules.length - 1, `${owner}: "${last}" must be the last rule of styles.css, so that no later rule replaces its size. Put new rules above it.`);
});

test('no field rule is more specific than the touch rule', () => {
  const limit = specificity(last);
  for (const rule of rules) {
    if (!setsFont(rule.body)) continue;
    for (const selector of split(rule.selector)) {
      if (!isField(selector)) continue;
      assert.ok(compare(specificity(selector), limit) <= 0,
        `${owner}: "${selector}" sets the font of a field and is more specific than "${last}". With Touch it can give the field less than 16 px. Use a selector with one class and one element, such as ".field input".`);
    }
  }
});

test('the specificity helper counts the selectors of styles.css', () => {
  assert.deepEqual(specificity('.field input'), [0, 1, 1]);
  assert.deepEqual(specificity(last), [0, 1, 1]);
  assert.deepEqual(specificity('.sheet .field input'), [0, 2, 1]);
  assert.deepEqual(specificity('#q'), [1, 0, 0]);
  assert.ok(isField('.field input') && isField('select.position') && !isField('.field-error'));
});

test('a group of the left panel does not grow with a long title', () => {
  assert.match(css, /\.destination-group \{[^}]*grid-template-columns: minmax\(0, 1fr\)/,
    'The human test of 0.1.0-rc.18 found that a long release title widened the left panel, with a scrollbar and no ellipsis. Fix: give .destination-group the column minmax(0, 1fr) in styles.css.');
});

