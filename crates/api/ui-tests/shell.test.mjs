// The shell models (ADR 0042). Each test checks one rule of ADR 0039 and of
// musicindex ADR 0027 R-NAV-2.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  mediumKind, roleChoice, toggleRole, isEditable, isReadOnly, reviewHost, moveNote, isPublisher, isPlaylist, trackKey, isTrack,
  leftPanel, releasesList, listQuery, queryText,
  startNav, pushNav, backNav, forwardNav, canBack, canForward, setScroll, releasesView,
  narrowStack, openFeedView, navAtWidth, widthNav, wideView,
} from '../ui/shell.js';

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

test('the medium kind gives the dot and the label', () => {
  assert.deepEqual(mediumKind('publisher', ['label'], true), { dot: 'label', label: 'Label' });
  assert.deepEqual(mediumKind('MUSIC'), { dot: 'feed', label: 'Album' });
  assert.deepEqual(mediumKind('musicL'), { dot: 'playlist', label: 'Playlist' });
  assert.deepEqual(mediumKind(undefined), { dot: 'feed', label: 'Feed' });
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

test('with no open feed, the left panel gives Releases at the top', () => {
  const panel = leftPanel({ view: { name: 'releases' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(panel.kind, 'destinations');
  assert.equal(panel.top[0].id, 'releases');
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts', 'settings', 'lock']);
  assert.equal(panel.top[0].current, true);
  assert.equal(JSON.stringify(panel).includes('Select a feed'), false);
});

test('the row Releases is current on the view add-feed (ADR 0072 item 5)', () => {
  const panel = leftPanel({ view: { name: 'add-feed' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(panel.top.find((e) => e.id === 'releases').current, true, 'ADR 0072 item 5: Releases is current on Add feeds.');
  assert.equal(panel.top[0].id, 'releases', 'ADR 0072 item 1: Releases is the first destination.');
  assert.equal(panel.top[0].opens, true, 'ADR 0072 item 5: on Add feeds, the row Releases must open Releases. Set opens in leftPanel of crates/api/ui/shell.js.');
  const releases = leftPanel({ view: { name: 'releases' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(releases.top[0].opens, false);
});

test('Unlock Studio takes the place of Lock Studio on a server with a locked vault', () => {
  const panel = leftPanel({ view: { name: 'releases' }, feeds: [], model: null, vaultOpen: false, vaultLocked: true });
  assert.deepEqual(panel.bottom.map((e) => [e.id, e.label]), [['hosts', 'Hosts'], ['settings', 'Settings'], ['unlock', 'Unlock Studio']]);
});

test('Lock Studio shows only when the vault is open', () => {
  const panel = leftPanel({ view: { name: 'hosts' }, feeds: [], model: null, vaultOpen: false });
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts', 'settings']);
  assert.equal(panel.bottom[0].current, true);
});

test('an open album gives the outline in the sequence of ADR 0039 item 6', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [album], model, vaultOpen: true });
  assert.equal(panel.kind, 'outline');
  assert.deepEqual(panel.feed, {
    id: 1, title: 'Night Songs', image: null, glow: 'var(--c-feed)', typeLabel: 'Album',
    page: 'album', current: false, label: 'Night Songs, Album',
  });
  assert.deepEqual(panel.items[0], { id: 'releases', label: 'Releases' });
  assert.deepEqual(labels(panel.items), [
    'Releases', 'First', 'Untitled track', 'Add tracks',
    'Live shows', 'Add a live show', 'Value routes', 'Credits and links', 'Feed settings', 'Files',
  ]);
  assert.equal(panel.items[1].number, 1);
  assert.equal(panel.items[1].current, true);
  assert.equal(panel.items[2].current, false);
  assert.equal(panel.items[3].action, 'add-tracks', 'ADR 0073 item 11: the outline has one row "Add tracks". Give it the action add-tracks.');
  assert.ok(!panel.items.some((item) => item.action === 'add-track' || item.action === 'import'),
    'ADR 0073 item 11: the outline has no row with the action add-track or import. Remove it.');
  assert.equal(panel.items[4].heading, true);
  assert.equal(panel.items[5].action, 'add-live');
  assert.equal(panel.items[6].page, 'recipients');
});

test('the live shows come after the tracks, one entry for each live item', () => {
  const withLive = {
    ...model,
    live_items: [
      { _cid: 11, status: 'pending', item: { guid: 'l-1', title: 'Launch' } },
      { _cid: 12, status: 'live', item: { title: '' } },
    ],
  };
  const view = { name: 'feed', id: 1, page: 'live', track: 'l-1' };
  const panel = leftPanel({ view, feeds: [album], model: withLive, vaultOpen: false });
  assert.deepEqual(labels(panel.items).slice(3, 9), [
    'Add tracks', 'Live shows', 'Launch', 'Untitled show', 'Add a live show', 'Value routes',
  ]);
  const entry = panel.items[5];
  assert.equal(entry.page, 'live');
  assert.equal(entry.track, 'l-1');
  assert.equal(entry.status, 'Planned');
  assert.equal(entry.current, true);
  assert.equal(panel.items[6].track, 'cid-12');
  assert.equal(panel.items[6].status, 'Live now');
  assert.equal(panel.items[6].current, false);
  assert.equal(panel.items[7].action, 'add-live');
  // The track entries do not become current on a live page.
  assert.equal(panel.items.filter((i) => i.current).length, 1);
});

test('a playlist and a publisher feed have no live shows', () => {
  const withLive = { ...model, live_items: [{ _cid: 1, status: 'pending', item: { guid: 'l-1' } }] };
  const pub = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [publisher], model: withLive, vaultOpen: false });
  const list = leftPanel({ view: { name: 'feed', id: 4 }, feeds: [{ ...album, id: 4, medium: 'musicL' }], model: withLive, vaultOpen: false });
  for (const panel of [pub, list]) {
    assert.equal(labels(panel.items).includes('Live shows'), false);
  }
});

test('an open album before its draft loads gives the Releases link and Files', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model: null, vaultOpen: false });
  assert.deepEqual(labels(panel.items), ['Releases', 'Files']);
});

test('a publisher feed gives Feed settings and Files, and a feed that the editor does not open gives no page', () => {
  // musicindex ADR 0033 R-WORD-3: the page has the role word.
  const label = { ...publisher, publisher_roles: ['label'], roles_confirmed: true };
  const pub = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [label], model: null, vaultOpen: false });
  assert.deepEqual(labels(pub.items), ['Releases', 'Feed settings', 'Files']);
  const noRole = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [publisher], model: null, vaultOpen: false });
  assert.deepEqual(labels(noRole.items), ['Releases', 'Feed settings', 'Files']);
  const podcast = { ...readOnly, medium: 'podcast' };
  const read = leftPanel({ view: { name: 'feed', id: 3 }, feeds: [podcast], model: null, vaultOpen: false });
  assert.deepEqual(labels(read.items), ['Releases']);
  assert.equal(pub.feed.label, `${pub.feed.title}, Label`);
  assert.equal(read.feed.label, `${read.feed.title}, ${read.feed.typeLabel}`);
  assert.equal(read.feed.current, true);
});

