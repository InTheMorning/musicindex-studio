// The model of the Recipients pages (ADR 0091, task 054c).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  recipientRow, rowLines, placesText, useRow, outcomeText, causeText, blockedText,
  initialStatuses, addressLines, lnaddressOf, nodeOf, recipientTitle, lookupHint, memberStart, startType, operationLine, suggestionField, breakParts, OTHER_ADDRESS, startWrite, finishWrite, nextToWrite, statusText, writableIds, writeLabel, retryStatuses, nameOf, statusError,
  walletActions, walletSheet, walletSheetStart, walletTypeChange, walletDetailsLine, walletSheetBody, walletSheetReady, WALLET_TYPES, CUSTOM_KEY_HINT,
} from '../ui/recipients.js';

test('the row of a used recipient gives the title, the count and the lines', () => {
  const row = recipientRow({ name: 'Ann', recipient_type: 'lnaddress', address: 'ann@example.com', feeds: 3, saved_id: null });
  assert.deepEqual(row, { title: 'Ann', count: '3 feeds', lines: ['Lightning address ann@example.com'] });
  assert.equal(recipientRow({ name: 'Ann', address: 'a', feeds: 1 }).count, '1 feed');
});

test('the count of a saved recipient that no feed uses says so', () => {
  const row = recipientRow({ name: 'Cara', recipient_type: 'node', address: '02ab12cd34ef5678', feeds: 0, saved_id: 4 });
  assert.equal(row.count, 'Saved, not used');
  assert.deepEqual(row.lines, ['Lightning node 02ab12cd…5678'],
    'ADR 0095 item 7: a node key with no provider shows its first 8 and its last 4 characters.');
  assert.equal(recipientRow({ name: 'Dan', address: '03cd', feeds: 0, saved_id: null }).count, '0 feeds',
    'ADR 0091 item 2: only a saved recipient shows "Saved, not used". Give the number of feeds for each other recipient.');
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
  // A button label uses title case (musicindex ADR 0023 R-CASE-1, ADR 0099 item 12).
  assert.equal(writeLabel(2), 'Write 2 Feeds');
  assert.equal(writeLabel(1), 'Write 1 Feed');
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
    { label: 'Lightning address', address: 'tj@getalby.com', account: null },
    { label: 'Lightning node', address: '02ab12cd34ef5678', account: '01abc' }],
  'ADR 0095 item 8: the header shows the full key and the account.');
  assert.equal(lnaddressOf(wallet), 'tj@getalby.com');
  assert.equal(nodeOf(wallet), '02ab12cd34ef5678');
  const row = recipientRow({ ...wallet, name: 'TJ Doerfel' });
  assert.equal(row.count, '7 feeds');
  assert.deepEqual(row.lines, ['Lightning address tj@getalby.com', 'Lightning node 02ab12cd…5678 · account 01abc']);
  const keyFirst = { ...wallet, addresses: [wallet.addresses[1], wallet.addresses[0]] };
  assert.equal(recipientRow(keyFirst).lines[0], 'Lightning address tj@getalby.com',
    'ADR 0095 item 7: the Lightning address is first in the row.');
  assert.equal(addressLines(keyFirst)[0].label, 'Lightning address',
    'ADR 0095 item 8: the header has the lines of item 7, the Lightning address first.');
});

const keyA = '03d55f4d0000000000000000000000000000000000000000000000000000aa67';

test('rowLines gives a provider node with its domain and its account', () => {
  const entry = { address: keyA, addresses: [
    { address: keyA, recipient_type: 'node', provider: 'fountain.fm', custom_value: '01IMQkt4BFzAiSynxcQQqd' },
    { address: 'boostbot@fountain.fm', recipient_type: 'lnaddress' }] };
  assert.deepEqual(rowLines(entry), ['Lightning address boostbot@fountain.fm', 'fountain.fm node · account 01IMQkt4BFzAiSynxcQQqd'],
    'ADR 0095 item 7: a provider node shows its domain and no key.');
  assert.deepEqual(addressLines(entry)[1], { label: 'fountain.fm node', address: keyA, account: '01IMQkt4BFzAiSynxcQQqd' },
    'ADR 0095 item 8: the header shows the full key of a provider node, after the Lightning address.');
});

