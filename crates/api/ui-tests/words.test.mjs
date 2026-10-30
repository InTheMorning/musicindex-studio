// The words for a publisher feed (musicindex ADR 0033 R-WORD-3 to R-WORD-5).
// Each test examines the UI modules or the type label of a publisher role.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { mediumKind, releasesList } from '../ui/shell.js';

const MESSAGE = `musicindex ADR 0033 R-WORD-4: most artists do not know the RSS word
"publisher". Show the role of a publisher feed: "Artist", "Label" or
the word of its one role. Use "Publisher" only for a feed with two or
more roles and no artist role, or for a feed that is definitely not an
artist and that states no role. Use "Feed" when the role is not known.
A technical view can show the RSS name, for example \`podcast:publisher\`.
Studio cannot make the test of R-WORD-3 item 3, so it does not use "Publisher".`;

const uiDir = new URL('../ui/', import.meta.url);
const sources = readdirSync(uiDir).filter((name) => /\.(js|html)$/.test(name));

// The text of each string and each template of a JavaScript file, with its
// line. A comment is not text. The code in `${}` of a template is code, and
// its own strings are text. An HTML file is all text, except its comments.
// The UI code has no regular expression with a quote mark, so the scan does
// not parse regular expressions.
function textSegments(source, isHtml) {
  if (isHtml) return [{ text: source.replace(/<!--[\s\S]*?-->/g, (c) => c.replace(/[^\n]/g, ' ')), line: 1 }];
  const segments = [];
  // Each frame is code ({ mode: 'code' }), a string or a template part.
  const stack = [{ mode: 'code' }];
  let line = 1;
  const open = (mode, quote) => { const seg = { text: '', line }; segments.push(seg); stack.push({ mode, quote, seg }); };
  for (let i = 0; i < source.length; i++) {
    const c = source[i];
    const top = stack[stack.length - 1];
    if (c === '\n') line++;
    if (top.mode === 'quote' || top.mode === 'template') {
      if (c === '\\') { top.seg.text += source.slice(i, i + 2); if (source[i + 1] === '\n') line++; i++; continue; }
      if (top.mode === 'quote' && c === top.quote) { stack.pop(); continue; }
      if (top.mode === 'template' && c === '`') { stack.pop(); continue; }
      if (top.mode === 'template' && c === '$' && source[i + 1] === '{') { stack.push({ mode: 'code', depth: 0 }); i++; continue; }
      top.seg.text += c;
      continue;
    }
    if (c === '/' && source[i + 1] === '/') { while (i + 1 < source.length && source[i + 1] !== '\n') i++; continue; }
    if (c === '/' && source[i + 1] === '*') {
      const e = source.indexOf('*/', i + 2);
      for (let k = i + 1; k < e + 2; k++) if (source[k] === '\n') line++;
      i = e + 1;
      continue;
    }
    if (c === "'" || c === '"') { open('quote', c); continue; }
    if (c === '`') { open('template'); continue; }
    if (c === '{' && stack.length > 1) { top.depth++; continue; }
    if (c === '}' && stack.length > 1) {
      if (top.depth > 0) { top.depth--; continue; }
      // The end of `${}`: the template goes on in a new text part.
      stack.pop();
      stack.pop();
      open('template');
      continue;
    }
  }
  return segments;
}

// Is the word at `at` of a text segment interface text? A whole string
// 'publisher' is the RSS value. A word joined to other letters, a digit,
// `_`, `-` or `.` is a code name or a property, for example
// `album-publisher`. An RSS name (`podcast:publisher`,
// `medium="publisher"`) can show in a technical view (R-WORD-4 items 2
// and 3).
function isInterfaceText(text, at) {
  const end = at + 'publisher'.length;
  if (text === 'publisher') return false;
  if (/[\w$.-]/.test(text[at - 1] ?? '') || /[\w$-]/.test(text[end] ?? '')) return false;
  if (text.slice(at - 'podcast:'.length, at) === 'podcast:') return false;
  if (/medium=["']$/.test(text.slice(Math.max(0, at - 8), at))) return false;
  return true;
}

test('no interface text in the UI uses the word publisher', () => {
  const found = [];
  for (const name of sources) {
    const source = readFileSync(new URL(name, uiDir), 'utf8');
    for (const seg of textSegments(source, name.endsWith('.html'))) {
      for (const match of seg.text.matchAll(/publisher/gi)) {
        if (!isInterfaceText(seg.text, match.index)) continue;
        const line = seg.line + seg.text.slice(0, match.index).split('\n').length - 1;
        found.push(`crates/api/ui/${name}:${line}`);
      }
    }
  }
  assert.deepEqual(found, [], `${MESSAGE}\nFound: ${found.join(', ')}`);
});

test('a publisher feed lists each role, or shows Feed with a hollow dot', () => {
  // musicindex ADR 0037 R-WORD-7 and ADR 0033 R-WORD-3 item 3, ADR 0084.
  assert.deepEqual(mediumKind('publisher', ['artist'], true), { dot: 'artist', label: 'Artist' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['label'], true), { dot: 'label', label: 'Label' }, MESSAGE);
  // The HeyCitizen feed of 2026-10-29, with roles that Studio inferred.
  assert.deepEqual(mediumKind('publisher', ['artist', 'label', 'host', 'author', 'producer'], false),
    { dot: 'artist hollow', label: 'Artist · Label · Producer · Host · Author' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['producer', 'label'], true), { dot: 'label', label: 'Label · Producer' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['sponsor'], true), { dot: 'sponsor', label: 'Sponsor' }, MESSAGE);
  // Hosting comes before producer and host (musicindex ADR 0036 R-ROLE-1).
  assert.deepEqual(mediumKind('publisher', ['host', 'producer', 'hosting'], true), { dot: 'hosting', label: 'Hosting · Producer · Host' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', [], true), { dot: 'feed hollow', label: 'Feed' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher'), { dot: 'feed hollow', label: 'Feed' }, MESSAGE);
});

test('a feed with no cover shows the logo shape in the color of its kind', () => {
  // musicindex ADR 0021 R-ART-2 and ADR 0033 R-WORD-3 item 1.
  const glow = (feed) => releasesList({ feeds: [feed], connectors: [], sort: 'newest', view: 'rows' }).rows[0].glow;
  const base = { id: 1, title: 'A', connector_id: 'c1', path: 'a', added_at: 1, draft_state: 'none', image: null };
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true }), 'var(--c-artist)');
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['label'], roles_confirmed: false }), 'var(--c-label)');
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['host'], roles_confirmed: true }), 'var(--c-host)');
  assert.equal(glow({ ...base, medium: 'publisher' }), 'var(--c-feed)');
  assert.equal(glow({ ...base, medium: 'musicL' }), 'var(--c-playlist)');
  assert.equal(glow({ ...base, medium: 'music' }), 'var(--c-feed)');
});
