import test from 'node:test';
import assert from 'node:assert/strict';
import { op3Hint, op3Page } from '../ui/op3.js';

test('op3Hint gives no text when no track or each track has the prefix', () => {
  assert.equal(op3Hint({ prefixed: 0, tracks: 9 }), null);
  assert.equal(op3Hint({ prefixed: 9, tracks: 9 }), null);
  assert.equal(op3Hint({ prefixed: 0, tracks: 0 }), null);
  assert.equal(op3Hint(undefined), null);
});

test('op3Hint gives the count for a mixed set', () => {
  assert.equal(op3Hint({ prefixed: 2, tracks: 9 }), '2 of 9 tracks use OP3');
});

test('op3Page gives the OP3 page of the feed GUID', () => {
  assert.equal(
    op3Page('d5c805fb-6016-4038-9cda-2e68d2b8dcc4'),
    'https://op3.dev/show/d5c805fb-6016-4038-9cda-2e68d2b8dcc4',
  );
});
