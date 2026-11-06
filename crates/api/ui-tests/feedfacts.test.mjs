import { test } from 'node:test';
import assert from 'node:assert/strict';
import { versionLabel, hasSpace, spaceNotice } from '../ui/feedfacts.js';

test('a version label has no quote mark and no weak prefix', () => {
  assert.equal(versionLabel('"6ae0bb73-3c71da"'), '6ae0bb73-3c71da');
  assert.equal(versionLabel('W/"abc"'), 'abc');
  assert.equal(versionLabel('1793889138-12345678901234'), '1793889138-12345678');
});

test('the notice counts the addresses with a space (ADR 0023 item 9)', () => {
  assert.equal(hasSpace('https://a.example/b c.xml'), true);
  assert.equal(hasSpace('https://a.example/b%20c.xml'), false);
  assert.equal(hasSpace(null), false);
  assert.equal(spaceNotice(1).action, 'Correct the Address');
  assert.equal(spaceNotice(3).action, 'Correct All 3');
  assert.match(spaceNotice(3).text, /^3 addresses have a space/);
});

