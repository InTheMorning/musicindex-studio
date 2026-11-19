import test from 'node:test';
import assert from 'node:assert/strict';
import { shouldAddTracksAfterNewRelease } from '../ui/addfeeds.js';

test('only a new album without a source opens Add tracks', () => {
  assert.equal(shouldAddTracksAfterNewRelease('album', null), true);
  assert.equal(shouldAddTracksAfterNewRelease('album', 42), false);
  assert.equal(shouldAddTracksAfterNewRelease('playlist', null), false);
  for (const type of ['publisher', 'show']) {
    assert.equal(shouldAddTracksAfterNewRelease(type, null), false);
  }
});
