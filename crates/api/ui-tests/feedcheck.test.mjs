import test from 'node:test';
import assert from 'node:assert/strict';
import {
  tabLabel, checkCount, checkRows, unreadableRows, repairLabel, hasShowGrant, checkTexts,
  PAGE_FINDING_TEXT, pageRefusalView, guidChangeView,
  addressPanelView, repairAddresses, peoplePanelView, ADDRESS_WRITE_LINE,
  releaseChecklist, wrongAddressSet,
  CHECK_LEAD, CHECK_NONE, CHECK_GRANT, CHECK_READ_ONLY, CHECK_UNREADABLE,
  COPY_SHEET_TEXT, WAYS_SHEET_TEXT,
} from '../ui/feedcheck.js';

const finding = (kind, repair, extra = {}) => ({
  kind, level: 'notice', title: `T ${kind}`, reason: `R ${kind}`, entry: null, repair, url: null, ...extra,
});

test('the release checklist uses the five finding kinds and their existing actions', () => {
  const owner = 'ADR 0219 items 6 to 8: check releaseChecklist in feedcheck.js.';
  assert.equal(releaseChecklist(check([])), null, owner);
  const one = releaseChecklist(check([finding('no_tracks', 'add_tracks')]));
  assert.equal(one.title, 'This release needs 1 thing', owner);
  assert.deepEqual(one.rows.map((row) => [row.label, row.mark, row.open?.button ?? null]), [
    ['Tracks', 'cross', 'Add Tracks'], ['Cover image', 'check', null],
    ['Audio file of each track', 'check', null], ['Payment recipient', 'check', null], ['Artist page', 'check', null],
  ], owner);
  const many = releaseChecklist(check([
    finding('no_tracks', 'add_tracks'), finding('no_cover', 'add_a_cover'),
    finding('no_audio_file', 'open_the_track', { entry: 'track-1' }),
    finding('no_audio_file', 'open_the_track', { entry: 'track-2' }),
    finding('no_recipient', 'add_a_recipient'),
  ]));
  assert.equal(many.title, 'This release needs 4 things', owner);
  assert.deepEqual(many.rows.map((row) => [row.label, row.mark, row.open?.button ?? null]), [
    ['Tracks', 'cross', 'Add Tracks'], ['Cover image', 'cross', 'Add a Cover'],
    ['2 tracks have no audio file', 'cross', 'Open the Track'], ['Payment recipient', 'cross', 'Add a Recipient'], ['Artist page', 'check', null],
  ], owner);
  assert.equal(many.rows[2].open.track, 'track-1', owner);
});

test('the GUID panel asks for the feed title and accepts trimmed text', () => {
  const owner = 'ADR 0249 item 3: check guidChangeView in feedcheck.js.';
  const change = { known: 'old', new: 'new', title: 'This Is It' };
  const view = guidChangeView(change, 'this is it');
  assert.equal(view.title, 'This write changes the GUID', owner);
  assert.equal(view.guids, 'Known: old → New: new', owner);
  assert.deepEqual(view.lines, [
    'Payments from other music feeds that name old stop.',
    'Apps can show the feed as a new show.',
  ], owner);
  assert.equal(view.keep, 'Keep the Old GUID', owner);
  assert.equal(view.prompt, 'Type This Is It to change the GUID', owner);
  assert.equal(view.confirmed, false, owner);
  assert.equal(guidChangeView(change, '  This Is It ').confirmed, true, owner);
  assert.equal(guidChangeView({ ...change, title: null }).prompt, 'Type CHANGE to change the GUID', owner);
  assert.equal(guidChangeView(null), null, owner);
});

test('an artist page finding adds a checklist row', () => {
  const checklist = releaseChecklist(check([finding('no_artist_page', 'link_a_page')]));
  assert.equal(checklist.title, 'This release needs 1 thing');
  assert.deepEqual(checklist.rows.map((row) => [row.label, row.mark, row.open?.button ?? null]), [
    ['Tracks', 'check', null], ['Cover image', 'check', null], ['Audio file of each track', 'check', null],
    ['Payment recipient', 'check', null], ['Artist page', 'cross', 'Link a Page'],
  ]);
});

