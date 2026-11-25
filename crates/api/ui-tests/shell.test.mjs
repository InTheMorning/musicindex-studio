// The shell models (ADR 0042). Each test checks one rule of ADR 0258 and of
// musicindex ADR 0080 R-NAV-2.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  pageTabs, tabsFit, mediumKind, feedKind, trackCountText, roleChoice, roleHint, keptRoleFolds, linkRoleChoices, toggleRole, isEditable, isReadOnly, reviewHost, moveNote, isPublisher, isPlaylist, trackKey, isTrack,
  leftPanel, groupDestinations, releasesList, overviewModel, overviewFilter, legacyView, legacyNav, addressSearch, unlinkedPlaylists, sortLabel, sortEntries, compareNames, byName, segmentKey,
  startNav, pushNav, backNav, forwardNav, canBack, canForward, setScroll, overviewView,
  narrowStack, openFeedView, publisherGroups, placeText, pageSort, pageSortLabel, PAGE_SORTS, draftLabel, newFeedView, navAtWidth, rowPath, pagePath, toggleRow, revealRows, widthNav, wideView, publisherArt, mosaicCells, otherFeedText, mediumProblemView, mediumProblemCause,
} from '../ui/shell.js';
import { stateMark } from '../ui/outsidecheck.js';

const album = { id: 1, title: 'Night Songs', medium: 'music', connector_id: 'c1', path: 'night', added_at: 100, draft_state: 'none', image: null };
const publisher = { id: 2, title: 'Label', medium: 'publisher', connector_id: 'c1', path: 'label', added_at: 200, draft_state: 'none' };
const readOnly = { id: 3, title: 'Blocked', medium: 'music', connector_id: null, path: 'b.xml', added_at: 50, draft_state: 'none' };
const model = {
  items: [
    { guid: 'g-1', title: 'First' },
    { _cid: 7, title: '' },
  ],
};
const labels = (items) => items.map((item) => item.label);

test('link role choices show only page roles in the sequence of ROLES', () => {
  assert.deepEqual(linkRoleChoices(['hosting', 'artist', 'label'], 'hosting artist curator'), [
    { role: 'artist', label: 'Artist', checked: true },
    { role: 'label', label: 'Label', checked: false },
    { role: 'hosting', label: 'Publisher', checked: true },
  ]);
});

test('link role choices allow no selected role and no page roles', () => {
  assert.deepEqual(linkRoleChoices(['hosting'], 'artist'), [
    { role: 'hosting', label: 'Publisher', checked: false },
  ]);
  assert.deepEqual(linkRoleChoices(['artist'], null), [
    { role: 'artist', label: 'Artist', checked: false },
  ]);
  assert.deepEqual(linkRoleChoices([], 'artist'), []);
  assert.deepEqual(linkRoleChoices(undefined, undefined), []);
});

test('the medium kind gives the dot and the label', () => {
  assert.deepEqual(mediumKind('publisher', ['label'], true), { dot: 'label', label: 'Label' });
  assert.deepEqual(mediumKind('MUSIC'), { dot: 'feed', label: 'Album' });
  assert.deepEqual(mediumKind('musicL'), { dot: 'playlist', label: 'Playlist' });
  assert.deepEqual(mediumKind(undefined), { dot: 'feed', label: 'Feed' });
});

test('feedKind gives the release kind and defaults to Album', () => {
  const owner = 'ADR 0256 item 14: feedKind shows kind_shown. Fix feedKind in crates/api/ui/shell.js.';
  assert.equal(feedKind({ medium: 'music', kind_shown: 'album' }).label, 'Album', owner);
  assert.equal(feedKind({ medium: 'music', kind_shown: 'release' }).label, 'Release', owner);
  assert.equal(feedKind({ medium: 'music', kind_shown: 'ep' }).label, 'EP', owner);
  assert.equal(feedKind({ medium: 'music', kind_shown: 'single' }).label, 'Single', owner);
  assert.equal(feedKind({ medium: 'music' }).label, 'Album', owner);
});

test('a feed with no role has a hollow dot in each state', () => {
  const owner = 'ADR 0253 item 8: a feed with no role has a hollow dot. Fix feedKind in crates/api/ui/shell.js.';
  for (const roles_confirmed of [false, true]) {
    for (const roles_linked of [false, true]) {
      const feed = { medium: 'publisher', publisher_roles: [], roles_confirmed, roles_linked };
      assert.deepEqual(feedKind(feed), { dot: 'feed hollow', label: 'Feed' }, owner);
      assert.equal(feedKind({ ...feed, release_ids: [7] }).dot, 'feed hollow', owner);
    }
  }
});

test('the Role hint shows agreement for linked roles, including confirmed roles', () => {
  const owner = 'ADR 0253 item 18: linked roles show agreement. Fix roleHint in crates/api/ui/shell.js.';
  for (const roles_confirmed of [false, true]) {
    for (const readOnly of [false, true]) {
      assert.equal(roleHint({ publisher_roles: ['hosting'], roles_linked: true, roles_confirmed }, readOnly),
        'The feed and its releases give the same role.', owner);
      assert.equal(roleHint({ publisher_roles: [], roles_linked: true, roles_confirmed }, readOnly), null, owner);
    }
  }
});

test('the Role hint shows inferred roles only when they are unconfirmed and unlinked', () => {
  const feed = { publisher_roles: ['hosting'], roles_linked: false, roles_confirmed: false };
  assert.equal(roleHint(feed), 'Studio found this role in the feed: Publisher. Select the roles to confirm them.');
  assert.equal(roleHint({ ...feed, publisher_roles: ['label', 'hosting'] }, true),
    'Studio found these roles in the feed: Label, Publisher.');
  assert.equal(roleHint({ ...feed, roles_confirmed: true }), null);
  assert.equal(roleHint({ ...feed, publisher_roles: [] }), null);
  assert.equal(roleHint({}), null);
});

test('trackCountText gives a singular or plural track count', () => {
  assert.equal(trackCountText(1), '1 track');
  assert.equal(trackCountText(7), '7 tracks');
  assert.equal(trackCountText(0), '0 tracks');
  assert.equal(trackCountText(null), null);
});

test('an album or a publisher feed with a connector is editable', () => {
  assert.equal(isEditable(album), true);
  assert.equal(isEditable(publisher), true);
  assert.equal(isEditable({ ...album, medium: 'musicL' }), true);
  assert.equal(isPublisher(publisher), true);
  assert.equal(isPublisher(album), false);
});

test('a read-only album, publisher feed and playlist are editable, and a read-only podcast feed is not', () => {
  assert.equal(isEditable(readOnly), true);
  assert.equal(isEditable({ ...publisher, connector_id: null }), true);
  assert.equal(isEditable({ ...readOnly, medium: 'musicL' }), true);
  assert.equal(isEditable({ ...readOnly, medium: 'podcast' }), false);
});

test('only a feed with no connector is read-only', () => {
  assert.equal(isReadOnly(readOnly), true);
  assert.equal(isReadOnly({ ...readOnly, connector_id: undefined }), true);
  assert.equal(isReadOnly(album), false);
});

test('the outline of a read-only album has the item IDs of the album with a connector', () => {
  const ids = (feed) => leftPanel({ view: { name: 'feed', id: feed.id }, feeds: [feed], model, vaultOpen: true }).items.map((item) => item.id);
  assert.deepEqual(ids(readOnly), ids({ ...readOnly, connector_id: 'c1' }));
});

test('reviewHost gives host, choose and move', () => {
  assert.deepEqual(reviewHost({ hasHost: true, isReadOnly: false, feedMove: null }), { kind: 'host' });
  assert.deepEqual(reviewHost({ hasHost: false, isReadOnly: true, feedMove: null }), { kind: 'choose' });
  assert.deepEqual(reviewHost({ hasHost: true, isReadOnly: true, feedMove: { connector_id: 4 } }), { kind: 'move', connectorId: 4 });
});

test('moveNote gives a text for each method', () => {
  assert.equal(moveNote('none'), 'Studio writes nothing at the address before the move.');
  assert.match(moveNote('redirect'), /redirect/);
  assert.match(moveNote('pointer'), /pointer file/);
});

test('a track key uses the GUID, or the client ID with no GUID', () => {
  assert.equal(trackKey(model.items[0]), 'g-1');
  assert.equal(trackKey(model.items[1]), 'cid-7');
  assert.equal(isTrack(model.items[1], 'cid-7'), true);
  assert.equal(isTrack(model.items[0], 'g-1'), true);
  assert.equal(isTrack(model.items[0], 'g-2'), false);
});

test('with no open feed, the left panel gives Overview at the top (ADR 0263 item 1)', () => {
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(panel.kind, 'destinations');
  assert.equal(panel.top[0].id, 'overview');
  assert.deepEqual(panel.bottom.map((e) => e.id), ['people', 'hosts', 'settings', 'lock']);
  assert.equal(panel.top[0].current, true);
  assert.equal(JSON.stringify(panel).includes('Select a feed'), false);
});

test('Overview stays current on the Overview page', () => {
  const releases = leftPanel({ view: { name: 'overview' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(releases.top.find((e) => e.id === 'overview').current, true);
  assert.equal(releases.top[0].id, 'overview');
  assert.equal(releases.top[0].opens, false);
});

test('Unlock Studio takes the place of Lock Studio on a server with a locked vault', () => {
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [], model: null, vaultOpen: false, vaultLocked: true });
  assert.deepEqual(panel.bottom.map((e) => [e.id, e.label]), [['people', 'People'], ['hosts', 'File hosts'], ['settings', 'Settings'], ['unlock', 'Unlock Studio']]);
});

test('Lock Studio shows only when the vault is open', () => {
  const panel = leftPanel({ view: { name: 'hosts' }, feeds: [], model: null, vaultOpen: false });
  assert.deepEqual(panel.bottom.map((e) => e.id), ['people', 'hosts', 'settings']);
  assert.equal(panel.bottom[1].current, true);
});

test('an open album gives the outline of ADR 0239 item 1', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [album], model, vaultOpen: true });
  assert.equal(panel.kind, 'outline');
  assert.deepEqual(panel.feed, {
    id: 1, title: 'Night Songs', image: null, glow: 'var(--c-feed)', typeLabel: 'Album', dot: 'feed',
    page: 'album', first: false, current: false, label: 'Night Songs, Album',
  });
  assert.deepEqual(panel.items[0], { id: 'catalog', label: 'Catalog' });
  assert.deepEqual(labels(panel.items), [
    'Catalog', 'First', 'Untitled track', 'Add tracks',
  ]);
  assert.equal(panel.items[1].number, 1);
  assert.equal(panel.items[1].current, true);
  assert.equal(panel.items[2].current, false);
  assert.equal(panel.items[3].action, 'add-tracks', 'ADR 0212 item 15: the outline has one row "Add tracks". Give it the action add-tracks.');
  assert.ok(!panel.items.some((item) => item.action === 'add-track' || item.action === 'import'),
    'ADR 0212 item 15: the outline has no row with the action add-track or import. Remove it.');
});

