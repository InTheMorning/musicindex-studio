// The guard of musicindex ADR 0065 R-FIT-5 item 3. tools/fit-check.mjs is a
// copy of fit/web/fit-check.mjs at one musicindex-lib commit. The test compares
// the bytes of the copy with the recorded hash of that commit. When
// ../musicindex-lib is on this computer, the test also compares the bytes with
// the file at that commit.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const copy = join(root, 'tools/fit-check.mjs');
const lib = join(root, '../musicindex-lib');
const commit = 'bbcca58';
const sha256 = '61db8ea35da129c76bae7c3c08112a1c3f2a599b3be824759544586f2707a44d';
const fix = `musicindex ADR 0065 R-FIT-5 item 3: tools/fit-check.mjs must have the bytes of fit/web/fit-check.mjs at musicindex-lib commit ${commit}.\n`
  + 'Do not change the copy. Change the module in musicindex-lib, then copy it again and record the new commit and hash in this test.';

test('the fit check copy has the bytes of the recorded hash', () => {
  const hash = createHash('sha256').update(readFileSync(copy)).digest('hex');
  assert.equal(hash, sha256, fix);
});

test('the fit check copy has the bytes of the musicindex-lib commit', (t) => {
  if (!existsSync(join(lib, '.git'))) {
    t.skip('../musicindex-lib is not on this computer');
    return;
  }
  const original = execFileSync('git', ['-C', lib, 'show', `${commit}:fit/web/fit-check.mjs`]);
  assert.ok(original.equals(readFileSync(copy)), fix);
});
