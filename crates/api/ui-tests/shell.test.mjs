// The shell models (ADR 0042). Each test checks one rule of ADR 0039 and of
// musicindex ADR 0027 R-NAV-2.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  mediumKind, isEditable, isPublisher, isPlaylist, trackKey, isTrack,
  leftPanel, releasesList, listQuery, queryText,
  startNav, pushNav, backNav, forwardNav, canBack, canForward, setScroll,
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
  assert.deepEqual(mediumKind('publisher'), { dot: 'publisher', label: 'Publisher' });
  assert.deepEqual(mediumKind('MUSIC'), { dot: 'feed', label: 'Album' });
  assert.deepEqual(mediumKind('musicL'), { dot: 'playlist', label: 'Playlist' });
  assert.deepEqual(mediumKind(undefined), { dot: 'feed', label: 'Feed' });
});

test('an album or a publisher feed with a connector is editable', () => {
  assert.equal(isEditable(album), true);
  assert.equal(isEditable(publisher), true);
  assert.equal(isEditable(readOnly), false);
  assert.equal(isEditable({ ...album, medium: 'musicL' }), true);
  assert.equal(isEditable({ ...album, medium: 'musicL', connector_id: null }), false);
  assert.equal(isPublisher(publisher), true);
  assert.equal(isPublisher(album), false);
});

test('a track key uses the GUID, or the client ID with no GUID', () => {
  assert.equal(trackKey(model.items[0]), 'g-1');
  assert.equal(trackKey(model.items[1]), 'cid-7');
  assert.equal(isTrack(model.items[1], 'cid-7'), true);
  assert.equal(isTrack(model.items[0], 'g-1'), true);
  assert.equal(isTrack(model.items[0], 'g-2'), false);
});

test('with no open feed, the left panel gives the four destinations', () => {
  const panel = leftPanel({ view: { name: 'releases' }, feeds: [album], model: null, vaultOpen: true });
  assert.equal(panel.kind, 'destinations');
  assert.deepEqual(panel.top.map((e) => e.id), ['releases', 'add-feed']);
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts', 'lock']);
  assert.equal(panel.top[0].current, true);
  assert.equal(JSON.stringify(panel).includes('Select a feed'), false);
});

test('Unlock Studio takes the place of Lock Studio on a server with a locked vault', () => {
  const panel = leftPanel({ view: { name: 'releases' }, feeds: [], model: null, vaultOpen: false, vaultLocked: true });
  assert.deepEqual(panel.bottom.map((e) => [e.id, e.label]), [['hosts', 'Hosts'], ['unlock', 'Unlock Studio']]);
});

test('Lock Studio shows only when the vault is open', () => {
  const panel = leftPanel({ view: { name: 'hosts' }, feeds: [], model: null, vaultOpen: false });
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts']);
  assert.equal(panel.bottom[0].current, true);
});

test('an open album gives the outline in the sequence of ADR 0039 item 6', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1, page: 'track', track: 'g-1' }, feeds: [album], model, vaultOpen: true });
  assert.equal(panel.kind, 'outline');
  assert.deepEqual(panel.feed, { id: 1, title: 'Night Songs', image: null, typeLabel: 'Album' });
  assert.deepEqual(panel.items[0], { id: 'releases', label: 'Releases' });
  assert.deepEqual(labels(panel.items), [
    'Releases', 'Album', 'First', 'Untitled track', 'Add a track', 'Import files',
    'Live shows', 'Add a live show', 'Value routes', 'Credits and links', 'Files',
  ]);
  assert.equal(panel.items[2].number, 1);
  assert.equal(panel.items[2].current, true);
  assert.equal(panel.items[1].current, false);
  assert.equal(panel.items[4].action, 'add-track');
  assert.equal(panel.items[5].action, 'import');
  assert.equal(panel.items[6].heading, true);
  assert.equal(panel.items[7].action, 'add-live');
  assert.equal(panel.items[8].page, 'recipients');
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
  assert.deepEqual(labels(panel.items).slice(5, 11), [
    'Import files', 'Live shows', 'Launch', 'Untitled show', 'Add a live show', 'Value routes',
  ]);
  const entry = panel.items[7];
  assert.equal(entry.page, 'live');
  assert.equal(entry.track, 'l-1');
  assert.equal(entry.status, 'Planned');
  assert.equal(entry.current, true);
  assert.equal(panel.items[8].track, 'cid-12');
  assert.equal(panel.items[8].status, 'Live now');
  assert.equal(panel.items[8].current, false);
  assert.equal(panel.items[9].action, 'add-live');
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

test('an open album before its draft loads gives the Releases link, Album and Files', () => {
  const panel = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model: null, vaultOpen: false });
  assert.deepEqual(labels(panel.items), ['Releases', 'Album', 'Files']);
});

test('a publisher feed gives Publisher and Files, and a read-only feed gives Feed', () => {
  const pub = leftPanel({ view: { name: 'feed', id: 2 }, feeds: [publisher], model: null, vaultOpen: false });
  assert.deepEqual(labels(pub.items), ['Releases', 'Publisher', 'Files']);
  const read = leftPanel({ view: { name: 'feed', id: 3 }, feeds: [readOnly], model: null, vaultOpen: false });
  assert.deepEqual(labels(read.items), ['Releases', 'Feed']);
  assert.equal(read.items[1].current, true);
});

test('the Releases list gives the newest feed first, with the greater ID first on a tie', () => {
  const tie = { ...album, id: 9, added_at: 200 };
  const list = releasesList({ feeds: [album, publisher, readOnly, tie], connectors: [{ id: 'c1', name: 'Local' }], sort: 'newest', view: 'rows' });
  assert.equal(list.letters, null);
  assert.deepEqual(list.rows.map((r) => r.id), [9, 2, 1, 3]);
  assert.equal(list.rows[1].host, 'Local');
  assert.equal(list.rows[3].host, 'Read-only');
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
  assert.deepEqual(narrowStack({ name: 'add-feed' }), [root, { name: 'add-feed' }]);
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
    'Album', 'First', 'Untitled track', 'Add a track', 'Import files', 'Live shows', 'Add a live show',
    'Value routes', 'Credits and links', 'Files',
  ]);
  assert.equal(narrow.items.some((item) => item.current), false);
  const wide = leftPanel({ view: { name: 'feed', id: 1 }, feeds: [album], model, vaultOpen: true });
  assert.equal(wide.items[0].id, 'releases');
});

test('the root page has no current destination, and gives Hosts at the bottom', () => {
  const panel = leftPanel({ view: { name: 'root' }, feeds: [album], model: null, vaultOpen: false });
  assert.equal(panel.kind, 'destinations');
  assert.deepEqual(panel.top.map((e) => e.current), [false, false]);
  assert.deepEqual(panel.bottom.map((e) => e.id), ['hosts']);
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
  assert.deepEqual(labels(panel.items), ['Releases', 'Playlist', 'Tracks', 'Value routes', 'Files']);
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
