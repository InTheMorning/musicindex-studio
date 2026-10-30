// The title case of the button labels (task 058, musicindex ADR 0023
// R-CASE-1). The test finds each button label in the templates of the UI
// modules, and in the models that give a label to a button.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { titleCase, isTitleCase } from '../ui/casing.js';
import { CONFIRMS } from '../ui/confirms.js';
import { makeFields, MAKE_TYPES } from '../ui/addfeeds.js';
import { nextAction } from '../ui/live.js';
import { sources } from '../ui/picker.js';
import { signProgressLabel } from '../ui/signing.js';

const uiDir = new URL('../ui/', import.meta.url);

test('titleCase gives the title style of the Apple HIG', () => {
  assert.equal(titleCase('Discard the draft'), 'Discard the Draft');
  assert.equal(titleCase('Sign in with Amber or a bunker'), 'Sign In with Amber or a Bunker');
  assert.equal(titleCase('Yes, I have feeds'), 'Yes, I Have Feeds');
  assert.equal(titleCase('Move to this file host'), 'Move to This File Host');
  assert.equal(titleCase('Add to a level'), 'Add to a Level');
  assert.equal(titleCase('Set up a file host'), 'Set Up a File Host');
  assert.equal(titleCase('Choose a file to'), 'Choose a File To');
  assert.equal(titleCase('＋ Add the track'), '＋ Add the Track');
  assert.equal(titleCase('Import from msp 2.0'), 'Import from MSP 2.0');
  assert.equal(titleCase('Add the nsite'), 'Add the nsite');
});

test('isTitleCase refuses a label in sentence case', () => {
  assert.ok(isTitleCase('Discard the Draft'));
  assert.ok(!isTitleCase('Discard the draft'));
  assert.ok(!isTitleCase('Add The Folder'));
});

// Each text that the search below finds by error, with the reason. A name
// is in `NAMES` of casing.js.
const NOT_LABELS = new Map([
  ['Open Untitled feed', 'The text "Untitled feed" replaces the title of a feed. It is not a label word.'],
]);

const SKIP_TAGS = new Set(['select', 'option', 'input', 'textarea']);

// The index after the `}` that ends the expression that starts at `start`
// (the index after `${`).
function endOfExpression(src, start) {
  let depth = 0;
  for (let i = start; i < src.length; i++) {
    const c = src[i];
    if (c === "'" || c === '"') { i = endOfString(src, i); continue; }
    if (c === '`') { i = endOfTemplate(src, i); continue; }
    if (c === '{') depth++;
    if (c === '}') { if (depth === 0) return i + 1; depth--; }
  }
  throw new Error('An expression has no end.');
}

function endOfString(src, start) {
  for (let i = start + 1; i < src.length; i++) {
    if (src[i] === '\\') { i++; continue; }
    if (src[i] === src[start]) return i;
  }
  throw new Error('A string has no end.');
}

function endOfTemplate(src, start) {
  for (let i = start + 1; i < src.length; i++) {
    if (src[i] === '\\') { i++; continue; }
    if (src[i] === '$' && src[i + 1] === '{') { i = endOfExpression(src, i + 2) - 1; continue; }
    if (src[i] === '`') return i;
  }
  throw new Error('A template has no end.');
}

// The end of an opening tag that starts at `start`: the index after `>`.
function endOfTag(src, start) {
  for (let i = start; i < src.length; i++) {
    if (src[i] === '$' && src[i + 1] === '{') { i = endOfExpression(src, i + 2) - 1; continue; }
    if (src[i] === '"') { i = endOfString(src, i); continue; }
    if (src[i] === '>') return i + 1;
  }
  throw new Error('A tag has no end.');
}

const MAX_ALTERNATIVES = 64;

function product(parts) {
  let result = [''];
  for (const part of parts) {
    const next = [];
    for (const head of result) for (const tail of part) next.push(head + tail);
    result = next.slice(0, MAX_ALTERNATIVES);
  }
  return result;
}

// The texts that a JavaScript expression can give as a literal string. An
// expression with no literal gives the placeholder "#". A literal after a
// comparison is a value to compare, so it is not a text. A nested `html`
// template is not a label text.
function expressionTexts(expr) {
  const texts = [];
  for (let i = 0; i < expr.length; i++) {
    const c = expr[i];
    if (c === "'" || c === '"') {
      const end = endOfString(expr, i);
      const before = expr.slice(0, i).trimEnd();
      if (!/[!=]==?$/.test(before) && !/\bhtml$/.test(before)) texts.push(...literalTexts(expr.slice(i + 1, end)));
      i = end;
    } else if (c === '`') {
      const end = endOfTemplate(expr, i);
      const before = expr.slice(0, i);
      if (!/\bhtml$/.test(before)) texts.push(...templateTexts(expr.slice(i + 1, end)));
      i = end;
    }
  }
  return texts.length > 0 ? texts : ['#'];
}

function literalTexts(text) {
  return [text];
}

