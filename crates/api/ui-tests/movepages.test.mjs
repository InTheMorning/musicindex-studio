import test from 'node:test';
import assert from 'node:assert/strict';
import {
  NEW_PAGE_PREFIX, releaseCreditFields,
  pageActions, pageSheet, pageNow, pageChoiceRows, pageReleaseRows, previewPath, pageChangeBody,
  startSelection, pageSheetResult, pageChangeNotice, gatherPagesModel,
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

const owner254 = 'ADR 0254 items 22 to 28: the model of the sheets. Fix crates/api/ui/movepages.js.';
const tree = [
  page(1, 'Zed', ['artist'], { parent_ids: [10], release_ids: [101, 102] }),
  page(2, 'Amy', ['artist'], { release_ids: [103] }),
  page(3, 'Bob', ['artist'], { parent_ids: [20] }),
  page(10, 'Label A', ['label'], { parent_ids: [20], child_ids: [1] }),
  page(11, 'Label B', ['label'], { child_ids: [] }),
  page(12, 'Label C', ['label'], { connector_id: null }),
  page(20, 'Host A', ['hosting'], { child_ids: [10, 3] }),
  page(21, 'Host B', ['hosting'], { child_ids: [] }),
  page(30, 'Curator', ['curator']),
  { id: 101, title: 'First', medium: 'music' },
  { id: 102, title: 'Second', medium: 'music' },
  { id: 103, title: 'Third', medium: 'music' },
];
const byId = (id) => tree.find((feed) => feed.id === id);

test('the actions of each rank (ADR 0254 item 23), with no "Also on another label…"', () => {
  const labels = (id) => pageActions(byId(id), tree).map((entry) => entry.label);
  assert.deepEqual(labels(1), ['Change label…', 'Go independent…'], owner254);
  assert.deepEqual(labels(2), ['Sign to a label…', 'Choose a publisher…'], owner254);
  assert.deepEqual(labels(3), ['Sign to a label…', 'Change publisher…'], owner254);
  assert.deepEqual(labels(10), ['Change publisher…'], owner254);
  assert.deepEqual(labels(11), ['Choose a publisher…'], owner254);
  assert.deepEqual(labels(20), [], owner254);
  assert.deepEqual(labels(30), [], owner254);
  assert.ok(!['1', '2', '10', '11'].some((id) => labels(Number(id)).some((text) => /Also on another label/.test(text))), owner254);
  assert.deepEqual(pageActions(byId(1), tree).map((entry) => entry.id), ['change_label', 'go_independent'], owner254);
  assert.deepEqual(pageActions(byId(2), tree).map((entry) => entry.id), ['sign_to_label', 'change_publisher'], owner254);
});

test('an artist with a publisher above it and no label gets Change publisher', () => {
  const artist = page(5, 'Eve', ['artist'], { parent_ids: [20] });
  assert.deepEqual(pageActions(artist, [...tree, artist]).map((entry) => entry.label), ['Sign to a label…', 'Change publisher…'], owner254);
});

test('each sheet has its question and its button (ADR 0254 item 24)', () => {
  const zed = byId(1);
  const sheet = (action) => { const { title, button, role, listTitle, noneHint } = pageSheet(zed, action); return { title, button, role, listTitle, noneHint }; };
  assert.deepEqual(sheet('sign_to_label'), { title: 'Which label releases Zed?', button: 'Change Label', role: 'label',
    listTitle: 'Releases on the new label', noneHint: 'Select a label.' }, owner254);
  assert.deepEqual(sheet('change_label'), sheet('sign_to_label'), owner254);
  assert.deepEqual(sheet('go_independent'), { title: 'Make Zed independent', button: 'Go Independent', role: null,
    listTitle: 'Releases with no label after the change', noneHint: '' }, owner254);
  assert.deepEqual(sheet('change_publisher'), { title: 'Who publishes Zed?', button: 'Change Publisher', role: 'hosting',
    listTitle: 'Releases that change publisher', noneHint: 'Select a publisher.' }, owner254);
  assert.equal(pageSheet(zed, 'add_label'), null, 'ADR 0257 item 14: no sheet "Also on another label…".');
  assert.equal(pageNow(zed, tree), 'On Label A, published by Host A', owner254);
});

test('the list of labels has only writable pages, and marks the label now (ADR 0254 item 25)', () => {
  const rows = pageChoiceRows(tree, byId(1), 'change_label');
  assert.deepEqual(rows.map(({ id, title, now }) => ({ id, title, now })), [
    { id: 10, title: 'Label A', now: true },
    { id: 11, title: 'Label B', now: false },
  ], owner254);
  assert.equal(rows[0].sub, 'Published by Host A', owner254);
  assert.deepEqual(pageChoiceRows(tree, byId(1), 'change_label', 'label b').map((row) => row.id), [11], owner254);
  assert.deepEqual(pageChoiceRows(tree, byId(1), 'go_independent'), [], owner254);
});

test('the list of publishers starts with "Not published" and marks the publisher now', () => {
  const rows = pageChoiceRows(tree, byId(1), 'change_publisher');
  assert.deepEqual(rows.map(({ id, title, now }) => ({ id, title, now })), [
    { id: null, title: 'Not published', now: false },
    { id: 20, title: 'Host A', now: true },
    { id: 21, title: 'Host B', now: false },
  ], owner254);
  assert.equal(rows[1].sub, '2 pages', owner254);
  assert.equal(rows[2].sub, '0 pages', owner254);
  assert.equal(pageChoiceRows(tree, byId(2), 'change_publisher')[0].now, true, owner254);
  // A label that is also a publisher publishes its own releases.
  const selfHost = page(50, 'Self Host', ['label', 'hosting']);
  assert.equal(pageChoiceRows([...tree, selfHost], selfHost, 'change_publisher')[0].now, false, owner254);
  assert.equal(pageNow(selfHost, [...tree, selfHost]), 'Published by Self Host', owner254);
  assert.deepEqual(pageChoiceRows(tree, byId(2), 'change_publisher', 'host b').map((row) => row.id), [21], owner254);
});

test('a page below the page is not in the list', () => {
  const rows = pageChoiceRows(tree, byId(20), 'change_publisher');
  assert.ok(!rows.some((row) => row.id === 10), 'A label below the publisher cannot be its publisher.');
});

test('the release rows show the checks', () => {
  assert.deepEqual(pageReleaseRows(tree, byId(1), [102]), [
    { id: 101, title: 'First', checked: false }, { id: 102, title: 'Second', checked: true },
  ], owner254);
});

test('the preview gives the start selection (ADR 0254 item 28)', () => {
  assert.deepEqual(startSelection({ start_selection: [101] }), [101], owner254);
  assert.deepEqual(startSelection(null), [], owner254);
  assert.equal(previewPath(byId(1), 'change_label', { selected: 10, above: 10 }),
    '/feeds/1/page-change?action=change_label&page=10&above=10', owner254);
  assert.equal(previewPath(byId(1), 'change_publisher', { releases: [] }),
    '/feeds/1/page-change?action=change_publisher&releases=', owner254);
  assert.equal(previewPath(byId(1), 'change_publisher', { releases: [101, 102] }),
    '/feeds/1/page-change?action=change_publisher&releases=101,102', owner254);
  assert.deepEqual(pageChangeBody('change_publisher', null, [101]), { action: 'change_publisher', page: null, releases: [101] }, owner254);
  assert.deepEqual(pageChangeBody('change_label', 11, [101]), { action: 'change_label', page: 11, releases: [101] }, owner254);
});

test('the hints and the result texts come from the preview (ADR 0254 item 25)', () => {
  const zed = byId(1);
  const preview = (extra) => ({ start_selection: [101], result: { place: 'On Label B, published by Host A', changed_releases: 1,
    kept_release_titles: [], nothing_changes: false, ...extra } });
  assert.deepEqual(pageSheetResult(zed, 'change_label', undefined, preview()), { hint: 'Select a label.', after: null, changes: null, keeps: null, ready: false }, owner254);
  assert.equal(pageSheetResult(zed, 'change_publisher', undefined, preview()).hint, 'Select a publisher.', owner254);
  assert.equal(pageSheetResult(zed, 'change_publisher', null, preview({ nothing_changes: true })).hint,
    'Nothing changes. Select a different one or a release.', owner254);
  assert.equal(pageSheetResult(zed, 'change_publisher', null, preview({ nothing_changes: true })).ready, false, owner254);
  assert.equal(pageSheetResult(zed, 'go_independent', undefined, preview({ nothing_changes: true })).hint,
    'Nothing changes. Select a different one or a release.', owner254);
  const one = pageSheetResult(zed, 'change_label', 11, preview({ kept_release_titles: ['Second'] }));
  assert.deepEqual(one, { hint: null, after: 'After: Zed is on Label B, published by Host A.', changes: 'Changes 1 release.',
    keeps: 'Second keeps its label.', ready: true }, owner254);
  const two = pageSheetResult(zed, 'change_publisher', 21, preview({ changed_releases: 0, place: 'Independent, published by Host B', kept_release_titles: ['A', 'B', 'C'] }));
  assert.equal(two.after, 'After: Zed is independent, published by Host B.', owner254);
  assert.equal(two.changes, 'Changes 0 releases.', owner254);
  assert.equal(two.keeps, 'A, B and C keep their publisher.', owner254);
  assert.equal(pageSheetResult(zed, 'change_label', 11, null).ready, false, owner254);
  assert.equal(pageChangeNotice(zed, { place: 'On Label B, not published' }), 'Zed is now on Label B, not published. Not written yet.', owner254);
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
