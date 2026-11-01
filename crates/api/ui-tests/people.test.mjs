// The model of the People area (ADR 0099, task 065a).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  peopleSections, PEOPLE_EMPTY, suggestionRow, personRow, makePersonStart, makePersonBody, sheetFeeds,
  checkedIds, toggledRow, reviewLabel, operationBody, addResultLine,
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

test('the page person has the sections of ADR 0099 item 4 in tier sequence', () => {
  const sections = pageSequence('person');
  assert.deepEqual(sections.map((s) => s.id), ['profile', 'actions', 'where', 'payment', 'credits', 'support']);
  assert.equal(tierError('person', sections), null);
  assert.ok(!sections.some((s) => /feeds|check/.test(s.id)),
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
    lines: ['Fiddle · Member of The Ultras', 'Pays Lightning address sam@getalby.com'],
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
  assert.deepEqual(row.lines, ['Pays Lightning address jo@x.example · 2 wallets']);
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
  assert.deepEqual(rows[0], { id: 1, title: 'Neon Diner', lines: ['Pays Lightning address mk@x.example', 'Credited as Vocals'], checked: true });
  assert.deepEqual(rows[1].lines, ['Credited']);
  assert.match(rows[2].lines[0], /^Pays Lightning node /);
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
  initials, personMatches, filterPeople, filterPayees, payeesWithNoPerson, openChanges, changeTitle, feedsToWrite,
  changeRow, afterDiscard, memberChoices, makeSheetTitle, creditItemRow, payeeItemRow, toggledItem, dismissBody, addPersonBody,
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

test('the sheet Make a Person gives its texts, rows and menu', () => {
  assert.equal(makeSheetTitle('Mary'), 'Make a Person: Mary');
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
