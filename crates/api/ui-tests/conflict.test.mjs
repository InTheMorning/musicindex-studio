import test from 'node:test';
import assert from 'node:assert/strict';
import { conflictNotice } from '../ui/conflict.js';

test('conflict notice gives both times and omits an unknown file time', () => {
  const notice = conflictNotice({ draft_started: 100, file_changed: 200 });
  assert.equal(notice.heading, 'The hosted file changed after your draft started. Studio did not write.');
  assert.deepEqual(notice.times, [
    `Draft started ${new Date(100000).toLocaleString()}`,
    `File changed ${new Date(200000).toLocaleString()}`,
  ]);
  assert.equal(notice.cause, 'The change can come from a different collection, a different Studio version, a different tool or a different person.');
  assert.deepEqual(conflictNotice({ draft_started: 100 }).times, [notice.times[0]]);
  // A conflict with no details, or no error, gives the text with no times.
  assert.deepEqual(conflictNotice(null).times, []);
  assert.deepEqual(conflictNotice(undefined).times, []);
});
