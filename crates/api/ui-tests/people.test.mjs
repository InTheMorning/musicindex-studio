// The model of the People area (ADR 0099, task 065a).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  changeSetView, peopleSections, PEOPLE_EMPTY, PEOPLE_SERVER_NOTE, suggestionRow, personRow, makePersonStart, makePersonBody, sheetFeeds,
  checkedIds, toggledRow, reviewLabel, operationBody, addResultLine,
  lookupAddress, showFindEmpty,
} from '../ui/people.js';
import { pageSequence, tierError } from '../ui/pages.js';

const SUGGESTION = {
  name: 'Mary Kate Ultra',
  credit_feeds: 3,
  payee_feeds: 6,
  evidence: ['same_name', 'same_wallet'],
  credits: [
    { name: 'Mary Kate Ultra', href: '', npub: 'npub1a', same_name: true },
    { name: 'MK Ultra', href: 'https://x.example', npub: 'npub1a', same_name: false },
  ],
  recipients: [
    { key: { address: 'a@x.example', custom_key: null, custom_value: null }, recipient_type: 'lnaddress', same_name: true },
    { key: { address: '02ab', custom_key: null, custom_value: null }, recipient_type: 'node', same_name: false },
  ],
};

test('the People page gives the sections of ADR 0099 item 2 in sequence', () => {
  const all = { people: [{}], suggestions: [{}], changes: [{}], payees: [{}] };
  assert.deepEqual(peopleSections(all), ['search', 'actions', 'changes', 'people', 'suggestions', 'payees'],
    'ADR 0099 item 2: with a person, the suggestions come after the people. Fix peopleSections in crates/api/ui/people.js.');
});

test('with no person, the suggestions come before the people', () => {
  assert.deepEqual(peopleSections({ suggestions: [{}], payees: [{}] }), ['search', 'actions', 'suggestions', 'people', 'payees']);
});

test('a section with no data is left out, but people always shows', () => {
  assert.deepEqual(peopleSections({}), ['search', 'actions', 'people']);
  assert.equal(PEOPLE_EMPTY, 'No people yet. Check a suggestion above, or add a person.');
});

test('ADR 0108 item 3: the People page of a server starts with the line about the operator, and the desktop has none', () => {
  assert.equal(PEOPLE_SERVER_NOTE, 'The operator of this server can read the people that you keep.');
  assert.deepEqual(peopleSections({ server: true }), ['note', 'search', 'actions', 'people'],
    'ADR 0108 item 3: a server shows the line first. Fix peopleSections in crates/api/ui/people.js.');
  assert.ok(!peopleSections({ server: false }).includes('note'),
    'ADR 0108 item 3: the desktop has no such line. Fix peopleSections in crates/api/ui/people.js.');
  assert.ok(!peopleSections({}).includes('note'));
});

test('the page person has no feed list with check boxes', () => {
  const sections = pageSequence('person');
  assert.ok(!sections.some((s) => /feeds/.test(s.id)),
    'ADR 0099 item 6: the page of a person has no feed list with check boxes. Remove the section from crates/api/ui/pages.js.');
});

test('the row of a suggestion gives the counts and the evidence', () => {
  assert.deepEqual(suggestionRow(SUGGESTION), {
    title: 'Mary Kate Ultra',
    lines: ['Credit in 3 feeds · Payee in 6 feeds', 'The same name is in the credits and in the value blocks'],
  });
  const npub = suggestionRow({ name: 'Sam', credit_feeds: 1, payee_feeds: 3, evidence: ['same_npub'], recipients: [{}] });
  assert.deepEqual(npub.lines, ['Credit in 1 feed · Payee in 3 feeds', 'The same Nostr key is in the credits']);
  const wallet = suggestionRow({ name: 'Wal', credit_feeds: 0, payee_feeds: 2, evidence: ['same_wallet'], recipients: [{}] });
  assert.deepEqual(wallet.lines, ['Payee in 2 feeds', 'The same wallet is in the value blocks']);
});

test('a suggestion with no recipient says so', () => {
  const row = suggestionRow({ name: 'Jo Keys', credit_feeds: 4, payee_feeds: 0, evidence: ['same_npub'], recipients: [] });
  assert.deepEqual(row.lines, ['Credit in 4 feeds', 'No payment address found']);
});

test('the row of a person gives the role, the membership, the address and the count', () => {
  const person = {
    name: 'Sam Fiddle', usual_role: 'Fiddle', members: [7],
    default_address: { address: 'sam@getalby.com', custom_key: null, custom_value: null },
    wallets: [{ key: { address: 'sam@getalby.com' }, recipient_type: 'lnaddress' }],
    pays: [1, 2], credited: [2, 3],
  };
  assert.deepEqual(personRow(person, { 7: 'The Ultras' }), {
    title: 'Sam Fiddle',
    lines: ['Fiddle · Linked to The Ultras', 'Pays LNURL address sam@getalby.com'],
    count: '3 feeds',
  });
});

test('the row of a person with two wallets and no role gives the wallet count', () => {
  const person = {
    name: 'Jo', usual_role: '', members: [],
    default_address: { address: 'jo@x.example' },
    wallets: [{ key: { address: 'jo@x.example' }, recipient_type: 'lnaddress' }, { key: { address: '02ab' }, recipient_type: 'node' }],
    feeds: 1, pays: 1, credited: 0,
  };
  const row = personRow(person, new Map());
  assert.deepEqual(row.lines, ['Pays LNURL address jo@x.example · 2 wallets']);
  assert.equal(row.count, '1 feed');
  assert.deepEqual(personRow({ name: 'Jo Keys', usual_role: 'Keyboards', wallets: [], feeds: 4 }, {}).lines,
    ['Keyboards', 'No payment address']);
});

test('Make a Person starts an item with a different name not selected', () => {
  const start = makePersonStart(SUGGESTION);
  assert.deepEqual(start.credits.map((r) => r.checked), [true, false],
    'ADR 0099 item 3: an item with a different name starts not selected. Fix makePersonStart in crates/api/ui/people.js.');
  assert.deepEqual(start.recipients.map((r) => r.checked), [true, false]);
  assert.deepEqual(start.keys, [{ item: 'npub1a', checked: true }]);
  assert.equal(start.name, 'Mary Kate Ultra');
  assert.equal(start.memberOf, '');
});

test('the body of Make a Person has only the checked items', () => {
  const state = makePersonStart(SUGGESTION);
  assert.deepEqual(makePersonBody(state), {
    name: 'Mary Kate Ultra',
    credits: [['Mary Kate Ultra', '', 'npub1a']],
    recipients: [[SUGGESTION.recipients[0].key, 'lnaddress']],
    keys: ['npub1a'],
    member_of: null,
  });
  state.keys[0].checked = false;
  state.memberOf = '7';
  const body = makePersonBody(state);
  assert.deepEqual(body.keys, []);
  assert.equal(body.member_of, 7);
});

const PLACES = [
  { feed_id: 1, pays: [{ address: 'mk@x.example' }], roles: ['Vocals'] },
  { feed_id: 2, pays: [], roles: [''] },
  { feed_id: 3, pays: [{ address: '02f4c1a9d0e3b27c' }], roles: [] },
];

test('the feeds of a sheet start selected, with their lines', () => {
  const rows = sheetFeeds(PLACES, { 1: 'Neon Diner', 2: 'Midnight Arcade', 3: 'Basement Tapes' });
  assert.deepEqual(rows.map((r) => r.checked), [true, true, true],
    'ADR 0099 item 7: each check box starts selected. Fix sheetFeeds in crates/api/ui/people.js.');
  assert.deepEqual(rows[0], { id: 1, title: 'Neon Diner', lines: ['Pays LNURL address mk@x.example', 'Credited as Vocals'], checked: true });
  assert.deepEqual(rows[1].lines, ['Credited']);
  assert.match(rows[2].lines[0], /^Pays Node address /);
});

test('the selection of a row turns over, and checkedIds gives the selected rows', () => {
  const rows = sheetFeeds(PLACES, {});
  const next = toggledRow(rows, 2);
  assert.deepEqual(checkedIds(next), [1, 3]);
  assert.deepEqual(checkedIds(rows), [1, 2, 3], 'toggledRow must give a new list.');
});

test('the review label gives the feed count', () => {
  assert.equal(reviewLabel(1), 'Review in 1 Feed');
  assert.equal(reviewLabel(3), 'Review in 3 Feeds');
});

test('each operation of a person gives the body of task 064', () => {
  const key = { address: 'a@x.example', custom_key: null, custom_value: null };
  assert.deepEqual(operationBody('use_address', { key }), { kind: 'use_address', key });
  assert.deepEqual(operationBody('rename', { name: 'Ann', payment: true, credit: false }),
    { kind: 'rename', name: 'Ann', payment: true, credit: false });
  assert.deepEqual(operationBody('add_to_releases', { payee: true, percent: '5', credit: true, supportLink: 4 }),
    { kind: 'add_to_releases', payee_percent: 5, credit: true, support_link: 4 });
  assert.deepEqual(operationBody('add_to_releases', { payee: false, percent: '5', credit: true }),
    { kind: 'add_to_releases', payee_percent: null, credit: true, support_link: null });
  assert.deepEqual(operationBody('remove', { payee: true, credit: true }), { kind: 'remove', payee: true, credit: true });
  assert.throws(() => operationBody('merge', {}), /not an operation of a person/);
});

const part = (name, changed, cause = null) => ({ part: name, changed, cause: cause && { cause } });
const changed = (...parts) => ({ outcome: 'changed', parts });
const same = (...parts) => ({ outcome: 'not_changed', parts });

test('the result line of Add to All Releases gives the mockup text', () => {
  const outcomes = [
    ...Array.from({ length: 7 }, () => changed(part('payee', true), part('credit', true))),
    ...Array.from({ length: 3 }, () => same(part('payee', false, 'has_recipient'), part('credit', false, 'has_credit'))),
  ];
  assert.equal(addResultLine(outcomes), '10 releases · 7 get the payee · 7 get the credit · 3 have both already');
});

test('the result line leaves out a count of 0 and names one part', () => {
  const one = [changed(part('payee', true)), same(part('payee', false, 'has_recipient')), same(part('payee', false, 'no_value_block'))];
  assert.equal(addResultLine(one), '3 releases · 1 get the payee · 1 have it already');
  const all = [same(part('payee', false, 'has_recipient'), part('credit', false, 'has_credit'), part('support_link', false, 'has_support_link'))];
  assert.equal(addResultLine(all), '1 release · 1 have all already');
  assert.equal(addResultLine([]), '');
});

import {
  initials, personMatches, filterPeople, filterPayees, payeesWithNoPerson, peopleOrder, openChanges, changeTitle, feedsToWrite,
  whereEntries, whereSummary, whereLine,
  changeRow, afterDiscard, afterMissing, memberChoices, creditItemRow, payeeItemRow, toggledItem, dismissBody, addPersonBody,
} from '../ui/people.js';

test('the search matches the name, an address and a key, with no regard to capitals', () => {
  const person = { name: 'Mary Kate', wallets: [{ key: { address: 'MK@fountain.fm' } }], keys: [{ npub: 'npub1abc' }] };
  assert.ok(personMatches(person, 'mary'));
  assert.ok(personMatches(person, 'mk@FOUNTAIN'));
  assert.ok(personMatches(person, 'NPUB1A'));
  assert.ok(personMatches(person, ''));
  assert.ok(!personMatches(person, 'sam'));
  assert.equal(filterPeople([person, { name: 'Sam' }], 'sam').length, 1);
  const payee = { name: 'BoostBot', address: '03d5', addresses: [{ address: 'bot@x.fm' }] };
  assert.equal(filterPayees([payee], 'bot@X').length, 1);
  assert.equal(filterPayees([payee], 'zzz').length, 0);
});

test('the lists of payees and of open change sets', () => {
  assert.deepEqual(payeesWithNoPerson([{ saved_id: null, name: 'a' }, { saved_id: 3, name: 'b' }]).map((r) => r.name), ['a']);
  assert.deepEqual(openChanges([{ id: 1, open: true }, { id: 2, open: false }]).map((s) => s.id), [1]);
  assert.equal(initials('mary kate ultra'), 'MK');
});

// musicindex ADR 0053 R-LIST-12: a list of names sorts by name, with numbers
// as numbers, and never by an address.
test('the payees with no person sort by name, then by the address that a row shows', () => {
  const payees = [
    { saved_id: null, name: '', address: 'aaa@x.example' },
    { saved_id: null, name: 'Track 15', address: '02bb' },
    { saved_id: null, name: 'zoe', address: '01aa' },
    { saved_id: null, name: 'Track 2', address: '03cc' },
    { saved_id: null, name: 'Anna', address: 'zzz@x.example' },
    { saved_id: null, name: null, address: '00ff' },
  ];
  assert.deepEqual(payeesWithNoPerson(payees).map((r) => r.name || r.address),
    ['Anna', 'Track 2', 'Track 15', 'zoe', '00ff', 'aaa@x.example']);
});

// musicindex ADR 0053 R-LIST-9 and ADR 0043 R-LIST-7, ADR 0153: People has
// "A to Z", the default, and "Newest", and each one reverses.
test('the People list sorts A to Z or Newest, and reverses', () => {
  const people = [
    { id: 1, name: 'zoe', created_at: 300 },
    { id: 2, name: 'Anna 10', created_at: 100 },
    { id: 3, name: 'anna 9', created_at: 200 },
    { id: 4, name: '4 Hero', created_at: 200 },
  ];
  const az = peopleOrder(people, {});
  assert.equal(az.sort, 'az');
  assert.deepEqual(az.sorts, ['az', 'newest']);
  assert.deepEqual(az.people.map((p) => p.id), [4, 3, 2, 1]);
  assert.deepEqual(az.letters, [{ letter: '#', firstId: 4 }, { letter: 'A', firstId: 3 }, { letter: 'Z', firstId: 1 }]);
  const za = peopleOrder(people, { sort: 'az', reversed: true });
  assert.equal(za.reversed, true);
  assert.deepEqual(za.people.map((p) => p.id), [1, 2, 3, 4]);
  assert.deepEqual(za.letters.map((g) => g.letter), ['Z', 'A', '#']);
  const newest = peopleOrder(people, { sort: 'newest' });
  assert.deepEqual(newest.people.map((p) => p.id), [1, 4, 3, 2]);
  assert.equal(newest.letters, null);
  assert.deepEqual(peopleOrder(people, { sort: 'newest', reversed: true }).people.map((p) => p.id), [2, 3, 4, 1]);
  assert.equal(peopleOrder(people, { sort: 'unknown' }).sort, 'az');
});

test('a change row gives the operation, the subject and the feeds to write', () => {
  const feeds = [
    { outcome: { outcome: 'changed' }, written_at: null },
    { outcome: { outcome: 'changed' }, written_at: 5 },
    { outcome: { outcome: 'unchanged' }, written_at: null },
  ];
  assert.equal(feedsToWrite({ feeds }), 1);
  const person = changeRow({ operation: { kind: 'rename_person' }, person_id: 7, feeds }, [{ id: 7, name: 'Mary' }]);
  assert.deepEqual(person, { title: 'Rename · Mary', text: '1 feed to write' });
  const wallet = changeRow({ operation: { kind: 'remove' }, recipient: { address: 'a@x' }, feeds }, [], [{ address: 'a@x', name: 'Ann' }]);
  assert.equal(wallet.title, 'Remove · Ann');
  assert.equal(changeTitle({ kind: 'add_to_releases' }), 'Add to All Releases');
});

