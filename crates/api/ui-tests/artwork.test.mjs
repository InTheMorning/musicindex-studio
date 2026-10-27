import { test } from 'node:test';
import assert from 'node:assert/strict';
import { PURPOSES, purposeTokens, withPurpose, nearRatio, imageFacts } from '../ui/artwork.js';

test('nearRatio gives the near ratio of the list', () => {
  assert.equal(nearRatio(1376, 768), '16/9');
  assert.equal(nearRatio(3000, 3000), '1/1');
  assert.equal(nearRatio(3000, 1000), '3/1');
  assert.equal(nearRatio(1080, 1920), '9/16');
});

test('nearRatio divides by the greatest common divisor outside the list', () => {
  assert.equal(nearRatio(1400, 1000), '7/5');
});

test('nearRatio gives null for no size', () => {
  assert.equal(nearRatio(0, 100), null);
  assert.equal(nearRatio(100, 0), null);
  assert.equal(nearRatio(null, 100), null);
  assert.equal(nearRatio(undefined, undefined), null);
});

test('withPurpose adds a token at the end', () => {
  assert.equal(withPurpose('artwork', 'banner', true), 'artwork banner');
  assert.equal(withPurpose(null, 'banner', true), 'banner');
});

test('withPurpose removes a token and keeps the others in place', () => {
  assert.equal(withPurpose('artwork hero banner', 'artwork', false), 'hero banner');
  assert.equal(withPurpose('artwork hero banner', 'banner', false), 'artwork hero');
});

test('withPurpose gives null after the last token', () => {
  assert.equal(withPurpose('banner', 'banner', false), null);
  assert.equal(withPurpose(null, 'banner', false), null);
});

test('purposeTokens reads two spaces as one separator', () => {
  assert.deepEqual(purposeTokens('artwork  social'), ['artwork', 'social']);
  assert.deepEqual(purposeTokens(null), []);
});

test('PURPOSES has the tokens of the specification', () => {
  assert.deepEqual(PURPOSES, ['artwork', 'social', 'canvas', 'banner', 'publisher', 'circular', 'poster']);
});

test('imageFacts gives text values with the JSON names', () => {
  assert.deepEqual(imageFacts(1376, 768, 'image/png'), {
    width: '1376', height: '768', media_type: 'image/png', aspect_ratio: '16/9',
  });
  assert.deepEqual(imageFacts(0, 0, null), { width: null, height: null, media_type: null, aspect_ratio: null });
});