test('a playlist has no live shows, and a publisher feed has no outline (ADR 0254 item 7)', () => {
  const withLive = { ...model, live_items: [{ _cid: 1, status: 'pending', item: { guid: 'l-1' } }] };
  const pub = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [publisher], model: withLive, vaultOpen: false });
  const list = leftPanel({ view: { name: 'feed', id: 4 }, feeds: [{ ...album, id: 4, medium: 'musicL' }], model: withLive, vaultOpen: false });
  assert.equal(labels(list.items).includes('Live shows'), false);
  assert.equal(pub.kind, 'destinations', 'ADR 0254 item 7: a publisher feed has no outline. Fix leftPanel in crates/api/ui/shell.js.');
  const outline = leftPanel({ view: { name: 'outline', id: 2 }, feeds: [publisher], model: withLive, vaultOpen: false });
  assert.equal(outline.kind, 'destinations', 'ADR 0254 item 7: the outline view of a publisher feed shows the tree.');
});

test('a medium problem view gives its repair text and menu state', () => {
  const typo = { kind: 'misspelled_tag', tag: '<podcast:meduim>music</podcast:meduim>', line: 12, suggested: 'music' };
  assert.deepEqual(mediumProblemView(typo, true), {
    title: 'Studio does not know the type of this feed',
    text: 'Line 12 has <podcast:meduim>music</podcast:meduim>. Studio reads only <podcast:medium>.',
    menu: true, start: 'music', hostLine: null,
  }, 'ADR 0250 items 4, 5 and 9: check mediumProblemView in shell.js.');
  assert.equal(mediumProblemView({ ...typo, line: null, suggested: null }, false).text,
    'This feed has <podcast:meduim>music</podcast:meduim>. Studio reads only <podcast:medium>.');
  assert.equal(mediumProblemView({ kind: 'other_namespace', tag: '<medium>music</medium>', line: 3, suggested: 'musicL' }, true).start, 'musicL');
  assert.equal(mediumProblemView({ kind: 'misspelled_value', tag: '<podcast:medium>msuic</podcast:medium>', line: 8, suggested: 'publisher' }, true).start, 'publisher');
  assert.equal(mediumProblemView({ kind: 'no_tag', tag: null, line: null, suggested: null }, false).hostLine,
    'Connect the folder that holds this file in File hosts to correct it.');
  assert.equal(mediumProblemView({ kind: 'no_tag', tag: null, line: null, suggested: null }, true).text,
    'This feed has no <podcast:medium> tag.');
});

test('a medium problem cause names each problem kind', () => {
  assert.equal(mediumProblemCause({ kind: 'misspelled_tag', tag: '<podcast:meduim>music</podcast:meduim>' }), 'misspelled tag <podcast:meduim>music</podcast:meduim>');
  assert.equal(mediumProblemCause({ kind: 'other_namespace', tag: '<medium>music</medium>' }), 'misspelled tag <medium>music</medium>');
  assert.equal(mediumProblemCause({ kind: 'misspelled_value', tag: '<podcast:medium>msuic</podcast:medium>' }), 'misspelled tag <podcast:medium>msuic</podcast:medium>');
  assert.equal(mediumProblemCause({ kind: 'no_tag', tag: null }), 'no medium tag', 'ADR 0250 item 10: check mediumProblemCause in shell.js.');
});

test('an open album before its draft loads gives the Catalog link', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model: null, vaultOpen: false });
  assert.deepEqual(labels(panel.items), ['Catalog']);
});

test('a feed that the editor does not open gives no page, and a publisher feed has no outline', () => {
  // musicindex ADR 0033 R-WORD-3: the page has the role word.
  const label = { ...publisher, publisher_roles: ['label'], roles_confirmed: true };
  const pub = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [label], model: null, vaultOpen: false });
  assert.equal(pub.kind, 'destinations', 'ADR 0254 item 7: a publisher feed has no outline. Fix leftPanel in crates/api/ui/shell.js.');
  assert.equal(pub.top.find((row) => row.group === 2).current, true, 'ADR 0254 item 7: the row of the open feed is current.');
  const podcast = { ...readOnly, medium: 'podcast' };
  const read = leftPanel({ view: { name: 'feed', id: 3 }, feeds: [podcast], model: null, vaultOpen: false });
  assert.deepEqual(labels(read.items), ['Catalog']);
  assert.equal(read.feed.label, `${read.feed.title}, ${read.feed.typeLabel}`);
  assert.equal(read.feed.current, true);
});

test('the Releases list gives the newest feed first, with the greater ID first on a tie', () => {
  const tie = { ...album, id: 9, added_at: 200 };
  const list = releasesList({ feeds: [album, publisher, readOnly, tie], connectors: [{ id: 'c1', name: 'Local' }], sort: 'newest', view: 'rows', group: 'none' });
  assert.equal(list.letters, null);
  // ADR 0085 item 6: a feed with roles opens from its destination, not from
  // a row of Releases.
  assert.deepEqual(list.rows.map((r) => r.id), [9, 1, 3]);
  assert.equal(list.rows[1].host, 'Local');
  assert.equal(list.rows[2].host, 'Read-only');
});

