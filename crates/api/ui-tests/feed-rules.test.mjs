// The guard of ADR 0065 item 12. A feed rule that task 042 moved to the
// core must not come back into the UI. Each test checks one removed function.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import * as cover from '../ui/cover.js';
import * as listing from '../ui/listing.js';

test('cover.js exports no withCover', () => {
  assert.equal('withCover' in cover, false,
    'ADR 0065 item 6: withCover is a feed rule. The core applies it at the draft save. Send the address in image.');
});

test('listing.js exports no isHidden, withHidden or withPlatformBlock', () => {
  const items = { isHidden: 8, withHidden: 8, withPlatformBlock: 9 };
  for (const [name, item] of Object.entries(items)) {
    assert.equal(name in listing, false,
      `ADR 0065 item ${item}: ${name} is a feed rule. The core applies it. Call POST /feeds/{id}/listing and use "hidden" from the draft response.`);
  }
});

test('editor.js defines no function alternateEnclosures', () => {
  const text = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.equal(/function\s+alternateEnclosures\b/.test(text), false,
    'ADR 0065 item 10: alternateEnclosures is a feed rule. The core makes the list with lossless_pair. Use facts.alternate_enclosures from the import.');
});
