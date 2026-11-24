import test from 'node:test';
import assert from 'node:assert/strict';
import {
  NEW_PAGE_PREFIX, releaseCreditFields,
  movePageTitle, pageMoveRoles, pageMoveBody, pageMoveMenus, pageMoveStart, pageMoveLines, gatherPagesModel,
} from '../ui/movepages.js';

const page = (id, title, roles, extra = {}) => ({ id, title, medium: 'publisher', publisher_roles: roles, connector_id: 'c', ...extra });
const feeds = [
  page(1, 'Zed', ['artist'], { parent_ids: [10] }),
  page(2, 'Amy', ['artist']),
  page(10, 'Label A', ['label'], { parent_ids: [20] }),
  page(20, 'Host A', ['hosting']),
  page(30, 'Read Only', ['artist'], { connector_id: null }),
];
const owner = 'ADR 0240 items 12 to 17: the model of the sheet. Fix crates/api/ui/movepages.js.';

test('page moves show the allowed roles, current parent, and changed page lines', () => {
  const artist = { ...feeds[0], release_ids: [101, 102] };
  const label = { ...feeds[2], release_ids: [101, 102] };
  assert.equal(movePageTitle(artist), 'Move Zed', owner);
  assert.equal(movePageTitle(label), 'Move Label A', owner);
  assert.deepEqual(pageMoveRoles(artist), ['label', 'hosting'], owner);
  assert.deepEqual(pageMoveRoles(label), ['hosting'], owner);
  assert.deepEqual(pageMoveMenus(feeds, artist, { label: '10', hosting: '' }).map((menu) => menu.role), ['label', 'hosting'], owner);
  assert.deepEqual(pageMoveMenus(feeds, label, { hosting: '' }).map((menu) => menu.role), ['hosting'], owner);
  assert.deepEqual(pageMoveStart(feeds, artist), { label: '10', hosting: '20' }, owner);
  assert.deepEqual(pageMoveStart(feeds, label), { hosting: '20' }, owner);
  assert.deepEqual(pageMoveLines([...feeds, { id: 11, title: 'Label B' }], artist, { label: '11', hosting: '20' }),
    ['Changes 2 releases:', '+ Label B', '− Label A'], owner);
  assert.deepEqual(pageMoveLines([...feeds, page(21, 'Host B', ['hosting'])], label, { hosting: '21' }),
    ['Changes 2 releases:', '+ Host B', '− Host A'], owner);
});

test('page move bodies use selected page ids and null for None', () => {
  const artist = { ...feeds[0] };
  const label = { ...feeds[2] };
  assert.deepEqual(pageMoveBody(artist, { label: '12', hosting: '' }), { label_id: 12, hosting_id: null }, owner);
  assert.deepEqual(pageMoveBody(label, { hosting: '12' }), { hosting_id: 12 }, owner);
});

