// The display value of the explicit switch (ADR 0064 item 4).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isExplicit } from '../ui/explicit.js';

test('isExplicit gives true for true and yes in any letter case, with or without space', () => {
  assert.equal(isExplicit('true'), true);
  assert.equal(isExplicit('Yes'), true);
  assert.equal(isExplicit(' yes '), true);
  assert.equal(isExplicit('TRUE'), true);
});

test('isExplicit gives false for each other text and for null', () => {
  for (const text of ['false', 'no', 'clean', '', null, undefined]) {
    assert.equal(isExplicit(text), false);
  }
});