test('the Releases list gives each release its track count text', () => {
  const list = releasesList({ feeds: [{ ...album, track_count: 2 }, { ...album, id: 4 }], connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  assert.equal(list.rows.find((row) => row.id === 1).countText, '2 tracks');
  assert.equal(list.rows.find((row) => row.id === 4).countText, null);
});

test('the Releases list gives the draft label of each draft state', () => {
  const states = ['new', 'changed', 'unchanged', 'none'].map((s, i) => ({ ...album, id: i + 1, draft_state: s }));
  const list = releasesList({ feeds: states, connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  const byId = Object.fromEntries(list.rows.map((r) => [r.id, r.draftLabel]));
  assert.equal(byId[1], 'New, not written');
  assert.equal(byId[2], 'Saved, not written');
  assert.equal(byId[3], 'No changes');
  assert.equal(byId[4], 'No changes');
});

test('the Releases list marks a feed that apps do not list (ADR 0063)', () => {
  const feeds = [{ ...album, id: 1, hidden: true }, { ...album, id: 2 }];
  const list = releasesList({ feeds, connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  const byId = Object.fromEntries(list.rows.map((r) => [r.id, r.hidden]));
  assert.equal(byId[1], true);
  assert.equal(byId[2], false);
});

test('the A to Z sort gives the letter groups, with "#" after Z', () => {
  const feeds = [
    { id: 1, title: 'beta', medium: 'music' },
    { id: 2, title: '2 More', medium: 'music' },
    { id: 3, title: 'Alpha', medium: 'music' },
    { id: 4, title: 'Blue', medium: 'music' },
    { id: 5, title: 'Zed', medium: 'music' },
  ];
  const list = releasesList({ feeds, connectors: [], sort: 'az', view: 'tiles', group: 'none' });
  assert.equal(list.sort, 'az');
  assert.equal(list.view, 'tiles');
  assert.deepEqual(list.rows.map((r) => r.id), [3, 1, 4, 5, 2]);
  assert.deepEqual(list.letters, [
    { letter: 'A', firstId: 3 },
    { letter: 'B', firstId: 1 },
    { letter: 'Z', firstId: 5 },
    { letter: '#', firstId: 2 },
  ]);
});

test('a feed with no title sorts by its path', () => {
  const feeds = [{ id: 1, path: 'zz.xml', medium: 'music' }, { id: 2, path: 'aa.xml', medium: 'music' }];
  const list = releasesList({ feeds, connectors: [], sort: 'az', view: 'rows', group: 'none' });
  assert.deepEqual(list.rows.map((r) => r.title), ['aa.xml', 'zz.xml']);
});

test('the reverse command changes the sequence of the rows (musicindex ADR 0043 R-LIST-7)', () => {
  const feeds = [
    { id: 1, title: 'Bravo', medium: 'music', added_at: 1 },
    { id: 2, title: 'Alpha', medium: 'music', added_at: 3 },
    { id: 3, title: 'Charlie', medium: 'music', added_at: 2 },
  ];
  const ids = (sort, reversed) => releasesList({ feeds, connectors: [], sort, view: 'rows', reversed, group: 'none' }).rows.map((r) => r.id);
  assert.deepEqual(ids('newest', false), [2, 3, 1]);
  assert.deepEqual(ids('newest', true), [1, 3, 2]);
  assert.deepEqual(ids('az', false), [2, 1, 3]);
  assert.deepEqual(ids('az', true), [3, 1, 2]);
});

test('a reversed feed group reverses its feed order', () => {
  const publisher = { id: 9, title: 'Label', medium: 'publisher', roles: ['label'], release_ids: [1, 2] };
  const feeds = [publisher, { id: 1, title: 'One', medium: 'music' }, { id: 2, title: 'Two', medium: 'music' }];
  const ids = (reversed) => releasesList({ feeds, connectors: [], sort: 'order', view: 'rows', reversed, group: 9 }).rows.map((r) => r.id);
  assert.deepEqual(ids(false), [1, 2]);
  assert.deepEqual(ids(true), [2, 1]);
});

test('a reversed A to Z list shows its letters from Z to A (musicindex ADR 0043 R-LIST-7 item 7)', () => {
  const feeds = [{ id: 1, title: 'Alpha', medium: 'music' }, { id: 2, title: 'Zulu', medium: 'music' }];
  const list = releasesList({ feeds, connectors: [], sort: 'az', view: 'rows', reversed: true, group: 'none' });
  assert.deepEqual(list.letters.map((g) => g.letter), ['Z', 'A']);
  assert.equal(list.reversed, true);
});

test('each sort has its reversed name (musicindex ADR 0043 R-LIST-7 item 5)', () => {
  assert.equal(sortLabel('newest', false), 'Newest');
  assert.equal(sortLabel('newest', true), 'Oldest');
  assert.equal(sortLabel('az', false), 'A to Z');
  assert.equal(sortLabel('az', true), 'Z to A');
  assert.equal(sortLabel('order', false), 'Page Order');
  assert.equal(sortLabel('order', true), 'Page Order, Reversed');
});

test('after a feed and Back, the Releases entry has the marked feed and its scroll', () => {
  let nav = startNav({ name: 'overview' });
  nav = setScroll(nav, 120);
  nav = pushNav(nav, { name: 'feed', id: 1 });
  assert.equal(canBack(nav), true);
  nav = backNav(nav);
  assert.equal(nav.index, 0);
  assert.equal(nav.entries[0].marked, 1);
  assert.equal(nav.entries[0].scroll, 120);
  assert.equal(canForward(nav), true);
  nav = forwardNav(nav);
  assert.equal(nav.entries[nav.index].name, 'feed');
});

test('Back at index 0 on a feed puts a marked Releases entry before it', () => {
  let nav = startNav({ name: 'feed', id: 4 });
  assert.equal(canBack(nav), true);
  nav = backNav(nav);
  assert.deepEqual(nav.entries, [{ name: 'overview', marked: 4 }, { name: 'feed', id: 4 }]);
  assert.equal(nav.index, 0);
  nav = forwardNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'feed', id: 4 });
});

test('Forward is false when no entry follows, and Back is false at the start', () => {
  const nav = startNav({ name: 'overview' });
  assert.equal(canBack(nav), false);
  assert.equal(canForward(nav), false);
  assert.deepEqual(forwardNav(nav), nav);
});

test('a new view drops the entries after the current one', () => {
  let nav = startNav({ name: 'overview' });
  nav = pushNav(nav, { name: 'hosts' });
  nav = pushNav(nav, { name: 'people' });
  nav = backNav(nav);
  nav = pushNav(nav, { name: 'feed', id: 2 });
  assert.deepEqual(nav.entries.map((e) => e.name), ['overview', 'hosts', 'feed']);
  assert.equal(canForward(nav), false);
});

test('the navigation functions do not change their argument', () => {
  const nav = startNav({ name: 'overview' });
  const before = JSON.stringify(nav);
  pushNav(nav, { name: 'feed', id: 1 });
  backNav(startNav({ name: 'feed', id: 1 }));
  setScroll(nav, 50);
  forwardNav(nav);
  assert.equal(JSON.stringify(nav), before);
});

test('the narrow stack gives the root page below each view (ADR 0258 item 13)', () => {
  const root = { name: 'root' };
  const releases = { name: 'overview' };
  assert.deepEqual(narrowStack({ name: 'root' }), [root]);
  assert.deepEqual(narrowStack({ name: 'overview' }), [root, releases]);
  assert.deepEqual(narrowStack({ name: 'change-set', id: 3 }), [root, { name: 'people' }, { name: 'change-set', id: 3 }]);
  assert.deepEqual(narrowStack({ name: 'hosts' }), [root, { name: 'hosts' }]);
  assert.deepEqual(narrowStack({ name: 'outline', id: 1 }), [root, releases, { name: 'feed', id: 1 }, { name: 'outline', id: 1 }]);
  const feed = { name: 'feed', id: 1, page: 'track', track: 'g-1' };
  assert.deepEqual(narrowStack(feed), [root, releases, feed]);
});

test('a selection on Releases opens the feed at narrow width and at wide width (ADR 0223 item 8)', () => {
  assert.deepEqual(openFeedView(4, true), { name: 'feed', id: 4 });
  assert.deepEqual(openFeedView(4, false), { name: 'feed', id: 4 });
});

test('the outline at narrow width has no Catalog row, and the wide outline keeps it', () => {
  const narrow = leftPanel({ view: { name: 'outline', id: 1 }, feeds: [album], model, vaultOpen: true, narrow: true });
  assert.equal(narrow.kind, 'outline');
  assert.equal(narrow.items.some((item) => item.id === 'catalog'), false);
  assert.deepEqual(labels(narrow.items), [
    'First', 'Untitled track', 'Add tracks',
  ]);
  assert.equal(narrow.items.some((item) => item.current), false);
  const wide = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model, vaultOpen: true });
  assert.equal(wide.items[0].id, 'catalog');
});

test('the root page has no current destination, and gives People, File hosts and Settings at the bottom', () => {
  const panel = leftPanel({ view: { name: 'root' }, feeds: [album], model: null, vaultOpen: false });
  assert.equal(panel.kind, 'destinations');
  assert.equal(panel.top.every((e) => e.current === false), true);
  assert.deepEqual(panel.bottom.map((e) => e.id), ['people', 'hosts', 'settings']);
});

test('ADR 0241 items 1 and 8 and ADR 0242 items 1, 3 and 6: leftPanel adds the footer rows in sequence', () => {
  const base = { view: { name: 'root' }, feeds: [], model: null, vaultOpen: false };
  const owner = 'ADR 0241 items 1 and 8, ADR 0242 items 1, 3 and 6: fix leftPanel in crates/api/ui/shell.js.';
  const server = leftPanel({ ...base, serverMode: true, vaultLocked: true, collectionCount: 2, narrow: true });
  assert.deepEqual(server.bottom.map((item) => item.id), ['people', 'hosts', 'settings', 'unlock', 'signout'], owner);
  assert.equal(server.bottom.at(-1).label, 'Sign Out', owner);
  const two = leftPanel({ ...base, collection: 'Collection: Test', collectionCount: 2, vaultOpen: true });
  assert.deepEqual(two.bottom.map((item) => item.id), ['people', 'hosts', 'settings', 'collection', 'lock'], owner);
  assert.equal(two.bottom[3].label, 'Collection: Test', owner);
  assert.equal(two.bottom[3].mark, true, owner);
  const one = leftPanel({ ...base, collection: null, collectionCount: 1 });
  assert.deepEqual(one.bottom.map((item) => item.id), ['people', 'hosts', 'settings', 'collection'], owner);
  assert.equal(one.bottom[3].label, 'Switch Collection…', owner);
  assert.equal(one.bottom[3].mark, false, owner);
  assert.deepEqual(leftPanel({ ...base, collectionCount: 0 }).bottom.map((item) => item.id), ['people', 'hosts', 'settings'], owner);
});

test('Back from the outline marks the feed on Releases, and Back from a part keeps no mark', () => {
  let nav = { entries: narrowStack({ name: 'outline', id: 1 }), index: 3 };
  nav = backNav(nav);
  assert.equal(nav.entries[nav.index].name, 'feed');
  assert.equal(nav.entries[nav.index - 1].marked, undefined);
  nav = backNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'overview', marked: 1 });
  assert.equal(nav.index, 1);
  nav = backNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'root' });
});

test('the root page has no Back, and the outline and the feed have Back', () => {
  assert.equal(canBack({ entries: [{ name: 'root' }], index: 0 }), false);
  assert.equal(canBack({ entries: [{ name: 'outline', id: 1 }], index: 0 }), true);
  assert.equal(canBack({ entries: narrowStack({ name: 'outline', id: 1 }), index: 3 }), true);
});

test('a width change gives the stack, or the view at wide width', () => {
  const wideNav = startNav({ name: 'feed', id: 2 });
  assert.deepEqual(navAtWidth(wideNav, true), widthNav({ name: 'feed', id: 2 }, true));
  const narrowNav = { entries: narrowStack({ name: 'feed', id: 2 }), index: 2 };
  assert.deepEqual(navAtWidth(narrowNav, false), startNav({ name: 'feed', id: 2 }));
  assert.deepEqual(navAtWidth(narrowNav, true), narrowNav);
  assert.deepEqual(wideView({ name: 'outline', id: 5 }), { name: 'feed', id: 5 });
  assert.deepEqual(wideView({ name: 'root' }), { name: 'overview' });
});

test('a musicL feed is a playlist, with no regard to case', () => {
  assert.equal(isPlaylist({ medium: 'musicL' }), true);
  assert.equal(isPlaylist({ medium: 'MUSICL' }), true);
  assert.equal(isPlaylist(album), false);
  assert.equal(isPlaylist({}), false);
});

test('a playlist has only its head and the row Catalog in the outline', () => {
  const playlist = { ...album, id: 4, medium: 'musicL' };
  const panel = leftPanel({ view: { name: 'feed', id: 4, page: 'tracks' }, feeds: [playlist], model, vaultOpen: false });
  assert.deepEqual(labels(panel.items), ['Catalog'], 'ADR 0239 item 4: a playlist has no track pages.');
});

test('the tabs fit at the exact width and not at 1 px less', () => {
  const owner = 'ADR 0197 item 1: check tabsFit in shell.js.';
  // Three segments of 60 + 8 px, two gaps of 2 px and a tray padding of 2 px at each side.
  const input = { labelWidths: [60, 60, 60], gap: 2, padding: 2 };
  assert.equal(tabsFit({ ...input, available: 3 * 68 + 2 * 2 + 4 }), true, owner);
  assert.equal(tabsFit({ ...input, available: 3 * 68 + 2 * 2 + 4 - 1 }), false, owner);
});

test('each tab segment has the width of the longest label', () => {
  const owner = 'ADR 0197 item 1: check tabsFit in shell.js.';
  const input = { labelWidths: [30, 90, 40], gap: 2, padding: 2 };
  assert.equal(tabsFit({ ...input, available: 3 * 98 + 4 + 4 }), true, owner);
  assert.equal(tabsFit({ ...input, available: 3 * 98 + 4 + 4 - 1 }), false, owner);
  assert.equal(tabsFit({ labelWidths: [], gap: 2, padding: 2, available: 0 }), true, owner);
});

test('the page of a feed gives the tabs of its type', () => {
  const tabs = (feed, view) => pageTabs(feed, { name: 'feed', id: feed.id, ...view }).map((tab) => tab.label);
  const owner = 'ADR 0239 item 2: check pageTabs in shell.js.';
  assert.deepEqual(tabs(album), ['Release', 'Live Shows', 'Value Routes', 'Credits', 'Files', 'Issues', 'Settings'], 'ADR 0225 item 1: the first tab of a release is "Release". Check pageTabs in shell.js.');
  assert.equal(tabs({ ...album, kind_shown: 'single' })[0], 'Release', 'ADR 0225 item 1: the first tab does not show the kind. Check pageTabs in shell.js.');
  const artist = { ...publisher, publisher_roles: ['artist'], roles_confirmed: true };
  assert.deepEqual(tabs(artist), ['Artist', 'Value Routes', 'Credits', 'Files', 'Issues', 'Settings'], owner);
  assert.deepEqual(tabs({ ...album, medium: 'musicL' }), ['Playlist', 'Tracks', 'Value Routes', 'Files', 'Issues', 'Settings'], owner);
  assert.deepEqual(tabs({ ...readOnly, medium: 'podcast' }), [], owner);
});

test('the tab Issues comes before Settings and shows the count of the findings', () => {
  const owner = 'ADR 0174 item 17: check pageTabs in shell.js.';
  const labels = (feed, count) => pageTabs(feed, { name: 'feed', id: feed.id }, count).map((tab) => tab.label);
  const artist = { ...publisher, publisher_roles: ['artist'], roles_confirmed: true };
  for (const feed of [album, artist, { ...album, medium: 'musicL' }]) {
    const withCount = labels(feed, 5);
    assert.equal(withCount[withCount.length - 2], 'Issues (5)', owner);
    assert.equal(withCount[withCount.length - 1], 'Settings', owner);
    assert.equal(labels(feed, 0)[withCount.length - 2], 'Issues', owner);
    assert.equal(labels(feed)[withCount.length - 2], 'Issues', owner);
  }
  const current = pageTabs(album, { name: 'feed', id: 1, page: 'check' }, 2).filter((tab) => tab.current).map((tab) => tab.page);
  assert.deepEqual(current, ['check'], owner);
});

