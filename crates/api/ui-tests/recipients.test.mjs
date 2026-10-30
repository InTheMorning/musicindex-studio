// The model of the Recipients pages (ADR 0091, task 054c).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  recipientRow, sortRecipients, placesText, useRow, outcomeText, causeText, blockedText,
  initialStatuses, startWrite, finishWrite, nextToWrite, statusText, writableIds, writeLabel, retryStatuses, changeSetRow, nameOf, statusError,
} from '../ui/recipients.js';

test('the summary line of a used recipient gives the name, the type and the feeds', () => {
  const row = recipientRow({ name: 'Ann', recipient_type: 'lnaddress', address: 'ann@example.com', feeds: 3, saved_id: null });
  assert.deepEqual(row, { title: 'Ann', type: 'lnaddress', text: '3 feeds' });
  assert.equal(recipientRow({ name: 'Ann', address: 'a', feeds: 1 }).text, '1 feed');
});

test('the summary line of a saved recipient that no feed uses says so', () => {
  const row = recipientRow({ name: 'Cara', recipient_type: 'node', address: '02ab', feeds: 0, saved_id: 4 });
  assert.equal(row.text, 'Saved, not used');
  assert.equal(row.type, 'node (keysend)');
  assert.equal(recipientRow({ name: 'Dan', address: '03cd', feeds: 0, saved_id: null }).text, '0 feeds',
    'ADR 0091 item 2: only a saved recipient shows "Saved, not used". Give the number of feeds for each other recipient.');
});

test('the list is A to Z by name, and a recipient with no name uses its address', () => {
  const sorted = sortRecipients([{ name: 'bob', address: 'x' }, { name: null, address: 'ann@x' }, { name: 'Cara', address: 'y' }]);
  assert.deepEqual(sorted.map((r) => r.address), ['ann@x', 'x', 'y']);
});

test('the places text of a feed with the channel and 12 tracks', () => {
  assert.equal(placesText({ channel: true, items: 12, live_items: 0 }), 'Channel · 12 tracks');
  assert.equal(placesText({ channel: true, items: 1, live_items: 2 }), 'Channel · 1 track · 2 live items');
});

test('the places text of a feed with only live items', () => {
  assert.equal(placesText({ channel: false, items: 0, live_items: 1 }), '1 live item');
  assert.equal(useRow({ title: 'Show', channel: false, items: 0, live_items: 3, writable: false }).text, '3 live items · Read-only');
});

test('the outcome text of each cause of task 054a', () => {
  const causes = ['no_recipient', 'has_recipient', 'has_new_address', 'no_value_block', 'only_fees',
    'split_not_number', 'bad_percent', 'in_open_change_set'];
  const seen = new Set();
  for (const cause of causes) {
    const text = causeText({ cause });
    assert.ok(text.length > 20 && text !== causeText({ cause: 'unknown' }), `the cause ${cause} needs its own text`);
    assert.ok(!seen.has(text), `the cause ${cause} needs a text that is different`);
    seen.add(text);
    assert.deepEqual(outcomeText({ outcome: 'not_changed', cause: { cause } }), { changed: false, text: `Not changed · ${text}` });
  }
  assert.equal(causeText({ cause: 'cannot_start', reason: 'the feed is not loaded' }), 'Studio cannot start the change: the feed is not loaded');
  assert.equal(outcomeText({ outcome: 'changed', channel: true, items: 2, live_items: 0 }).text, 'Changed · Channel · 2 tracks');
});

test('a feed that Studio cannot write gives its cause', () => {
  assert.match(blockedText('read_only_no_host'), /file host/);
  assert.match(blockedText('vault_locked'), /Unlock Studio/);
  assert.equal(blockedText(null), null);
});

const set = {
  feeds: [
    { feed_id: 1, outcome: { outcome: 'changed', channel: true, items: 0, live_items: 0 }, blocked: null, written_at: null },
    { feed_id: 2, outcome: { outcome: 'changed', channel: true, items: 0, live_items: 0 }, blocked: null, written_at: null },
    { feed_id: 3, outcome: { outcome: 'not_changed', cause: { cause: 'no_recipient' } }, blocked: null, written_at: null },
    { feed_id: 4, outcome: { outcome: 'changed', channel: true, items: 0, live_items: 0 }, blocked: 'read_only_no_host', written_at: null },
  ],
};

test('only a changed feed that is not blocked needs a write', () => {
  assert.deepEqual(writableIds(set), [1, 2]);
  assert.equal(writeLabel(2), 'Write 2 feeds');
  assert.equal(writeLabel(1), 'Write 1 feed');
});

test('the write status after one failure keeps the next feed Waiting', () => {
  let statuses = initialStatuses(set);
  assert.deepEqual(Object.keys(statuses), ['1', '2']);
  assert.equal(statusText(statuses[1]), 'Waiting');
  assert.equal(nextToWrite(statuses, [1, 2]), 1);
  statuses = startWrite(statuses, 1);
  assert.equal(statusText(statuses[1]), 'Writing');
  assert.equal(statusText(statuses[2]), 'Waiting');
  statuses = finishWrite(statuses, 1, 'The hosted file changed.');
  assert.equal(statusText(statuses[1]), 'Not written');
  assert.equal(statusError(statuses[1]), 'The hosted file changed.');
  assert.equal(statusText(statuses[2]), 'Waiting');
  assert.equal(nextToWrite(statuses, [1, 2]), 2);
  statuses = finishWrite(startWrite(statuses, 2), 2, null);
  assert.equal(statusText(statuses[2]), 'Written');
  assert.equal(nextToWrite(statuses, [1, 2]), null);
  assert.equal(retryStatuses(statuses)[1].state, 'waiting');
});

test('the row of an open change set gives its operation and the changed feeds', () => {
  const row = changeSetRow({ operation: { kind: 'rename' }, recipient: { address: 'ann@example.com' }, feeds: set.feeds });
  assert.deepEqual(row, { title: 'Rename · ann@example.com', text: '3 feeds changed' });
});

test('a change set names its recipient by name, or by address when no recipient has the key', () => {
  const list = [{ address: 'ann@example.com', custom_key: '', custom_value: null, name: 'Ann' }];
  const set = { operation: { kind: 'remove' }, recipient: { address: 'ann@example.com', custom_key: null, custom_value: null }, feeds: [] };
  assert.equal(changeSetRow(set, list).title, 'Remove · Ann',
    'ADR 0091 item 21: the review names the recipient. Find the name with nameOf of recipients.js.');
  assert.equal(nameOf([], set.recipient), 'ann@example.com');
});