test('gather pages lists writable pages, their start selection, and the note (ADR 0243 items 8 to 10)', () => {
  const level = page(40, 'Host A', ['hosting']);
  const candidates = [
    page(1, 'Artist A', ['artist'], { release_ids: [10, 11] }),
    page(2, 'No Role', [], { release_ids: [12] }),
    page(3, 'Nested', ['artist'], { parent_ids: [40] }),
    page(4, 'Read Only', ['artist'], { connector_id: null }),
    page(5, 'Label A', ['label']),
  ];
  const model = gatherPagesModel([level, ...candidates], level, 'artist');
  assert.equal(model.title, 'Move artists to Host A', 'ADR 0243 item 8: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.deepEqual(model.rows.map(({ id, title, releases, role, checked }) => ({ id, title, releases, role, checked })), [
    { id: 1, title: 'Artist A', releases: 2, role: 'Artist', checked: false },
    { id: 2, title: 'No Role', releases: 1, role: 'No role', checked: true },
  ], 'ADR 0243 items 8 to 10: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.equal(model.button, 'Move 1 Artist', 'ADR 0243 item 9: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.equal(model.ready, true, 'ADR 0243 item 9: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.equal(model.note, 'A page with no role gets the role Artist.', 'ADR 0243 item 10: fix gatherPagesModel in crates/api/ui/movepages.js.');
  const empty = gatherPagesModel([level, candidates[0]], level, 'artist', []);
  assert.equal(empty.button, 'Move 0 Artists', 'ADR 0243 item 9: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.equal(empty.ready, false, 'ADR 0243 item 9: fix gatherPagesModel in crates/api/ui/movepages.js.');
  assert.equal(empty.note, null, 'ADR 0243 item 10: fix gatherPagesModel in crates/api/ui/movepages.js.');
});

test('a page move with no link change has an instruction and no heading (ADR 0243 item 6)', () => {
  const artist = { ...feeds[0], release_ids: [101] };
  assert.deepEqual(pageMoveLines(feeds, artist, { label: '10', hosting: '20' }), ['Select a label or a publisher page.'],
    'ADR 0243 item 6: fix pageMoveLines in crates/api/ui/movepages.js.');
});

const credit = (id, title) => ({ id, title });
const credits = {
  artist: credit(1, 'Zed'), label: credit(10, 'Label A'), publisher: credit(20, 'Host A'),
  follows: credit(20, 'Host A'), publisher_follows: true,
  other_artists: [], other_labels: [], other_publishers: [], alias: null,
};

test('release menus use writable pages and core selections without parent defaults', () => {
  const view = releaseCreditFields(credits, feeds);
  assert.deepEqual(view.menus.map(({ role, value }) => [role, value]), [['artist', '1'], ['label', '10'], ['hosting', '20']]);
  assert.deepEqual(view.menus[0].options.map((option) => option[1]), ['Amy', 'Zed', 'New Artist Page…']);
  assert.deepEqual(view.menus[1].options.at(-2), ['', 'Independent (no label)']);
  assert.deepEqual(view.menus[2].options.at(-2), ['', 'Not published']);
  for (const menu of view.menus) assert.equal(menu.options.at(-1)[0], `${NEW_PAGE_PREFIX}${menu.role}`);
  const empty = releaseCreditFields({ ...credits, artist: null, label: null, publisher: null, follows: null }, feeds);
  assert.equal(empty.menus[0].placeholder, 'Select an artist page');
  assert.equal(empty.menus[0].value, '');
  assert.ok(!empty.menus[0].options.some(([value]) => value === ''));
  assert.equal(empty.menus[1].value, '');
  assert.equal(empty.menus[2].value, '');
});

test('the core gives the publisher that follows even when the feed tree differs', () => {
  const view = releaseCreditFields({ ...credits, follows: credit(999, 'Other publisher') }, feeds);
  assert.equal(view.follows, 999);
  assert.match(view.menus[2].hint, /^Follows the label\./);
  assert.match(view.menus[2].hint, /Here the label goes in Label\./);
  assert.equal(view.followLabel, null);
  const independent = releaseCreditFields({ ...credits, label: null, follows: null }, feeds);
  assert.match(independent.menus[2].hint, /^Follows the artist page\./);
  assert.equal(independent.follows, null);
});

test('a selected publisher uses the core flag and offers the correct follow button', () => {
  const view = releaseCreditFields({ ...credits, publisher_follows: false }, feeds);
  assert.equal(view.menus[2].hint, 'You selected the publisher.');
  assert.equal(view.followLabel, 'Follow the Label Again');
  assert.equal(releaseCreditFields({ ...credits, label: null, publisher_follows: false }, feeds).followLabel, 'Follow the Artist Page Again');
});

test('other linked pages keep their file sequence and names in the hint', () => {
  const view = releaseCreditFields({ ...credits, other_artists: [credit(2, 'Amy'), credit(null, 'Outside artist')] }, feeds);
  assert.equal(view.menus[0].otherHint, 'Also linked in the file: Amy, Outside artist.');
  assert.equal(view.menus[1].otherHint, null);
  const outside = releaseCreditFields({ ...credits, artist: credit(null, 'Outside artist') }, feeds);
  assert.deepEqual(outside.menus[0].options[0], ['external', 'Outside artist', true]);
  assert.equal(outside.menus[0].value, 'external');
  const readOnly = releaseCreditFields({ ...credits, artist: credit(30, 'Read Only') }, feeds);
  assert.deepEqual(readOnly.menus[0].options[0], ['30', 'Read Only', true]);
});

test('the alias section and its hint use the alias that the core gives', () => {
  const view = releaseCreditFields(credits, feeds);
  assert.equal(view.aliasOpen, false);
  assert.equal(view.aliasPlaceholder, 'Zed');
  assert.match(view.aliasHint, /“Zed”/);
  const alias = releaseCreditFields({ ...credits, alias: 'Zed feat. Amy' }, feeds);
  assert.equal(alias.aliasOpen, true);
  assert.equal(alias.alias, 'Zed feat. Amy');
  assert.equal(alias.aliasHint, 'The artist name of this release in podcast apps and music files: “Zed feat. Amy”. The release stays in the catalog of the artist page above.');
});