test('the payment page address mark uses wrong address findings', () => {
  const owner = 'ADR 0219 item 10: check wrongAddressSet in feedcheck.js.';
  assert.deepEqual([...wrongAddressSet(check([
    finding('wrong_address_type', 'use_lnurl_address', { url: 'name@example.com' }),
    finding('no_cover', 'add_a_cover'),
  ]))], ['name@example.com'], owner);
  assert.equal(wrongAddressSet(null).size, 0, owner);
});

test('an old link GUID uses the target title and the repair route', () => {
  const owner = 'ADR 0247 item 8: fix checkRows in feedcheck.js.';
  const row = checkRows(check([finding('link_guid_old', 'use_the_guid_of', {
    level: 'warning', target_title: 'This Is It', entry: null, url: 'https://music.example/this-is-it.xml',
  })]))[0];
  assert.equal(row.button, 'Use the GUID of This Is It', owner);
  assert.equal(row.action, 'repair', owner);
  assert.deepEqual(row.body, {
    kind: 'link_guid_old', entry: null, url: 'https://music.example/this-is-it.xml',
  }, owner);
});
const check = (findings, extra = {}) => ({ revision: 'r1', read_only: false, read: true, unreadable: [], findings, ...extra });

test('the tab label has the count, and no count for no finding', () => {
  const owner = 'ADR 0228 item 13: check tabLabel in feedcheck.js.';
  assert.equal(tabLabel(5), 'Issues (5)', owner);
  assert.equal(tabLabel(1), 'Issues (1)', owner);
  assert.equal(tabLabel(0), 'Issues', owner);
  assert.equal(tabLabel(undefined), 'Issues', owner);
  assert.equal(checkCount(null), 0, owner);
  assert.equal(checkCount(check([finding('no_language', 'select_a_language', { level: 'question' })])), 1, owner);
});

test('the model gives one row for each finding with its level and its button', () => {
  const owner = 'ADR 0174 items 7 and 19: check checkRows in feedcheck.js.';
  const rows = checkRows(check([
    finding('no_self_link', 'add_the_link'),
    finding('http_file', 'use_the_https_address', { entry: 'g-1', url: 'http://a.example/x.mp3' }),
    finding('no_file_size', 'read_the_size', { entry: 'g-1' }),
    finding('no_duration', 'open_the_track', { entry: 'g-2' }),
    finding('no_language', 'select_a_language', { level: 'question' }),
    finding('move_not_confirmed', null),
  ]));
  assert.deepEqual(rows.map((r) => r.button), [
    'Add the Link', 'Use the https Address', 'Read the Size', 'Open the Track', null, 'Select a Language',
  ], owner);
  assert.deepEqual(rows.map((r) => r.level), ['Notice', 'Notice', 'Notice', 'Notice', 'Notice', 'Question'], owner);
  assert.equal(rows[0].title, 'T no_self_link', owner);
  assert.equal(rows[0].reason, 'R no_self_link', owner);
  assert.deepEqual(rows[1].body, { kind: 'http_file', entry: 'g-1', url: 'http://a.example/x.mp3' }, owner);
  assert.equal(rows[1].action, 'repair', owner);
  assert.equal(rows[3].action, 'open-track', owner);
  assert.equal(rows[3].track, 'g-2', owner);
  assert.equal(rows[3].body, null, owner);
  assert.equal(rows[5].action, 'open-language', owner);
  assert.equal(rows[4].action, null, owner);
});

