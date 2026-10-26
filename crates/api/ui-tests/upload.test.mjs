// The upload models (ADR 0059, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  PART_SIZE, partCount, missingParts, partRange, findUpload, receivedBytes, progressPercent,
} from '../ui/upload.js';

const MIB = 1024 * 1024;

test('the part count rounds up, and an empty file has no part', () => {
  assert.equal(partCount(0, PART_SIZE), 0);
  assert.equal(partCount(1, PART_SIZE), 1);
  assert.equal(partCount(8 * MIB, PART_SIZE), 1);
  assert.equal(partCount(8 * MIB + 1, PART_SIZE), 2);
  assert.equal(partCount(20 * MIB, PART_SIZE), 3);
});

test('the missing parts are those that are not received, from small to large', () => {
  assert.deepEqual(missingParts(20 * MIB, PART_SIZE, []), [0, 1, 2]);
  assert.deepEqual(missingParts(20 * MIB, PART_SIZE, [2, 0]), [1]);
  assert.deepEqual(missingParts(20 * MIB, PART_SIZE, [0, 1, 2]), []);
  assert.deepEqual(missingParts(20 * MIB, PART_SIZE, [1, 1, 7]), [0, 2]);
  assert.deepEqual(missingParts(0, PART_SIZE, []), []);
});

test('the range of a part ends at the end of the file for the last part', () => {
  assert.deepEqual(partRange(0, 20 * MIB, PART_SIZE), { start: 0, end: 8 * MIB });
  assert.deepEqual(partRange(1, 20 * MIB, PART_SIZE), { start: 8 * MIB, end: 16 * MIB });
  assert.deepEqual(partRange(2, 20 * MIB, PART_SIZE), { start: 16 * MIB, end: 20 * MIB });
  assert.deepEqual(partRange(0, 5, PART_SIZE), { start: 0, end: 5 });
});

test('the range of a part outside the file is null', () => {
  assert.equal(partRange(3, 20 * MIB, PART_SIZE), null);
  assert.equal(partRange(-1, 20 * MIB, PART_SIZE), null);
  assert.equal(partRange(0, 0, PART_SIZE), null);
  assert.equal(partRange(1.5, 20 * MIB, PART_SIZE), null);
});

test('an open upload is found by the name and the size', () => {
  const uploads = [
    { upload: 'a', name: 'one.wav', size: 10, received: [0] },
    { upload: 'b', name: 'two.wav', size: 10, received: [] },
    { upload: 'c', name: 'one.wav', size: 11, received: [0, 1] },
    { upload: 'd', name: 'one.wav', size: 10, received: [0, 1] },
  ];
  assert.equal(findUpload(uploads, 'two.wav', 10).upload, 'b');
  assert.equal(findUpload(uploads, 'one.wav', 10).upload, 'd');
  assert.equal(findUpload(uploads, 'one.wav', 12), null);
  assert.equal(findUpload([], 'one.wav', 10), null);
});

test('the progress is the sum of the received parts', () => {
  assert.equal(receivedBytes(20 * MIB, PART_SIZE, []), 0);
  assert.equal(receivedBytes(20 * MIB, PART_SIZE, [2]), 4 * MIB);
  assert.equal(receivedBytes(20 * MIB, PART_SIZE, [0, 2, 2]), 12 * MIB);
  assert.equal(receivedBytes(20 * MIB, PART_SIZE, [0, 1, 2]), 20 * MIB);
  assert.equal(progressPercent(12 * MIB, 20 * MIB), 60);
  assert.equal(progressPercent(0, 0), 0);
  assert.equal(progressPercent(20 * MIB, 20 * MIB), 100);
});