test('a track page has no tabs, and the page of a show keeps the tab Live Shows', () => {
  const owner = 'ADR 0239 items 3 and 5: check pageTabs in shell.js.';
  assert.deepEqual(pageTabs(album, { name: 'feed', id: 1, page: 'track', track: 'g-1' }), [], owner);
  const current = (page) => pageTabs(album, { name: 'feed', id: 1, page }).filter((tab) => tab.current).map((tab) => tab.page);
  assert.deepEqual(current(undefined), ['album'], owner);
  assert.deepEqual(current('live'), ['lives'], owner);
  assert.deepEqual(current('settings'), ['settings'], owner);
  const panel = (view) => leftPanel({ view: { name: 'feed', id: 1, ...view }, feeds: [album], model, vaultOpen: false }).feed;
  assert.equal(panel({ page: 'settings' }).current, true, owner);
  assert.equal(panel({ page: 'settings' }).first, false, owner);
  assert.equal(panel({ page: 'track', track: 'g-1' }).current, false, owner);
});

test('a feed row with only rss_image_url gives that address as the cover of the outline and of the release row', () => {
  const demu = { ...album, image: null, rss_image_url: 'https://m.example/demu.png' };
  const panel = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [demu], model, vaultOpen: true });
  assert.equal(panel.feed.image, 'https://m.example/demu.png');
  const list = releasesList({ feeds: [demu], connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  assert.equal(list.rows[0].image, 'https://m.example/demu.png');
});

test('the itunes:image of a feed row comes before its rss_image_url', () => {
  const both = { ...album, image: 'https://m.example/a.png', rss_image_url: 'https://m.example/b.png' };
  const list = releasesList({ feeds: [both], connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  assert.equal(list.rows[0].image, 'https://m.example/a.png');
});

test('no outline has an item for the first page of the feed (ADR 0069)', () => {
  const list = { ...album, id: 4, medium: 'musicL' };
  const feeds = [album, list, publisher, readOnly];
  for (const feed of feeds) {
    const panel = leftPanel({ view: { name: 'feed', id: feed.id }, feeds, model, vaultOpen: true });
    const ids = (panel.items || []).map((item) => item.id);
    for (const id of ['album', 'publisher', 'playlist', 'feed']) assert.equal(ids.includes(id), false);
  }
});

test('the head is current on the first page and not on a track page (ADR 0069)', () => {
  const first = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model, vaultOpen: true });
  assert.equal(first.feed.current, true);
  assert.equal(first.items.some((item) => item.current), false);
  const track = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [album], model, vaultOpen: true });
  assert.equal(track.feed.current, false);
});

test('overviewView gives the scroll of the nearest Releases entry and does not change nav (ADR 0069)', () => {
  const nav = {
    entries: [{ name: 'overview', marked: 9, scroll: 40 }, { name: 'feed', id: 1 }, { name: 'feed', id: 1, page: 'files' }],
    index: 2,
  };
  const copy = JSON.parse(JSON.stringify(nav));
  assert.deepEqual(overviewView(nav, 1), { name: 'overview', marked: 1, scroll: 40 });
  assert.deepEqual(nav, copy);
  const none = { entries: [{ name: 'feed', id: 1 }], index: 0 };
  assert.deepEqual(overviewView(none, 1), { name: 'overview', marked: 1 });
  const later = { entries: [{ name: 'feed', id: 1 }, { name: 'overview', scroll: 5 }], index: 0 };
  assert.deepEqual(overviewView(later, 1), { name: 'overview', marked: 1 });
});

test('Settings is current on the view settings', () => {
  const panel = leftPanel({ view: { name: 'settings' }, feeds: [], model: null, vaultOpen: true });
  assert.deepEqual(panel.bottom.map((e) => [e.id, e.current]), [['people', false], ['hosts', false], ['settings', true], ['lock', false]]);
});

test('A change set of a move marks File hosts and is below it (ADR 0148 item 17)', () => {
  const view = { name: 'change-set', id: 3, area: 'hosts' };
  const panel = leftPanel({ view, feeds: [], model: null, vaultOpen: true });
  assert.equal(panel.bottom.find((e) => e.id === 'hosts').current, true);
  assert.equal(panel.bottom.find((e) => e.id === 'people').current, false);
  assert.deepEqual(narrowStack(view), [{ name: 'root' }, { name: 'hosts' }, view]);
});

test('a page move change set belongs to Releases', () => {
  const view = { name: 'change-set', id: 8, area: 'releases' };
  const panel = leftPanel({ view, feeds: [], model: null, vaultOpen: true });
  assert.equal(panel.top[0].current, true, 'ADR 0243 item 3: fix leftPanel in crates/api/ui/shell.js.');
  assert.equal(panel.bottom.find((item) => item.id === 'people').current, false,
    'ADR 0243 item 3: fix leftPanel in crates/api/ui/shell.js.');
  assert.deepEqual(narrowStack(view), [{ name: 'root' }, { name: 'overview' }, view],
    'ADR 0243 item 3: fix narrowStack in crates/api/ui/shell.js.');
});

test('People is current on each view of its area (ADR 0203 item 36)', () => {
  for (const name of ['people', 'person', 'check', 'wallet', 'change-set']) {
    const panel = leftPanel({ view: { name }, feeds: [], model: null, vaultOpen: true });
    const people = panel.bottom.find((e) => e.id === 'people');
    assert.equal(people.current, true, `ADR 0203 item 36: People is current on the view ${name}.`);
    assert.equal(people.icon, 'users', 'ADR 0203 item 36: People has the Lucide icon users.');
  }
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [], model: null, vaultOpen: true });
  assert.deepEqual(panel.bottom.map((e) => e.id), ['people', 'hosts', 'settings', 'lock'],
    'ADR 0203 item 36: the bottom is People, File hosts, Settings, then Lock Studio.');
  assert.equal(panel.bottom.find((e) => e.id === 'people').current, false);
});

test('the narrow stack of the People area (ADR 0203 items 36 and 64)', () => {
  const root = { name: 'root' };
  const people = { name: 'people' };
  assert.deepEqual(narrowStack({ name: 'people' }), [root, people]);
  assert.deepEqual(narrowStack({ name: 'person', id: 4 }), [root, people, { name: 'person', id: 4 }]);
  assert.deepEqual(narrowStack({ name: 'wallet', recipient: { address: 'a' } }), [root, people, { name: 'wallet', recipient: { address: 'a' } }]);
  assert.deepEqual(narrowStack({ name: 'change-set', id: 3 }), [root, people, { name: 'change-set', id: 3 }]);
  assert.deepEqual(narrowStack({ name: 'check', id: 4 }), [root, people, { name: 'person', id: 4 }, { name: 'check', id: 4 }]);
});

test('the Role menu gives Artist, Label, Publisher or Custom for a set of roles', () => {
  // ADR 0082 item 7.
  assert.equal(roleChoice([]), '');
  assert.equal(roleChoice(['artist']), 'artist');
  assert.equal(roleChoice(['label']), 'label');
  assert.equal(roleChoice(['artist', 'host']), 'custom');
  assert.equal(roleChoice(['producer']), 'custom');
  // ADR 0083 item 2.
  assert.equal(roleChoice(['hosting']), 'hosting');
  assert.equal(roleChoice(['hosting', 'producer']), 'custom');
});

test('the role headings are Publishers, Labels, Curators, Artists and No role', () => {
  const owner = 'ADR 0253 item 16: use four role headings before No role. Fix HEADINGS in crates/api/ui/shell.js.';
  const feeds = ['artist', 'curator', 'label', 'hosting', null, 'producer'].map((role, i) => ({
    id: i + 1, medium: 'publisher', title: role || 'Bare', publisher_roles: role ? [role] : [],
  }));
  const panel = leftPanel({ view: { name: 'overview' }, feeds, model: null, vaultOpen: true });
  assert.deepEqual(panel.top.filter((row) => row.heading).map((row) => row.heading),
    ['Publishers', 'Labels', 'Curators', 'Artists', 'No role'], owner);
  assert.deepEqual(panel.top.slice(-2).map((row) => row.label), ['Bare', 'producer'], owner);
});

test('a check box adds or removes a role, and keeps one or more roles', () => {
  assert.deepEqual(toggleRole(['label'], 'artist', true), ['artist', 'label']);
  assert.deepEqual(toggleRole(['artist', 'label'], 'artist', false), ['label']);
  const last = ['host'];
  assert.equal(toggleRole(last, 'host', false), last);
});

// ---- Destinations below Releases (ADR 0085) ----

const artist = { id: 20, title: 'Zed', medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true, release_ids: [22, 21], added_at: 1 };
const label = { id: 30, title: 'Acme', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: false, release_ids: [22], added_at: 2 };
const first = { id: 21, title: 'First', medium: 'music', publisher_ids: [20], added_at: 30, draft_state: 'none' };
const shared = { id: 22, title: 'Shared', medium: 'music', publisher_ids: [20, 30], added_at: 10, draft_state: 'none' };
const mixtape = { id: 23, title: 'Mixtape', medium: 'musicL', publisher_ids: [], added_at: 20, draft_state: 'none' };
const catalog = [artist, label, first, shared, mixtape];

test('the panel gives Overview, the tree, then "No artist yet" (ADR 0254 item 2)', () => {
  const message = 'ADR 0254 item 2: the panel gives Overview, the top rows of the tree A to Z, then "No artist yet". Use groupDestinations of crates/api/ui/shell.js.';
  const panel = leftPanel({ view: { name: 'releases', group: 'none' }, feeds: catalog, model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.id, e.label, e.count]),
    [['overview', 'Overview', undefined], ['tree-30', 'Acme', 1], ['tree-20', 'Zed', 2], ['group-none', 'No artist yet', 1]], message);
  assert.deepEqual(panel.top.map((e) => e.current), [false, false, false, true], message);
  assert.equal(panel.top[0].opens, true, 'ADR 0085 item 1: on a destination below Overview, the row Overview opens Overview.');
  assert.deepEqual(panel.top.map((e) => e.dot), [undefined, 'label hollow', 'artist hollow', null], message);
  const linked = leftPanel({ view: { name: 'overview' }, feeds: [artist, first], model: null, vaultOpen: false });
  assert.deepEqual(linked.top.map((e) => e.id), ['overview', 'tree-20'], 'ADR 0254 item 2: "No artist yet" shows only when a release has no artist page.');
});

