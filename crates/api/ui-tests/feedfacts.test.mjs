import { test } from 'node:test';
import assert from 'node:assert/strict';
import { versionLabel, hasSpace, spaceNotice, newFeedAddress, newFeedAddressLine } from '../ui/feedfacts.js';

test('a version label has no quote mark and no weak prefix', () => {
  assert.equal(versionLabel('"6ae0bb73-3c71da"'), '6ae0bb73-3c71da');
  assert.equal(versionLabel('W/"abc"'), 'abc');
  assert.equal(versionLabel('1793889138-12345678901234'), '1793889138-12345678');
});

test('the notice counts the addresses with a space (ADR 0023 item 9)', () => {
  assert.equal(hasSpace('https://a.example/b c.xml'), true);
  assert.equal(hasSpace('https://a.example/b%20c.xml'), false);
  assert.equal(hasSpace(null), false);
  assert.equal(spaceNotice(1).action, 'Correct the Address');
  assert.equal(spaceNotice(3).action, 'Correct All 3');
  assert.match(spaceNotice(3).text, /^3 addresses have a space/);
});

test('the tag shows when the feed has it, with the comparison of the core (ADR 0021 item 4)', () => {
  const url = 'https://b.example/a.xml';
  assert.equal(newFeedAddress({ new_feed_url: null, address_differences: [] }), null);
  const other = newFeedAddress({
    new_feed_url: url,
    public_url: 'https://a.example/a.xml',
    address_differences: [{ source: 'new_feed_url', url }],
  });
  assert.equal(other.url, url);
  assert.equal(other.note, null);
  const own = newFeedAddress({ new_feed_url: url, public_url: url, address_differences: [] });
  assert.equal(own.note, 'This is the address of this feed.');
  const noAddress = newFeedAddress({ new_feed_url: url, public_url: null, address_differences: [] });
  assert.equal(noAddress.note, null);
});

test('the Files page line shows only for a new feed address (ADR 0184 items 1 and 2)', () => {
  const url = 'https://next.example/feed.xml';
  assert.equal(newFeedAddressLine({ new_feed_url: url, new_feed_address_is_new: true }), url);
  assert.equal(newFeedAddressLine({ new_feed_url: url, new_feed_address_is_new: false }), null);
});