test('the Releases list gives the newest feed first, with the greater ID first on a tie', () => {
  const tie = { ...album, id: 9, added_at: 200 };
  const list = releasesList({ feeds: [album, publisher, readOnly, tie], connectors: [{ id: 'c1', name: 'Local' }], sort: 'newest', view: 'rows' });
  assert.equal(list.letters, null);
  // ADR 0085 item 6: a feed with roles opens from its destination, not from
  // a row of Releases.
  assert.deepEqual(list.rows.map((r) => r.id), [9, 1, 3]);
  assert.equal(list.rows[1].host, 'Local');
  assert.equal(list.rows[2].host, 'Read-only');
});

test('the Releases list gives the draft label of each draft state', () => {
  const states = ['new', 'changed', 'unchanged', 'none'].map((s, i) => ({ ...album, id: i + 1, draft_state: s }));
  const list = releasesList({ feeds: states, connectors: [], sort: 'newest', view: 'rows' });
  const byId = Object.fromEntries(list.rows.map((r) => [r.id, r.draftLabel]));
  assert.equal(byId[1], 'New, not written');
  assert.equal(byId[2], 'Saved, not written');
  assert.equal(byId[3], 'No changes');
  assert.equal(byId[4], 'No changes');
});

test('the Releases list marks a feed that apps do not list (ADR 0063)', () => {
  const feeds = [{ ...album, id: 1, hidden: true }, { ...album, id: 2 }];
  const list = releasesList({ feeds, connectors: [], sort: 'newest', view: 'rows' });
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
  const list = releasesList({ feeds, connectors: [], sort: 'az', view: 'tiles' });
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
  const list = releasesList({ feeds, connectors: [], sort: 'az', view: 'rows' });
  assert.deepEqual(list.rows.map((r) => r.title), ['aa.xml', 'zz.xml']);
});

