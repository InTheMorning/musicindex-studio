// The words for a publisher feed (musicindex ADR 0033 R-WORD-3 to R-WORD-5).
// Each test examines the UI modules or the type label of a publisher role.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { mediumKind, relLabel, feedKind, kindGlow, roleWord, FIRST_CLASS_ROLES } from '../ui/shell.js';

const MESSAGE = `ADR 0253 item 13: use "Publisher" only as the screen word of the role hosting or in "published by". Fix the text in crates/api/ui.`;

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
  const word = text.slice(at).match(/^publishers?/i)?.[0];
  if (!word) return false;
  const end = at + word.length;
  // ADR 0257 item 5 gives these hints for the Publisher field.
  if (text === 'A release has one publisher. A music file calls the label "Publisher". Here the label goes in Label.'
    || text === 'You selected the publisher.') return false;
  if (text === 'publisher' || text === 'publishers') return false;
  // ADR 0254 items 23 to 25 give these exact texts to the sheet of a page action.
  if (['Change Publisher', 'Search for a publisher', 'Releases that change publisher', 'Select a publisher.',
    'Change publisher…', 'Choose a publisher…', 'No publisher matches.'].includes(text)) return false;
  // ADR 0250 item 5 gives this exact label to the medium repair menu.
  const repairLabel = 'Artist or label page (';
  if (text.slice(Math.max(0, at - repairLabel.length), at) === repairLabel && text[end] === ')') return false;
  if (/value=["']$/.test(text.slice(Math.max(0, at - 7), at))) return false;
  if (/[\w$.-]/.test(text[at - 1] ?? '') || /[\w$-]/.test(text[end] ?? '')) return false;
  if (text.slice(at - 'podcast:'.length, at) === 'podcast:') return false;
  if (/medium=["']$/.test(text.slice(Math.max(0, at - 8), at))) return false;
  // A role label or heading uses one screen word (ADR 0253 item 13).
  if ((word === 'Publisher' || word === 'Publishers') && text.trim() === word) return false;
  // Only this occurrence can name the page of the role `hosting`.
  if (word === 'publisher' && /^ pages?\b/.test(text.slice(end))) return false;
  return true;
}

test('the Publisher rule permits only role labels, headings and publisher pages', () => {
  const refused = (text) => [...text.matchAll(/publishers?/gi)]
    .some((match) => isInterfaceText(text, match.index));
  for (const text of ['Publisher', 'Publishers', 'Start with your publisher page.']) {
    assert.equal(refused(text), false, text);
  }
  for (const text of ['publisher feed', 'Publisher feed', 'a publisher']) {
    assert.equal(refused(text), true, text);
  }
  assert.equal(refused('Your publisher pages'), false);
  assert.equal(refused('A publisher feed on your publisher page.'), true);
  assert.equal(refused('A Publisher feed, published by Example.'), true);
  assert.equal(refused('Your publisher pageant'), true);
  assert.equal(refused("role === 'hosting' ? 'Publisher' : ''"), true);
  assert.equal(refused("['hosting', 'Publisher']"), true);
});

test('the screen word Publisher names hosting and published by', () => {
  const found = [];
  for (const name of sources) {
    const source = readFileSync(new URL(name, uiDir), 'utf8');
    for (const seg of textSegments(source, name.endsWith('.html'))) {
      for (const match of seg.text.matchAll(/publishers?/gi)) {
        if (!isInterfaceText(seg.text, match.index)) continue;
        const line = seg.line + seg.text.slice(0, match.index).split('\n').length - 1;
        found.push(`crates/api/ui/${name}:${line}`);
      }
    }
  }
  assert.deepEqual(found, [], `${MESSAGE}\nFound: ${found.join(', ')}`);
  assert.match(readFileSync(new URL('app.js', uiDir), 'utf8'), /Artist or label page \(publisher\)/,
    'ADR 0250 item 5: name the value in mediumProblemView of crates/api/ui/shell.js.');
});

test('a publisher feed lists each role and names hosting Publisher', () => {
  // musicindex ADR 0037 R-WORD-7 and ADR 0033 R-WORD-3 item 3, ADR 0084.
  assert.deepEqual(mediumKind('publisher', ['artist'], true), { dot: 'artist', label: 'Artist' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['label'], true), { dot: 'label', label: 'Label' }, MESSAGE);
  // The HeyCitizen feed of 2026-10-29, with roles that Studio inferred.
  assert.deepEqual(mediumKind('publisher', ['artist', 'label', 'host', 'author', 'producer'], false),
    { dot: 'artist hollow', label: 'Artist · Label · Producer · Host · Author' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['producer', 'label'], true), { dot: 'label', label: 'Label · Producer' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['sponsor'], true), { dot: 'sponsor', label: 'Sponsor' }, MESSAGE);
  // Curator comes after the first-class roles and before producer
  // (musicindex ADR 0081 R-CURATOR-1 items 1, 3 and 4).
  assert.deepEqual(mediumKind('publisher', ['curator', 'artist'], true), { dot: 'artist', label: 'Artist · Curator' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['producer', 'curator'], true), { dot: 'curator', label: 'Curator · Producer' }, MESSAGE);
  // Hosting comes before producer and host (musicindex ADR 0036 R-ROLE-1).
  assert.deepEqual(mediumKind('publisher', ['host', 'producer', 'hosting'], true), { dot: 'hosting', label: 'Publisher · Producer · Host' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher', [], true), { dot: 'feed hollow', label: 'Feed' }, MESSAGE);
  assert.deepEqual(mediumKind('publisher'), { dot: 'feed hollow', label: 'Feed' }, MESSAGE);
  assert.deepEqual(FIRST_CLASS_ROLES.map(roleWord), ['Artist', 'Label', 'Publisher'], MESSAGE);
  assert.deepEqual(mediumKind('publisher', ['label', 'hosting'], true), { dot: 'label', label: 'Label · Publisher' }, MESSAGE);
});

test('a feed with no cover shows the logo shape in the color of its kind', () => {
  // musicindex ADR 0021 R-ART-2 and ADR 0033 R-WORD-3 item 1.
  const glow = (feed) => kindGlow(feedKind(feed));
  const base = { id: 1, title: 'A', connector_id: 'c1', path: 'a', added_at: 1, draft_state: 'none', image: null };
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true }), 'var(--c-artist)');
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['label'], roles_confirmed: false }), 'var(--c-label)');
  assert.equal(glow({ ...base, medium: 'publisher', publisher_roles: ['host'], roles_confirmed: true }), 'var(--c-host)');
  assert.equal(glow({ ...base, medium: 'publisher' }), 'var(--c-feed)');
  assert.equal(glow({ ...base, medium: 'musicL' }), 'var(--c-playlist)');
  assert.equal(glow({ ...base, medium: 'music' }), 'var(--c-feed)');
});