test('the new finding buttons use their required actions', () => {
  const rows = checkRows(check([
    finding('no_tracks', 'add_tracks'), finding('no_cover', 'add_a_cover'),
    finding('no_recipient', 'add_a_recipient'), finding('wrong_address_type', 'use_lnurl_address', { url: 'name@example.com' }),
  ]));
  assert.deepEqual(rows.map((row) => [row.button, row.action]), [
    ['Add Tracks', 'open-tracks'], ['Add a Cover', 'open-cover'],
    ['Add a Recipient', 'open-recipient'], ['Use LNURL Address', 'repair'],
  ]);
});

test('the short address finding has the button Add https:// with the repair action', () => {
  const owner = 'ADR 0245 item 1: check REPAIRS and ACTIONS in feedcheck.js.';
  const rows = checkRows(check([
    finding('short_address', 'add_https', { level: 'stop', url: 'www.a.com', field: 'persons/0/href' }),
  ]));
  assert.equal(rows[0].button, 'Add https://', owner);
  assert.equal(rows[0].action, 'repair', owner);
  assert.equal(rows[0].stop, true, owner);
  assert.deepEqual(rows[0].body, { kind: 'short_address', entry: null, url: 'www.a.com', field: 'persons/0/href' }, owner);
});

test('a read-only feed gives rows with no button', () => {
  const owner = 'ADR 0174 item 20: check checkRows in feedcheck.js.';
  const rows = checkRows(check([finding('no_self_link', null), finding('no_language', null, { level: 'question' })], { read_only: true }));
  assert.equal(rows.length, 2, owner);
  assert.deepEqual(rows.map((r) => r.button), [null, null], owner);
  assert.deepEqual(rows.map((r) => r.action), [null, null], owner);
});

test('warning rows come first and the count includes them', () => {
  const owner = 'ADR 0175 item 7.';
  const rows = checkRows(check([
    finding('no_language', 'select_a_language', { level: 'question' }),
    finding('index_address_differs', null, { url: 'https://old.example/feed.xml' }),
    finding('copy_other_recipients', null, { level: 'warning', report: 'Feed GUID: g-1\nPublic address of the feed: https://a.example/feed.xml\nCopy URL: https://b.example/feed.xml' }),
    finding('no_self_link', 'add_the_link'),
  ]));
  assert.deepEqual(rows.map((row) => row.level), ['Warning', 'Notice', 'Notice', 'Question'], owner);
  assert.equal(checkCount(check([finding('copy_other_recipients', null, { level: 'warning' })])), 1, owner);
});

test('a one-sided page link opens the page write action', () => {
  const row = checkRows(check([finding('unlisted_by_page', null, {
    title: 'HeyCitizen does not list this release.', page_id: 42, page_title: 'HeyCitizen',
  })]))[0];
  assert.equal(row.button, 'Write HeyCitizen…');
  assert.equal(row.action, 'write-page');
  assert.equal(row.pageId, 42);
});

test('stop findings come first and stop the write', () => {
  const rows = checkRows(check([
    finding('no_self_link', 'add_the_link'),
    finding('no_artist_page', 'link_a_page', { level: 'stop' }),
  ]));
  assert.deepEqual([rows[0].level, rows[0].stop, rows[0].button], ['Stops the write', true, 'Link a Page']);
});

test('read-only index findings keep their two sheet actions', () => {
  const rows = checkRows(check([
    finding('no_self_link', null),
    finding('copy_other_recipients', null, { level: 'warning', report: 'Feed GUID: g' }),
    finding('index_address_differs', null, { url: 'https://old.example/feed.xml' }),
  ], { read_only: true }));
  assert.deepEqual(rows.map((row) => row.button), ['Show What to Do', null, 'Show the Ways']);
  assert.deepEqual(rows.map((row) => row.action), ['show-copy', null, 'show-ways']);
  assert.equal(COPY_SHEET_TEXT.title, 'A Copy Pays Other People');
  assert.equal(WAYS_SHEET_TEXT.title, 'Music Index Has Another Address');
});