test('the page of a feed with roles gives its releases in feed order, with a note for another feed', () => {
  const message = 'ADR 0085 items 4 and 5: the releases of the feed in its sequence, and "Also on" names each other feed. Use releasesList of crates/api/ui/shell.js.';
  const list = releasesList({ feeds: catalog, connectors: [], sort: undefined, view: 'rows', group: 20 });
  assert.deepEqual(list.rows.map((r) => r.id), [22, 21], message);
  assert.equal(list.sort, 'order', message);
  assert.deepEqual(list.sorts, ['order', 'newest', 'az'], message);
  assert.deepEqual(list.head, { id: 20, title: 'Zed', typeLabel: 'Artist', dot: 'artist hollow' }, message);
  assert.deepEqual(list.rows.map((r) => r.note), ['Also on Acme', null], message);
  const az = releasesList({ feeds: catalog, connectors: [], sort: 'az', view: 'rows', group: 20 });
  assert.deepEqual(az.rows.map((r) => r.id), [21, 22]);
});

test('No artist yet gives the releases with no artist page', () => {
  const none = releasesList({ feeds: catalog, connectors: [], sort: 'order', view: 'rows', group: 'none' });
  assert.deepEqual(none.rows.map((r) => r.id), [23], 'ADR 0254 item 2: "No artist yet" gives each release with no artist page.');
  assert.equal(none.sort, 'newest', 'ADR 0254 item 2: "No artist yet" has the sorts of Releases.');
  assert.equal(none.head.title, 'No artist yet');
  assert.equal(none.head.note, 'These releases have no artist page.');
});

test('No artist yet includes a release linked only to a label page (ADR 0254 item 2)', () => {
  const lab = { id: 24, title: 'Some Label', medium: 'publisher', publisher_roles: ['label'], connector_id: 'c1' };
  const release = { id: 25, title: 'Linked elsewhere', medium: 'music', publisher_ids: [24], added_at: 25, draft_state: 'none' };
  const list = releasesList({ feeds: [lab, release], connectors: [], sort: 'newest', view: 'rows', group: 'none' });
  assert.deepEqual(list.rows.map((row) => row.id), [25]);
  assert.equal(list.head.note, 'These releases have no artist page.');
  const withArtist = { ...lab, publisher_roles: ['artist'] };
  assert.deepEqual(releasesList({ feeds: [withArtist, release], connectors: [], sort: 'newest', view: 'rows', group: 'none' }).rows, []);
});

test('a destination below Releases is a page of the root at narrow width', () => {
  const view = { name: 'releases', group: 20 };
  assert.deepEqual(narrowStack(view), [{ name: 'root' }, view], 'ADR 0085 item 7: the root page opens the destination.');
});

// ---- The Catalog tree (ADR 0254) ----

const host = { id: 40, title: 'Some Host', medium: 'publisher', publisher_roles: ['hosting'], roles_confirmed: true, release_ids: [51, 52], child_ids: [41, 42], parent_ids: [] };
const labelA = { id: 41, title: 'Label A', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [51], child_ids: [43, 44], parent_ids: [40] };
const labelB = { id: 42, title: 'Label B', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [52], child_ids: [], parent_ids: [40] };
const artistX = { id: 43, title: 'Artist X', medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [41] };
const beatLab = { id: 44, title: 'Beat Lab', medium: 'publisher', publisher_roles: ['producer'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [41] };
const album1 = { id: 51, title: 'Album 1', medium: 'music', publisher_ids: [40, 41, 43, 44], added_at: 1, draft_state: 'none' };
const album2 = { id: 52, title: 'Album 2', medium: 'music', publisher_ids: [40, 42], added_at: 2, draft_state: 'none' };
const tree = [host, labelA, labelB, artistX, beatLab, album1, album2];

const rows = (panel) => panel.top.filter((row) => row.depth !== undefined);
const catalogOf = (feeds, open = [], view = { name: 'overview' }) => rows(leftPanel({ view, feeds, model: null, vaultOpen: false, open }));

test('the top rows are the feeds with no feed above them, A to Z under their headings (ADR 0254 item 3)', () => {
  const message = 'ADR 0254 item 3: the top rows are the feeds with no feed above them, under the role headings. Use groupDestinations of crates/api/ui/shell.js.';
  const alone = { ...artistX, id: 45, title: 'Alone', parent_ids: [] };
  const zed = { ...artistX, id: 46, title: 'Zed Artist', parent_ids: [] };
  const bare = { ...artistX, id: 47, title: 'Bare', publisher_roles: [], parent_ids: [] };
  const lists = { ...artistX, id: 48, title: 'Lists', publisher_roles: ['curator'], parent_ids: [] };
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [...tree, zed, alone, bare, lists], model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.label, e.heading]), [
    ['Overview', undefined], ['Some Host', 'Publishers'], ['Lists', 'Curators'], ['Alone', 'Artists'], ['Zed Artist', undefined], ['Bare', 'No role'], ['No artist yet', undefined],
  ], message);
  assert.deepEqual(rows(panel).map((e) => e.depth), [0, 0, 0, 0, 0], message);
});

test('an open row shows the feeds directly below it, in rank, then A to Z (ADR 0254 item 5)', () => {
  const message = 'ADR 0254 item 5: below an open row, the feeds of child_ids in the rank of ADR 0253 item 15, then A to Z. Use groupDestinations of crates/api/ui/shell.js.';
  const lists = { ...beatLab, id: 46, title: 'Lists', publisher_roles: ['curator', 'producer'], parent_ids: [41] };
  const feeds = tree.map((f) => (f.id === 41 ? { ...f, child_ids: [44, 43, 46] } : f)).concat(lists);
  assert.deepEqual(catalogOf(feeds).map((r) => r.key), ['40'], 'A closed row shows no row below it.');
  assert.deepEqual(catalogOf(feeds, ['40']).map((r) => [r.key, r.depth, r.expandable, r.open]),
    [['40', 0, true, true], ['40/41', 1, true, false], ['40/42', 1, false, false]], message);
  assert.deepEqual(catalogOf(feeds, ['40', '40/41']).map((r) => r.label),
    ['Some Host', 'Label A', 'Lists', 'Artist X', 'Beat Lab', 'Label B'], message);
  assert.deepEqual(catalogOf(feeds, new Set(['40'])).map((r) => r.key), ['40', '40/41', '40/42'], 'The open rows can be a Set.');
});

test('a feed below two feeds shows below each one, with its second line (ADR 0254 items 5 and 8)', () => {
  const message = 'ADR 0254 items 5 and 8: a feed below two feeds shows below each one, and names the other feeds. Use groupDestinations of crates/api/ui/shell.js.';
  const both = { ...artistX, parent_ids: [41, 42] };
  const feeds = tree.map((f) => (f.id === 43 ? both : f.id === 42 ? { ...f, child_ids: [43] } : f));
  const open = catalogOf(feeds, ['40', '40/41', '40/42']);
  const artistRows = open.filter((r) => r.group === 43);
  assert.deepEqual(artistRows.map((r) => [r.key, r.also]), [['40/41/43', 'Also on Label B'], ['40/42/43', 'Also on Label A']], message);
  assert.equal(open.find((r) => r.key === '40/41').also, null, message);
  // With no other label, the line names the other publishers.
  const other = { id: 45, title: 'Other Host', medium: 'publisher', publisher_roles: ['hosting'], release_ids: [], child_ids: [43], parent_ids: [] };
  const mixed = tree.map((f) => (f.id === 43 ? { ...artistX, parent_ids: [41, 45] } : f)).concat(other);
  const rowsOf = catalogOf(mixed, ['40', '40/41', '45']).filter((r) => r.group === 43);
  assert.deepEqual(rowsOf.map((r) => [r.key, r.also]), [['45/43', 'Also on Label A'], ['40/41/43', 'Also published by Other Host']], message);
  const three = [...feeds, { id: 46, title: 'Label C', medium: 'publisher', publisher_roles: ['label'], release_ids: [], child_ids: [43], parent_ids: [] }]
    .map((f) => (f.id === 43 ? { ...f, parent_ids: [41, 42, 46] } : f));
  assert.equal(catalogOf(three, ['40', '40/41']).find((r) => r.key === '40/41/43').also, 'Also on Label B and Label C', message);
});

test('a row gives the releases, the pages below, or the two counts (ADR 0254 item 6)', () => {
  const message = 'ADR 0254 item 6: a hosting row gives the pages below it, a row of another level role gives its releases, and a feed with both gives both. Use groupDestinations of crates/api/ui/shell.js.';
  const both = { ...host, id: 61, title: 'Both', publisher_roles: ['artist', 'hosting'], release_ids: [51], child_ids: [42], parent_ids: [] };
  const feeds = [host, both, labelA, labelB, artistX];
  const byKey = new Map(catalogOf(feeds).map((r) => [r.key, r]));
  assert.deepEqual(byKey.get('40').counts, { releases: null, pages: 2 }, message);
  assert.equal(byKey.get('40').countText, '2', message);
  assert.equal(byKey.get('40').countLabel, '2 pages below', message);
  assert.deepEqual(byKey.get('61').counts, { releases: 1, pages: 1 }, message);
  assert.equal(byKey.get('61').countText, '1 · 1', message);
  assert.equal(byKey.get('61').countLabel, '1 release, 1 page below', message);
  const labelRows = catalogOf(feeds, ['40']);
  assert.deepEqual(labelRows.find((r) => r.key === '40/41').counts, { releases: 1, pages: null }, message);
});

test('a tree row opens the page of its feed and is current for that row only (ADR 0254 items 5 and 7)', () => {
  const message = 'ADR 0254 item 7: a selection opens the page of the feed, and the row is current. Use groupDestinations of crates/api/ui/shell.js.';
  const both = { ...artistX, parent_ids: [41, 42] };
  const feeds = tree.map((f) => (f.id === 43 ? both : f.id === 42 ? { ...f, child_ids: [43] } : f));
  const open = ['40', '40/41', '40/42'];
  const view = { name: 'feed', id: 43, from: { group: 42, trail: [40] } };
  const current = catalogOf(feeds, open, view).filter((r) => r.current).map((r) => r.key);
  assert.deepEqual(current, ['40/42/43'], message);
  const row = catalogOf(feeds, open, view).find((r) => r.key === '40/41/43');
  assert.deepEqual(row.view, { name: 'feed', id: 43, from: { group: 41, trail: [40] } }, message);
  assert.deepEqual(catalogOf(feeds, [], view).find((r) => r.key === '40').view, { name: 'feed', id: 40 }, message);
  // The page of a feed that no row names uses the feed directly above it with the highest rank.
  const plain = catalogOf(feeds, open, { name: 'feed', id: 43 }).filter((r) => r.current).map((r) => r.key);
  assert.deepEqual(plain, ['40/41/43'], message);
  assert.equal(catalogOf(feeds, open, view).find((r) => r.key === '40/42/43').opens, false, message);
  assert.deepEqual(groupDestinations(feeds, { name: 'feed', id: 51 }, open).filter((r) => r.current), [], 'A release page makes no row current.');
});