test('Discard opens the person of a person change set, else People (ADR 0099 item 12)', () => {
  assert.deepEqual(afterDiscard({ person_id: 7 }), { name: 'person', id: 7 });
  assert.deepEqual(afterDiscard({ person_id: null }), { name: 'people' });
});

test('a person that is gone opens the person that kept its items, else People', () => {
  const merged = new Map([['4', 9]]);
  assert.deepEqual(afterMissing(4, merged), { name: 'person', id: 9 });
  assert.deepEqual(afterMissing('4', merged), { name: 'person', id: 9 });
  assert.deepEqual(afterMissing(5, merged), { name: 'people' });
});

test('the sheet Check gives its rows and menu', () => {
  assert.deepEqual(memberChoices([
    { id: 2, medium: 'publisher', title: 'Zed' }, { id: 1, medium: 'music', title: 'Album' }, { id: 3, medium: 'publisher', title: 'Ann' },
  ]), [['3', 'Ann'], ['2', 'Zed']]);
  assert.equal(creditItemRow({ roles: ['Vocals'], feeds: [1, 2] }, { 1: 'A', 2: 'B' }).lines[0], 'A, B');
  assert.match(payeeItemRow({ key: { address: 'a@x.fm' }, recipient_type: 'lnaddress', feeds: [1], same_name: false, name: 'MK' }).lines[0], /the name in that feed is "MK"/);
  const state = makePersonStart(SUGGESTION);
  assert.equal(toggledItem(state, 'recipients', 0).recipients[0].checked, !state.recipients[0].checked);
  assert.deepEqual(dismissBody({ identity: 'i', item_keys: ['k'] }), { identity: 'i', items: ['k'] });
  assert.deepEqual(addPersonBody('  Sam '), { name: 'Sam' });
});

// ---- Task 065c: the page of a person and its sheets ----
import {
  keyValue, keyFromValue, memberLine, memberText, walletGroups, addressOptions, usedText, keyEvidenceText, checkRow,
  savedValues, pendingKeys, pendingStatus, isPending, saveBody, leaveText, checkGroups, findingLine, checkFeedLines, checkKeys,
  feedRows, levelRows, feedsHeading, SHEET_TITLES, sheetStart, sheetBody, sheetReady, LINK_FORMS, linkFields,
  linkStart, linkBody, linkReady, profileLine, changeLine, addIntro,
  lookupKind, lookupGroups, LOOKUP_LEVELS, hasLine, fillRows, fillLabel, fillPatch, fillSource, musicindexFeedUrl,
} from '../ui/people.js';

const KEY = (address, custom_value = null) => ({ address, custom_key: null, custom_value });
const MK = {
  id: 7, name: 'Mary Kate', usual_role: 'Vocals', img: '', href: '', members: [3],
  default_address: KEY('mk@x.example'),
  wallets: [{
    id: 1, key: KEY('02ab', 'acct1'), recipient_type: 'node',
    members: [
      { key: KEY('02ab', 'acct1'), recipient_type: 'node' },
      { key: KEY('mk@x.example'), recipient_type: 'lnaddress' },
    ],
  }],
  support_links: [{ id: 4, label: 'Support Mary', url: 'https://x.example/s' }],
  pays: [1, 2], credited: [2, 5], feeds: 3,
};
const RECIPIENTS = [{
  address: '02ab', addresses: [
    { address: '02ab', custom_key: null, custom_value: 'acct1', recipient_type: 'node', provider: 'fountain.fm', feeds: 4 },
    { address: 'mk@x.example', custom_key: null, custom_value: null, recipient_type: 'lnaddress', feeds: 2 },
  ],
}];

test('a key makes a menu value and back', () => {
  const key = KEY('02ab', 'acct1');
  assert.deepEqual(keyFromValue(keyValue(key)), key);
  assert.deepEqual(keyFromValue(keyValue(KEY('a@x'))), KEY('a@x'));
});

test('the lines of an address follow ADR 0099 item 15', () => {
  assert.deepEqual(memberLine({ address: 'a@x.example', recipient_type: 'lnaddress' }),
    { label: 'LNURL address', address: 'a@x.example', account: null });
  assert.equal(memberText({ address: '02ab', recipient_type: 'node', provider: 'fountain.fm', custom_value: 'acct1' }), 'fountain.fm node · account acct1');
  assert.equal(memberText({ address: 'a@x.example' }), 'LNURL address a@x.example');
  assert.equal(memberText({ address: '03d55f4d0000aa67', recipient_type: 'node' }), 'Node address 03d55f4d…aa67');
  assert.equal(memberText({ address: '03d55f4d0000aa67', recipient_type: 'node', custom_value: 'acct1' }), 'Node address 03d55f4d…aa67 · account acct1');
  assert.equal(usedText(0), 'Not used');
  assert.equal(usedText(1), '1 feed');
});

test('a wallet gives its addresses, the LNURL addresses first', () => {
  const groups = walletGroups(MK, RECIPIENTS);
  assert.equal(groups.length, 1);
  assert.deepEqual(groups[0].members.map((m) => m.key.address), ['mk@x.example', '02ab']);
  assert.deepEqual(addressOptions(groups).map(([, text]) => text), ['LNURL address mk@x.example', 'fountain.fm node · account acct1']);
  const alone = walletGroups({ ...MK, wallets: [{ id: 1, key: KEY('02ab'), recipient_type: 'node' }] }, []);
  assert.deepEqual(alone[0].members.map((m) => m.key.address), ['02ab']);
});

test('the evidence of a key and the row Check your feeds have their texts', () => {
  assert.equal(keyEvidenceText('user'), 'Added by you');
  assert.equal(keyEvidenceText('nostr'), 'Found in a Nostr profile');
  const lines = (n) => checkRow({ ...MK, needs_correction: n }).lines;
  // ADR 0156 item 6: the row gives only the state of the check.
  assert.deepEqual(lines(0), [{ text: 'Each feed is correct', error: false }]);
  assert.deepEqual(lines(1)[0], { text: '1 feed needs a correction', error: true });
  assert.deepEqual(lines(2)[0], { text: '2 feeds need a correction', error: true });
  const row = checkRow(MK);
  assert.equal(row.title, 'Check your feeds');
  assert.equal(row.count, '3 feeds');
});

test('the saved values and the pending changes of a person (ADR 0116 items 1 to 3)', () => {
  const saved = savedValues(MK);
  assert.deepEqual(saved, {
    name: 'Mary Kate', usual_role: 'Vocals', usual_group: '', img: '', href: '', default_address: keyValue(KEY('mk@x.example')), members: ['3'], pages: [],
    wallets: [], keys: [],
  });
  assert.deepEqual(pendingKeys(saved, saved), []);
  assert.equal(saveBody(saved, saved), null);
  assert.equal(pendingStatus(0), 'No changes');
  assert.equal(pendingStatus(1), '1 change not saved');
  assert.equal(pendingStatus(2), '2 changes not saved');
  const name = { ...saved, name: 'Mary' };
  assert.deepEqual(pendingKeys(saved, name), ['name']);
  assert.deepEqual(saveBody(saved, name), { fields: { name: 'Mary', usual_role: 'Vocals', usual_group: '', img: '', href: '' } });
  const role = { ...saved, usual_group: 'Band' };
  assert.deepEqual(pendingKeys(saved, role), ['usual_role'], 'ADR 0116 item 1: the role and its group are one control.');
  assert.ok(isPending(saved, role, 'usual_role'));
  assert.deepEqual(saveBody(saved, role), { fields: { name: 'Mary Kate', usual_role: 'Vocals', usual_group: 'Band', img: '', href: '' } });
  const address = { ...saved, default_address: keyValue(KEY('new@x.example')) };
  assert.deepEqual(pendingKeys(saved, address), ['default_address']);
  assert.deepEqual(saveBody(saved, address), { default_address: KEY('new@x.example') });
  assert.deepEqual(saveBody(saved, { ...saved, default_address: '' }), { default_address: null });
  assert.deepEqual(saveBody(saved, { ...saved, members: ['3', '5'] }), { members: [3, 5] });
  assert.deepEqual(saveBody(saved, { ...saved, members: [] }), { members: [] });
  assert.deepEqual(saveBody(saved, { ...saved, img: 'https://x.example/p.jpg', name: 'Mary' }), {
    fields: { name: 'Mary', usual_role: 'Vocals', usual_group: '', img: 'https://x.example/p.jpg', href: '' },
  });
});

test('the leave sheet names each pending control (ADR 0116 item 8)', () => {
  assert.equal(leaveText('DuhLaurien', ['img', 'href']),
    'DuhLaurien has 2 changes that are not saved: the picture address and the information address.');
  assert.equal(leaveText('Mary Kate', ['name']), 'Mary Kate has 1 change that is not saved: the name.');
});

test('the lines of each finding and the group of each state (ADR 0111 items 9 to 11)', () => {
  const profiles = new Map([['npub1abcdefghijklmnop', { npub: 'npub1abcdefghijklmnop', name: 'City Radio' }]]);
  assert.deepEqual(findingLine({ kind: 'credit_key', role: 'Host', npub: '' }, 'Ann'), { text: 'No Nostr key in the credit', error: true });
  assert.deepEqual(findingLine({ kind: 'credit_key', role: 'Host', npub: 'npub1abcdefghijklmnop' }, 'Ann', profiles),
    { text: 'Nostr key of City Radio · not the key of Ann', error: true });
  assert.equal(findingLine({ kind: 'credit_details', role: 'Host', img: true, href: true }, 'Ann').text,
    'The picture and information address in the credit differs from Ann');
  assert.equal(findingLine({ kind: 'credit_details', role: 'Host', img: false, href: true }, 'Ann').text,
    'The information address in the credit differs from Ann');
  assert.equal(findingLine({ kind: 'credit_role' }, 'Ann').error, true);
  const key = KEY('mk@x.example');
  assert.deepEqual(findingLine({ kind: 'not_default', key }, 'Ann'), { text: 'Pays LNURL address mk@x.example · not your default address', error: false });
  assert.equal(findingLine({ kind: 'not_default', key }, 'Ann', null, 'not_default').text, 'Pays LNURL address mk@x.example · the same wallet as your default address');
  assert.deepEqual(findingLine({ kind: 'other_wallet', key }, 'Ann'), { text: 'Pays LNURL address mk@x.example · not a wallet of Ann', error: true });
  const check = { feeds: [
    { feed_id: 1, state: 'correct', findings: [] },
    { feed_id: 2, state: 'not_default', findings: [{ kind: 'not_default', key }] },
    { feed_id: 3, state: 'needs_correction', findings: [{ kind: 'credit_key', role: 'Host', npub: 'npub1abcdefghijklmnop' }, { kind: 'credit_role' }] },
  ] };
  assert.deepEqual(checkGroups(check).map((g) => [g.title, g.feeds.map((f) => f.feed_id)]),
    [['Needs a correction', [3]], ['Not your default address', [2]], ['Correct', [1]]]);
  assert.deepEqual(checkGroups({ feeds: [check.feeds[0]] }).map((g) => g.title), ['Correct']);
  assert.deepEqual(checkGroups(null), []);
  assert.deepEqual(checkKeys(check), ['npub1abcdefghijklmnop']);
  assert.deepEqual(checkFeedLines(check.feeds[2], ['Album'], 'Ann', profiles).map((l) => l.error), [false, true, true]);
});

test('the operation menu labels each feed with its first role (ADR 0111 item 3)', () => {
  assert.deepEqual(memberChoices([{ id: 2, medium: 'publisher', title: 'HeyCitizen', publisher_roles: ['artist', 'label'] }]), [['2', 'HeyCitizen · Artist']]);
});

test('the feed rows give the type first, each selected', () => {
  const feeds = [{ id: 2, title: 'Neon Diner', medium: 'music' }];
  const rows = feedRows([{ feed_id: 2, pays: [KEY('02ab')], roles: ['Vocals'] }], feeds);
  assert.equal(rows[0].lines[0], 'Album');
  assert.deepEqual(rows[0].lines.slice(1), ['Pays Node address 02ab', 'Credited as Vocals']);
  assert.ok(rows[0].checked);
  assert.equal(feedsHeading(rows), 'Feeds · 1 of 1 selected');
  assert.deepEqual(levelRows([2, 9], feeds).map((r) => [r.title, r.checked]), [['Neon Diner', true], ['Feed 9', true]]);
  // The artist or label feed of the level is not a release (ADR 0098 item 20).
  const level = [...feeds, { id: 4, title: 'The Ultras', medium: 'publisher' }];
  assert.deepEqual(levelRows([4, 2], level).map((r) => r.title), ['Neon Diner']);
});

test('each sheet starts with its fields and the button counts feeds', () => {
  const groups = walletGroups(MK, RECIPIENTS);
  assert.deepEqual(Object.keys(SHEET_TITLES), ['use_address', 'rename', 'add_to_releases', 'remove']);
  assert.equal(sheetStart('use_address', MK, groups).key, keyValue(KEY('mk@x.example')));
  assert.deepEqual(sheetStart('rename', MK, groups), { name: 'Mary Kate', payment: true, credit: true });
  assert.equal(sheetStart('add_to_releases', MK, groups).payee, true);
  assert.equal(sheetStart('add_to_releases', { ...MK, wallets: [] }, []).payee, false);
  assert.equal(addIntro('Ann'), 'Studio adds Ann to each release of the artist or label that you select.');
});

test('the body of a sheet is the body of the operation', () => {
  assert.deepEqual(sheetBody('use_address', { key: keyValue(KEY('a@x')) }), { kind: 'use_address', key: KEY('a@x') });
  assert.deepEqual(sheetBody('add_to_releases', { owner: '3', payee: true, percent: '5', credit: false, supportLink: '4' }),
    { kind: 'add_to_releases', payee_percent: 5, credit: false, support_link: 4 });
  assert.deepEqual(sheetBody('add_to_releases', { owner: '3', payee: false, percent: '', credit: true, supportLink: '' }),
    { kind: 'add_to_releases', payee_percent: null, credit: true, support_link: null });
});

test('the primary button needs a feed and a part', () => {
  assert.ok(!sheetReady('rename', { name: 'Ann', payment: true, credit: true }, []));
  assert.ok(sheetReady('rename', { name: 'Ann', payment: false, credit: true }, [1]));
  assert.ok(!sheetReady('rename', { name: 'Ann', payment: false, credit: false }, [1]));
  assert.ok(!sheetReady('rename', { name: ' ', payment: true, credit: true }, [1]));
  assert.ok(!sheetReady('remove', { payee: false, credit: false }, [1]));
  assert.ok(sheetReady('remove', { payee: true, credit: false }, [1]));
  assert.ok(!sheetReady('add_to_releases', { payee: true, percent: '', credit: false, supportLink: '' }, [1]));
  assert.ok(sheetReady('add_to_releases', { payee: true, percent: '5', credit: false, supportLink: '' }, [1]));
  assert.ok(sheetReady('use_address', { key: 'x' }, [1]));
  assert.ok(!sheetReady('use_address', { key: '' }, [1]));
});