// The texts of a template body: its static parts and its expressions.
function templateTexts(body) {
  const parts = [];
  let last = 0;
  for (let i = 0; i < body.length; i++) {
    if (body[i] === '$' && body[i + 1] === '{') {
      parts.push([body.slice(last, i)]);
      const end = endOfExpression(body, i + 2);
      parts.push(expressionTexts(body.slice(i + 2, end - 1)));
      last = end;
      i = end - 1;
    }
  }
  parts.push([body.slice(last)]);
  return product(parts);
}

// The text of the content of an element: the tags go out, and each
// expression gives its alternatives.
function contentTexts(content) {
  const parts = [];
  let text = '';
  const flush = () => { parts.push([text]); text = ''; };
  for (let i = 0; i < content.length; i++) {
    if (content[i] === '$' && content[i + 1] === '{') {
      flush();
      const end = endOfExpression(content, i + 2);
      parts.push(expressionTexts(content.slice(i + 2, end - 1)));
      i = end - 1;
    } else if (content[i] === '<' && /[a-z/$]/.test(content[i + 1] || '')) {
      i = endOfTag(content, i) - 1;
    } else {
      text += content[i];
    }
  }
  flush();
  return product(parts).map((label) => label.replace(/\s+/g, ' ').trim());
}

// Each button element of a source: the label texts, the line and the tag.
export function buttonLabels(src) {
  const found = [];
  const open = /<([a-z][a-z0-9]*)\b/g;
  let match;
  while ((match = open.exec(src)) !== null) {
    const tag = match[1];
    if (SKIP_TAGS.has(tag)) continue;
    let end;
    try { end = endOfTag(src, match.index + match[0].length); } catch { continue; }
    const head = src.slice(match.index, end);
    const classes = (/\bclass=(?:"([^"]*)"|\$\{([^}]*)\})/.exec(head) || []).slice(1).join(' ');
    const tokens = classes.split(/[^A-Za-z0-9_-]+/);
    if (!tokens.includes('button')) continue;
    if (tokens.includes('link-row') || tokens.includes('row')) continue;
    if (head.endsWith('/>')) continue;
    const close = src.indexOf(`</${tag}>`, end);
    if (close === -1) continue;
    const line = src.slice(0, match.index).split('\n').length;
    for (const label of contentTexts(src.slice(end, close))) {
      if (/\p{L}/u.test(label) || label === '#') found.push({ label, line, tag });
    }
  }
  return found;
}

function fail(label, where) {
  return `musicindex ADR 0023 R-CASE-1: the button label "${label}" in ${where} is not in title case. Write "${titleCase(label)}".`;
}

test('buttonLabels finds a label, a busy text and a ternary', () => {
  const src = 'html`<button class="button primary" disabled=${a > 1}>${busy ? \'Adding…\' : `Add ${n} feed${n === 1 ? \'\' : \'s\'}`}</button>`';
  assert.deepEqual(buttonLabels(src).map((f) => f.label), ['Adding…', 'Add # feed', 'Add # feeds']);
  assert.deepEqual(buttonLabels('html`<button class="link-row" type="button">Add a person</button>`'), []);
  assert.deepEqual(buttonLabels('html`<select class="button"><option>a b</option></select>`'), []);
});

test('each button label in the UI templates is in title case', () => {
  const names = readdirSync(uiDir).filter((name) => name.endsWith('.js'));
  let count = 0;
  const problems = [];
  for (const name of names) {
    const src = readFileSync(new URL(name, uiDir), 'utf8');
    for (const { label, line } of buttonLabels(src)) {
      count++;
      if (NOT_LABELS.has(label)) continue;
      if (!isTitleCase(label)) problems.push(fail(label, `${name}:${line}`));
    }
  }
  assert.deepEqual(problems, []);
  assert.ok(count > 100, `The search found only ${count} button labels. Check the search.`);
});

test('each label that a model gives to a button is in title case', () => {
  const labels = [];
  for (const [name, entry] of Object.entries(CONFIRMS)) labels.push([entry('X', 'track').action, `CONFIRMS.${name}`]);
  for (const [type] of MAKE_TYPES) {
    const fields = makeFields(type);
    labels.push([fields.button, `makeFields(${type})`], [fields.busyLabel, `makeFields(${type})`]);
  }
  for (const status of ['pending', 'live']) labels.push([nextAction(status).label, `nextAction(${status})`]);
  for (const source of sources('audio', [])) labels.push([source.label, 'sources of picker.js']);
  const sortLabels = /const SORT_LABELS = \{([^}]*)\}/.exec(readFileSync(new URL('app.js', uiDir), 'utf8'));
  for (const [, label] of sortLabels[1].matchAll(/'([^']*)'/g)) labels.push([label, 'SORT_LABELS of app.js']);
  labels.push([signProgressLabel({ done: 1, total: 2 }), 'signProgressLabel']);
  const problems = labels.filter(([label]) => !isTitleCase(label)).map(([label, where]) => fail(label, where));
  assert.deepEqual(problems, []);
});
