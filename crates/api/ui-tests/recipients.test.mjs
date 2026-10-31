// The model of the Recipients pages (ADR 0091, task 054c).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  recipientRow, sortRecipients, placesText, useRow, outcomeText, causeText, blockedText,
  initialStatuses, addressLines, lnaddressOf, nodeOf, recipientTitle, pickerRows, lookupHint, memberStart, startType, operationTitle, operationLine, startWrite, finishWrite, nextToWrite, statusText, writableIds, writeLabel, retryStatuses, changeSetRow, nameOf, statusError,
} from '../ui/recipients.js';

test('the summary line of a used recipient gives the name, the address and the feeds', () => {
  const row = recipientRow({ name: 'Ann', recipient_type: 'lnaddress', address: 'ann@example.com', feeds: 3, saved_id: null });
  assert.deepEqual(row, { title: 'Ann', text: 'ann@example.com · 3 feeds' });
  assert.equal(recipientRow({ name: 'Ann', address: 'a', feeds: 1 }).text, 'a · 1 feed');
});

test('the summary line of a saved recipient that no feed uses says so', () => {
  const row = recipientRow({ name: 'Cara', recipient_type: 'node', address: '02ab12cd34ef5678', feeds: 0, saved_id: 4 });
  assert.equal(row.text, '02ab12cd…5678 · Saved, not used',
    'ADR 0095 item 7: a node key shows its first 8 and its last 4 characters.');
  assert.equal(recipientRow({ name: 'Dan', address: '03cd', feeds: 0, saved_id: null }).text, '03cd · 0 feeds',
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
    'split_not_number', 'bad_percent', 'in_open_change_set', 'no_node_recipient'];
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

const wallet = {
  name: null, address: '02ab12cd34ef5678', recipient_type: 'node', custom_key: '906608', custom_value: '01abc', feeds: 7,
  addresses: [
    { address: 'tj@getalby.com', recipient_type: 'lnaddress', name: null, custom_key: null, custom_value: null },
    { address: '02ab12cd34ef5678', recipient_type: 'node', name: 'TJ node', custom_key: '906608', custom_value: '01abc' },
  ],
};

test('a wallet of two addresses gives its title, row and lines', () => {
  assert.equal(recipientTitle(wallet), 'tj@getalby.com');
  assert.deepEqual(addressLines(wallet), [
    { address: 'tj@getalby.com', type: 'lnaddress', custom: null },
    { address: '02ab12cd34ef5678', type: 'node (keysend)', custom: '01abc' }]);
  assert.equal(lnaddressOf(wallet), 'tj@getalby.com');
  assert.equal(nodeOf(wallet), '02ab12cd34ef5678');
  assert.equal(recipientRow({ ...wallet, name: 'TJ Doerfel' }).text, 'tj@getalby.com · 02ab12cd…5678 · 01abc · 7 feeds',
    'ADR 0095 item 7: the row shows each address of the wallet, a node key with its custom value, then the feeds.');
  const keyFirst = { ...wallet, addresses: [wallet.addresses[1], wallet.addresses[0]] };
  assert.equal(recipientRow(keyFirst).text, 'tj@getalby.com · 02ab12cd…5678 · 01abc · 7 feeds',
    'ADR 0095 item 7: the Lightning address is first in the row.');
});

test('the operation form starts at the member of the wallet with the type of the menu', () => {
  assert.equal(startType(wallet, 'change_address'), 'node');
  assert.equal(startType(wallet, 'add'), 'lnaddress');
  assert.deepEqual(memberStart(wallet, 'node'), { address: '02ab12cd34ef5678', custom_key: '906608', custom_value: '01abc' });
  assert.deepEqual(memberStart(wallet, 'lnaddress'), { address: 'tj@getalby.com', custom_key: '', custom_value: '' });
  // A Lightning address with a found lookup: the key of the wallet is a node
  // key that no feed has, so the form starts at the Lightning address.
  const looked = { name: null, address: '03cd', recipient_type: 'lnaddress', custom_key: '906608', custom_value: '01xyz', feeds: 1,
    addresses: [{ address: 'm@fountain.fm', recipient_type: 'lnaddress', name: null, custom_key: null, custom_value: null, lookup: 'found' }] };
  assert.equal(startType(looked, 'change_address'), 'lnaddress',
    'ADR 0095 item 14: the start values come from the members of the wallet, not from its key.');
  assert.deepEqual(memberStart(looked, 'lnaddress'), { address: 'm@fountain.fm', custom_key: '', custom_value: '' });
  assert.deepEqual(memberStart(looked, 'node'), { address: '', custom_key: '', custom_value: '' });
});

test('an entry with no addresses gives one line', () => {
  const entry = { name: null, address: '03cd', recipient_type: 'node' };
  assert.equal(recipientTitle(entry), '03cd');
  assert.deepEqual(addressLines(entry), [{ address: '03cd', type: 'node (keysend)', custom: null }]);
  assert.equal(lnaddressOf(entry), null);
  assert.equal(nodeOf(entry), null);
});

test('the picker gives one row for each address, with the name of the entry', () => {
  const rows = pickerRows([{ ...wallet, name: 'TJ Doerfel' }]);
  assert.deepEqual(rows.map((r) => [r.address, r.name]), [['tj@getalby.com', 'TJ Doerfel'], ['02ab12cd34ef5678', 'TJ node']]);
});

test('the keysend hint names the domain', () => {
  assert.match(lookupHint('chadf@getalby.com'), /getalby\.com/);
});

test('the new operation and the new cause have texts', () => {
  assert.equal(operationTitle({ kind: 'use_lnaddress' }), 'Use the Lightning address');
  assert.equal(operationLine({ kind: 'use_lnaddress', address: 'a@b.c' }, 'TJ'),
    'Studio replaces the node key of TJ with a@b.c in the selected feeds.');
  assert.equal(causeText({ cause: 'no_node_recipient' }), 'Each value block has the Lightning address already.');
});

test('the type menu of the operation sheet has no placeholder', async () => {
  const { readFileSync } = await import('node:fs');
  const text = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = text.indexOf('id="op-type"');
  const element = text.slice(start, text.indexOf('/>', start));
  assert.ok(!element.includes('placeholder'),
    'ADR 0095 item 14: the Type menu of the operation sheet has no placeholder. Remove it.');
  const form = text.slice(text.indexOf('<div class="group rows">', text.indexOf("kind === 'change_address' || kind === 'add'")));
  assert.ok(form.indexOf('${typeMenu}') < form.indexOf('id="op-address"'),
    'ADR 0095 item 14: the Type menu is the first field of the operation sheet. Put it before the address field.');
});
