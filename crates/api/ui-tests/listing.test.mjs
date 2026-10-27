// The listing rule (ADR 0063, ADR 0042). Each test checks one rule of the
// block tags of a channel.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isHidden, withHidden, platformBlocks, withPlatformBlock } from '../ui/listing.js';

const plain = (value, cid) => ({ value, id: null, _cid: cid });
const platform = (id, value, cid) => ({ value, id, _cid: cid });

const LISTED = { title: 'T', itunes_block: null, blocks: [] };
const HIDDEN = { title: 'T', itunes_block: 'yes', blocks: [plain('yes', 1), platform('spotify', 'no', 2)] };

test('isHidden reads "yes" in any letter case from itunes:block or a podcast:block for each app', () => {
  assert.equal(isHidden(LISTED), false);
  assert.equal(isHidden({ title: 'T' }), false);
  assert.equal(isHidden(HIDDEN), true);
  assert.equal(isHidden({ itunes_block: ' YES ', blocks: [] }), true);
  assert.equal(isHidden({ itunes_block: null, blocks: [plain('Yes', 1)] }), true);
  assert.equal(isHidden({ itunes_block: 'No', blocks: [plain('no', 1)] }), false);
});

test('a block for one platform does not hide the feed', () => {
  assert.equal(isHidden({ itunes_block: null, blocks: [platform('spotify', 'yes', 1)] }), false);
});

test('a hide of a feed with no block tag adds itunes:block and a podcast:block for each app', () => {
  const next = withHidden(LISTED, true);
  assert.equal(next.itunes_block, 'Yes');
  assert.deepEqual(next.blocks, [{ value: 'yes', id: null }]);
  assert.equal(isHidden(next), true);
});

test('a hide changes the tags that the feed has and keeps the platform block', () => {
  const shown = { itunes_block: 'No', blocks: [plain('no', 1), platform('spotify', 'no', 2)] };
  const next = withHidden(shown, true);
  assert.equal(next.itunes_block, 'Yes');
  assert.deepEqual(next.blocks, [plain('yes', 1), platform('spotify', 'no', 2)]);
});

test('a show writes "no" in each tag that the feed has, and adds no tag', () => {
  const next = withHidden(HIDDEN, false);
  assert.equal(next.itunes_block, 'No');
  assert.deepEqual(next.blocks, [plain('no', 1), platform('spotify', 'no', 2)]);
  assert.equal(isHidden(next), false);
  assert.deepEqual(withHidden(LISTED, false), LISTED);
  const onlyItunes = withHidden({ itunes_block: 'Yes', blocks: [] }, false);
  assert.equal(onlyItunes.itunes_block, 'No');
  assert.deepEqual(onlyItunes.blocks, []);
});

test('platformBlocks lists only the blocks with an id, with their positions', () => {
  assert.deepEqual(platformBlocks(LISTED), []);
  const rows = platformBlocks(HIDDEN);
  assert.equal(rows.length, 1);
  assert.equal(rows[0].index, 1);
  assert.equal(rows[0].id, 'spotify');
  assert.equal(rows[0].hidden, false);
});

test('withPlatformBlock changes only the block at its position', () => {
  const next = withPlatformBlock(HIDDEN, 1, true);
  assert.deepEqual(next.blocks, [plain('yes', 1), platform('spotify', 'yes', 2)]);
  assert.equal(next.itunes_block, 'yes');
});