test('the copy finding carries the complete report text', () => {
  const report = 'Feed GUID: g-1\nPublic address of the feed: https://a.example/feed.xml\nCopy URL: https://b.example/feed.xml';
  const row = checkRows(check([finding('copy_other_recipients', null, { level: 'warning', report })]))[0];
  assert.equal(row.report, report);
  assert.match(row.report, /g-1/);
  assert.match(row.report, /https:\/\/a\.example\/feed\.xml/);
  assert.match(row.report, /https:\/\/b\.example\/feed\.xml/);
});

test('Open the Track has no button for a track with no GUID', () => {
  const rows = checkRows(check([finding('no_duration', 'open_the_track')]));
  assert.equal(rows[0].button, null, 'ADR 0174 item 7: check checkRows in feedcheck.js.');
});

test('the repair labels follow the core names', () => {
  assert.equal(repairLabel('use_the_public_address'), 'Use the Public Address');
  assert.equal(repairLabel(null), null);
  assert.equal(repairLabel('other'), null);
});

test('page findings use the words of ADR 0213 items 18 and 19', () => {
  const owner = 'ADR 0213 items 18 and 19: show the page finding words and Link a Page. Fix feedcheck.js.';
  assert.deepEqual(PAGE_FINDING_TEXT, {
    no_artist_page: 'This release has no artist page.',
    no_publisher_page: 'This feed has no artist, label or hosting page.',
  }, owner);
  const rows = checkRows(check([
    finding('no_artist_page', 'link_a_page', { title: PAGE_FINDING_TEXT.no_artist_page }),
    finding('no_publisher_page', 'link_a_page', { title: PAGE_FINDING_TEXT.no_publisher_page }),
  ]));
  assert.deepEqual(rows.map((row) => row.button), ['Link a Page', 'Link a Page'], owner);
  assert.deepEqual(rows.map((row) => row.action), ['open-pages', 'open-pages'], owner);
});

test('the refusal panel of Review links a page (ADR 0218 items 1 to 6)', () => {
  const owner = 'ADR 0218 items 1 to 6: the refusal panel gives its texts, the menu and Link and Continue. Fix pageRefusalView in feedcheck.js.';
  assert.deepEqual(pageRefusalView('no_artist_page', false, 2), {
    title: 'This release has no artist page.',
    line: 'Music Index lists each release under its artist.',
    label: 'Artist page',
    write: 'Link an artist page first.',
    placeholder: 'Select a page',
    button: 'Link and Continue',
    menu: true,
    linkDisabled: true,
  }, owner);
  assert.equal(pageRefusalView('no_artist_page', true, 2).linkDisabled, false, owner);
  const show = pageRefusalView('no_publisher_page', false, 1);
  assert.deepEqual([show.title, show.line, show.label, show.write],
    ['This feed has no artist, label or hosting page.', 'Music Index lists each feed under its page.', 'Page', 'Link a page first.'], owner);
  const none = pageRefusalView('no_artist_page', false, 0);
  assert.deepEqual([none.menu, none.button, none.linkDisabled], [false, 'Make an Artist Page…', false], owner);
  assert.equal(pageRefusalView(null), null, owner);
});

test('each address that a read did not reach gives a notice', () => {
  const rows = unreadableRows(check([], { unreadable: ['https://a.example/x.mp3', 'https://b.example/y.mp3'] }));
  assert.deepEqual(rows.map((r) => r.title), ['https://a.example/x.mp3', 'https://b.example/y.mp3'], 'ADR 0174 item 16');
  assert.equal(rows[0].reason, CHECK_UNREADABLE);
  assert.equal(CHECK_UNREADABLE, 'Studio could not read this address.');
  assert.deepEqual(unreadableRows(null), []);
});