test('the forms of a link give the fields and the body', () => {
  assert.deepEqual(Object.keys(LINK_FORMS), ['wallets', 'credit-names', 'support-links']);
  const start = linkStart('wallets');
  assert.equal(start.recipient_type, 'lnaddress');
  assert.deepEqual(linkFields('wallets', start).map((f) => f.id), ['recipient_type', 'address']);
  assert.deepEqual(linkFields('wallets', { recipient_type: 'node' }).map((f) => f.id), ['recipient_type', 'address', 'custom_key', 'custom_value']);
  assert.deepEqual(linkBody('wallets', { recipient_type: 'node', address: ' 02ab ', custom_key: '', custom_value: 'a' }),
    { recipient_type: 'node', address: '02ab', custom_key: null, custom_value: 'a' });
  assert.ok(linkReady('support-links', { label: '', url: 'https://x.example' }));
  assert.deepEqual(linkBody('support-links', { label: ' L ', url: 'u' }), { label: 'L', url: 'u' });
});

test('a credit name sends its Nostr key from the picker (ADR 0123)', () => {
  assert.equal(linkStart('credit-names').npub, null);
  assert.deepEqual(linkFields('credit-names', linkStart('credit-names')).map((f) => f.id), ['name', 'href']);
  assert.deepEqual(linkBody('credit-names', { name: ' Ann ', href: '', npub: 'npub1a' }), { name: 'Ann', href: '', npub: 'npub1a' });
  assert.deepEqual(linkBody('credit-names', { name: 'Ann', href: 'https://x.example', npub: null }), { name: 'Ann', href: 'https://x.example', npub: null });
  assert.ok(linkReady('credit-names', { name: 'Ann', href: '', npub: null }));
  assert.ok(!linkReady('credit-names', { name: ' ', href: '', npub: 'npub1a' }));
});

test('no link form has a plain field for a Nostr key (ADR 0123)', () => {
  for (const [kind, form] of Object.entries(LINK_FORMS)) {
    for (const f of form.fields) {
      assert.notEqual(f.id, 'npub',
        `ADR 0123 ${kind}: a link form has no plain field for a Nostr key. Use NostrRow or NostrSheet of editor.js with nostrKey: true on the form.`);
    }
  }
});

test('the usual role line and the membership of a person', () => {
  assert.deepEqual(profileLine(MK, { 3: 'The Ultras' }), { role: 'Vocals', members: [{ id: 3, title: 'The Ultras' }] });
  assert.deepEqual(profileLine(MK, { 3: 'The Ultras' }, [{ id: 3, medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true }]),
    { role: 'Vocals', members: [{ id: 3, title: 'The Ultras', dot: 'artist', role: 'Artist' }] });
});

test('the credit of a member has its own title and line (ADR 0118 item 12)', () => {
  assert.equal(changeTitle({ kind: 'credit_feed' }), 'Add to the Credits');
  assert.equal(changeLine({ feeds: [{}], operation: { kind: 'credit_feed' } }, 'DuhLaurien'), 'Add DuhLaurien as a credit in 1 feed');
});

test('the review line of a change set of a person names the operation', () => {
  const feeds = [{}, {}];
  assert.equal(changeLine({ feeds, operation: { kind: 'rename_person', name: 'Ann Ultra', payment: true, credit: true } }, 'Ann'),
    'Rename Ann to Ann Ultra in 2 feeds (payment name and credit name)');
  assert.equal(changeLine({ feeds, operation: { kind: 'rename_person', name: 'Ann Ultra', payment: true, credit: false } }, 'Ann Ultra'),
    'Rename to Ann Ultra in 2 feeds (payment name)');
  assert.equal(changeLine({ feeds, operation: { kind: 'use_address', address: 'ann@example.com', recipient_type: 'lnaddress' } }, 'Ann'),
    'Use LNURL address ann@example.com for Ann');
  assert.equal(changeLine({ feeds, operation: { kind: 'add_to_releases', payee: { percent: 5 }, credit: {}, support_link: null } }, 'Ann'),
    'Add Ann as a payee with 5% and a credit in 2 feeds');
  assert.equal(changeLine({ feeds, operation: { kind: 'remove_person', payee: true, credit: true } }, 'Ann'),
    'Remove Ann as a payee and a credit from 2 feeds');
  assert.equal(changeLine({ feeds, operation: { kind: 'merge' } }, 'Ann'), '');
});

import { readFileSync, readdirSync } from 'node:fs';

test('no text of the People area shows an XML attribute or the words "change set" (ADR 0099 items 13 and 17)', () => {
  for (const name of ['people.js', 'peoplepages.js', 'recipients.js']) {
    const source = readFileSync(new URL(`../ui/${name}`, import.meta.url), 'utf8')
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .split('\n')
      .filter((line) => !line.trim().startsWith('//'))
      .join('\n');
    assert.ok(!/method=/.test(source),
      `ADR 0099 item 17: ${name} shows an XML attribute such as method="lnaddress". Name the type with "LNURL Address" instead.`);
    assert.ok(!/change sets?/i.test(source),
      `ADR 0099 item 13: ${name} has a text with the words "change set". Say "changes" instead.`);
  }
});

test('the words of an address line and a change title follow ADR 0099 items 15 and 18', () => {
  assert.equal(memberText({ address: 'a@b.c', recipient_type: 'lnaddress' }), 'LNURL address a@b.c');
  assert.equal(memberText({ address: '03aa', recipient_type: 'node', provider: 'fountain.fm', custom_value: 'x1' }), 'fountain.fm node · account x1');
  assert.equal(memberText({ address: '03aa', recipient_type: 'node', custom_value: 'x1' }), 'Node address 03aa · account x1');
  assert.equal(changeTitle({ kind: 'add' }), 'Add to All Releases');
});

// ---- The recipient picker (task 066a) ----
import { recipientPickRows, recipientFromPerson } from '../ui/people.js';

const PICK_PEOPLE = [
  { id: 1, name: 'Mary Kate', img: '', default_address: { address: '03aa', custom_key: '696969', custom_value: 'acct1' },
    wallets: [{ key: { address: '03aa', custom_key: '696969', custom_value: 'acct1' }, recipient_type: 'node' }] },
  { id: 2, name: 'Jo Keys', img: '', default_address: null, wallets: [] },
];
const PICK_RECIPIENTS = [
  { name: 'Mary wallet', saved_id: 1, addresses: [{ address: '03aa', custom_key: '696969', custom_value: 'acct1', recipient_type: 'node', provider: 'fountain.fm' }] },
  { name: 'BoostBot', saved_id: null, addresses: [{ address: 'bot@x.example', recipient_type: 'lnaddress' }] },
];

test('recipientPickRows gives a person row with its default address and leaves out a person with no wallet', () => {
  const { people, payees } = recipientPickRows(PICK_PEOPLE, PICK_RECIPIENTS);
  assert.deepEqual(people.map((r) => r.name), ['Mary Kate']);
  assert.equal(people[0].line, 'Adds fountain.fm node · account acct1');
  assert.deepEqual(payees.map((r) => r.name), ['BoostBot']);
  assert.equal(payees[0].line, 'LNURL address bot@x.example');
});

test('recipientFromPerson gives the name and the values of the default address, and no split', () => {
  assert.deepEqual(recipientFromPerson(PICK_PEOPLE[0]), {
    name: 'Mary Kate', recipient_type: 'node', address: '03aa', split: '', custom_key: '696969', custom_value: 'acct1', fee: null,
  });
});

test('the type menu of the value editor uses the words of ADR 0099 item 14', async () => {
  const { readFileSync } = await import('node:fs');
  const source = readFileSync(new URL('../ui/values.js', import.meta.url), 'utf8');
  assert.match(source, /label: WALLET_TYPES\[0\]\[1\]/);
  assert.match(source, /label: WALLET_TYPES\[1\]\[1\]/);
  const { WALLET_TYPES } = await import('../ui/recipients.js');
  assert.deepEqual(WALLET_TYPES.map(([, label]) => label), ['Node Address', 'LNURL Address']);
  assert.match(source, /Add to People/);
});

test('a default address that is not a wallet row gets the type of its form', () => {
  const person = { name: 'Ann', default_address: { address: 'ann@example.com' }, wallets: [{ id: 1, key: { address: '02ab' }, recipient_type: 'node' }] };
  assert.equal(recipientFromPerson(person).recipient_type, 'lnaddress');
  const node = { name: 'Bo', default_address: { address: '03cd' }, wallets: [] };
  assert.equal(recipientFromPerson(node).recipient_type, 'node');
});

import { creditPickRows, creditFromPerson, supportPickRows, fundingFromLink, addPickLabel, nostrPickRows } from '../ui/people.js';

const CREDIT_SRC = [
  { id: 1, name: 'Mary Kate', usual_role: 'Vocals', img: 'https://x/mk.jpg', href: 'https://x', keys: [{ npub: 'npub1mku7abcdefghijklmnopq2xh' }, { npub: 'npub1second' }],
    support_links: [{ id: 4, label: 'Support Mary Kate', url: 'https://x/support' }] },
  { id: 2, name: 'Jo Keys', usual_role: '', img: '', href: null, keys: [], support_links: [] },
];

test('creditFromPerson gives the start values of a credit', () => {
  assert.deepEqual(creditFromPerson(CREDIT_SRC[0]), { name: 'Mary Kate', role: 'Vocals', group: null, img: 'https://x/mk.jpg', href: 'https://x', npub: 'npub1mku7abcdefghijklmnopq2xh' });
  assert.deepEqual(creditFromPerson(CREDIT_SRC[1]), { name: 'Jo Keys', role: null, group: null, img: null, href: null, npub: null });
});

test('creditPickRows gives the role and key line and marks a person in the list', () => {
  const rows = creditPickRows(CREDIT_SRC, []);
  assert.equal(rows.length, 2);
  assert.match(rows[0].line, /^Vocals · /);
  assert.equal(rows[1].line, '');
  assert.equal(rows[0].taken, false);
  assert.equal(creditPickRows(CREDIT_SRC, [{ name: 'other', npub: 'npub1mku7abcdefghijklmnopq2xh' }])[0].taken, true);
  assert.equal(creditPickRows(CREDIT_SRC, [{ name: ' jo keys ', npub: null }])[1].taken, true);
});

test('supportPickRows gives each support link and marks an address in the list', () => {
  const rows = supportPickRows(CREDIT_SRC, []);
  assert.equal(rows.length, 1);
  assert.equal(rows[0].name, 'Support Mary Kate');
  assert.equal(rows[0].line, 'Mary Kate · https://x/support');
  assert.equal(rows[0].taken, false);
  assert.equal(supportPickRows(CREDIT_SRC, [{ label: 'a', url: 'https://x/support' }])[0].taken, true);
  assert.deepEqual(fundingFromLink(CREDIT_SRC[0].support_links[0]), { label: 'Support Mary Kate', url: 'https://x/support' });
});

test('a fee payee gets the fee mark, and a picked fee payee starts as a fee', async () => {
  const { withFee, memberText } = await import('../ui/people.js');
  const { rowLines } = await import('../ui/recipients.js');
  const fee = { address: '03ae9f910000005e33', recipient_type: 'node', fee: true };
  assert.equal(withFee(memberText(fee), fee), 'Node address 03ae9f91…5e33 · fee');
  assert.deepEqual(rowLines({ addresses: [fee] }), ['Node address 03ae9f91…5e33 · fee']);
  const rows = recipientPickRows([], [{ saved_id: null, name: 'Podcast Index', addresses: [{ ...fee, name: 'Podcast Index' }] }]);
  assert.equal(rows.payees[0].line, 'Node address 03ae9f91…5e33 · fee');
  assert.equal(rows.payees[0].recipient.fee, 'true');
});

test('personKeys gives each address of each wallet of a person, and no payee', async () => {
  const { personKeys, recipientIdentity } = await import('../ui/people.js');
  const keys = personKeys([
    { saved_id: 4, addresses: [{ address: 'ann@example.com' }, { address: '02ab', custom_value: 'acct' }] },
    { saved_id: null, addresses: [{ address: '03cd' }] },
  ]);
  assert.ok(keys.has(recipientIdentity({ address: 'ann@example.com' })));
  assert.ok(keys.has(recipientIdentity({ address: '02ab', custom_value: 'acct' })));
  assert.ok(!keys.has(recipientIdentity({ address: '03cd' })));
});

// ---- Task 067d: profiles, members and delete ----
import {
  resolveBody, profileMap, keyRow, keyWarning, withProfiles, profileAddressNote, keyItemRow, NO_PROFILE, NIP05_OK,
} from '../ui/people.js';
import { CONFIRMS } from '../ui/confirms.js';

const NPUB = 'npub1abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrs';
const profilesOf = (profile) => profileMap([{ npub: NPUB, picture: null, nip05: null, nip05_ok: false, ...profile }]);

test('resolveBody sends each key one time, 50 keys at most', () => {
  assert.deepEqual(resolveBody(['a', 'a', null, 'b']), { npubs: ['a', 'b'] });
  assert.equal(resolveBody(Array.from({ length: 80 }, (_, i) => `k${i}`)).npubs.length, 50);
});

test('a key row shows the profile name, the short key and NIP-05 checked', () => {
  const row = keyRow(NPUB, profilesOf({ name: 'Mary Kate', nip05_ok: true }), 'Mary Kate');
  assert.equal(row.title, 'Mary Kate');
  assert.equal(row.lines.length, 2);
  assert.equal(row.lines[1], NIP05_OK);
  assert.equal(row.warning, null);
  assert.deepEqual(keyRow(NPUB, profilesOf({ name: 'Mary Kate' }), 'Mary Kate').lines.slice(1), []);
});

test('a key row shows the short key while the profiles load or when they fail', () => {
  const row = keyRow(NPUB, null, 'Mary Kate');
  assert.equal(row.title, NPUB.length > 14 ? row.title : NPUB);
  assert.deepEqual(row.lines, []);
  assert.equal(row.warning, null);
});

test('a key with no profile says so and has no warning', () => {
  const row = keyRow(NPUB, profilesOf({ name: null }), 'Mary Kate');
  assert.deepEqual(row.lines, [NO_PROFILE]);
  assert.equal(row.warning, null);
  assert.equal(NO_PROFILE, 'No Nostr profile found');
});

test('the warning of ADR 0103 item 18 shows for a different profile name only', () => {
  assert.equal(keyWarning(NPUB, profilesOf({ name: 'Someone Else' }), 'Mary Kate'),
    'The profile name is not "Mary Kate". Check this key.');
  assert.equal(keyWarning(NPUB, profilesOf({ name: 'Mary Kate Ultra Official' }), 'mary  kate ultra'), null);
  assert.equal(keyWarning(NPUB, profilesOf({ name: null }), 'Mary Kate'), null);
  assert.equal(keyWarning(NPUB, null, 'Mary Kate'), null);
});

test('Make a Person starts a key with a warning not selected', () => {
  const suggestion = { ...SUGGESTION, credits: [{ name: 'Mary Kate Ultra', href: '', npub: NPUB, same_name: true }] };
  assert.equal(makePersonStart(suggestion).keys[0].checked, true);
  assert.equal(makePersonStart(suggestion, profilesOf({ name: 'Someone Else' })).keys[0].checked, false);
  assert.equal(makePersonStart(suggestion, profilesOf({ name: 'Mary Kate Ultra' })).keys[0].checked, true);
  assert.equal(withProfiles(makePersonStart(suggestion), profilesOf({ name: 'Someone Else' })).keys[0].checked, false);
  assert.equal(keyItemRow(NPUB, profilesOf({ name: 'Someone Else' }), 'Mary Kate').warning !== null, true);
});

