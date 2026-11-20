import test from 'node:test';
import assert from 'node:assert/strict';
import { shouldAddTracksAfterNewRelease } from '../ui/addfeeds.js';

test('only a new release opens Add tracks', () => {
  assert.equal(shouldAddTracksAfterNewRelease('album'), true);
  assert.equal(shouldAddTracksAfterNewRelease('playlist'), false);
  for (const type of ['publisher', 'show']) {
    assert.equal(shouldAddTracksAfterNewRelease(type), false);
  }
});