test('the path and the page above of a page follow the selected row, or else the highest rank (ADR 0254 items 12 and 12a)', () => {
  const message = 'ADR 0254 items 12 and 12a: the path follows the selected row, or else the feed directly above with the highest rank, then A to Z. Use rowPath and pagePath of crates/api/ui/shell.js.';
  const both = { ...artistX, parent_ids: [41, 42] };
  const feeds = tree.map((f) => (f.id === 43 ? both : f.id === 42 ? { ...f, child_ids: [43] } : f));
  assert.deepEqual(rowPath(feeds, 43, { group: 42, trail: [40] }), [40, 42, 43], message);
  assert.deepEqual(rowPath(feeds, 43), [40, 41, 43], message);
  // A selected row that does not hold is ignored.
  assert.deepEqual(rowPath(feeds, 43, { group: 44, trail: [40] }), [40, 41, 43], message);
  assert.deepEqual(rowPath(feeds, 40), [40], message);
  const path = pagePath(feeds, 43, { group: 42, trail: [40] });
  assert.deepEqual(path.names.map((n) => [n.id, n.title]), [[40, 'Some Host'], [42, 'Label B']], message);
  assert.deepEqual(path.names.map((n) => n.view), [{ name: 'feed', id: 40 }, { name: 'feed', id: 42, from: { group: 40, trail: [] } }], message);
  assert.equal(path.above, 42, message);
  assert.equal(path.top, false, message);
  assert.deepEqual(pagePath(feeds, 40), { names: [], above: null, top: true }, message);
  // The highest rank comes first, then A to Z.
  const hostB = { id: 45, title: 'Another Host', medium: 'publisher', publisher_roles: ['hosting'], release_ids: [], child_ids: [43], parent_ids: [] };
  const rank = feeds.map((f) => (f.id === 43 ? { ...f, parent_ids: [41, 45] } : f)).concat(hostB);
  assert.deepEqual(rowPath(rank, 43), [45, 43], message);
});

test('a new publisher feed is current and each row above it is open (ADR 0254 item 43)', () => {
  const message = 'ADR 0254 item 43: a new feed opens with its row current and each row above it open. Use revealRows and newFeedView of crates/api/ui/shell.js.';
  const from = { group: 41, trail: [40] };
  const view = newFeedView(43, from);
  assert.deepEqual([...revealRows([], from)].sort(), ['40', '40/41'], message);
  assert.deepEqual([...revealRows(['40/42'], from)].sort(), ['40', '40/41', '40/42'], message);
  const found = catalogOf(tree, revealRows([], from), view);
  assert.deepEqual(found.map((r) => r.key), ['40', '40/41', '40/41/43', '40/41/44', '40/42'], message);
  assert.deepEqual(found.filter((r) => r.current).map((r) => r.key), ['40/41/43'], message);
  assert.equal(revealRows(undefined, undefined).size, 0, message);
});

test('the button of a row opens or closes it, and does not change the set before (ADR 0254 item 5)', () => {
  const before = new Set(['40']);
  assert.deepEqual([...toggleRow(before, '40/41')].sort(), ['40', '40/41']);
  assert.deepEqual([...toggleRow(before, '40')], []);
  assert.deepEqual([...before], ['40']);
});

test('a new feed keeps the view of its source level', () => {
  assert.deepEqual(newFeedView(91, { group: 40, trail: [] }), { name: 'feed', id: 91, from: { group: 40, trail: [] } });
  assert.deepEqual(newFeedView(92, { group: 41, trail: [40] }), { name: 'feed', id: 92, from: { group: 41, trail: [40] } });
  assert.deepEqual(newFeedView(93), { name: 'feed', id: 93 });
});

test('the level tile model puts the level feed before its releases (ADR 0181 items 2 and 3)', () => {
  const owner = { ...labelA, draft_state: 'changed' };
  const release = { ...album1, draft_state: 'none' };
  const list = releasesList({ feeds: [owner, release], connectors: [], sort: 'order', view: 'tiles', group: owner.id });
  assert.deepEqual(list.levelTile, {
    id: owner.id,
    title: owner.title,
    typeLabel: 'Label',
    dot: 'label hollow',
    glow: 'var(--c-label)',
    image: null,
    draftLabel: 'Saved, not written',
  });
  assert.deepEqual(list.rows.map((row) => row.id), [release.id]);
  assert.equal(list.nested, false, 'ADR 0228 item 19: the tiles view has no indent.');
});

test('artist release rows show a differing artist and a missing page link', () => {
  const artist = { id: 90, title: 'HeyCitizen', medium: 'publisher', publisher_roles: ['artist'], release_ids: [91, 92, 93] };
  const feeds = [artist,
    { id: 91, title: 'Release A', medium: 'music', author: 'Matt Finlay', unlisted_by: [90] },
    { id: 92, title: 'Release B', medium: 'music', author: 'heycitizen ' },
    { id: 93, title: 'Release C', medium: 'music' },
  ];
  const rows = releasesList({ feeds, connectors: [], sort: 'order', view: 'rows', group: 90 }).rows;
  assert.deepEqual(rows.map((row) => [row.artistLine, row.unlistedLine]), [
    ['Artist: Matt Finlay', '⚠ HeyCitizen does not list it yet'],
    [null, null],
    [null, null],
  ]);
  assert.equal(releasesList({ feeds, connectors: [], sort: 'order', view: 'rows', group: 44 }).rows.some((row) => row.unlistedLine), false);
});

test('the release rows of a group have an indent below the level row (ADR 0228 item 19)', () => {
  const message = 'ADR 0228 item 19: the rows view of a group sets nested. Use releasesList of crates/api/ui/shell.js.';
  const owner = { ...labelA, draft_state: 'none' };
  const release = { ...album1, draft_state: 'none' };
  assert.equal(releasesList({ feeds: [owner, release], connectors: [], sort: 'order', view: 'rows', group: owner.id }).nested, true, message);
  assert.equal(releasesList({ feeds: [owner, release], connectors: [], sort: 'order', view: 'rows' }).nested, false, message);
});

test('a release opened from a page goes back to that page with the row "Catalog" (ADR 0254 items 7 and 43)', () => {
  const message = 'ADR 0254 items 7 and 43: the first outline row is "Catalog", and it opens the page that opened the release. Use outlineItems of crates/api/ui/shell.js.';
  const from = { group: 41, trail: [40] };
  const panel = leftPanel({ view: { name: 'feed', id: 51, from }, feeds: tree, model: null, vaultOpen: false });
  const up = panel.items.find((i) => i.id === 'catalog');
  assert.equal(up.label, 'Catalog', message);
  assert.deepEqual(up.target, { name: 'feed', id: 41, from: { group: 40, trail: [] } }, message);
  const top = leftPanel({ view: { name: 'feed', id: 51, from: { group: 40, trail: [] } }, feeds: tree, model: null, vaultOpen: false });
  assert.deepEqual(top.items.find((i) => i.id === 'catalog').target, { name: 'feed', id: 40 }, message);
  const plain = leftPanel({ view: { name: 'feed', id: 51 }, feeds: tree, model: null, vaultOpen: false });
  assert.deepEqual(plain.items.find((i) => i.id === 'catalog'), { id: 'catalog', label: 'Catalog' }, message);
});

test('the tree is the root page of the narrow stack, and "‹ Catalog" goes back to it (ADR 0254 item 10)', () => {
  const message = 'ADR 0254 item 10: at 760 px or less the tree is the root page. Use narrowStack of crates/api/ui/shell.js.';
  const root = { name: 'root' };
  const from = { group: 41, trail: [40] };
  const page = { name: 'feed', id: 43, from };
  assert.deepEqual(narrowStack(page, tree), [root, page], message);
  assert.deepEqual(narrowStack({ name: 'feed', id: 40 }, tree), [root, { name: 'feed', id: 40 }], message);
  const release = { name: 'feed', id: 51, from };
  const parent = { name: 'feed', id: 41, from: { group: 40, trail: [] } };
  assert.deepEqual(narrowStack(release, tree), [root, parent, release], message);
  assert.deepEqual(narrowStack({ name: 'feed', id: 51 }, tree), [root, { name: 'overview' }, { name: 'feed', id: 51 }], message);
  const outline = { name: 'outline', id: 51, from };
  assert.deepEqual(narrowStack(outline, tree), [root, parent, release, outline], message);
  assert.deepEqual(widthNav(page, true, tree).entries, [root, page], message);
  assert.deepEqual(openFeedView(51, true, from), { name: 'feed', id: 51, from });
});

test('the note of a release leaves out the feeds above and below the page', () => {
  const message = 'ADR 0254 item 9: "Also on" names only the feeds that are not above or below the page. Use relatedLevels of crates/api/ui/shell.js.';
  const labelC = { id: 46, title: 'Label C', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [] };
  const shared = { ...album1, publisher_ids: [40, 41, 43, 44, 46] };
  const list = releasesList({ feeds: [host, labelA, labelB, artistX, beatLab, labelC, shared, album2], connectors: [], sort: 'order', view: 'rows', group: 41 });
  assert.deepEqual(list.rows.map((r) => r.note), ['Also on Label C'], message);
});

test('a feed with the roles artist and label is in Artists', () => {
  const message = 'ADR 0253 item 16: the first role gives the heading. Use headingIndex of crates/api/ui/shell.js.';
  const own = { id: 60, title: 'HeyCitizen', medium: 'publisher', publisher_roles: ['artist', 'label', 'producer'], roles_confirmed: true, release_ids: [], child_ids: [], parent_ids: [] };
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [own], model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.label, e.heading]), [['Overview', undefined], ['HeyCitizen', 'Artists']], message);
});

test('a publisher feed shows its cover, then the mosaic of its releases, then the logo shape', () => {
  const message = 'musicindex ADR 0021 R-ART-3 item 2: own image, then a mosaic of the album covers, then the placeholder. Use publisherArt of crates/api/ui/shell.js.';
  const a = { id: 71, medium: 'music', image: 'https://e.x/a.jpg' };
  const b = { id: 72, medium: 'music', image: 'https://e.x/b.jpg' };
  const same = { id: 73, medium: 'music', image: 'https://e.x/a.jpg' };
  const bare = { id: 74, medium: 'music', image: null };
  const pub = { id: 70, medium: 'publisher', image: null, release_ids: [71, 72, 73, 74] };
  const feeds = [pub, a, b, same, bare];
  assert.deepEqual(publisherArt({ ...pub, image: 'https://e.x/own.jpg' }, feeds), ['https://e.x/own.jpg'], message);
  assert.deepEqual(publisherArt(pub, feeds), ['https://e.x/a.jpg', 'https://e.x/b.jpg'], message);
  assert.deepEqual(publisherArt({ ...pub, release_ids: [74] }, feeds), [], message);
  assert.deepEqual(mosaicCells(['1', '2']), ['1', '2', '2', '1']);
  assert.deepEqual(mosaicCells(['1', '2', '3']), ['1', '2', '3', '1']);
  assert.deepEqual(mosaicCells(['1', '2', '3', '4', '5']), ['1', '2', '3', '4']);
  const panel = leftPanel({ view: { name: 'overview' }, feeds, model: null, vaultOpen: false });
  assert.deepEqual(panel.top.find((e) => e.group === 70).art, ['https://e.x/a.jpg', 'https://e.x/b.jpg'], 'musicindex ADR 0204 R-DOT-3 item 6: a destination row has the art of its feed.');
});