test('the LNURL address line names the profile and the key', () => {
  const short = keyRow(NPUB, null).title;
  assert.equal(profileAddressNote(NPUB, profilesOf({ name: 'Mary Kate' })), `From the Nostr profile Mary Kate (${short}) · keysend checked`);
  assert.equal(profileAddressNote(NPUB, null), `From the Nostr profile ${short} · keysend checked`);
  assert.equal(profileAddressNote(NPUB, profilesOf({ name: null })), `From the Nostr profile ${short} · keysend checked`);
});

test('a wallet gives each member, and the menu offers each one', () => {
  const alby = {
    ...MK,
    wallets: [{
      id: 2, key: KEY('02cd'), recipient_type: 'node',
      members: [{ key: KEY('02cd'), recipient_type: 'node' }, { key: KEY('a@getalby.com'), recipient_type: 'lnaddress' }],
    }],
  };
  const groups = walletGroups(alby, [{ addresses: [{ address: '02cd', custom_key: null, custom_value: null, provider: 'Alby', feeds: 2 }] }]);
  assert.deepEqual(groups[0].members.map((m) => m.key.address), ['a@getalby.com', '02cd']);
  assert.equal(groups[0].members[1].member.provider, 'Alby');
  assert.equal(groups[0].members[1].count, 2);
  assert.deepEqual(addressOptions(groups).map(([, text]) => text), ['LNURL address a@getalby.com', 'Alby node']);
});

test('ADR 0102: the person page ends with the delete, the confirmation and the sheet title', () => {
  const sections = pageSequence('person');
  assert.deepEqual(sections[sections.length - 1], { id: 'delete', tier: 'limits' });
  const confirm = CONFIRMS.deletePerson('Mary Kate');
  assert.equal(confirm.title, 'Delete Mary Kate from People?');
  assert.equal(confirm.text, 'Studio deletes this person and its links. Studio changes no feed.');
  assert.equal(confirm.action, 'Delete the Person');
  assert.equal(confirm.role, 'destructive');
  assert.equal(SHEET_TITLES.remove, 'Remove from Feeds');
});

// ---- Add to a person, more from your feeds and merge (task 067e) ----
import {
  personChoices, hasItem, addToRows, addSelectedCount, addItemsBody, addItemsLabel, moreRows, moreItemBody, causeText,
  mergeLinks, mergeResult, mergeBody, mergeTitle, addToTitle,
} from '../ui/people.js';

const KEYP = (address, custom_key = null, custom_value = null) => ({ address, custom_key, custom_value });
const HAVING = {
  id: 7,
  name: 'Sir HeyCitizen',
  credit_names: [{ id: 1, identity: 'i1', name: 'Mary Kate Ultra', href: '', npub: 'npub1a' }],
  keys: [{ id: 2, npub: 'npub1k' }],
  wallets: [{ id: 3, key: KEYP('a@x.example'), recipient_type: 'lnaddress', members: [{ key: KEYP('02ab', 'k', 'v') }] }],
  support_links: [{ id: 4, label: 'Tip', url: 'https://t.example' }],
};

test('personChoices sorts A to Z and leaves one person out', () => {
  const list = [{ id: 2, name: 'beta' }, { id: 1, name: 'Alpha' }, { id: 3, name: 'Gamma' }];
  assert.deepEqual(personChoices(list), [['1', 'Alpha'], ['2', 'beta'], ['3', 'Gamma']]);
  assert.deepEqual(personChoices(list, 2), [['1', 'Alpha'], ['3', 'Gamma']]);
});

test('the rows of Add to a Person give New and Already in, with no check for Already in', () => {
  const state = {
    name: 'Mary Kate Ultra',
    credits: [
      { item: { name: 'Mary Kate Ultra', href: '', npub: 'npub1a', roles: [], feeds: [] }, checked: true },
      { item: { name: 'MK', href: '', npub: '', roles: ['vocals'], feeds: [] }, checked: true },
    ],
    recipients: [{ item: { key: KEYP('02ab', 'k', 'v'), recipient_type: 'node', feeds: [] }, checked: true }],
    keys: [{ item: 'npub1k', checked: true }, { item: 'npub1z', checked: false }],
  };
  const groups = addToRows(state, HAVING);
  const flat = groups.flatMap((g) => g.rows);
  assert.deepEqual(flat.map((r) => [r.group, r.already, r.checked]), [
    ['credits', true, false], ['credits', false, true], ['recipients', true, false], ['keys', true, false], ['keys', false, false],
  ]);
  assert.ok(flat[0].lines.includes('Already in Sir HeyCitizen'));
  assert.ok(flat[1].lines.includes('New'));
  assert.equal(hasItem(HAVING, 'recipients', { key: KEYP('a@x.example') }), true);
  assert.equal(addSelectedCount(state, HAVING), 1);
  assert.deepEqual(addItemsBody(state, HAVING), { credits: [['MK', '', '']], recipients: [], keys: [] });
  assert.equal(addItemsLabel(1), 'Add 1 Item');
  assert.equal(addItemsLabel(3), 'Add 3 Items');
  assert.equal(addToTitle('X'), 'Add to a Person: X');
});

test('the rows of More from your feeds give the item, its feeds and its cause', () => {
  const items = [
    { kind: 'credit', name: 'Hey', href: '', npub: 'npub1k', roles: ['vocals'], feeds: [1, 2], cause: 'same_npub' },
    { kind: 'payee', key: KEYP('a@x.example'), name: 'Hey', recipient_type: 'lnaddress', feeds: [1], cause: 'same_wallet' },
  ];
  const rows = moreRows(items, new Map([[1, 'One'], [2, 'Two']]));
  // With no profile, a credit with a key shows the short key.
  assert.equal(rows[0].title, 'npub1k');
  assert.ok(rows[0].lines.includes('Nostr key in the credit vocals'));
  // With a profile, it shows the profile name and the warning of ADR 0103 item 18.
  const named = moreRows(items, null, new Map([['npub1k', { npub: 'npub1k', name: 'City Radio' }]]), 'Hey')[0];
  assert.equal(named.title, 'City Radio');
  assert.equal(named.warning, 'The profile name is not "Hey". Check this key.');
  // ADR 0156 item 7: a credit in two releases has the title "2 releases".
  assert.equal(moreRows([{ ...items[0], npub: '' }])[0].title, '2 releases');
  assert.ok(rows[0].lines.includes('One, Two'));
  assert.ok(rows[0].lines.includes(causeText('same_npub')));
  assert.equal(rows[1].title, 'LNURL address a@x.example');
  assert.ok(rows[1].lines.includes('The wallet is a wallet of this person'));
  assert.deepEqual(rows[0].body, { credits: [['Hey', '', 'npub1k']], recipients: [], keys: [] });
  assert.deepEqual(moreItemBody(items[1]), { credits: [], recipients: [[KEYP('a@x.example'), 'lnaddress']], keys: [] });
  for (const c of ['same_name', 'same_credit_name', 'same_npub', 'same_wallet']) assert.notEqual(causeText(c), '');
  assert.notEqual(new Set(rows.map((r) => r.key)).size, 1);
});

test('the merge sheet gives the links of the other person and the result line', () => {
  const other = {
    id: 9, name: 'Other', keys: [{ id: 5, npub: 'npub1k' }, { id: 6, npub: 'npub1m' }],
    wallets: [{ id: 8, key: KEYP('b@x.example'), recipient_type: 'lnaddress' }],
    credit_names: [{ id: 10, identity: 'i2', name: 'Vox' }], support_links: [],
  };
  const links = mergeLinks(other);
  assert.deepEqual(links.map((l) => l.lines[0] || 'wallet'), ['wallet', 'Nostr key', 'Nostr key', 'Credit name']);
  assert.deepEqual(mergeLinks(null), []);
  const result = mergeResult(HAVING, other);
  assert.equal(result.title, 'Sir HeyCitizen');
  assert.equal(result.lines[0], '2 Nostr keys · 2 wallets · 2 credit names · 1 support link');
  assert.match(result.lines[1], /stay as in Sir HeyCitizen/);
  assert.deepEqual(mergeBody('9'), { other: 9 });
  assert.equal(mergeTitle('X'), 'Merge with X');
});

// ---- Task 075c: the sheets Check, Correct the Feeds and Add to a Person ----
import {
  NEW_PERSON, checkSheetTitle, checkSheetStart, checkSheetChoices, checkSheetHint, checkSheetPerson, checkSheetLabel, checkSheetReady,
  walletToPersonStart, walletToPersonReady, walletToPersonLabel, correctCount, correctButtonLabel, correctFeedIds,
  correctStart, correctKeyStart, correctOptions, otherPayeesLine, otherWalletCount, correctKeyChoices, correctBody, correctAnySelected, correctRows, correctFindingLines,
} from '../ui/people.js';
import { addWalletSections, walletBodyOf } from '../ui/recipients.js';

const CHECK_SUGGESTION = {
  identity: 'i', name: 'Mary', person_id: null, item_keys: [],
  credits: [{ name: 'Mary', href: '', npub: 'npub1m', roles: ['vocals'], feeds: [1], same_name: true }],
  recipients: [{ key: KEYP('mary@x.example'), recipient_type: 'lnaddress', feeds: [1], same_name: true }],
};
const CHECK_PEOPLE = [{ id: 9, name: 'Mary', credit_names: [], keys: [], wallets: [] }, { id: 4, name: 'Anna', credit_names: [], keys: [], wallets: [] }];

test('the sheet Check starts on the person with the name, else on a new person', () => {
  assert.equal(checkSheetTitle('Mary'), 'Check: Mary');
  assert.equal(checkSheetStart(CHECK_SUGGESTION).personId, NEW_PERSON);
  const match = checkSheetStart({ ...CHECK_SUGGESTION, person_id: 9 });
  assert.equal(match.personId, '9');
  assert.deepEqual(checkSheetChoices(CHECK_PEOPLE), [[NEW_PERSON, 'A new person'], ['4', 'Anna'], ['9', 'Mary']]);
  assert.equal(checkSheetHint({ ...CHECK_SUGGESTION, person_id: 9 }, '9'), 'You have a person with this name. Studio adds the selected items to it.');
  assert.equal(checkSheetHint({ ...CHECK_SUGGESTION, person_id: 9 }, '4'), undefined);
  assert.equal(checkSheetHint(CHECK_SUGGESTION, NEW_PERSON), undefined);
});

test('the primary button of the sheet Check follows the person', () => {
  const fresh = checkSheetStart(CHECK_SUGGESTION);
  assert.equal(checkSheetPerson(fresh, CHECK_PEOPLE), null);
  assert.equal(checkSheetLabel(fresh, null), 'Make the Person');
  assert.equal(checkSheetReady(fresh, null), true);
  assert.equal(checkSheetReady({ ...fresh, name: '  ' }, null), false);
  const match = checkSheetStart({ ...CHECK_SUGGESTION, person_id: 9 });
  const person = checkSheetPerson(match, CHECK_PEOPLE);
  assert.equal(person.name, 'Mary');
  // The credit, the wallet and the key are new: three items.
  assert.equal(checkSheetLabel(match, person), 'Add 3 Items');
  assert.equal(checkSheetReady(match, person), true);
  const one = { ...match, credits: [], keys: [], recipients: match.recipients };
  assert.equal(checkSheetLabel(one, person), 'Add 1 Item');
  assert.equal(checkSheetReady({ ...one, recipients: [] }, person), false);
  // A new person shows no flag on its rows.
  assert.ok(!addToRows(fresh, null).flatMap((g) => g.rows).some((r) => r.lines.includes('New')));
});

const CORRECT_PERSON = {
  id: 7, name: 'HeyCitizen', usual_role: 'Audio Engineer', img: 'https://x.example/a.jpg', href: '',
  keys: [{ npub: 'npub1a' }], wallets: [{ id: 1 }],
};

test('the sheet Correct the Feeds starts with the payees of the name not selected', () => {
  const start = correctStart(CORRECT_PERSON);
  assert.deepEqual(start, { key: true, npub: 'npub1a', payment: true, details: true, role: true, other_payees: false });
  assert.deepEqual(correctOptions(CORRECT_PERSON, start).map((o) => o.label), [
    'Nostr key', 'Payment address', 'Picture and information address', 'Role', 'Replace other addresses named "HeyCitizen"']);
  assert.deepEqual(correctBody(start), { kind: 'correct', key: 'npub1a', payment: true, details: true, role: true, other_payees: false });
  assert.equal(correctBody({ ...start, key: false }).key, null);
  assert.equal(correctAnySelected({ ...start, key: false, payment: false, details: false, role: false }), false);
});

test('the sheet Correct the Feeds leaves out a change that the person cannot give, and has a menu for two keys', () => {
  const bare = { ...CORRECT_PERSON, keys: [], wallets: [] };
  assert.deepEqual(correctOptions(bare, correctStart(bare)).map((o) => o.id), ['details', 'role']);
  assert.equal(correctStart(bare).key, false);
  assert.deepEqual(correctKeyChoices(CORRECT_PERSON), []);
  assert.deepEqual(correctKeyChoices({ ...CORRECT_PERSON, keys: [{ npub: 'npub1a' }, { npub: 'npub1b' }] }, new Map([['npub1b', { name: 'City Radio' }]])).map((c) => c[1]),
    ['npub1a', 'City Radio']);
});

test('the count and the feeds of the correction come from the check', () => {
  const check = { feeds: [
    { feed_id: 1, state: 'needs_correction' }, { feed_id: 2, state: 'not_default' }, { feed_id: 3, state: 'correct' },
  ] };
  assert.equal(correctCount(check), 2);
  assert.deepEqual(correctFeedIds(check), [1, 2]);
  assert.equal(correctButtonLabel(1), 'Correct 1 Feed');
  assert.equal(correctButtonLabel(3), 'Correct 3 Feeds');
});

test('the rows of the correction leave out each feed that the core does not change', () => {
  const changed = { outcome: 'changed', channel: false, items: 2, live_items: 0 };
  const none = (cause) => ({ outcome: 'not_changed', cause: { cause } });
  const preview = { feeds: [
    { feed_id: 1, outcome: changed }, { feed_id: 2, outcome: none('no_credit') }, { feed_id: 3, outcome: none('no_recipient') },
    { feed_id: 4, outcome: none('has_new_address') }, { feed_id: 5, outcome: changed },
  ] };
  const places = new Map([[1, { title: 'One', lines: ['Album', 'Credited as vocals'] }]]);
  const { rows, omitted } = correctRows(preview, new Map([[5, 'Five']]), places);
  assert.deepEqual(rows, [
    { id: 1, title: 'One', checked: true, lines: ['Album · 2 tracks'] },
    { id: 5, title: 'Five', checked: true, lines: ['2 tracks'] },
  ]);
  assert.equal(omitted, 3);
  // A new preview keeps the selection of the user.
  const again = correctRows(preview, new Map(), places, [{ id: 1, checked: false }]);
  assert.equal(again.rows[0].checked, false);
});

