// The guards of ADR 0188 item 7. Each relative link of the README and the
// user guide points to a file that exists, and the guide index links each
// guide page.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const guide = join(root, 'docs/user-guide');

// The link targets of a Markdown text, with no fenced code.
export function linkTargets(text) {
  const plain = text.replace(/^(```|~~~)[^\n]*\n[\s\S]*?^\1[^\n]*$/gm, '');
  const targets = [];
  for (const match of plain.matchAll(/!?\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)/g)) {
    targets.push(match[1]);
  }
  for (const match of plain.matchAll(/<img\b[^>]*\bsrc="([^"]+)"/g)) targets.push(match[1]);
  return targets;
}

// The relative file of a link target, or null for a web address or an anchor.
export function relativeFile(target) {
  if (/^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith('#')) return null;
  return decodeURIComponent(target.split('#')[0]);
}

const pages = () => readdirSync(guide).filter((name) => name.endsWith('.md'));

test('each relative link in README.md and docs/user-guide/ points to a file that exists', () => {
  const files = [join(root, 'README.md'), ...pages().map((name) => join(guide, name))];
  const broken = [];
  for (const file of files) {
    for (const target of linkTargets(readFileSync(file, 'utf8'))) {
      const path = relativeFile(target);
      if (path && !existsSync(resolve(dirname(file), path))) {
        broken.push(`${relative(root, file)}: ${target}`);
      }
    }
  }
  assert.deepEqual(broken, [],
    'ADR 0188 item 7: a relative link points to a file that does not exist. Correct the link, or add the file.');
});

test('docs/user-guide/README.md has a link to each page of the guide', () => {
  const linked = new Set(linkTargets(readFileSync(join(guide, 'README.md'), 'utf8'))
    .map(relativeFile).filter(Boolean).map((path) => resolve(guide, path)));
  const missing = pages().filter((name) => name !== 'README.md' && !linked.has(join(guide, name)));
  assert.deepEqual(missing, [],
    'ADR 0188 item 7: the guide index has no link to these pages. Add a link to each page in docs/user-guide/README.md, in the group of its task.');
});

test('the link reader finds Markdown links and images, and skips code and web addresses', () => {
  const text = '[a](a.md) ![b](images/b.png "B")\n```sh\n[c](c.md)\n```\n[d](https://x.example) [e](#top) [f](f.md#part)';
  assert.deepEqual(linkTargets(text).map(relativeFile), ['a.md', 'images/b.png', null, null, 'f.md']);
});