test('a feed that the editor does not open is only in Other feeds', () => {
  const message = 'ADR 0090 item 2: Releases and "No artist yet" hold only albums and playlists, and "Other feeds" holds each other feed. Use isRelease of crates/api/ui/shell.js.';
  const show = { id: 80, title: 'A Podcast', medium: 'podcast', publisher_ids: [], added_at: 5, draft_state: 'none' };
  const single = { id: 81, title: 'A Single', medium: 'music', publisher_ids: [], added_at: 4, draft_state: 'none' };
  const feeds = [show, single];
  assert.deepEqual(releasesList({ feeds, connectors: [], sort: 'newest', view: 'rows', group: 'none' }).rows.map((r) => r.id), [81], message);
  const other = releasesList({ feeds, connectors: [], sort: 'newest', view: 'rows', group: 'other' });
  assert.deepEqual(other.rows.map((r) => r.id), [80], message);
  assert.equal(other.head.title, 'Other feeds', message);
  const panel = leftPanel({ view: { name: 'overview' }, feeds, model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.label, e.count]), [['Overview', undefined], ['No artist yet', 1], ['Other feeds', 1]], message);
  assert.equal(otherFeedText(show), 'This feed has the type “podcast”. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed.', 'ADR 0090 item 1: the page gives the type of the feed.');
  assert.match(otherFeedText({ medium: null }), /^This feed has no type\./);
});

test('a publisher feed with no role is in the heading No role', () => {
  const message = 'ADR 0090 item 2: "Other feeds" names only the feeds that the editor does not open. Use groupDestinations of crates/api/ui/shell.js.';
  const bare = { id: 90, title: 'Bare', medium: 'publisher', publisher_roles: [], release_ids: [], child_ids: [], parent_ids: [] };
  const panel = leftPanel({ view: { name: 'overview' }, feeds: [bare], model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.label, e.heading]), [['Overview', undefined], ['Bare', 'No role']], message);
});

test('each head and row with art and a type word gives its dot', () => {
  const message = 'musicindex ADR 0204 R-DOT-4: art with a type word shows its type dot. Fix: give `dot` from feedKind in shell.js.';
  const feeds = [
    { id: 20, title: 'Zed', medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true, release_ids: [1] },
    { id: 1, title: 'One', medium: 'music' },
  ];
  const list = releasesList({ feeds, connectors: [], sort: 'title', view: 'rows', group: 'none' });
  assert.ok(list.rows.length > 0);
  for (const row of list.rows) assert.ok(row.dot, message);
});


// musicindex ADR 0053 R-LIST-12: a number in a name counts as a number, and
// the case of a letter does not change the order.
test('a name order reads numbers as numbers', () => {
  assert.deepEqual(byName([{ p: '15.mp3' }, { p: '2.mp3' }, { p: '1.mp3' }, { p: 'cover.jpg' }], (f) => f.p).map((f) => f.p),
    ['1.mp3', '2.mp3', '15.mp3', 'cover.jpg']);
  assert.equal(compareNames('anna', 'Anna'), 0);
  assert.ok(compareNames('Track 9', 'Track 10') < 0);
  const titled = (id, title) => ({ ...album, id, title, path: `p${id}` });
  const list = releasesList({ feeds: [titled(1, 'Volume 10'), titled(2, 'Volume 9')], connectors: [], sort: 'az', view: 'rows', group: 'none' });
  assert.deepEqual(list.rows.map((r) => r.title), ['Volume 9', 'Volume 10']);
});

// musicindex ADR 0255 R-LIST-13 item 2, ADR 0155: each sort has two entries
// in words, and the reversed entry follows its usual entry.
test('the sort menu names both directions of each sort', () => {
  const labels = (sorts) => sortEntries(sorts).map((e) => e.label);
  assert.deepEqual(labels(['newest', 'az']), ['Newest', 'Oldest', 'A to Z', 'Z to A']);
  assert.deepEqual(labels(['order', 'newest', 'az']),
    ['Page Order', 'Page Order, Reversed', 'Newest', 'Oldest', 'A to Z', 'Z to A']);
  assert.deepEqual(sortEntries(['az'])[1], { sort: 'az', reversed: true, label: 'Z to A' });
});

// musicindex ADR 0051 R-SEG-1 item 7, ADR 0157: the keys of a segmented control.
test('the arrow keys, Home and End move the focus between the segments', () => {
  assert.equal(segmentKey(0, 6, 'ArrowRight'), 1);
  assert.equal(segmentKey(5, 6, 'ArrowRight'), 0, 'Right wraps to the first segment');
  assert.equal(segmentKey(0, 6, 'ArrowLeft'), 5, 'Left wraps to the last segment');
  assert.equal(segmentKey(3, 6, 'Home'), 0);
  assert.equal(segmentKey(3, 6, 'End'), 5);
  assert.equal(segmentKey(3, 6, 'Enter'), null, 'Enter selects through the click of the button');
  assert.equal(segmentKey(0, 0, 'ArrowRight'), null);
});

test('the releases that keep a page of a role show in one fold for each role', () => {
  const owner = 'ADR 0257 item 10a: name the releases that keep a page of a role. Fix keptRoleFolds in crates/api/ui/shell.js.';
  assert.deepEqual(keptRoleFolds(undefined), [], owner);
  assert.deepEqual(keptRoleFolds({}), [], owner);
  assert.deepEqual(keptRoleFolds({
    artist: [{ id: 3, title: 'Low Tide' }],
    hosting: [{ id: 4, title: 'Harbor EP' }, { id: 5, title: '' }],
  }), [
    { summary: '1 release keeps its artist page.', names: ['Low Tide'] },
    { summary: '2 releases keep their publisher page.', names: ['Harbor EP', 'Untitled release'] },
  ], owner);
});

// ---- The page of a publisher feed (ADR 0254 items 11 to 17) ----

const groupIds = (feed, feeds, options) => publisherGroups(feed, feeds, options).map((g) => g.id);

test('a publisher page gives the groups of its level roles, from the highest rank (ADR 0254 items 6b and 13)', () => {
  const message = 'ADR 0254 item 13: a publisher gives "Labels and artists published here", a label gives "Artists on this label" then Releases, an artist gives Releases. Fix publisherGroups in crates/api/ui/shell.js.';
  const groups = publisherGroups(host, tree);
  assert.deepEqual(groups.map((g) => [g.heading, g.add.map((a) => a.label), g.rows.map((r) => r.title)]), [
    ['Labels and artists published here', ['＋ Publish a label', '＋ Publish an artist'], ['Label A', 'Label B']],
  ], message);
  const label = publisherGroups(labelA, tree);
  assert.deepEqual(label.map((g) => [g.heading, g.add.map((a) => a.label), g.rows.map((r) => r.title)]), [
    ['Artists on this label', ['＋ Sign an artist'], ['Artist X']],
    ['Releases', [], ['Album 1']],
    ['No role', [], ['Beat Lab']],
  ], message);
  const curator = { ...artistX, id: 70, publisher_roles: ['curator'], release_ids: [51] };
  assert.deepEqual(publisherGroups(curator, [...tree, curator]).map((g) => [g.id, g.add.length]), [['releases', 0]], message);
  assert.deepEqual(publisherGroups(artistX, tree).map((g) => [g.id, g.add.map((a) => [a.label, a.ready])]),
    [['releases', [['＋ Add release', false]]]], 'An artist with no connector has no start parent, so its add row is not ready.');
  const writable = { ...artistX, connector_id: 'c1' };
  assert.deepEqual(publisherGroups(writable, tree)[0].add, [{ id: 'add-release', label: '＋ Add release', ready: true }], message);
});

test('a feed with the roles artist and hosting gives the publisher group, then Releases (ADR 0254 item 6b)', () => {
  const message = 'ADR 0254 item 6b: a feed with several level roles gives the groups of each role, from the highest rank. Fix publisherGroups in crates/api/ui/shell.js.';
  const both = { ...artistX, id: 61, title: 'Both', publisher_roles: ['artist', 'hosting'], release_ids: [51], child_ids: [41, 43], parent_ids: [] };
  const groups = publisherGroups(both, [...tree, both]);
  assert.deepEqual(groups.map((g) => g.id), ['pages', 'releases'], message);
  assert.deepEqual(groups[0].rows.map((r) => r.title), ['Label A', 'Artist X'], message);
  assert.deepEqual(groups[1].rows.map((r) => r.title), ['Album 1'], message);
  // A label of a feed with the roles label and hosting claims the artists.
  const both2 = { ...labelA, id: 62, publisher_roles: ['label', 'hosting'], child_ids: [41, 43], parent_ids: [] };
  const second = publisherGroups(both2, [...tree, both2]);
  assert.deepEqual(second.map((g) => [g.id, g.rows.map((r) => r.title)]),
    [['pages', ['Label A']], ['artists', ['Artist X']], ['releases', ['Album 1']]], message);
});

test('an empty group says its text (ADR 0254 item 14)', () => {
  const message = 'ADR 0254 item 14: an empty group says "Nothing published here yet.", "No artists on this label yet." or "No releases on this page.". Fix publisherGroups in crates/api/ui/shell.js.';
  const emptyHost = { ...host, child_ids: [], release_ids: [] };
  const emptyLabel = { ...labelB, child_ids: [], release_ids: [] };
  const emptyArtist = { ...artistX, release_ids: [] };
  assert.deepEqual(publisherGroups(emptyHost, [emptyHost]).map((g) => [g.rows.length, g.empty]), [[0, 'Nothing published here yet.']], message);
  assert.deepEqual(publisherGroups(emptyLabel, [emptyLabel]).map((g) => [g.rows.length, g.empty]),
    [[0, 'No artists on this label yet.'], [0, 'No releases on this page.']], message);
  assert.deepEqual(publisherGroups(emptyArtist, [emptyArtist]).map((g) => [g.rows.length, g.empty]), [[0, 'No releases on this page.']], message);
});

