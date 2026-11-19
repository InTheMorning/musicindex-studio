import { test } from 'node:test';
import assert from 'node:assert/strict';
import { writeResult } from '../ui/writeresult.js';

test('a feed with a public address gives its written line and address', () => {
  assert.deepEqual(writeResult(
    { title: 'Songs', public_url: 'https://example.test/feed.xml' },
    { kind: 's3', name: 'My bucket' },
  ), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    address: 'https://example.test/feed.xml',
  });
});

test('a linked page appears in the result', () => {
  assert.deepEqual(writeResult(
    { title: 'Songs', public_url: 'https://example.test/feed.xml' },
    { kind: 's3', name: 'My bucket' },
    { id: 7, title: 'Artist page' },
  ), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    changedPage: { id: 7, title: 'Artist page' },
    address: 'https://example.test/feed.xml',
  });
});

test('a local host without a public address gives the folder warning', () => {
  assert.deepEqual(writeResult({ title: 'Songs', public_url: null }, { kind: 'local', name: 'Music' }), {
    title: 'Written',
    line: 'Songs is written to Music.',
    warning: { title: 'Listeners cannot get it yet.', text: 'This folder has no public address.' },
  });
});

test('an S3 host without a public address gives the file host warning', () => {
  assert.deepEqual(writeResult({ title: 'Songs' }, { kind: 's3', name: 'My bucket' }), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    warning: { title: 'Listeners cannot get it yet.', text: 'This file host gives no public address.' },
  });
});