test('the change line of a correction names the selected changes', () => {
  const op = { kind: 'correct', npub: 'npub1a', payment: true, other_payees: false, img: 'x', href: null, role: null };
  assert.equal(changeLine({ operation: op, feeds: [{}, {}] }, 'Hey'), 'Correct the Nostr key, the payment address and the picture and the information address of Hey in 2 feeds');
});

test('the sheet Add to a Person of a payee starts on a new person with the name of the payee', () => {
  const start = walletToPersonStart('DuhLaurien');
  assert.deepEqual(start, { personId: NEW_PERSON, name: 'DuhLaurien' });
  assert.equal(walletToPersonLabel(start), 'Make the Person');
  assert.equal(walletToPersonReady({ ...start, name: ' ' }), false);
  assert.equal(walletToPersonLabel({ ...start, personId: '4' }), 'Add the Wallet');
  assert.equal(walletToPersonReady({ ...start, personId: '4', name: '' }), true);
});

test('the sheet Add a Wallet has its form first, then the payees with no person', () => {
  const recipients = [
    { address: 'saved@x.example', recipient_type: 'lnaddress', saved_id: 5, feeds: 1 },
    { address: 'duh@x.example', name: 'DuhLaurien', recipient_type: 'lnaddress', saved_id: null, feeds: 1 },
  ];
  const sections = addWalletSections(recipients);
  assert.deepEqual(sections.map((s) => s.id), ['different', 'payees']);
  assert.equal(sections[0].title, '');
  assert.deepEqual(sections[1].rows.map((r) => r.title), ['DuhLaurien']);
  assert.deepEqual(sections[1].rows[0].body, { address: 'duh@x.example', custom_key: null, custom_value: null, recipient_type: 'lnaddress' });
  assert.deepEqual(addWalletSections([]).map((s) => s.id), ['different']);
  assert.equal(walletBodyOf({ address: 'a', addresses: [{ address: '02ab', recipient_type: 'node' }, { address: 'a@x.example', recipient_type: 'lnaddress' }] }).address, 'a@x.example');
});

test('a feed row of Correct the Feeds lists only the findings that the selected changes correct (ADR 0111 item 14)', () => {
  const entry = { feed_id: 1, state: 'needs_correction', findings: [
    { kind: 'credit_key', role: 'audio engineer', npub: '' },
    { kind: 'credit_role' },
    { kind: 'other_wallet', key: { address: 'a@b.example' } },
  ] };
  assert.deepEqual(correctFindingLines(entry, { key: true, role: false, other_payees: false }, 'Ann'), ['No Nostr key in the credit'],
    'ADR 0111 item 14: a row lists the findings of the selected changes only. Check CORRECTED_BY in people.js.');
  assert.equal(correctFindingLines(entry, { key: true, role: true, other_payees: true }, 'Ann').length, 3);
  assert.deepEqual(correctFindingLines(undefined, { key: true }, 'Ann'), []);
});

test('ADR 0113: the list of suggestions is open, collapsed or none', async () => {
  const { detailList } = await import('../ui/people.js');
  const detail = { img: [{ value: 'https://a.example/1.jpg', sources: [{ kind: 'profile', npub: 'npub1abcdefghijklmnop' }] }, { value: 'https://a.example/2.jpg', sources: [] }], href: [] };
  const open = detailList('img', '  ', detail, null, new Map(), []);
  assert.equal(open.state, 'open');
  assert.equal(open.heading, 'Suggestions for the picture');
  assert.equal(open.rows.length, 2);
  const collapsed = detailList('img', 'https://x.example/p.jpg', detail, null, new Map(), []);
  assert.equal(collapsed.state, 'collapsed');
  assert.equal(collapsed.collapsedLine, '2 other values in your feeds and profiles');
  assert.equal(detailList('img', 'x', { img: [detail.img[0]] }, null, new Map(), []).collapsedLine, '1 other value in your feeds and profiles');
  const none = detailList('href', '', detail, null, new Map(), []);
  assert.equal(none.state, 'none');
  assert.equal(none.heading, 'Suggestions for the information address');
  assert.equal(detailList('href', '', null, null, new Map(), []).state, 'none');
});

test('ADR 0113: the source lines name each source and join with a dot', async () => {
  const { detailSourceLine, profileMap } = await import('../ui/people.js');
  const titles = new Map([[1, 'Night Signals'], [2, 'Low Tide'], [9, 'HeyCitizen']]);
  const feeds = [{ id: 9, title: 'HeyCitizen', medium: 'publisher', publisher_roles: ['artist'] }];
  const profiles = profileMap([{ npub: 'npub1named', name: 'Hey Citizen' }]);
  assert.equal(detailSourceLine([{ kind: 'profile', npub: 'npub1named' }], 'img', profiles, titles, feeds), 'From the Nostr profile Hey Citizen');
  assert.match(detailSourceLine([{ kind: 'profile', npub: 'npub1abcdefghijklmnopqrstuvwxyz' }], 'img', null, titles, feeds), /^From the Nostr profile npub1/);
  assert.equal(detailSourceLine([{ kind: 'credit', feed_ids: [1, 2] }], 'img', null, titles, feeds), 'In the credits of 2 feeds: Night Signals, Low Tide');
  assert.equal(detailSourceLine([{ kind: 'credit', feed_ids: [1] }], 'href', null, titles, feeds), 'In the credit of 1 feed: Night Signals');
  assert.equal(detailSourceLine([{ kind: 'member', feed_id: 9 }], 'img', null, titles, feeds), 'The image of the artist feed HeyCitizen');
  assert.equal(detailSourceLine([{ kind: 'member', feed_id: 9 }], 'href', null, titles, feeds), 'The link of the artist feed HeyCitizen');
  assert.equal(detailSourceLine([{ kind: 'profile', npub: 'npub1named' }, { kind: 'member', feed_id: 9 }], 'href', profiles, titles, feeds),
    'From the Nostr profile Hey Citizen · the link of the artist feed HeyCitizen');
});

test('the change of the payees of the name says what it replaces, and with which address (ADR 0111 item 15)', () => {
  const key = { address: 'heycitizen@getalby.com' };
  assert.equal(otherPayeesLine('HeyCitizen', key, 2),
    '2 payments to the name "HeyCitizen" go to an address that is not in this person. Studio replaces them with heycitizen@getalby.com. Select this only when that address is yours too.');
  assert.match(otherPayeesLine('HeyCitizen', key, 1), /^1 payment to the name "HeyCitizen" goes to .* Studio replaces it with heycitizen@getalby.com\./);
  assert.match(otherPayeesLine('HeyCitizen', null, 0), /^A payment to the name .* with the default address\./);
  const check = { feeds: [{ findings: [{ kind: 'other_wallet' }, { kind: 'credit_role' }] }, { findings: [{ kind: 'other_wallet' }] }] };
  assert.equal(otherWalletCount(check), 2);
  assert.equal(otherWalletCount(null), 0);
});

test('the key of Correct the Feeds is never a key with the warning by default (ADR 0103 item 18)', () => {
  const person = { ...CORRECT_PERSON, keys: [{ npub: 'npub1radio' }, { npub: 'npub1mine' }] };
  const profiles = new Map([['npub1radio', { name: 'City Radio' }], ['npub1mine', { name: 'Sir HeyCitizen' }]]);
  const start = correctStart(person);
  assert.equal(correctKeyStart(person, start, profiles).npub, 'npub1mine',
    'ADR 0103 item 18: the sheet must start on a key with no warning. Check correctKeyStart.');
  const only = { ...CORRECT_PERSON, keys: [{ npub: 'npub1radio' }] };
  const fields = correctKeyStart(only, correctStart(only), profiles);
  assert.equal(fields.key, false, 'A person with only keys of other accounts must not write a key by default.');
  const line = correctOptions(only, { ...fields, key: true }, profiles).find((o) => o.id === 'key').line;
  assert.match(line, /City Radio\. The profile name is not "HeyCitizen"\. Check this key\./);
});

// ---- Task 080b: Find on musicindex (ADR 0117) ----

const ROW = (kind, extra = {}) => ({
  feed_guid: 'g1', title: 'DuhLaurien', image_url: null, level: 'role', kind, releases: 3, host: 'wavlake.com',
  has: { picture: true, website: true, wallet: true, npub: false }, ...extra,
});

test('the dot and the type line of each kind of row (ADR 0117 item 4)', () => {
  assert.deepEqual(lookupKind(ROW({ type: 'role', roles: ['artist'] })), { dot: 'artist', text: 'Artist · 3 releases' });
  assert.deepEqual(lookupKind(ROW({ type: 'role', roles: ['label'] }, { releases: 1 })), { dot: 'label', text: 'Label · 1 release' });
  assert.deepEqual(lookupKind(ROW({ type: 'role', roles: ['label'] }, { releases: 2 })), { dot: 'label', text: 'Label · 2 releases' });
  assert.deepEqual(lookupKind(ROW({ type: 'inferred_artist' }, { releases: 1 })), { dot: 'artist hollow', text: 'Inferred artist · 1 release' });
  assert.deepEqual(lookupKind(ROW({ type: 'no_role' }, { level: 'two_way', releases: 2 })), { dot: 'feed hollow', text: 'Feed · two-way links, no role' });
  assert.deepEqual(lookupKind(ROW({ type: 'unverified' }, { level: 'name_only', releases: null })), { dot: 'feed hollow', text: 'Unverified link' });
});

test('the groups of a search keep the three levels in sequence and leave out an empty level (ADR 0117 item 4)', () => {
  assert.deepEqual(LOOKUP_LEVELS.map((l) => l.heading), ['Verified role', 'Verified two-way links', 'Name only']);
  const rows = [
    ROW({ type: 'unverified' }, { feed_guid: 'c', level: 'name_only' }),
    ROW({ type: 'role', roles: ['artist'] }, { feed_guid: 'a' }),
    ROW({ type: 'no_role' }, { feed_guid: 'b', level: 'two_way' }),
    ROW({ type: 'role', roles: ['label'] }, { feed_guid: 'd' }),
  ];
  const groups = lookupGroups(rows);
  assert.deepEqual(groups.map((g) => g.heading), ['Verified role', 'Verified two-way links', 'Name only']);
  assert.deepEqual(groups[0].rows.map((r) => r.feed_guid), ['a', 'd'], 'ADR 0117 item 4: a level keeps the sequence of the search.');
  assert.deepEqual(lookupGroups([rows[1]]).map((g) => g.heading), ['Verified role']);
  assert.deepEqual(lookupGroups([]), []);
});

test('the host and the values of a row, or null with no detail (ADR 0117 item 7)', () => {
  assert.equal(hasLine(ROW({ type: 'role', roles: ['artist'] })), 'wavlake.com · a picture, a website and a wallet');
  assert.equal(hasLine(ROW({ type: 'role', roles: ['artist'] }, { has: { picture: true, website: false, wallet: false, npub: false } })), 'wavlake.com · a picture');
  assert.equal(hasLine(ROW({ type: 'role', roles: ['artist'] }, { host: null, has: { picture: false, website: false, wallet: false, npub: false } })), null);
  assert.equal(hasLine(ROW({ type: 'role', roles: ['artist'] }, { has: { picture: false, website: false, wallet: false, npub: false } })), 'wavlake.com');
});

const RAW = (state = 'new') => ({ value: 'v', state });
const PENDING = savedValues(MK);

test('the start values and the rows of a fill (ADR 0117 items 9 to 12)', () => {
  const values = {
    title: 'DuhLaurien',
    picture: { value: 'https://x.example/p.jpg', state: 'new' },
    website: { value: 'https://duh.example', state: 'current' },
    wallets: [
      { value: 'duh@x.example', name: null, recipient_type: 'lnaddress', split: '100', lookup: 'found', key: KEY('duh@x.example'), state: 'new' },
      { value: '02ff', name: null, recipient_type: 'node', split: '5', lookup: null, key: KEY('02ff'), state: 'has_wallet' },
      { value: '03ee', name: null, recipient_type: 'node', split: '1', lookup: null, key: KEY('03ee'), state: { other_person: { id: 9, name: 'Ann' } } },
      { value: 'zz', name: null, recipient_type: 'node', split: null, lookup: null, key: null, state: 'new' },
    ],
    npub: null,
  };
  const rows = fillRows(values, PENDING);
  const byId = Object.fromEntries(rows.map((r) => [r.id, r]));
  assert.equal(byId.img.selected, true, 'ADR 0117 item 11: the picture starts selected when its field is empty.');
  assert.equal(byId.href.selectable, false, 'ADR 0117 item 12: the current value has no check box.');
  assert.deepEqual(byId.href.lines, ['Already the value of this person']);
  assert.equal(byId['wallet:0'].selectable, true);
  assert.equal(byId['wallet:0'].selected, false, 'ADR 0117 item 11: a wallet starts not selected.');
  assert.deepEqual(byId['wallet:0'].lines.slice(0, 3), ['LNURL address', 'The feed gives no name for this recipient.', 'The value block of the feed · 100%']);
  const named = fillRows({ ...values, wallets: [{ ...values.wallets[0], name: 'Music Side Project' }] }, PENDING);
  assert.ok(named.find((r) => r.id === 'wallet:0').lines.includes('Paid to: Music Side Project'),
    'ADR 0117 item 9: a wallet row names its recipient. The human test of 0.1.0-rc.19 found a row with no clue of who the recipient was.');
  assert.ok(byId['wallet:0'].lines.includes('Checked with keysend'));
  assert.ok(byId['wallet:0'].lines.includes('A feed can pay a band member, a collaborator or an app. Add this wallet only when this recipient is this person.'));
  assert.equal(byId['wallet:1'].selectable, false);
  assert.ok(byId['wallet:1'].lines.includes('Already a wallet of this person'));
  assert.equal(byId['wallet:2'].selectable, false);
  assert.ok(byId['wallet:2'].lines.includes('A wallet of Ann'));
  assert.equal(byId['wallet:3'].selectable, false, 'A wallet with no key cannot be selected.');
  assert.equal(byId.npub.selectable, false, 'A feed with no Nostr key has no check box.');
  assert.deepEqual(byId.npub.lines, ['This feed gives no Nostr key. Use “Add a Nostr Key” to search the Nostr profiles.']);
  const filled = fillRows({ ...values, picture: { value: 'https://x.example/p.jpg', state: 'new' } }, { ...PENDING, img: 'https://old.example/p.jpg' });
  assert.equal(filled.find((r) => r.id === 'img').selected, false, 'A picture starts not selected when its field has a value.');
});

test('a value that is not an http or https address is not in the rows (ADR 0117 item 10)', () => {
  const rows = fillRows({ picture: { value: 'ftp://x.example/p.jpg', state: 'new' }, website: { value: 'javascript:1', state: 'new' }, wallets: [], npub: null }, PENDING);
  assert.deepEqual(rows.map((r) => r.id), ['npub']);
});

test('a Nostr key of a feed has a check box and starts not selected (ADR 0117 items 9 and 11)', () => {
  const rows = fillRows({ wallets: [], npub: { value: 'npub1z', state: 'new' } }, PENDING);
  assert.equal(rows[0].selectable, true);
  assert.equal(rows[0].selected, false);
  assert.deepEqual(rows[0].lines, []);
  const other = fillRows({ wallets: [], npub: { value: 'npub1z', state: { other_person: { id: 9, name: 'Ann' } } } }, PENDING);
  assert.equal(other[0].selectable, false, 'A Nostr key of a different person has no check box, as a wallet of a different person.');
  assert.deepEqual(other[0].lines, ['A Nostr key of Ann']);
});