test('the query gives the sort and the view, with the defaults for unknown values', () => {
  assert.deepEqual(listQuery('?sort=az&view=tiles'), { sort: 'az', view: 'tiles' });
  assert.deepEqual(listQuery('sort=other&view=x'), { sort: 'newest', view: 'rows' });
  assert.deepEqual(listQuery(''), { sort: 'newest', view: 'rows' });
});

test('the query text holds only the values that are not the default', () => {
  assert.equal(queryText({ sort: 'az', view: 'tiles' }), 'sort=az&view=tiles');
  assert.equal(queryText({ sort: 'newest', view: 'tiles' }), 'view=tiles');
  assert.equal(queryText({ sort: 'newest', view: 'rows' }), '');
});

test('after a feed and Back, the Releases entry has the marked feed and its scroll', () => {
  let nav = startNav({ name: 'releases' });
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
  assert.deepEqual(nav.entries, [{ name: 'releases', marked: 4 }, { name: 'feed', id: 4 }]);
  assert.equal(nav.index, 0);
  nav = forwardNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'feed', id: 4 });
});

test('Forward is false when no entry follows, and Back is false at the start', () => {
  const nav = startNav({ name: 'releases' });
  assert.equal(canBack(nav), false);
  assert.equal(canForward(nav), false);
  assert.deepEqual(forwardNav(nav), nav);
});

test('a new view drops the entries after the current one', () => {
  let nav = startNav({ name: 'releases' });
  nav = pushNav(nav, { name: 'hosts' });
  nav = pushNav(nav, { name: 'add-feed' });
  nav = backNav(nav);
  nav = pushNav(nav, { name: 'feed', id: 2 });
  assert.deepEqual(nav.entries.map((e) => e.name), ['releases', 'hosts', 'feed']);
  assert.equal(canForward(nav), false);
});

test('the navigation functions do not change their argument', () => {
  const nav = startNav({ name: 'releases' });
  const before = JSON.stringify(nav);
  pushNav(nav, { name: 'feed', id: 1 });
  backNav(startNav({ name: 'feed', id: 1 }));
  setScroll(nav, 50);
  forwardNav(nav);
  assert.equal(JSON.stringify(nav), before);
});

test('the narrow stack gives the root page below each view (ADR 0039 item 11)', () => {
  const root = { name: 'root' };
  const releases = { name: 'releases' };
  assert.deepEqual(narrowStack({ name: 'root' }), [root]);
  assert.deepEqual(narrowStack({ name: 'releases' }), [root, releases]);
  assert.deepEqual(narrowStack({ name: 'hosts' }), [root, { name: 'hosts' }]);
  assert.deepEqual(narrowStack({ name: 'add-feed' }), [root, releases, { name: 'add-feed' }]);
  assert.deepEqual(narrowStack({ name: 'outline', id: 1 }), [root, releases, { name: 'outline', id: 1 }]);
  const feed = { name: 'feed', id: 1, page: 'track', track: 'g-1' };
  assert.deepEqual(narrowStack(feed), [root, releases, { name: 'outline', id: 1 }, feed]);
});

