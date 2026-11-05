import { test } from 'node:test';
import assert from 'node:assert/strict';
import { versionLabel } from '../ui/feedfacts.js';

test('a version label has no quote mark and no weak prefix', () => {
  assert.equal(versionLabel('"6ae0bb73-3c71da"'), '6ae0bb73-3c71da');
  assert.equal(versionLabel('W/"abc"'), 'abc');
  assert.equal(versionLabel('1793889138-12345678901234'), '1793889138-12345678');
});