test('the label of the Fill button for the number of fields (ADR 0117 item 13)', () => {
  assert.equal(fillLabel(1), 'Fill 1 Field');
  assert.equal(fillLabel(0), 'Fill 0 Fields');
  assert.equal(fillLabel(2), 'Fill 2 Fields');
});

test('a fill makes pending changes, and the save sends the added wallets and keys (ADR 0116, ADR 0117 item 14)', () => {
  const values = {
    picture: { value: 'https://x.example/p.jpg', state: 'new' },
    website: { value: 'https://duh.example', state: 'new' },
    wallets: [{ value: 'duh@x.example', name: null, recipient_type: 'lnaddress', split: '100', lookup: 'found', key: KEY('duh@x.example'), state: 'new' }],
    npub: { value: 'npub1z', state: 'new' },
  };
  const rows = fillRows(values, PENDING);
  const patch = fillPatch(rows, ['img', 'href', 'wallet:0', 'npub'], PENDING);
  assert.deepEqual(patch, {
    img: 'https://x.example/p.jpg', href: 'https://duh.example',
    wallets: [{ key: KEY('duh@x.example'), recipient_type: 'lnaddress' }], keys: ['npub1z'],
  });
  const values2 = { ...PENDING, ...patch };
  assert.deepEqual(pendingKeys(PENDING, values2), ['img', 'href', 'wallet', 'npub']);
  assert.equal(pendingStatus(pendingKeys(PENDING, values2).length), '4 changes not saved');
  assert.deepEqual(saveBody(PENDING, values2), {
    fields: { name: 'Mary Kate', usual_role: 'Vocals', usual_group: '', img: 'https://x.example/p.jpg', href: 'https://duh.example' },
    wallets: [{ key: KEY('duh@x.example'), recipient_type: 'lnaddress' }],
    keys: ['npub1z'],
  });
  assert.equal(leaveText('Mary Kate', pendingKeys(PENDING, values2)),
    'Mary Kate has 4 changes that are not saved: the picture address, the information address, a wallet and a Nostr key.');
  assert.deepEqual(fillPatch(rows, ['wallet:0'], values2), {}, 'A wallet that the page has already is not added twice.');
  assert.deepEqual(fillPatch(rows, [], PENDING), {});
});

test('a fill sends no request (ADR 0117 item 15)', () => {
  const real = globalThis.fetch;
  globalThis.fetch = () => { throw new Error('a fill must not send a request'); };
  try {
    fillPatch(fillRows({ picture: { value: 'https://x.example/p.jpg', state: 'new' }, wallets: [], npub: null }, PENDING), ['img'], PENDING);
  } finally {
    globalThis.fetch = real;
  }
});

test('the source line of a filled field names the kind of the row (ADR 0117 item 14)', () => {
  assert.equal(fillSource({ title: 'DuhLaurien', kind: { type: 'role', roles: ['artist'] } }), 'From Music Index: the artist feed DuhLaurien');
  assert.equal(fillSource({ title: 'Duh Trio', kind: { type: 'inferred_artist' } }), 'From Music Index: the artist feed Duh Trio');
  assert.equal(fillSource({ title: null, kind: { type: 'no_role' } }), 'From Music Index: the feed Untitled feed');
  assert.equal(musicindexFeedUrl('a b'), 'https://musicindex.org/#feed=a%20b');
});


// The credit of a member (ADR 0118 items 2, 4, 6, 9 and 10, task 081b).
import {
  creditOffers, creditAddTitle, creditRemoveTitle, creditAddNote, creditPublicNote, creditRemoveNote, creditOperation,
  REMOVE_CREDIT_OPERATION, feedWord, memberSectionNote, memberRowLine, LINK_HINT, LINK_EMPTY, linkChoices, linkRows,
  creditPosition, afterCredits,
} from '../ui/people.js';

const LABEL = { id: 3, medium: 'publisher', title: 'Band', connector_id: 'local', publisher_roles: ['artist'] };
const READ_ONLY = { id: 4, medium: 'publisher', title: 'Old Band', connector_id: null, publisher_roles: ['artist'] };
const OTHER = { id: 5, medium: 'publisher', title: 'Other Band', connector_id: 'local', publisher_roles: ['label'] };
const FEEDS = [LABEL, READ_ONLY, OTHER];

// The members of each feed: the person 7 with the membership, the credit, or both.
const members = (rows) => new Map(rows);
const PERSON = 7;

test('creditOffers gives an add for a new membership with no credit', () => {
  const m = members([['3', [{ person_id: PERSON, in_credits: false }]]]);
  assert.deepEqual(creditOffers([], ['3'], FEEDS, m, PERSON), [{ kind: 'add', feedId: '3' }]);
});

test('creditOffers gives a remove for a removed membership with a credit', () => {
  const m = members([['3', [{ person_id: PERSON, in_credits: true }]]]);
  assert.deepEqual(creditOffers(['3'], [], FEEDS, m, PERSON), [{ kind: 'remove', feedId: '3' }]);
});

test('creditOffers gives the remove first, then the add, when the feed changes', () => {
  const m = members([
    ['3', [{ person_id: PERSON, in_credits: true }]],
    ['5', [{ person_id: PERSON, in_credits: false }]],
  ]);
  assert.deepEqual(creditOffers(['3'], ['5'], FEEDS, m, PERSON), [
    { kind: 'remove', feedId: '3' },
    { kind: 'add', feedId: '5' },
  ]);
});

test('creditOffers gives no offer for a read-only feed', () => {
  const m = members([['4', [{ person_id: PERSON, in_credits: false }]]]);
  assert.deepEqual(creditOffers([], ['4'], FEEDS, m, PERSON), []);
});

test('creditOffers gives no add when the feed has the credit of the person', () => {
  const m = members([['3', [{ person_id: PERSON, in_credits: true }]]]);
  assert.deepEqual(creditOffers([], ['3'], FEEDS, m, PERSON), []);
});

test('creditOffers gives no offer when the membership does not change', () => {
  const m = members([['3', [{ person_id: PERSON, in_credits: true }]]]);
  assert.deepEqual(creditOffers(['3'], ['3'], FEEDS, m, PERSON), []);
  assert.deepEqual(creditOffers([], [], FEEDS, m, PERSON), []);
});

test('creditOffers ignores the rows of other persons', () => {
  const m = members([['3', [{ person_id: 99, in_credits: true }]]]);
  assert.deepEqual(creditOffers(['3'], [], FEEDS, m, PERSON), []);
});

test('the texts of the credit sheets and of the links (ADR 0132 items 2 and 10)', () => {
  assert.equal(LINK_HINT, 'Add the details of this person to your artist or label page. After Save, you can add a credit to each new feed. A read-only link shows the changes of a page on Music Index here. The links stay in Studio.');
  assert.equal(creditAddTitle('Sam', 'Band'), 'Add Sam to the Credits of Band?');
  assert.equal(creditRemoveTitle('Sam', 'Band'), 'Remove Sam from the Credits of Band?');
  assert.equal(creditAddNote('Sam', 'Band'), 'Sam is part of Band in Studio. The credits of the feed can also show it, so that apps and musicindex.org can find Sam.');
  assert.equal(creditPublicNote(false), 'The feed is public. The name, the picture and the information address go in it.');
  assert.equal(creditPublicNote(true), 'The feed is public. The name, the picture, the information address and the Nostr key go in it.');
  assert.equal(creditRemoveNote('Sam'), 'The feed then does not name Sam. Studio shows the review first.');
  assert.equal(memberSectionNote('artist'), 'The people that are part of this artist. A person that is not in the credits is only in Studio.');
});

test('the operations of a credit and its removal', () => {
  assert.deepEqual(creditOperation('Fiddle', 'Performer'), { kind: 'credit_feed', role: 'Fiddle', group: 'Performer' });
  assert.deepEqual(REMOVE_CREDIT_OPERATION, { kind: 'remove', payee: false, credit: true });
});

test('feedWord and memberRowLine give the words of the members list', () => {
  assert.equal(feedWord(LABEL), 'artist');
  assert.equal(feedWord({ medium: 'publisher', publisher_roles: [] }), 'feed');
  assert.equal(memberRowLine({ usual_role: 'Fiddle', in_credits: true }), 'Fiddle · In the credits of this feed');
  assert.equal(memberRowLine({ usual_role: '', in_credits: false }), 'Only in Studio');
  assert.equal(memberRowLine({ usual_role: 'Vocals', in_credits: false }), 'Vocals · Only in Studio');
});

// ---- Task 084b: the entries of the check (ADR 0120) ----
import {
  releaseWord, checkSummary, placeText, differenceBlocks, openLabel, useValuesLabel,
} from '../ui/people.js';

test('the word of a release and the summary of an entry (ADR 0120 items 1 and 4)', () => {
  assert.equal(releaseWord({ medium: 'music' }), 'album');
  assert.equal(releaseWord({ medium: 'musicl' }), 'playlist');
  assert.equal(releaseWord({ medium: 'publisher', publisher_roles: ['artist'] }), 'artist page');
  assert.equal(releaseWord({ medium: 'publisher', publisher_roles: ['label'] }), 'label page');
  const album = { medium: 'music' };
  const two = { findings: [{ kind: 'credit_details' }, { kind: 'credit_key' }] };
  assert.equal(checkSummary(two, album), 'Album · 2 credits differ');
  assert.equal(checkSummary({ findings: [{ kind: 'credit_role' }] }, { medium: 'musicl' }), 'Playlist · 1 credit differs');
  assert.equal(checkSummary({ findings: [{ kind: 'not_default' }] }, { medium: 'publisher', publisher_roles: ['artist'] }), 'Artist page · 1 payment differs');
  assert.equal(checkSummary({ findings: [{ kind: 'credit_role' }, { kind: 'other_wallet' }] }, { medium: 'publisher', publisher_roles: ['label'] }),
    'Label page · 1 credit and 1 payment differ');
  assert.equal(openLabel({ medium: 'music' }), 'Open the Album');
  assert.equal(openLabel({ medium: 'publisher', publisher_roles: ['label'] }), 'Open the Label Page');
  assert.equal(useValuesLabel('DuhLaurien'), "Use DuhLaurien's Values");
});

test('the position of a credit: the channel, two items, five items (ADR 0120 item 2)', () => {
  assert.equal(placeText([{ kind: 'channel' }]), 'Channel');
  const item = (title) => ({ kind: 'item', guid: title, title });
  assert.equal(placeText([item('Neon Diner'), item('Night Bus')]), 'Neon Diner, Night Bus');
  assert.equal(placeText([item('A'), item('B'), item('C')]), 'A, B, C');
  assert.equal(placeText([item('A'), item('B'), item('C'), item('D'), item('E')]), 'A, B and 3 more');
  assert.equal(placeText([{ kind: 'item', guid: '', title: '' }]), 'Untitled item');
});

test('the blocks of a credit with a picture and an information address (ADR 0120 item 2)', () => {
  const entry = { findings: [{
    kind: 'credit_details', role: 'vocals', img: false, href: true, feed_img: '', feed_href: '', person_img: '',
    person_href: 'https://duhlaurien.example', places: [{ kind: 'item', guid: 'g3', title: 'Neon Diner' }],
  }] };
  const blocks = differenceBlocks(entry, 'DuhLaurien', { medium: 'music' });
  assert.deepEqual(blocks, [{
    place: 'Neon Diner',
    what: 'Credit · Vocals · information address',
    lines: [
      { label: 'In the album', value: 'None', old: true },
      { label: 'DuhLaurien', value: 'https://duhlaurien.example', old: false },
    ],
  }]);
  const both = { findings: [{ ...entry.findings[0], img: true, feed_img: 'https://old.example/a.jpg', person_img: 'https://a.example/a.jpg' }] };
  const two = differenceBlocks(both, 'DuhLaurien', { medium: 'musicl' });
  assert.deepEqual(two.map((b) => b.what), ['Credit · Vocals · picture', 'Credit · Vocals · information address']);
  assert.equal(two[0].lines[0].label, 'In the playlist');
  assert.equal(two[0].lines[0].value, 'https://old.example/a.jpg');
});

test('the blocks of a credit key and a credit role, with None for a missing value (ADR 0120 item 2)', () => {
  const profiles = profilesOf({ name: 'City Radio' });
  const entry = { findings: [
    { kind: 'credit_key', role: 'host', group: 'person', npub: '', places: [{ kind: 'channel' }] },
    { kind: 'credit_role', places: [{ kind: 'channel' }] },
  ] };
  const [key, role] = differenceBlocks(entry, 'Ann', { medium: 'music' }, profiles, { keys: [{ npub: NPUB }], usual_role: '' });
  assert.deepEqual(key, {
    place: 'Channel',
    what: 'Credit · Host · Nostr key',
    lines: [
      { label: 'In the album', value: 'None', old: true },
      { label: 'Ann', value: 'City Radio', old: false },
    ],
  });
  assert.deepEqual(role.lines, [
    { label: 'In the album', value: 'None', old: true },
    { label: 'Ann', value: 'None', old: false },
  ]);
  assert.equal(role.what, 'Credit · No role');
});

test('the blocks of a payment: the address of the feed and the default address (ADR 0120 item 2)', () => {
  const key = KEY('mk@x.example');
  const blocks = differenceBlocks({ findings: [{ kind: 'other_wallet', key }] }, 'Ann', { medium: 'music' }, null, { default_address: null });
  assert.deepEqual(blocks, [{
    place: 'Payments',
    what: 'Payment · not a wallet of Ann',
    lines: [
      { label: 'In the album', value: 'LNURL address mk@x.example', old: true },
      { label: 'Ann', value: 'None', old: false },
    ],
  }]);
});

// ---- ADR 0132: a person can be part of many artists and labels ----

const LINK_FEEDS = [
  { id: 1, medium: 'publisher', title: 'Moss Choir', publisher_roles: ['artist'], connector_id: 1 },
  { id: 2, medium: 'publisher', title: 'Aurora Lane', publisher_roles: ['artist'], connector_id: 1 },
  { id: 3, medium: 'publisher', title: 'Cloud Host', publisher_roles: ['hosting'], connector_id: 1 },
  { id: 4, medium: 'publisher', title: 'Night Owl', publisher_roles: ['label', 'hosting'], connector_id: 1 },
  { id: 5, medium: 'publisher', title: 'Untagged', publisher_roles: [], connector_id: 1 },
  { id: 6, medium: 'music', title: 'An Album', connector_id: 1 },
];

test('ADR 0132 item 6: the menu leaves out a link and a hosting feed, and keeps a feed with no role', () => {
  assert.deepEqual(linkChoices(LINK_FEEDS, ['1']), [['2', 'Aurora Lane · Artist'], ['4', 'Night Owl · Label'], ['5', 'Untagged']]);
  assert.equal(linkChoices(LINK_FEEDS).length, 4);
});