test('a selection on Releases opens the outline at narrow width and the feed at wide width', () => {
  assert.deepEqual(openFeedView(4, true), { name: 'outline', id: 4 });
  assert.deepEqual(openFeedView(4, false), { name: 'feed', id: 4 });
});

test('the outline at narrow width has no Releases row, and the wide outline keeps it', () => {
  const narrow = leftPanel({ view: { name: 'outline', id: 1 }, feeds: [album], model, vaultOpen: true, narrow: true });
  assert.equal(narrow.kind, 'outline');
  assert.equal(narrow.items.some((item) => item.id === 'releases'), false);
  assert.deepEqual(labels(narrow.items), [
    'First', 'Untitled track', 'Add tracks', 'Live shows', 'Add a live show',
    'Value routes', 'Credits and links', 'Feed settings', 'Files',
  ]);
  assert.equal(narrow.items.some((item) => item.current), false);
  const wide = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model, vaultOpen: true });
  assert.equal(wide.items[0].id, 'releases');
});

test('the root page has no current destination, and gives Hosts and Settings at the bottom', () => {
  const panel = leftPanel({ view: { name: 'root' }, feeds: [album], model: null, vaultOpen: false });
  assert.equal(panel.kind, 'destinations');
  assert.equal(panel.top.every((e) => e.current === false), true);
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts', 'settings']);
});

test('Back from the outline marks the feed on Releases, and Back from a part keeps no mark', () => {
  let nav = { entries: narrowStack({ name: 'feed', id: 1 }), index: 3 };
  nav = backNav(nav);
  assert.equal(nav.entries[nav.index].name, 'outline');
  assert.equal(nav.entries[nav.index - 1].marked, undefined);
  nav = backNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'releases', marked: 1 });
  assert.equal(nav.index, 1);
  nav = backNav(nav);
  assert.deepEqual(nav.entries[nav.index], { name: 'root' });
});

test('the root page has no Back, and the outline and the feed have Back', () => {
  assert.equal(canBack({ entries: [{ name: 'root' }], index: 0 }), false);
  assert.equal(canBack({ entries: [{ name: 'outline', id: 1 }], index: 0 }), true);
  assert.equal(canBack({ entries: narrowStack({ name: 'outline', id: 1 }), index: 2 }), true);
});

test('a width change gives the stack, or the view at wide width', () => {
  const wideNav = startNav({ name: 'feed', id: 2 });
  assert.deepEqual(navAtWidth(wideNav, true), widthNav({ name: 'feed', id: 2 }, true));
  const narrowNav = { entries: narrowStack({ name: 'feed', id: 2 }), index: 3 };
  assert.deepEqual(navAtWidth(narrowNav, false), startNav({ name: 'feed', id: 2 }));
  assert.deepEqual(navAtWidth(narrowNav, true), narrowNav);
  assert.deepEqual(wideView({ name: 'outline', id: 5 }), { name: 'feed', id: 5 });
  assert.deepEqual(wideView({ name: 'root' }), { name: 'releases' });
});

test('a musicL feed is a playlist, with no regard to case', () => {
  assert.equal(isPlaylist({ medium: 'musicL' }), true);
  assert.equal(isPlaylist({ medium: 'MUSICL' }), true);
  assert.equal(isPlaylist(album), false);
  assert.equal(isPlaylist({}), false);
});