test('the texts of the page follow items 19 to 21', () => {
  const owner = 'ADR 0174 items 19 to 21: check checkTexts in feedcheck.js.';
  const rows = check([finding('no_self_link', 'add_the_link')]);
  assert.deepEqual(checkTexts(rows), { lead: CHECK_LEAD, readOnly: null, grant: null, none: null, reading: null }, owner);
  assert.equal(checkTexts(check([])).none, CHECK_NONE, owner);
  assert.equal(checkTexts(check([])).lead, null, owner);
  assert.equal(checkTexts(rows, { grant: true }).grant, CHECK_GRANT, owner);
  assert.equal(checkTexts(check([finding('no_self_link', null)], { read_only: true })).readOnly, CHECK_READ_ONLY, owner);
  assert.equal(checkTexts(check([finding('no_self_link', null)], { read_only: true })).lead, null, owner);
  assert.equal(checkTexts(check([], { read_only: true }), { grant: true }).grant, null, owner);
  assert.equal(checkTexts(null).none, null, owner);
  assert.notEqual(checkTexts(null).reading, null, owner);
  assert.equal(CHECK_LEAD, 'Studio does not change the feed until you select a button. Each change goes to the draft, and you review it before the write.');
  assert.equal(CHECK_NONE, 'Studio found no problem in this feed.');
});

test('a show grant is active until it ends', () => {
  const live = [{ item: { guid: 'live-1' }, start: '2026-11-10T10:00:00Z', end: '2026-11-10T11:00:00Z' }];
  const now = Date.parse('2026-11-10T12:00:00Z') / 1000;
  const owner = 'ADR 0205 item 49: check hasShowGrant in feedcheck.js.';
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: null }], live, now), true, owner);
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: 5 }], live, now), false, owner);
  assert.equal(hasShowGrant([{ item_guid: 'live-1', ended_at: null }], live, now + 3 * 86400), false, owner);
  assert.equal(hasShowGrant([], live, now), false, owner);
  assert.equal(hasShowGrant(null, live, now), false, owner);
});

const shortFinding = (extra = {}) => ({
  kind: 'short_address', level: 'stop', title: 'T', reason: 'R', entry: null, repair: 'add_https',
  field: 'items/0/persons/0/href', place: 'Credit · The Doerfels · Information address', url: 'www.doerfelverse.com', ...extra,
});

test('the address panel for one feed gives the texts of ADR 0245 items 6 and 7', () => {
  const owner = 'ADR 0245 items 6 and 7: check addressPanelView in feedcheck.js.';
  const view = addressPanelView([{ feedId: 4, title: 'Bloodshot Lies - The Album', check: check([shortFinding(), finding('no_cover', 'add_a_cover')]) }]);
  assert.equal(view.title, '1 feed has addresses that are not full', owner);
  assert.equal(view.button, 'Add https:// in 1 Feed', owner);
  assert.equal(view.write, 'Repair these addresses first.', owner);
  assert.equal(ADDRESS_WRITE_LINE, view.write, owner);
  assert.deepEqual(view.rows.map((row) => [row.feedTitle, row.place, row.address, row.newAddress]), [
    ['Bloodshot Lies - The Album', 'Credit · The Doerfels · Information address', 'www.doerfelverse.com', 'https://www.doerfelverse.com'],
  ], owner);
  assert.deepEqual(view.rows[0].body, { kind: 'short_address', entry: null, url: 'www.doerfelverse.com', field: 'items/0/persons/0/href' }, owner);
});

test('the address panel for two feeds counts the feeds and trims the new address', () => {
  const owner = 'ADR 0245 item 6: check addressPanelView in feedcheck.js.';
  const view = addressPanelView([
    { feedId: 1, title: 'One', check: check([shortFinding({ url: ' doerfelverse.com/artists ' }), shortFinding({ field: 'items/1/persons/0/href' })]) },
    { feedId: 2, title: 'Two', check: check([]) },
    { feedId: 3, title: 'Three', check: check([shortFinding()]) },
  ]);
  assert.equal(view.title, '2 feeds have addresses that are not full', owner);
  assert.equal(view.button, 'Add https:// in 2 Feeds', owner);
  assert.equal(view.rows.length, 3, owner);
  assert.equal(view.rows[0].newAddress, 'https://doerfelverse.com/artists', owner);
  assert.equal(addressPanelView([{ feedId: 2, title: 'Two', check: check([]) }]), null, owner);
  assert.equal(addressPanelView([]), null, owner);
});

