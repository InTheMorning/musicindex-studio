// The rows of "Advanced" (ADR 0063, ADR 0065). The core owns the listing rule.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import * as listing from '../ui/listing.js';

const { platformBlocks } = listing;

const plain = (value, cid) => ({ value, id: null, _cid: cid });
const platform = (id, value, cid) => ({ value, id, _cid: cid });

const LISTED = { title: 'T', itunes_block: null, blocks: [] };
const HIDDEN = { title: 'T', itunes_block: 'yes', blocks: [plain('yes', 1), platform('spotify', 'no', 2)] };

test('platformBlocks lists only the blocks with an id, with their positions', () => {
  assert.deepEqual(platformBlocks(LISTED), []);
  const rows = platformBlocks(HIDDEN);
  assert.equal(rows.length, 1);
  assert.equal(rows[0].index, 1);
  assert.equal(rows[0].id, 'spotify');
  assert.equal(rows[0].hidden, false);
});