test('a playlist gives Playlist, Tracks, Value routes and Files, with no add or import entry', () => {
  const playlist = { ...album, id: 4, medium: 'musicL' };
  const panel = leftPanel({ view: { name: 'feed', id: 4, page: 'tracks' }, feeds: [playlist], model, vaultOpen: false });
  assert.deepEqual(labels(panel.items), ['Releases', 'Tracks', 'Value routes', 'Feed settings', 'Files']);
  assert.equal(panel.items.find((item) => item.label === 'Tracks').current, true);
  assert.equal(panel.items.some((item) => item.action), false);
});

test('a feed row with only rss_image_url gives that address as the cover of the outline and of the release row', () => {
  const demu = { ...album, image: null, rss_image_url: 'https://m.example/demu.png' };
  const panel = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [demu], model, vaultOpen: true });
  assert.equal(panel.feed.image, 'https://m.example/demu.png');
  const list = releasesList({ feeds: [demu], connectors: [], sort: 'newest', view: 'rows' });
  assert.equal(list.rows[0].image, 'https://m.example/demu.png');
});

test('the itunes:image of a feed row comes before its rss_image_url', () => {
  const both = { ...album, image: 'https://m.example/a.png', rss_image_url: 'https://m.example/b.png' };
  const list = releasesList({ feeds: [both], connectors: [], sort: 'newest', view: 'rows' });
  assert.equal(list.rows[0].image, 'https://m.example/a.png');
});