test('repairAddresses repairs each row and passes the new revision on', async () => {
  const owner = 'ADR 0245 items 4 and 6: check repairAddresses in feedcheck.js.';
  const view = addressPanelView([
    { feedId: 1, title: 'One', check: check([shortFinding(), shortFinding({ field: 'items/1/persons/0/href' })], { revision: 'r1' }) },
    { feedId: 3, title: 'Three', check: check([shortFinding()], { revision: 'r9' }) },
  ]);
  const calls = [];
  const answers = await repairAddresses(view, async (feedId, body) => {
    calls.push([feedId, body.field, body.revision]);
    return { draft: { feedId }, check: { revision: `${body.revision}+` } };
  });
  assert.deepEqual(calls, [[1, 'items/0/persons/0/href', 'r1'], [1, 'items/1/persons/0/href', 'r1+'], [3, 'items/0/persons/0/href', 'r9']], owner);
  assert.deepEqual([...answers.keys()], [1, 3], owner);
  assert.equal(answers.get(1).check.revision, 'r1++', owner);
});

const peopleAnswer = (people) => ({ people });
const entry = (feedId, findings) => ({ feed_id: feedId, state: findings.length > 0 ? 'needs_correction' : 'correct', findings });

test('the People panel gives the texts of ADR 0245 item 11 for one feed', () => {
  const owner = 'ADR 0245 item 11: check peoplePanelView in feedcheck.js.';
  const view = peoplePanelView(peopleAnswer([{ person_id: 7, name: 'The Doerfels', feeds: [entry(4, [{ kind: 'credit_details', href: true }])] }]), (id) => `Feed ${id}`);
  assert.equal(view.title, '1 feed differs from People', owner);
  assert.equal(view.groups.length, 1, owner);
  assert.equal(view.groups[0].button, 'Use the People Values in 1 Feed', owner);
  assert.deepEqual(view.groups[0].feedIds, [4], owner);
  assert.equal(view.groups[0].entries[0].title, 'Feed 4', owner);
});

test('the People panel for two feeds and two persons counts each feed once', () => {
  const owner = 'ADR 0245 items 11 and 13: check peoplePanelView in feedcheck.js.';
  const view = peoplePanelView(peopleAnswer([
    { person_id: 7, name: 'Ann', feeds: [entry(1, [{ kind: 'credit_role' }]), entry(2, [{ kind: 'credit_role' }]), entry(3, [])] },
    { person_id: 8, name: 'Bob', feeds: [entry(2, [{ kind: 'credit_key' }])] },
    { person_id: 9, name: 'Cy', feeds: [entry(5, [])] },
  ]));
  assert.equal(view.title, '2 feeds differ from People', owner);
  assert.deepEqual(view.groups.map((group) => [group.name, group.button]), [
    ['Ann', 'Use the People Values in 2 Feeds'], ['Bob', 'Use the People Values in 1 Feed'],
  ], owner);
  assert.deepEqual(view.groups[0].feedIds, [1, 2], owner);
  assert.equal(peoplePanelView(peopleAnswer([{ person_id: 9, name: 'Cy', feeds: [entry(5, [])] }])), null, owner);
  assert.equal(peoplePanelView(null), null, owner);
});

test('the People button names People for two feeds of The Doerfels', () => {
  const owner = 'ADR 0246 item 7: check peoplePanelView in feedcheck.js.';
  const view = peoplePanelView(peopleAnswer([{
    person_id: 7,
    name: 'The Doerfels',
    feeds: [
      entry(4, [{ kind: 'credit_details', href: true }]),
      entry(5, [{ kind: 'credit_details', href: true }]),
    ],
  }]));
  assert.equal(view.groups[0].button, 'Use the People Values in 2 Feeds', owner);
});
