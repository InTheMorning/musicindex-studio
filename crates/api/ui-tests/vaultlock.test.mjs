import test from 'node:test';
import assert from 'node:assert/strict';
import { lockChoiceText } from '../ui/vaultlock.js';

test('each time of the lock has its words', () => {
  assert.equal(lockChoiceText(0), 'Never');
  assert.equal(lockChoiceText(1), '1 minute');
  assert.equal(lockChoiceText(10), '10 minutes');
  assert.equal(lockChoiceText(60), '1 hour');
});
