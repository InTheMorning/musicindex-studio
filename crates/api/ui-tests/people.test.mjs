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

// ---- Task 065c: the page of a person and its sheets ----
import {
  keyValue, keyFromValue, memberLine, memberText, walletGroups, addressOptions, usedText, keyEvidenceText, whereRow,
  feedRows, levelRows, feedsHeading, SHEET_TITLES, sheetStart, sheetBody, sheetReady, LINK_FORMS, linkFields,
  linkStart, linkBody, linkReady, personFieldsBody, profileLine, changeLine, addIntro,
} from '../ui/people.js';

const KEY = (address, custom_value = null) => ({ address, custom_key: null, custom_value });
const MK = {
  id: 7, name: 'Mary Kate', usual_role: 'Vocals', img: '', href: '', members: [3],
  default_address: KEY('mk@x.example'),
  wallets: [{ id: 1, key: KEY('02ab', 'acct1'), recipient_type: 'node' }],
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
    { label: 'Lightning address', address: 'a@x.example', account: null });
  assert.equal(memberText({ address: '02ab', recipient_type: 'node', provider: 'fountain.fm', custom_value: 'acct1' }), 'fountain.fm node · account acct1');
  assert.equal(memberText({ address: 'a@x.example' }), 'Lightning address a@x.example');
  assert.equal(memberText({ address: '03d55f4d0000aa67', recipient_type: 'node' }), 'Lightning node 03d55f4d…aa67');
  assert.equal(memberText({ address: '03d55f4d0000aa67', recipient_type: 'node', custom_value: 'acct1' }), 'Lightning node 03d55f4d…aa67 · account acct1');
  assert.equal(usedText(0), 'Not used');
  assert.equal(usedText(1), '1 feed');
});

test('a wallet gives its addresses, the Lightning addresses first', () => {
  const groups = walletGroups(MK, RECIPIENTS);
  assert.equal(groups.length, 1);
  assert.deepEqual(groups[0].members.map((m) => m.key.address), ['mk@x.example', '02ab']);
  assert.deepEqual(addressOptions(groups).map(([, text]) => text), ['Lightning address mk@x.example', 'fountain.fm node · account acct1']);
  const alone = walletGroups(MK, []);
  assert.deepEqual(alone[0].members.map((m) => m.key.address), ['02ab']);
});

test('the evidence of a key and the row Where used have their texts', () => {
  assert.equal(keyEvidenceText('user'), 'Added by you');
  assert.equal(keyEvidenceText('nostr'), 'Found in a Nostr profile');
  assert.deepEqual(whereRow(MK), { title: 'Where used', lines: ['Pays in 2 feeds · Credited in 2 feeds'], count: '3 feeds' });
});

test('the feed rows give the type first, each selected', () => {
  const feeds = [{ id: 2, title: 'Neon Diner', medium: 'music' }];
  const rows = feedRows([{ feed_id: 2, pays: [KEY('02ab')], roles: ['Vocals'] }], feeds);
  assert.equal(rows[0].lines[0], 'Album');
  assert.deepEqual(rows[0].lines.slice(1), ['Pays Lightning node 02ab', 'Credited as Vocals']);
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
  assert.deepEqual(Object.keys(LINK_FORMS), ['wallets', 'credit-names', 'keys', 'support-links']);
  const start = linkStart('wallets');
  assert.equal(start.recipient_type, 'lnaddress');
  assert.deepEqual(linkFields('wallets', start).map((f) => f.id), ['recipient_type', 'address']);
  assert.deepEqual(linkFields('wallets', { recipient_type: 'node' }).map((f) => f.id), ['recipient_type', 'address', 'custom_key', 'custom_value']);
  assert.deepEqual(linkBody('wallets', { recipient_type: 'node', address: ' 02ab ', custom_key: '', custom_value: 'a' }),
    { recipient_type: 'node', address: '02ab', custom_key: null, custom_value: 'a' });
  assert.ok(!linkReady('keys', { npub: ' ' }));
  assert.ok(linkReady('support-links', { label: '', url: 'https://x.example' }));
  assert.deepEqual(linkBody('support-links', { label: ' L ', url: 'u' }), { label: 'L', url: 'u' });
});

test('a change of one field sends the four fields of the person', () => {
  assert.deepEqual(personFieldsBody(MK, { usual_role: 'Fiddle' }), { name: 'Mary Kate', usual_role: 'Fiddle', img: '', href: '' });
  assert.deepEqual(profileLine(MK, { 3: 'The Ultras' }), { role: 'Vocals', members: [{ id: 3, title: 'The Ultras' }] });
});

test('the review line of a change set of a person names the operation', () => {
  const feeds = [{}, {}];
  assert.equal(changeLine({ feeds, operation: { kind: 'rename_person', name: 'Ann Ultra', payment: true, credit: true } }, 'Ann'),
    'Rename Ann to Ann Ultra in 2 feeds (payment name and credit name)');
  assert.equal(changeLine({ feeds, operation: { kind: 'rename_person', name: 'Ann Ultra', payment: true, credit: false } }, 'Ann Ultra'),
    'Rename to Ann Ultra in 2 feeds (payment name)');
  assert.equal(changeLine({ feeds, operation: { kind: 'use_address', address: 'ann@example.com', recipient_type: 'lnaddress' } }, 'Ann'),
    'Use Lightning address ann@example.com for Ann');
  assert.equal(changeLine({ feeds, operation: { kind: 'add_to_releases', payee: { percent: 5 }, credit: {}, support_link: null } }, 'Ann'),
    'Add Ann as a payee with 5% and a credit in 2 feeds');
  assert.equal(changeLine({ feeds, operation: { kind: 'remove_person', payee: true, credit: true } }, 'Ann'),
    'Remove Ann as a payee and a credit from 2 feeds');
  assert.equal(changeLine({ feeds, operation: { kind: 'merge' } }, 'Ann'), '');
});

import { readFileSync } from 'node:fs';

test('no text of the People area shows an XML attribute or the words "change set" (ADR 0099 items 13 and 17)', () => {
  for (const name of ['people.js', 'peoplepages.js', 'recipients.js']) {
    const source = readFileSync(new URL(`../ui/${name}`, import.meta.url), 'utf8')
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .split('\n')
      .filter((line) => !line.trim().startsWith('//'))
      .join('\n');
    assert.ok(!/method=/.test(source),
      `ADR 0099 item 17: ${name} shows an XML attribute such as method="lnaddress". Name the type with "Lightning Address" instead.`);
    assert.ok(!/change sets?/i.test(source),
      `ADR 0099 item 13: ${name} has a text with the words "change set". Say "changes" instead.`);
  }
});

test('the words of an address line and a change title follow ADR 0099 items 15 and 18', () => {
  assert.equal(memberText({ address: 'a@b.c', recipient_type: 'lnaddress' }), 'Lightning address a@b.c');
  assert.equal(memberText({ address: '03aa', recipient_type: 'node', provider: 'fountain.fm', custom_value: 'x1' }), 'fountain.fm node · account x1');
  assert.equal(memberText({ address: '03aa', recipient_type: 'node', custom_value: 'x1' }), 'Lightning node 03aa · account x1');
  assert.equal(changeTitle({ kind: 'add' }), 'Add to All Releases');
});