test('the group Releases has the sorts Feed order, Newest and A to Z (ADR 0254 item 13a)', () => {
  const message = 'ADR 0254 item 13a: Releases has the sorts Feed order (default), Newest and A to Z, and the sort is in the view. Fix publisherGroups and pageSort in crates/api/ui/shell.js.';
  const b = { id: 81, title: 'B Album', medium: 'music', added_at: 5 };
  const a = { id: 82, title: 'A Album', medium: 'music', added_at: 9 };
  const c = { id: 83, title: 'C Album', medium: 'music', added_at: 1 };
  const owner = { ...artistX, release_ids: [81, 83, 82] };
  const titles = (options) => publisherGroups(owner, [owner, a, b, c], options).find((g) => g.id === 'releases').rows.map((r) => r.title);
  assert.deepEqual(titles(), ['B Album', 'C Album', 'A Album'], message);
  assert.deepEqual(titles({ sort: 'newest' }), ['A Album', 'B Album', 'C Album'], message);
  assert.deepEqual(titles({ sort: 'az' }), ['A Album', 'B Album', 'C Album'], message);
  assert.deepEqual(titles({ sort: 'other' }), ['B Album', 'C Album', 'A Album'], message);
  assert.deepEqual(PAGE_SORTS.map(pageSortLabel), ['Feed order', 'Newest', 'A to Z'], message);
  assert.equal(pageSort({ name: 'feed', sort: 'az' }), 'az', message);
  assert.equal(pageSort({ name: 'feed', sort: 'x' }), 'order', message);
  assert.equal(pageSort(undefined), 'order', message);
});

test('a group of a different role heading shows only with rows and has no add row (ADR 0254 item 13)', () => {
  const message = 'ADR 0254 item 13: a group of a different role heading shows only when it has rows, and it has no add row. Fix publisherGroups in crates/api/ui/shell.js.';
  const curator = { ...artistX, id: 71, title: 'Lists', publisher_roles: ['curator'], parent_ids: [40] };
  const owner = { ...host, child_ids: [41, 71] };
  const groups = publisherGroups(owner, [owner, labelA, curator]);
  assert.deepEqual(groups.map((g) => [g.id, g.heading, g.add.length]), [['pages', 'Labels and artists published here', 2], ['other-2', 'Curators', 0]], message);
  assert.deepEqual(groupIds(host, [host]), ['pages'], message);
});

test('a row of a group opens its page with the path of this page (ADR 0254 items 12a and 16)', () => {
  const message = 'ADR 0254 items 12a and 16: a row opens the page of its feed and keeps the path of this page. Fix publisherGroups in crates/api/ui/shell.js.';
  const groups = publisherGroups(labelA, tree, { from: { group: 40, trail: [] } });
  const artist = groups[0].rows[0];
  assert.deepEqual(artist.view, { name: 'feed', id: 43, from: { group: 41, trail: [40] } }, message);
  assert.deepEqual([artist.dot, artist.typeLabel, artist.countText], ['artist hollow', 'Artist', '1 release'], message);
  assert.equal(publisherGroups(host, tree)[0].rows[0].countText, '1 release', message);
  const hosts = { ...host, id: 95, child_ids: [], parent_ids: [] };
  const upper = { ...host, id: 96, title: 'Upper', child_ids: [95] };
  assert.deepEqual(publisherGroups(upper, [upper, hosts]).map((g) => [g.heading, g.rows.map((r) => r.countText)]),
    [['Labels and artists published here', []], ['Publishers', ['0 pages']]], message);
  assert.deepEqual(groups[1].rows[0].view, { name: 'feed', id: 51, from: { group: 41, trail: [40] } }, message);
});

test('the place of a feed uses music words (ADR 0254 item 11)', () => {
  const message = 'ADR 0254 item 11: the place reads "On {label}, published by {publisher}", "Independent, published by {publisher}" or "Not published". Fix placeText in crates/api/ui/shell.js.';
  const north = { ...labelA, id: 90, title: 'Northwind Records', parent_ids: [91] };
  const river = { ...host, id: 91, title: 'Riverside', child_ids: [90, 92], parent_ids: [] };
  const onLabel = { ...artistX, id: 93, parent_ids: [90] };
  const indep = { ...artistX, id: 92, parent_ids: [91] };
  const lone = { ...artistX, id: 94, parent_ids: [] };
  const feeds = [north, river, onLabel, indep, lone];
  assert.equal(placeText(onLabel, feeds), 'On Northwind Records, published by Riverside', message);
  assert.equal(placeText(indep, feeds), 'Independent, published by Riverside', message);
  assert.equal(placeText(lone, feeds), 'Independent, not published', message);
  assert.equal(placeText(north, feeds), 'Published by Riverside', message);
  assert.equal(placeText({ ...north, parent_ids: [] }, feeds), 'Not published', message);
  assert.equal(placeText(river, feeds), null, message);
  // A label that is also a publisher publishes itself, so it has no place line.
  assert.equal(placeText({ ...north, parent_ids: [], publisher_roles: ['label', 'hosting'] }, feeds), null, message);
  assert.equal(draftLabel({ draft_state: 'changed' }), 'Saved, not written', message);
  assert.equal(draftLabel({}), 'No changes', message);
});

// Overview (ADR 0263 and ADR 0264).
const pg = (id, title, roles, extra = {}) => ({ id, title, medium: 'publisher', publisher_roles: roles, role_set: roles, ...extra });
const rel = (id, title, pubs, extra = {}) => ({ id, title, medium: 'music', publisher_ids: pubs, ...extra });
const pl = (id, title, pubs, extra = {}) => ({ id, title, medium: 'musicl', publisher_ids: pubs, ...extra });
const heads = (model, part) => (model.parts.find((p) => p.id === part) || { headings: [] }).headings;
const tileIds = (h) => (h.children.length > 0 ? h.children.flatMap(tileIds) : h.tiles.map((t) => t.id));

test('ADR 0264 items 1 to 3: label, artist, Independent and No artist yet headings, each release once', () => {
  const feeds = [
    pg(10, 'Label A', ['label']), pg(11, 'Artist B', ['artist']), pg(12, 'Artist C', ['artist']),
    rel(1, 'R1', [10, 11]), rel(2, 'R2', [11]), rel(3, 'R3', []), rel(4, 'R4', [10]), rel(5, 'R5', [12]),
  ];
  const m = overviewModel({ feeds });
  const h = heads(m, 'releases');
  assert.deepEqual(h.map((x) => x.title), ['Label A', 'Independent', 'No artist yet']);
  assert.equal(h[0].level, 1);
  assert.deepEqual(h[0].children.map((x) => x.title), ['Artist B', 'No artist yet']);
  assert.equal(h[0].children[0].level, 2);
  assert.deepEqual(h[1].children.map((x) => x.title), ['Artist B', 'Artist C']);
  const all = h.flatMap(tileIds).sort();
  assert.deepEqual(all, [1, 2, 3, 4, 5]);
  assert.equal(h[0].count, 2);
});

test('ADR 0264 item 4: tiles sort by newest date, then no date, then A to Z', () => {
  const feeds = [
    pg(11, 'Artist B', ['artist']),
    rel(1, 'Zed', [11], { release_date: '2020-01-01' }), rel(2, 'Beta', [11]), rel(3, 'Alpha', [11]),
    rel(4, 'New', [11], { release_date: '2024-05-05' }),
  ];
  const h = heads(overviewModel({ feeds }), 'releases');
  assert.deepEqual(tileIds(h[0]), [4, 1, 3, 2]);
});

test('ADR 0264 item 5: a playlist shows below each curator, and No curator yet comes last', () => {
  const feeds = [pg(20, 'Cur 1', ['curator']), pg(21, 'Cur 2', ['curator']), pl(7, 'P7', [20, 21]), pl(8, 'P8', [])];
  const h = heads(overviewModel({ feeds }), 'playlists');
  assert.deepEqual(h.map((x) => x.title), ['Cur 1', 'Cur 2', 'No curator yet']);
  assert.deepEqual(tileIds(h[0]), [7]);
  assert.deepEqual(tileIds(h[1]), [7]);
  assert.deepEqual(tileIds(h[2]), [8]);
});

test('ADR 0264 items 6 and 7: filters and search hide parts and empty headings', () => {
  const feeds = [pg(11, 'Artist B', ['artist']), rel(1, 'Night', [11]), rel(2, 'Day', [11]), pl(7, 'P7', [])];
  assert.deepEqual(overviewModel({ feeds, filter: 'releases' }).parts.map((p) => p.id), ['releases']);
  assert.deepEqual(overviewModel({ feeds, filter: 'playlists' }).parts.map((p) => p.id), ['playlists']);
  const found = overviewModel({ feeds, query: 'night' });
  assert.deepEqual(found.parts.map((p) => p.id), ['releases']);
  assert.deepEqual(tileIds(heads(found, 'releases')[0]), [1]);
  assert.deepEqual(overviewModel({ feeds, query: 'artist b' }).parts.flatMap((p) => p.headings).flatMap(tileIds).sort(), [1, 2]);
  const none = overviewModel({ feeds, query: 'zzz' });
  assert.equal(none.empty, true);
  assert.ok(none.emptyText.includes('zzz'));
  assert.equal(overviewModel({ feeds: [], filter: 'playlists' }).emptyText, 'No playlists yet.');
  assert.equal(overviewModel({ feeds: [] }).emptyText, 'No releases yet.');
});

test('ADR 0263 item 3: the counts line leaves out a zero count, and each notice opens its target', () => {
  const feeds = [pg(11, 'Artist B', ['artist']), rel(1, 'R1', [], { draft_state: 'new' }), pl(7, 'P7', [])];
  const m = overviewModel({ feeds });
  assert.ok(m.countsText.includes('1 artist'));
  assert.ok(!m.countsText.includes('label'));
  assert.ok(!m.countsText.includes('0'));
  const by = Object.fromEntries(m.notices.map((n) => [n.id, n.view]));
  assert.deepEqual(by['no-artist'], { name: 'releases', group: 'none' });
  assert.deepEqual(by['no-curator'], { name: 'releases', group: 'no-curator' });
  assert.deepEqual(by['not-written'], openFeedView(1, false, undefined));
  assert.deepEqual(overviewModel({ feeds: [pg(11, 'Artist B', ['artist'])] }).notices, []);
  assert.equal(unlinkedPlaylists(feeds).length, 1);
});

test('ADR 0263 item 8: an old Releases view and an old address open Overview', () => {
  assert.deepEqual(legacyView({ name: 'releases', sort: 'az', marked: 4, scroll: 9 }), { name: 'overview', marked: 4, scroll: 9 });
  const group = { name: 'releases', group: 'none' };
  assert.equal(legacyView(group), group);
  const nav = legacyNav({ entries: [{ name: 'releases' }, group], index: 1 });
  assert.equal(nav.entries[0].name, 'overview');
  assert.equal(nav.entries[1], group);
  assert.equal(addressSearch('?sort=az&reversed=1&view=tiles&x=2'), '?x=2');
  assert.equal(addressSearch('?sort=az'), '');
});