test('rowLines gives a node with no provider and no account', () => {
  assert.deepEqual(rowLines({ address: keyA, recipient_type: 'node' }), ['Lightning node 03d55f4d…aa67'],
    'ADR 0095 item 7: a node with no provider shows its short key.');
});

test('rowLines gives a node with no provider and an account', () => {
  assert.deepEqual(rowLines({ address: keyA, recipient_type: 'node', custom_value: 'abc' }), ['Lightning node 03d55f4d…aa67 · account abc']);
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
  assert.deepEqual(addressLines(entry), [{ label: 'Lightning node', address: '03cd', account: null }]);
  assert.equal(lnaddressOf(entry), null);
  assert.equal(nodeOf(entry), null);
});

test('the lookup hint has the text of ADR 0099 item 16', () => {
  assert.equal(lookupHint('chadf@getalby.com'),
    'Studio found no payment details for this address at getalby.com. Some apps cannot pay it.');
});

test('the suggestion field gives a menu with suggestions and a text field with none (ADR 0097)', () => {
  const none = suggestionField([]);
  assert.equal(none.menu, false);
  assert.equal(none.start, '');
  assert.equal(none.hint, 'Studio found no Lightning address for this wallet.');
  assert.equal(suggestionField(null).menu, false);
  const some = suggestionField([
    { address: 'ann@fountain.fm', sources: ['collection', 'provider'] },
    { address: 'ann@pay.example', sources: ['nostr'] },
  ]);
  assert.equal(some.menu, true);
  assert.equal(some.start, 'ann@fountain.fm');
  assert.deepEqual(some.options, [
    ['ann@fountain.fm', 'ann@fountain.fm'],
    ['ann@pay.example', 'ann@pay.example'],
    [OTHER_ADDRESS, 'Other address'],
  ]);
  assert.equal(some.hints['ann@fountain.fm'], 'Studio found this address in your feeds and at the node provider. It pays the same wallet.');
  assert.equal(some.hints['ann@pay.example'], 'Studio found this address in a Nostr profile. It pays the same wallet.');
  assert.equal(some.hints[OTHER_ADDRESS], undefined);
});

test('a title breaks after its @', () => {
  assert.deepEqual(breakParts('empatheyes@fountain.fm'), ['empatheyes@', 'fountain.fm']);
  assert.deepEqual(breakParts('StevenB'), ['StevenB']);
  assert.deepEqual(breakParts(null), ['']);
});

test('the new operation and the new cause have texts', () => {
  assert.equal(operationLine({ kind: 'use_lnaddress', address: 'a@b.c' }, 'TJ'),
    'Studio replaces the node key of TJ with a@b.c in the selected feeds.');
  assert.equal(causeText({ cause: 'no_node_recipient' }), 'Each value block has the Lightning address already.');
});

test('the address lines of a row and of a header are one function (ADR 0099 item 15)', () => {
  const entry = { address: keyA, addresses: [
    { address: keyA, recipient_type: 'node', provider: 'fountain.fm', custom_value: '01ab' },
    { address: 'bot@fountain.fm', recipient_type: 'lnaddress' }] };
  assert.deepEqual(rowLines(entry), ['Lightning address bot@fountain.fm', 'fountain.fm node · account 01ab']);
  assert.deepEqual(addressLines(entry).map((l) => l.label), ['Lightning address', 'fountain.fm node']);
});

test('the type menu of a wallet sheet gives Lightning Node and Lightning Address (ADR 0099 item 14)', () => {
  assert.deepEqual(WALLET_TYPES, [['node', 'Lightning Node'], ['lnaddress', 'Lightning Address']]);
  assert.equal(CUSTOM_KEY_HINT, 'Most wallets do not need it. Copy it from your wallet when it gives one.');
});

