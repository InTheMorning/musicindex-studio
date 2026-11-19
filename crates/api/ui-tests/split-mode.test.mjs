import { test } from 'node:test';
import assert from 'node:assert/strict';
import { percentages, scaleSplits, setPercent } from '../ui/recipients.js';
import { readFileSync } from 'node:fs';

test('scaleSplits uses integer division and gives the remainder to the first biggest split', () => {
  assert.deepEqual(scaleSplits([50, 50], 75), [38, 37]);
  assert.deepEqual(scaleSplits([1, 2, 3], 60), [10, 20, 30]);
  assert.deepEqual(scaleSplits([0, 0], 10), [0, 0]);
  assert.deepEqual(scaleSplits([3, 3, 3], 10), [4, 3, 3]);
});

test('setPercent scales each other recipient and keeps the total at 100', () => {
  const recipients = [{ split: '50', name: 'A' }, { split: '30', name: 'B' }, { split: '20', name: 'C' }];
  const result = setPercent(recipients, 0, 25);
  assert.equal(result[0].split, '25');
  assert.equal(result.reduce((sum, recipient) => sum + Number(recipient.split), 0), 100);
  assert.equal(percentages(result)[0], 25);
  assert.equal(result[1].name, 'B');
});

test('setPercent scales fee recipients with the other recipients', () => {
  const result = setPercent([{ split: '50' }, { split: '30' }, { split: '20', fee: 'true' }], 0, 25);
  assert.equal(result[2].split, '30');
  assert.equal(result.reduce((sum, recipient) => sum + Number(recipient.split), 0), 100);
});

test('setPercent refuses a split that is not numeric', () => {
  assert.equal(setPercent([{ split: '50' }, { split: 'abc' }], 0, 25), null);
});

test('the recipient form names both split modes and their states', () => {
  const source = readFileSync(new URL('../ui/values.js', import.meta.url), 'utf8');
  for (const text of ['Split as', 'of this block', 'Give a split.', 'One recipient gets each payment.']) assert.ok(source.includes(text), text);
});