test('ADR 0132 items 3 and 4: the rows are A to Z, each with its condition', () => {
  const saved = { members: ['1', '2'] };
  const rows = linkRows({ members: ['1', '2', '4'] }, saved, LINK_FEEDS, [2]);
  assert.deepEqual(rows.map((r) => [r.id, r.title, r.line]), [
    ['2', 'Aurora Lane', 'Artist · In the credits of this feed'],
    ['1', 'Moss Choir', 'Artist · Only in Studio'],
    ['4', 'Night Owl', 'Label · Not saved yet'],
  ]);
  assert.equal(linkRows({ members: ['5'] }, { members: ['5'] }, LINK_FEEDS)[0].line, 'Only in Studio');
  assert.equal(LINK_EMPTY, 'This person is not linked to an artist or label page yet.');
});

test('ADR 0132 item 8: each added and each removed link is one change, and Save sends the list', () => {
  const saved = savedValues({ ...MK, members: [1, 2] });
  assert.deepEqual(saved.members, ['1', '2']);
  const values = { ...saved, members: ['2', '4', '5'] };
  assert.deepEqual(pendingKeys(saved, values), ['member', 'member', 'member']);
  assert.equal(isPending(saved, values, 'members'), true);
  assert.deepEqual(saveBody(saved, values), { members: [2, 4, 5] });
  // The same links in a different sequence are no change.
  assert.deepEqual(pendingKeys(saved, { ...saved, members: ['2', '1'] }), []);
  assert.equal(saveBody(saved, { ...saved, members: ['2', '1'] }), null);
});

test('ADR 0132 item 11: one sheet for each feed, the removals first, each with its position', () => {
  const m = new Map([
    ['1', [{ person_id: 7, in_credits: true }]],
    ['2', [{ person_id: 7, in_credits: true }]],
    ['4', [{ person_id: 7, in_credits: false }]],
    ['5', [{ person_id: 7, in_credits: false }]],
  ]);
  assert.deepEqual(creditOffers(['1', '2'], ['4', '5'], LINK_FEEDS, m, 7), [
    { kind: 'remove', feedId: '1' }, { kind: 'remove', feedId: '2' },
    { kind: 'add', feedId: '4' }, { kind: 'add', feedId: '5' },
  ]);
  assert.deepEqual(creditOffers(['1'], ['1', '4'], LINK_FEEDS, m, 7), [{ kind: 'add', feedId: '4' }]);
  assert.equal(creditPosition(0, 1), '');
  assert.equal(creditPosition(0, 2), '1 of 2.');
  assert.equal(creditPosition(1, 2), '2 of 2.');
});

test('changeSetView gives a move the area of File hosts (ADR 0148 item 17)', () => {
  assert.deepEqual(changeSetView({ id: 4, operation: { kind: 'move_releases' } }), { name: 'change-set', id: 4, area: 'hosts' });
  assert.deepEqual(changeSetView({ id: 5, operation: { kind: 'rename' } }), { name: 'change-set', id: 5 });
});

test('ADR 0132 item 12: the view after the last sheet', () => {
  assert.equal(afterCredits([]), null);
  assert.deepEqual(afterCredits([8]), { name: 'change-set', id: 8 });
  assert.deepEqual(afterCredits([8, 9]), { name: 'people' });
});

test('ADR 0132 item 10: the UI has no text "Member of"', () => {
  const dir = new URL('../ui/', import.meta.url);
  for (const name of readdirSync(dir).filter((f) => f.endsWith('.js'))) {
    assert.equal(readFileSync(new URL(name, dir), 'utf8').includes('Member of'), false,
      `ADR 0132 item 10: ${name} has "Member of". Use the words "Part of".`);
  }
});

test('nostrPickRows gives one row for each key and leaves out the key of the field', () => {
  const people = [{ id: 1, name: 'Mary Kate', keys: [{ npub: 'npub1a' }, { npub: 'npub1b' }] }, { id: 2, name: 'Jo', keys: [] }];
  assert.deepEqual(nostrPickRows(people, null).map((r) => [r.name, r.npub]), [['Mary Kate', 'npub1a'], ['Mary Kate', 'npub1b']]);
  assert.deepEqual(nostrPickRows(people, 'npub1a').map((r) => r.npub), ['npub1b']);
  assert.deepEqual(nostrPickRows(null, null), []);
});

test('a row with a key warning adds only with the label Add Anyway', async () => {
  const { moreAddLabel } = await import('../ui/people.js');
  assert.equal(moreAddLabel({ warning: 'The profile name is not "Ann". Check this key.' }), 'Add Anyway');
  assert.equal(moreAddLabel({ warning: null }), 'Add');
  assert.equal(moreAddLabel({}), 'Add');
});

test('a feed row names each role of a credit on one line', async () => {
  const { sheetFeeds } = await import('../ui/people.js');
  const rows = (roles) => sheetFeeds([{ feed_id: 1, roles }], null)[0].lines;
  assert.deepEqual(rows(['composer', 'guitarist', '']), ['Credited as composer, guitarist']);
  assert.deepEqual(rows(['']), ['Credited']);
  assert.deepEqual(rows([]), []);
});

test('one row is the default address, with the parts of its key', async () => {
  const { isDefaultAddress, keyValue } = await import('../ui/people.js');
  const key = { address: '03a', custom_key: '696969', custom_value: '2' };
  assert.equal(isDefaultAddress(key, keyValue(key)), true);
  assert.equal(isDefaultAddress({ address: '03a' }, keyValue(key)), false);
  assert.equal(isDefaultAddress(key, ''), false);
});

test('a credit name row gives one labeled attribute for each value and joins its key', async () => {
  const { creditNameRows, keyOnlyRow, keysWithNoCreditName, creditFoldSummary, roleText } = await import('../ui/people.js');
  const npub = 'npub109pabcdefghijklmnopqrstuvwxyz5aaj';
  const person = {
    name: 'HeyCitizen', usual_role: 'Guitarist',
    credit_names: [{ id: 1, name: 'HeyCitizen', href: 'https://musicindex.org/', npub }, { id: 2, name: 'H. Citizen', href: '', npub: '' }],
    credit_name_uses: [{ id: 1, feeds: 3, roles: ['composer', 'guitarist'] }, { id: 2, feeds: 0, roles: [] }],
    keys: [{ id: 9, npub }, { id: 10, npub: 'npub1other', evidence: 'user' }],
  };
  const profiles = new Map([[npub, { npub, name: 'Sir HeyCitizen' }]]);
  const [first, second] = creditNameRows(person, profiles);
  assert.equal(first.title, 'HeyCitizen');
  assert.equal(first.keyId, 9);
  assert.deepEqual(first.attributes.map((a) => [a.label, a.value]), [['Used in', '3 feeds'], ['Roles', '[guitarist], composer'], ['Website', 'musicindex.org'], ['Nostr', 'Sir HeyCitizen']]);
  assert.deepEqual(second.attributes.map((a) => [a.label, a.value]), [['Used in', 'No feed yet']]);
  assert.equal(second.keyId, null);
  assert.deepEqual(keysWithNoCreditName(person).map((k) => k.id), [10]);
  const only = keyOnlyRow(person.keys[1], null, person.name);
  assert.equal(only.keyId, 10);
  assert.equal(only.title, 'Nostr key');
  assert.equal(only.attributes[0].mono, true);
  assert.equal(only.attributes[0].sub.at(-1), 'Added by you');
  assert.deepEqual(creditFoldSummary(person), { title: 'Credited as HeyCitizen', line: '2 credit names · 2 Nostr keys' });
  assert.deepEqual(creditFoldSummary({ name: 'Ann', credit_names: [], keys: [{ id: 1, npub: 'a' }] }), { title: 'Credited as Ann', line: '1 Nostr key' });
  assert.equal(creditFoldSummary({ name: 'Ann Lee', credit_names: [{ id: 1, name: 'A. Lee' }, { id: 2, name: 'ann lee' }] }).title, 'Credited as ann lee');
  assert.equal(creditFoldSummary({ name: 'Ann Lee', credit_names: [{ id: 1, name: 'A. Lee' }] }).title, 'Credited as A. Lee');
  assert.equal(roleText(['composer', 'guitarist'], ''), 'composer, guitarist');
  assert.equal(roleText(['composer'], 'Drummer'), 'composer');
});

test('a payee fills the form of Add a Wallet', async () => {
  const { walletValues, linkReady } = await import('../ui/people.js');
  assert.deepEqual(walletValues({ address: 'a@b.example', custom_key: null, custom_value: null, recipient_type: '' }),
    { recipient_type: 'lnaddress', address: 'a@b.example', custom_key: '', custom_value: '' });
  const node = walletValues({ address: '03a', custom_key: '696969', custom_value: '2', recipient_type: 'node' });
  assert.deepEqual(node, { recipient_type: 'node', address: '03a', custom_key: '696969', custom_value: '2' });
  assert.equal(linkReady('wallets', node), true);
});

test('the default address and its wallet are first', async () => {
  const { defaultFirst, keyValue } = await import('../ui/people.js');
  const member = (address) => ({ key: { address } });
  const groups = [{ id: 1, members: [member('a'), member('b')] }, { id: 2, members: [member('c'), member('d')] }];
  const order = (value) => defaultFirst(groups, value).map((g) => [g.id, g.members.map((m) => m.key.address)]);
  assert.deepEqual(order(keyValue({ address: 'd' })), [[2, ['d', 'c']], [1, ['a', 'b']]]);
  assert.deepEqual(order(''), [[1, ['a', 'b']], [2, ['c', 'd']]]);
});

test('a payee row stays selected while the form holds its values', async () => {
  const { walletSelected, walletValues } = await import('../ui/people.js');
  const body = { address: '03a', custom_key: '696969', custom_value: '2', recipient_type: 'node' };
  assert.equal(walletSelected(walletValues(body), body), true);
  assert.equal(walletSelected({ ...walletValues(body), custom_value: '3' }, body), false);
  assert.equal(walletSelected({ recipient_type: 'lnaddress', address: '' }, body), false);
});

test('the sheet Edit a Wallet starts with the address and names the other addresses', async () => {
  const { walletEdit, walletEditNote } = await import('../ui/people.js');
  const node = { key: { address: '03a', custom_key: '696969', custom_value: '2' }, member: { recipient_type: 'node' } };
  const ln = { key: { address: 'a@b.example', custom_key: null, custom_value: null }, member: { recipient_type: 'lnaddress' } };
  const edit = walletEdit({ id: 4, members: [node, ln] }, node);
  assert.equal(edit.walletId, 4);
  assert.deepEqual(edit.start, { recipient_type: 'node', address: '03a', custom_key: '696969', custom_value: '2' });
  assert.deepEqual(edit.others, ['a@b.example']);
  assert.match(walletEditNote(edit.others), /also holds a@b\.example\. A saved change replaces the wallet and removes that address/);
  assert.equal(walletEditNote([]), null);
});

test('a feed row names the tracks of a payment and of a credit', async () => {
  const { sheetFeeds } = await import('../ui/people.js');
  const track = (title) => ({ kind: 'item', guid: title, title });
  const [row] = sheetFeeds([{
    feed_id: 1,
    pays: [{ address: 'a@b.example' }],
    pay_places: [[{ kind: 'channel' }, track('One')]],
    roles: ['composer', 'guitarist'],
    role_places: [track('One'), track('Two'), track('Three'), track('Four')],
  }], null);
  assert.match(row.lines[0], /^Pays .*a@b\.example · Channel, One$/);
  assert.equal(row.lines[1], 'Credited as composer, guitarist · One, Two and 2 more');
});

test('the rows sort by split and the sheet keeps the fields of a recipient', async () => {
  const { splitOrder, recipientFields, recipientFromFields } = await import('../ui/recipients.js');
  assert.deepEqual(splitOrder([{ split: '1' }, { split: '99' }, { split: '' }, { split: '99' }, { split: 'x' }]), [1, 3, 0, 2, 4]);
  assert.deepEqual(recipientFields({ recipient_type: 'node' }), { name: '', recipient_type: 'node', address: '', custom_key: '', custom_value: '' });
  assert.deepEqual(recipientFromFields({ name: ' ', recipient_type: 'lnaddress', address: ' a@b.example ', custom_key: 'k', custom_value: 'v' }),
    { name: null, recipient_type: 'lnaddress', address: 'a@b.example', custom_key: null, custom_value: null });
  assert.deepEqual(recipientFromFields({ name: 'A', recipient_type: 'node', address: '03a', custom_key: '696969', custom_value: '2' }),
    { name: 'A', recipient_type: 'node', address: '03a', custom_key: '696969', custom_value: '2' });
});

test('a long list of a sheet starts closed, and each sheet takes the values of a feed', async () => {
  const { foldStartsOpen, creditChoices, fundingChoices, recipientChoices } = await import('../ui/people.js');
  const WHY = 'ADR 0145';
  assert.equal(foldStartsOpen(5), true, WHY);
  assert.equal(foldStartsOpen(6), false, WHY);
  const feed = { title: 'Anna', image_url: 'https://i.example/a.jpg', website: 'https://a.example', npub: null, routes: [
    { name: 'Anna', recipient_type: 'lnaddress', address: 'a@b.example', custom_key: null, custom_value: null },
    { name: null, recipient_type: 'node', address: null },
  ] };
  assert.deepEqual(creditChoices(feed)[0].values, { name: 'Anna', img: 'https://i.example/a.jpg', href: 'https://a.example' }, WHY);
  assert.deepEqual(creditChoices({}), [], WHY);
  assert.deepEqual(creditChoices({ ...feed, title: 'One Song' }, { artist: 'The Wolf' })[0].values, { name: 'The Wolf', img: 'https://i.example/a.jpg' }, 'ADR 0145 item 5: a feed title is not a name');
  assert.deepEqual(fundingChoices(feed)[0].values, { url: 'https://a.example' }, WHY);
  assert.deepEqual(fundingChoices({ website: null }), [], WHY);
  const withCredits = creditChoices({ ...feed, credits: [{ name: 'Anna', role: 'vocalist' }, { name: 'Bo', role: 'bassist', group: 'Music', npub: 'npub1bo' }, { name: 'Cy', role: null }] });
  assert.deepEqual(withCredits.map((c) => [c.title, c.line]), [['Anna', 'vocalist'], ['Bo', 'bassist'], ['Cy', 'No role']], 'ADR 0146 item 8');
  assert.equal(withCredits[0].values.role, 'vocalist', 'ADR 0146 item 8: the row keeps the role');
  assert.equal(withCredits[0].values.href, 'https://a.example', 'ADR 0146 item 8');
  assert.deepEqual(withCredits[1].values, { name: 'Bo', role: 'bassist', group: 'Music', npub: 'npub1bo' }, 'ADR 0146 item 8');
  const { nostrChoices } = await import('../ui/people.js');
  assert.deepEqual(nostrChoices(feed), [], WHY);
  assert.deepEqual(nostrChoices({ ...feed, npub: 'npub1a' })[0].values, { npub: 'npub1a', name: 'Anna', picture: 'https://i.example/a.jpg' }, WHY);
  const routes = recipientChoices(feed);
  assert.equal(routes.length, 1, WHY);
  assert.equal(routes[0].line, 'LNURL address a@b.example', WHY);
  assert.deepEqual(recipientChoices({ routes: [] }), [], WHY);
});

test('addPickLabel uses title case and the plural', () => {
  assert.equal(addPickLabel(1, 'Person', 'People'), 'Add 1 Person');
  assert.equal(addPickLabel(2, 'Person', 'People'), 'Add 2 People');
});