test('the page of a wallet has the five operations, the last one destructive (ADR 0099 item 7)', () => {
  const actions = walletActions(wallet);
  assert.deepEqual(actions.map((a) => [a.kind, a.label, a.role]), [
    ['change_address', 'Change Address', 'action'],
    ['rename', 'Rename', 'action'],
    ['use_lnaddress', 'Use the Lightning Address', 'action'],
    ['add', 'Add to All Releases', 'action'],
    ['remove', 'Remove', 'destructive']]);
  assert.ok(!walletActions({ address: 'a@b.c', recipient_type: 'lnaddress' }).some((a) => a.kind === 'use_lnaddress'),
    'ADR 0095: a wallet with no node key has nothing to replace.');
});

test('each sheet of a wallet lists the feeds, each selected, and counts them in its button (ADR 0099 item 7)', () => {
  const uses = [
    { feed_id: 1, title: 'One', channel: true, items: 2, live_items: 0, writable: true },
    { feed_id: 2, title: '', channel: false, items: 0, live_items: 1, writable: false }];
  for (const kind of ['change_address', 'rename', 'use_lnaddress', 'remove']) {
    const sheet = walletSheet(kind, uses);
    assert.deepEqual(sheet.rows.map((r) => [r.id, r.checked]), [[1, true], [2, true]]);
    assert.equal(sheet.button, 'Review in 2 Feeds');
  }
  assert.deepEqual(walletSheet('rename', uses).rows[0], { id: 1, title: 'One', lines: ['Channel · 2 tracks'], checked: true });
  assert.equal(walletSheet('rename', [uses[0]]).button, 'Review in 1 Feed');
  assert.equal(walletSheet('add', uses).title, 'Add to All Releases');
  assert.deepEqual(walletSheet('add', uses).rows, [], 'The feeds of Add to All Releases come from the artist or label that the user selects.');
});

test('the fields of a wallet sheet start at the member with the type and give the body of the route', () => {
  const fields = walletSheetStart('change_address', wallet);
  assert.equal(fields.type, 'node');
  assert.equal(fields.address, '02ab12cd34ef5678');
  assert.deepEqual(walletSheetBody('change_address', { ...fields, address: ' 03ff ', custom_value: '' }),
    { kind: 'change_address', address: '03ff', custom_key: '906608', custom_value: null, recipient_type: 'node' });
  const toAddress = walletTypeChange(wallet, fields, 'lnaddress');
  assert.equal(toAddress.address, 'tj@getalby.com');
  assert.deepEqual(walletSheetBody('change_address', toAddress),
    { kind: 'change_address', address: 'tj@getalby.com', custom_key: null, custom_value: null, recipient_type: 'lnaddress' });
  assert.equal(walletTypeChange(wallet, { ...fields, address: 'typed' }, 'lnaddress').address, 'typed');
  assert.equal(walletSheetStart('use_lnaddress', wallet).address, 'tj@getalby.com');
  assert.deepEqual(walletSheetBody('remove', fields), { kind: 'remove' });
});

test('Add to All Releases needs a percentage, an address and one feed; its details have a summary line', () => {
  const fields = { ...walletSheetStart('add', wallet), percent: '' };
  assert.equal(walletSheetReady('add', fields, [1]), false);
  assert.equal(walletSheetReady('add', { ...fields, percent: '5' }, []), false);
  assert.equal(walletSheetReady('add', { ...fields, percent: '5' }, [1]), true);
  assert.equal(walletSheetReady('remove', fields, [1]), true);
  assert.equal(walletDetailsLine({ type: 'lnaddress', address: 'tj@getalby.com' }), 'Lightning Address · tj@getalby.com');
  assert.equal(walletDetailsLine({ type: 'node', address: '' }), 'Lightning Node · No address yet');
  assert.equal(walletSheetBody('add', { ...fields, percent: '5', name: ' ' }).percent, 5);
});
