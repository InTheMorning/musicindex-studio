// The guard of musicindex ADR 0076 R-ADDRESS-2 item 7. `shortKey` is the
// JavaScript copy of `short_address` of the crate musicindex-value. The test
// gives each vector of the copy of value/tests/short_address.json to
// `shortKey`. The copy must have the bytes of the file at the commit that
// crates/core/Cargo.toml pins. When ../musicindex-lib is on this computer, the
// test also compares the bytes with the file at that commit.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { shortKey } from '../ui/nostrkey.js';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const copy = join(root, 'crates/api/ui-tests/vectors/short_address.json');
const lib = join(root, '../musicindex-lib');
const recorded = '5e35cf5c39a8ff68c64267156d0a7165c43f1826';
const sha256 = '23e65319277b48a95c5ce5be976b328f87baf984e72fc78598407dcdc3d9e124';
const fix = `musicindex ADR 0076 R-ADDRESS-2 item 7: crates/api/ui-tests/vectors/short_address.json must have the bytes of value/tests/short_address.json at musicindex-lib commit ${recorded}.\n`
  + 'Do not change the copy. Copy the file again from the commit that crates/core/Cargo.toml pins, and record that commit and the new hash in this test.';

test('shortKey gives the expected text for each vector of musicindex ADR 0076', () => {
  for (const { input, expected } of JSON.parse(readFileSync(copy, 'utf8'))) {
    assert.equal(shortKey(input), expected,
      `musicindex ADR 0076 R-ADDRESS-1: shortKey(${JSON.stringify(input)}) must give ${JSON.stringify(expected)}. Change shortKey in crates/api/ui/nostrkey.js to the rule of short_address.`);
  }
});

test('the recorded commit is the pin of musicindex-value', () => {
  const cargo = readFileSync(join(root, 'crates/core/Cargo.toml'), 'utf8');
  const pin = cargo.match(/^musicindex-value = .*rev = "([0-9a-f]+)"/m)?.[1];
  assert.equal(pin, recorded, fix);
});

test('the vectors copy has the bytes of the recorded hash', () => {
  const hash = createHash('sha256').update(readFileSync(copy)).digest('hex');
  assert.equal(hash, sha256, fix);
});

test('the vectors copy has the bytes of the musicindex-lib commit', (t) => {
  if (!existsSync(join(lib, '.git'))) {
    t.skip('../musicindex-lib is not on this computer');
    return;
  }
  const original = execFileSync('git', ['-C', lib, 'show', `${recorded}:value/tests/short_address.json`]);
  assert.ok(original.equals(readFileSync(copy)), fix);
});