test('a row of the copy sheet names a shared node and its account, and the footer has no zero', async () => {
  const { memberText, addRecipientsLabel, trackHeading } = await import('../ui/people.js');
  const WHY = 'ADR 0145 items 9, 12 and 13';
  assert.equal(memberText({ address: '03b6f6', recipient_type: 'node', provider: 'fountain.fm', custom_value: 'abc' }), 'fountain.fm node · account abc', WHY);
  assert.equal(memberText({ address: 'a'.repeat(66), recipient_type: 'node', provider: null, custom_value: '5' }), `Node address ${'a'.repeat(8)}…aaaa · account 5`, WHY);
  assert.equal(addRecipientsLabel(0), 'Add Recipients', WHY);
  assert.equal(addRecipientsLabel(1), 'Add 1 Recipient', WHY);
  assert.equal(addRecipientsLabel(2), 'Add 2 Recipients', WHY);
  assert.equal(trackHeading('Song', 'Song'), 'Recipients of this track', WHY);
  assert.equal(trackHeading('Song', 'Album'), 'Recipients of Song', WHY);
});

test('an edit sheet starts from the saved values', async () => {
  const { creditNameEdit, supportLinkEdit, LINK_FORMS } = await import('../ui/people.js');
  const WHY = 'ADR 0146 item 6';
  const credit = creditNameEdit({ id: 4, name: 'Ann', href: '', npub: '' });
  assert.equal(credit.linkId, 4, WHY);
  assert.deepEqual(credit.start, { name: 'Ann', href: '', npub: null }, WHY);
  assert.deepEqual(supportLinkEdit({ id: 5, label: 'Tip', url: 'https://x' }).start, { label: 'Tip', url: 'https://x' }, WHY);
  for (const form of Object.values(LINK_FORMS)) assert.ok(form.editTitle && form.editButton, WHY);
});

test('a Nostr result takes a missing name and picture from the person that holds the key', async () => {
  const { withPersonValues } = await import('../ui/people.js');
  const WHY = 'ADR 0146 item 5';
  const people = [{ name: 'Ann', img: 'https://i.example/a.jpg', keys: [{ npub: 'npub1a' }] }];
  assert.deepEqual(withPersonValues({ npub: 'npub1a', name: null, picture: null }, people), { npub: 'npub1a', name: 'Ann', picture: 'https://i.example/a.jpg' }, WHY);
  assert.deepEqual(withPersonValues({ npub: 'npub1a', name: 'Own', picture: 'https://own' }, people), { npub: 'npub1a', name: 'Own', picture: 'https://own' }, WHY);
  assert.deepEqual(withPersonValues({ npub: 'npub1b', name: null, picture: null }, people), { npub: 'npub1b', name: null, picture: null }, WHY);
  assert.deepEqual(withPersonValues({ npub: 'npub1b', name: null, picture: null }, null), { npub: 'npub1b', name: null, picture: null }, WHY);
});

test('lookupAddress marks only a start text as plain (ADR 0151 item 3)', () => {
  assert.equal(lookupAddress('/musicindex/search', 'In:Flux', true), '/musicindex/search?q=In%3AFlux&plain=true');
  assert.equal(lookupAddress('/musicindex/search', 'kind:single', false), '/musicindex/search?q=kind%3Asingle');
});

test('showFindEmpty needs a request that gave no rows (ADR 0151 item 9)', () => {
  assert.equal(showFindEmpty({ rows: [], error: null, searched: true }), true);
  assert.equal(showFindEmpty({ rows: [], error: null, searched: false }), false);
  assert.equal(showFindEmpty({ rows: [{}], error: null, searched: true }), false);
  assert.equal(showFindEmpty({ rows: [], error: new Error('x'), searched: true }), false);
  assert.equal(showFindEmpty({ rows: null, error: null, searched: true }), false);
});

// ADR 0156 items 2 and 3: the summary line and the lines of a feed of the
// section "Where this person is".
test('where this person is: the summary in words and one line for each place', () => {
  const places = [
    { kind: 'channel', pays: true, percent: 15, roles: ['host'] },
    { kind: 'item', guid: 'a', title: "It's Christmastime Again!", number: 1, pays: true, percent: 15, roles: [] },
    { kind: 'item', guid: 'b', title: 'Night Song', number: 2, pays: true, percent: 10, roles: ['vocalist'] },
    { kind: 'item', guid: 'c', title: 'Low Tide', number: 3, pays: false, percent: null, roles: ['vocalist'] },
  ];
  assert.equal(whereSummary(places, 'album'), 'Paid on the album and 2 tracks · Credited as host and vocalist on the album and 2 tracks');
  assert.equal(whereSummary([places[3]], 'album'), 'Credited as vocalist on 1 track');
  assert.deepEqual(whereLine(places[0], 'album'), { label: 'Album', detail: '15.0 % split · Host' });
  assert.deepEqual(whereLine(places[1], 'album'), { label: "1 It's Christmastime Again!", detail: '15.0 % split' });
  assert.deepEqual(whereLine(places[3], 'album'), { label: '3 Low Tide', detail: 'Vocalist' });
  assert.deepEqual(whereLine({ kind: 'item', title: '', pays: true, percent: null, roles: [] }, 'album'), { label: 'Untitled track', detail: 'Paid' });
  const feeds = [{ id: 7, title: 'The Heycitizen Experience', medium: 'music' }];
  const [entry] = whereEntries([{ feed_id: 7, places }], feeds);
  assert.equal(entry.title, 'The Heycitizen Experience');
  assert.equal(entry.lines.length, 4);
  assert.equal(whereEntries([{ feed_id: 9, places: [] }], feeds)[0].title, 'Feed 9');
});

// ADR 0156 item 7: a credit row names its release, then each role with its tracks.
test('a credit of More from your feeds names the release and each role with its tracks', () => {
  const item = {
    kind: 'credit', name: 'MaryKateUltra', href: '', npub: '', roles: ['host', 'vocalist'], feeds: [7], cause: 'same_name',
    feed_places: [{ feed_id: 7, roles: [
      { role: 'host', places: [{ kind: 'channel' }] },
      { role: 'vocalist', places: [{ kind: 'item', title: 'Night Song', number: 3 }, { kind: 'item', title: 'Low Tide', number: 5 }] },
    ] }],
  };
  const feeds = [{ id: 7, title: 'The Heycitizen Experience', medium: 'music' }];
  const [row] = moreRows([item], new Map([[7, 'The Heycitizen Experience']]), null, 'Mary', feeds);
  assert.equal(row.title, 'The Heycitizen Experience');
  assert.deepEqual(row.lines, ['Host · the album', 'Vocalist · track 3 "Night Song", track 5 "Low Tide"', 'Credited as "MaryKateUltra"', causeText('same_name')]);
  const two = moreRows([{ ...item, feeds: [7, 8], feed_places: [...item.feed_places, { feed_id: 8, roles: [{ role: 'host', places: [{ kind: 'channel' }] }] }] }],
    new Map([[7, 'One'], [8, 'Two']]), null, 'Mary', feeds)[0];
  assert.equal(two.title, '2 releases');
  assert.ok(two.lines.includes('Two: Host · the album'));
});

// ADR 0158 item 2: the fold of hidden suggestions is last, and only when one
// is hidden.
test('the hidden suggestions come at the end of the People page', () => {
  const base = { people: [{ id: 1 }], suggestions: [], changes: [], payees: [] };
  assert.equal(peopleSections(base).includes('hidden'), false);
  assert.equal(peopleSections({ ...base, hidden: [{ identity: 'name:ann' }] }).at(-1), 'hidden');
});

// ADR 0158 item 1: the button names what it does.
test('the suggestion sheet has "Hide This Suggestion" and no "Not This Person"', () => {
  const source = readFileSync(new URL('../ui/peoplepages.js', import.meta.url), 'utf8');
  assert.equal(source.includes('Not This Person'), false, 'ADR 0158 item 1: rename the button in crates/api/ui/peoplepages.js.');
  assert.ok(source.includes('Hide This Suggestion'));
});

// ADR 0158 item 5: a credit row of the suggestion sheets has the form of
// ADR 0156 item 7.
test('a credit row of a suggestion sheet names its release, then each role with its tracks', () => {
  const item = {
    name: 'Ann Bell', roles: ['producer', 'vocalist'], feeds: [7], same_name: true,
    feed_places: [{ feed_id: 7, roles: [
      { role: 'producer', places: [{ kind: 'channel' }] },
      { role: 'vocalist', places: [{ kind: 'item', title: 'Track 2', number: 2 }] },
    ] }],
  };
  const feeds = [{ id: 7, title: 'Test Album', medium: 'music' }];
  const row = creditItemRow(item, new Map([[7, 'Test Album']]), feeds);
  assert.equal(row.title, 'Test Album');
  assert.deepEqual(row.lines, ['Producer · the album', 'Vocalist · track 2 "Track 2"']);
  const other = creditItemRow({ ...item, name: 'A. Bell', same_name: false }, new Map([[7, 'Test Album']]), feeds);
  assert.equal(other.lines.at(-1), 'Credited as "A. Bell"');
});

import { pickLink, pageSuggestions, READ_ONLY_LINK, PAGE_MISSING } from '../ui/people.js';

const PAGE = { feed_guid: 'g-ann', title: 'Ann Bell Music' };
const PAGE_FEEDS = [{ id: 3, title: 'Our Label', podcast_guid: 'AAAA-1111', publisher_roles: ['label'] }];

test('a read-only page is a pending change, and Save sends the list (ADR 0159 items 3 and 6)', () => {
  const saved = savedValues({ ...MK, pages: [PAGE] });
  assert.deepEqual(saved.pages, [PAGE]);
  assert.equal(saveBody(saved, saved), null);
  const added = { ...saved, pages: [PAGE, { feed_guid: 'g-2', title: 'Zed' }] };
  assert.deepEqual(pendingKeys(saved, added), ['page']);
  assert.deepEqual(saveBody(saved, added), { pages: added.pages });
  assert.deepEqual(saveBody(saved, { ...saved, pages: [] }), { pages: [] });
  assert.match(leaveText('Ann', ['page']), /the artists and labels/);
});

test('a result of the collection is a link, and each other result is a read-only page (ADR 0159 item 5)', () => {
  const values = savedValues(MK);
  assert.equal(pickLink({ feed_guid: 'aaaa-1111', title: 'Our Label' }, PAGE_FEEDS, values), null, 'MK already links feed 3.');
  const own = savedValues({ ...MK, members: [] });
  assert.deepEqual(pickLink({ feed_guid: 'aaaa-1111' }, PAGE_FEEDS, own), { members: ['3'] });
  assert.equal(pickLink({ feed_guid: 'AAAA-1111' }, PAGE_FEEDS, { ...own, members: ['3'] }), null, 'A linked feed gives no change.');
  assert.deepEqual(pickLink({ feed_guid: 'g-ann', title: 'Ann Bell Music' }, PAGE_FEEDS, own), { pages: [PAGE] });
  assert.equal(pickLink({ feed_guid: 'G-ANN' }, PAGE_FEEDS, { ...own, pages: [PAGE] }), null);
});

test('a read-only row says "Read only", and a page that Music Index does not know has a note (ADR 0159 items 5 and 10)', () => {
  const saved = savedValues({ ...MK, members: [], pages: [PAGE] });
  const values = { ...saved, pages: [...saved.pages, { feed_guid: 'g-2', title: 'Zed' }] };
  const rows = linkRows(values, saved, PAGE_FEEDS, [], [{ feed_guid: 'g-ann', missing: true }]);
  assert.deepEqual(rows.map((r) => [r.title, r.line]), [
    ['Ann Bell Music', `${READ_ONLY_LINK} · ${PAGE_MISSING}`],
    ['Zed', `${READ_ONLY_LINK} · Not saved yet`],
  ]);
  assert.equal(rows[0].page, 'g-ann');
});

test('a page that differs gives suggestions, and a page that agrees gives none (ADR 0159 items 8 and 9)', () => {
  const values = savedValues({ ...MK, pages: [PAGE] });
  const differs = [{ ...PAGE, missing: false, picture: 'https://img.example/a.png', website: 'https://ann.example', npub: 'npub1ann',
    wallets: [{ key: { address: 'ann@x.example', custom_key: null, custom_value: null }, recipient_type: 'lnaddress', name: 'Ann' }] }];
  const out = pageSuggestions(differs, values);
  assert.deepEqual(out.img.map((r) => [r.value, r.line]), [['https://img.example/a.png', 'New picture on "Ann Bell Music"']]);
  assert.deepEqual(out.href.map((r) => r.line), ['New website on "Ann Bell Music"']);
  assert.deepEqual(out.keys.map((r) => r.value), ['npub1ann']);
  assert.deepEqual(out.wallets.map((r) => [r.value, r.recipient_type]), [['ann@x.example', 'lnaddress']]);
  // "Use" makes a pending value, and the row goes.
  const used = { ...values, img: 'https://img.example/a.png', href: 'https://ann.example', keys: ['npub1ann'],
    wallets: [{ key: { address: 'Ann@x.example' }, recipient_type: 'lnaddress' }] };
  assert.deepEqual(pageSuggestions(differs, used), { img: [], href: [], keys: [], wallets: [] });
  const agrees = [{ ...PAGE, missing: false, picture: null, website: null, npub: null, wallets: [] }];
  assert.deepEqual(pageSuggestions(agrees, values), { img: [], href: [], keys: [], wallets: [] });
  assert.deepEqual(pageSuggestions([{ ...differs[0], missing: true }], values).img, [], 'A missing page gives no suggestion.');
  assert.deepEqual(pageSuggestions(differs, { ...values, pages: [] }).img, [], 'A removed page gives no suggestion.');
});

import { linkFillLabel, linkRow, LINK_READ_ONLY_LINE, LINK_OWN_LINE } from '../ui/people.js';

test('the primary button of the sheet "From Music Index" says what it does (ADR 0163 item 9)', () => {
  assert.equal(linkFillLabel(true, 2), 'Link and Fill 2 Fields');
  assert.equal(linkFillLabel(true, 0), 'Link');
  assert.equal(linkFillLabel(false, 1), 'Fill 1 Field');
  assert.equal(linkFillLabel(false, 0), null);
});

test('the row "Link this page" has three states (ADR 0163 items 5 to 7)', () => {
  const own = savedValues({ ...MK, members: [] });
  const outside = linkRow({ feed_guid: 'g-ann' }, PAGE_FEEDS, own);
  assert.deepEqual(outside, { selectable: true, selected: true, line: LINK_READ_ONLY_LINE, already: false });
  assert.equal(LINK_READ_ONLY_LINE, 'Read only. Subsequent changes of this page show on this person.');
  const collection = linkRow({ feed_guid: 'AAAA-1111' }, PAGE_FEEDS, own);
  assert.deepEqual(collection, { selectable: true, selected: true, line: LINK_OWN_LINE, already: false });
  assert.equal(LINK_OWN_LINE, 'Your page. The details of this person go to this page.');
  assert.equal(linkRow({ feed_guid: 'aaaa-1111' }, PAGE_FEEDS, savedValues(MK)).already, true);
  assert.equal(linkRow({ feed_guid: 'aaaa-1111' }, PAGE_FEEDS, savedValues(MK)).selectable, false);
  assert.equal(linkRow({ feed_guid: 'g-ann' }, PAGE_FEEDS, { ...own, pages: [PAGE] }).already, true);
});
