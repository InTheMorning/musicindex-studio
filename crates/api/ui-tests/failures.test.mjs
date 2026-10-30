// The words for a feed that Studio could not read (musicindex ADR 0023
// R-MSG-1).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { FAIL_TEXT, failedLine } from '../ui/failures.js';

// The kinds of the core, from `FailKind` in crates/core/src/discover.rs, in
// the snake case of their JSON form.
const rust = readFileSync(new URL('../../core/src/discover.rs', import.meta.url), 'utf8');
const body = rust.slice(rust.indexOf('pub enum FailKind'), rust.indexOf('}', rust.indexOf('pub enum FailKind')));
const kinds = [...body.matchAll(/^\s{4}([A-Z][A-Za-z]+),$/gm)]
  .map((m) => m[1].replace(/[A-Z]/g, (c, i) => (i ? '_' : '') + c.toLowerCase()));

test('each kind of the core has a text', () => {
  assert.ok(kinds.length >= 10, `found ${kinds.length} kinds`);
  for (const kind of kinds) {
    assert.ok(FAIL_TEXT[kind], `musicindex ADR 0023 R-MSG-1: the failure kind ${kind} has no text. Add it to FAIL_TEXT in failures.js.`);
  }
});

test('a release that gives a sitemap shows its title, its site and a reason', () => {
  const line = failedLine({
    source: '62d41c72-d06c-49e4-bb1f-e59f32c3a397',
    kind: 'not_a_feed',
    title: 'Everything Is Lit Remixes',
    url: 'https://www.hogstory.net/uploads/everythingislitremixes/everything_is_lit_remixes_feed.xml',
    message: 'the document has no rss/channel element',
  });
  assert.equal(line.name, 'Everything Is Lit Remixes');
  assert.equal(line.site, 'www.hogstory.net');
  assert.match(line.text, /not a feed/);
});

test('no line shows the raw message of the core', () => {
  for (const kind of [...kinds, 'a_new_kind']) {
    const line = failedLine({ source: 'https://x.example/f.xml', kind, title: null, url: null, message: 'RAW-LIBRARY-TEXT' });
    assert.ok(!JSON.stringify(line).includes('RAW-LIBRARY-TEXT'),
      `musicindex ADR 0023 R-MSG-1 item 4: the line of ${kind} shows the raw message.`);
  }
});

test('a failure with no title and no address shows its source', () => {
  const line = failedLine({ source: 'backup file', kind: 'bad_backup', title: null, url: null, message: 'x' });
  assert.equal(line.name, 'backup file');
  assert.equal(line.site, null);
});