test('no outline has an item for the first page of the feed (ADR 0069)', () => {
  const list = { ...album, id: 4, medium: 'musicL' };
  const feeds = [album, list, publisher, readOnly];
  for (const feed of feeds) {
    const panel = leftPanel({ view: { name: 'feed', id: feed.id }, feeds, model, vaultOpen: true });
    const ids = panel.items.map((item) => item.id);
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

test('releasesView gives the scroll of the nearest Releases entry and does not change nav (ADR 0069)', () => {
  const nav = {
    entries: [{ name: 'releases', marked: 9, scroll: 40 }, { name: 'feed', id: 1 }, { name: 'feed', id: 1, page: 'files' }],
    index: 2,
  };
  const copy = JSON.parse(JSON.stringify(nav));
  assert.deepEqual(releasesView(nav, 1), { name: 'releases', marked: 1, scroll: 40 });
  assert.deepEqual(nav, copy);
  const none = { entries: [{ name: 'feed', id: 1 }], index: 0 };
  assert.deepEqual(releasesView(none, 1), { name: 'releases', marked: 1 });
  const later = { entries: [{ name: 'feed', id: 1 }, { name: 'releases', scroll: 5 }], index: 0 };
  assert.deepEqual(releasesView(later, 1), { name: 'releases', marked: 1 });
});

test('Settings is current on the view settings', () => {
  const panel = leftPanel({ view: { name: 'settings' }, feeds: [], model: null, vaultOpen: true });
  assert.deepEqual(panel.bottom.map((e) => [e.id, e.current]), [['hosts', false], ['settings', true], ['lock', false]]);
});

test('the Role menu gives Artist, Label or Custom for a set of roles', () => {
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

test('the destinations below Releases are each feed with roles A to Z, then Not linked', () => {
  const message = 'ADR 0085 items 1 and 3: list each feed with roles A to Z, then "Not linked". Use groupDestinations of crates/api/ui/shell.js.';
  const panel = leftPanel({ view: { name: 'releases', group: 'none' }, feeds: catalog, model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.id, e.label, e.count]),
    [['releases', 'Releases', undefined], ['group-30', 'Acme', 1], ['group-20', 'Zed', 2], ['group-none', 'Not linked', 1]], message);
  assert.deepEqual(panel.top.map((e) => e.current), [false, false, false, true], message);
  assert.equal(panel.top[0].opens, true, 'ADR 0085 item 1: on a destination below Releases, the row Releases opens Releases.');
  assert.deepEqual(panel.top.map((e) => e.dot), [undefined, 'label hollow', 'artist', null], message);
  const linked = leftPanel({ view: { name: 'releases' }, feeds: [artist, first], model: null, vaultOpen: false });
  assert.deepEqual(linked.top.map((e) => e.id), ['releases', 'group-20'], 'ADR 0085 item 3: "Not linked" shows only when a release has no link.');
});

test('the page of a feed with roles gives its releases in feed order, with a note for another feed', () => {
  const message = 'ADR 0085 items 4 and 5: the releases of the feed in its sequence, and "Also on" names each other feed. Use releasesList of crates/api/ui/shell.js.';
  const list = releasesList({ feeds: catalog, connectors: [], sort: undefined, view: 'rows', group: 20 });
  assert.deepEqual(list.rows.map((r) => r.id), [22, 21], message);
  assert.equal(list.sort, 'order', message);
  assert.deepEqual(list.sorts, ['order', 'newest', 'az'], message);
  assert.deepEqual(list.head, { id: 20, title: 'Zed', typeLabel: 'Artist', dot: 'artist' }, message);
  assert.deepEqual(list.rows.map((r) => r.note), ['Also on Acme', null], message);
  const az = releasesList({ feeds: catalog, connectors: [], sort: 'az', view: 'rows', group: 20 });
  assert.deepEqual(az.rows.map((r) => r.id), [21, 22]);
});

test('Not linked gives the releases with no link, and Releases gives each release', () => {
  const none = releasesList({ feeds: catalog, connectors: [], sort: 'order', view: 'rows', group: 'none' });
  assert.deepEqual(none.rows.map((r) => r.id), [23], 'ADR 0085 item 3: "Not linked" gives each release with no link.');
  assert.equal(none.sort, 'newest', 'ADR 0085 item 4: "Not linked" has no feed order.');
  assert.equal(none.head.title, 'Not linked');
  const all = releasesList({ feeds: catalog, connectors: [], sort: 'newest', view: 'rows' });
  assert.deepEqual(all.rows.map((r) => r.id), [21, 23, 22], 'ADR 0085 item 6: Releases gives each album and playlist.');
  assert.equal(all.head, null);
  assert.equal(all.rows.every((r) => r.note === null), true);
});

test('a destination below Releases is a page of the root at narrow width', () => {
  const view = { name: 'releases', group: 20 };
  assert.deepEqual(narrowStack(view), [{ name: 'root' }, view], 'ADR 0085 item 7: the root page opens the destination.');
});

// ---- The levels of the hierarchy (ADR 0088) ----

const host = { id: 40, title: 'Some Host', medium: 'publisher', publisher_roles: ['hosting'], roles_confirmed: true, release_ids: [51, 52], child_ids: [41, 42], parent_ids: [] };
const labelA = { id: 41, title: 'Label A', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [51], child_ids: [43, 44], parent_ids: [40] };
const labelB = { id: 42, title: 'Label B', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [52], child_ids: [], parent_ids: [40] };
const artistX = { id: 43, title: 'Artist X', medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [41] };
const beatLab = { id: 44, title: 'Beat Lab', medium: 'publisher', publisher_roles: ['producer'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [41] };
const album1 = { id: 51, title: 'Album 1', medium: 'music', publisher_ids: [40, 41, 43, 44], added_at: 1, draft_state: 'none' };
const album2 = { id: 52, title: 'Album 2', medium: 'music', publisher_ids: [40, 42], added_at: 2, draft_state: 'none' };
const tree = [host, labelA, labelB, artistX, beatLab, album1, album2];

test('level 0 gives only the top feeds, under their role headings', () => {
  const message = 'ADR 0088 items 3 and 4: level 0 lists each feed with no feed above it, under its role heading. Use groupDestinations of crates/api/ui/shell.js.';
  const panel = leftPanel({ view: { name: 'releases' }, feeds: [...tree, { ...artistX, id: 45, title: 'Alone', parent_ids: [] }], model: null, vaultOpen: false });
  assert.deepEqual(panel.top.map((e) => [e.label, e.heading]),
    [['Releases', undefined], ['Some Host', 'Hosting'], ['Alone', 'Artists']], message);
});

test('a level gives the way up, the head, the feeds below and the releases', () => {
  const message = 'ADR 0088 item 5: a level gives "‹" to the level above, the head, the feeds directly below under their headings, then the releases. Use feedLevel of crates/api/ui/shell.js.';
  const level = leftPanel({ view: { name: 'releases', group: 41, trail: [40] }, feeds: tree, model: null, vaultOpen: false });
  assert.equal(level.kind, 'level', message);
  assert.equal(level.back.label, 'Some Host', message);
  assert.deepEqual(level.back.view, { name: 'releases', group: 40, trail: [], marked: 41 }, message);
  assert.equal(level.head.title, 'Label A', message);
  assert.deepEqual(level.sections.map((s) => [s.heading, s.items.map((i) => i.label)]),
    [['Artists', ['Artist X']], ['Producers', ['Beat Lab']]], message);
  assert.deepEqual(level.sections[0].items[0].view, { name: 'releases', group: 43, trail: [40, 41] }, message);
  assert.deepEqual(level.releases.map((r) => [r.title, r.from]), [['Album 1', { group: 41, trail: [40] }]], message);
  const top = leftPanel({ view: { name: 'releases', group: 40 }, feeds: tree, model: null, vaultOpen: false });
  assert.equal(top.back.label, 'Releases', message);
  assert.deepEqual(top.back.view, { name: 'releases', marked: 40 }, message);
});

test('a feed opened from a level goes back to that level', () => {
  const message = 'ADR 0088 item 6: the first outline row names the level. Use outlineItems of crates/api/ui/shell.js.';
  const from = { group: 41, trail: [40] };
  const panel = leftPanel({ view: { name: 'feed', id: 51, from }, feeds: tree, model: null, vaultOpen: false });
  const up = panel.items.find((i) => i.id === 'releases');
  assert.equal(up.label, 'Label A', message);
  assert.deepEqual(up.target, { name: 'releases', group: 41, trail: [40], marked: 51 }, message);
  const plain = leftPanel({ view: { name: 'feed', id: 51 }, feeds: tree, model: null, vaultOpen: false });
  assert.equal(plain.items.find((i) => i.id === 'releases').label, 'Releases');
});

test('each level is a page of the narrow stack', () => {
  const message = 'ADR 0088 item 7: the levels above are pages below. Use narrowStack of crates/api/ui/shell.js.';
  const root = { name: 'root' };
  const level = { name: 'releases', group: 43, trail: [40, 41] };
  assert.deepEqual(narrowStack(level), [root, { name: 'releases', group: 40, trail: [] }, { name: 'releases', group: 41, trail: [40] }, level], message);
  const from = { group: 41, trail: [40] };
  const feed = { name: 'feed', id: 51, from };
  assert.deepEqual(narrowStack(feed), [root, { name: 'releases', group: 40, trail: [] }, { name: 'releases', group: 41, trail: [40] }, { name: 'outline', id: 51, from }, feed], message);
  assert.deepEqual(openFeedView(51, true, from), { name: 'outline', id: 51, from });
});

test('the note of a release leaves out the feeds above and below the level', () => {
  const message = 'ADR 0088 item 8: "Also on" names only the feeds that are not above or below the level. Use relatedLevels of crates/api/ui/shell.js.';
  const labelC = { id: 46, title: 'Label C', medium: 'publisher', publisher_roles: ['label'], roles_confirmed: true, release_ids: [51], child_ids: [], parent_ids: [] };
  const shared = { ...album1, publisher_ids: [40, 41, 43, 44, 46] };
  const list = releasesList({ feeds: [host, labelA, labelB, artistX, beatLab, labelC, shared, album2], connectors: [], sort: 'order', view: 'rows', group: 41 });
  assert.deepEqual(list.rows.map((r) => r.note), ['Also on Label C'], message);
});