test('the rel of a link shows each of its roles', () => {
  const message = 'musicindex ADR 0037 R-WORD-7: the rel of a link shows each role in the sequence of musicindex ADR 0036. Use relLabel of crates/api/ui/shell.js.';
  assert.equal(relLabel('artist'), 'Artist', message);
  assert.equal(relLabel('producer label artist'), 'Artist · Label · Producer', message);
  assert.equal(relLabel('Label'), 'Label', message);
  assert.equal(relLabel(''), 'No role', message);
  assert.equal(relLabel(null), 'No role', message);
});

test('a role that the two sides of a link give has a filled dot', () => {
  const message = 'ADR 0253 item 8: roles_linked fills the dot. A confirmed role fills it only when no link exists. Fix feedKind in crates/api/ui/shell.js.';
  const base = { id: 1, title: 'A', medium: 'publisher', publisher_roles: ['artist', 'label'], roles_confirmed: false };
  assert.equal(feedKind({ ...base, roles_linked: true }).dot, 'artist', message);
  assert.equal(feedKind({ ...base, roles_linked: false }).dot, 'artist hollow', message);
  assert.equal(feedKind({ ...base, roles_confirmed: true, release_ids: [] }).dot, 'artist', message);
  assert.equal(feedKind({ ...base, roles_confirmed: true, release_ids: [2] }).dot, 'artist hollow', message);
});

// ADR 0092 item 2: a UI text that names a connector says "file host". The
// word "host" alone in a UI text is the role `host`, or a network name.
// Each permitted text is a piece of a string that holds the word "host".
const HOST_ALLOWED = [
  // The role `host` of a publisher feed and of a person.
  "['host', 'Hosts']",
  "p.role || 'host'",
  'Empty means host.',
  // The person of the hosting role (ADR 0238 item 5).
  'Type the name of the person who hosts',
  // The person of the hosting role on the Welcome page (ADR 0240 item 2).
  'The artists and labels that you host go on it.',
  // The network names of a server (ADR 0092 item 4).
  'Studio uses the host name when you leave this empty.',
  'Studio uses the host name of the first server when you leave this empty.',
  'Host key fingerprint',
  'Host key</dt>',
  'The server gives a new host key.',
  'Virtual host style',
  // A web site or a verb, not a connector.
  "feed.host || 'Unknown host'",
  'edits them where you host them.',
  'redirect at the old host',
  'Ask the old host for an HTTP 301 redirect',
  // A CSS class.
  'class="panel host"',
  'class="host-head"',
  // The role host of a credit (ADR 0112).
  "|| 'host · cast'",
  'as Host.',
  // The host of a linked page (ADR 0199 item 1).
  'On ${hosts.get(p.feed_guid)}',
];

test('no UI text names a connector with the word host alone', () => {
  const found = [];
  for (const name of sources) {
    if (!name.endsWith('.js')) continue;
    const source = readFileSync(new URL(name, uiDir), 'utf8');
    const lines = source.split('\n');
    for (const seg of textSegments(source, false)) {
      for (const match of seg.text.matchAll(/hosts?(?![\w$-])/gi)) {
        const before = seg.text.slice(0, match.index);
        if (/file $/i.test(before) || /[\w$./-]$/.test(before)) continue;
        if (seg.text === match[0]) continue;
        const line = seg.line + before.split('\n').length - 1;
        const lineText = lines[line - 1] || '';
        if (HOST_ALLOWED.some((allowed) => seg.text.includes(allowed) || lineText.includes(allowed))) continue;
        found.push(`crates/api/ui/${name}:${line}`);
      }
    }
  }
  assert.deepEqual(found, [],
    `ADR 0092 item 2: a UI text names a connector with the word "host" alone. Write "file host" or "file hosts". `
    + `If the text names the role host or a network name, add it to HOST_ALLOWED in crates/api/ui-tests/words.test.mjs. Found: ${found.join(', ')}`);
});
