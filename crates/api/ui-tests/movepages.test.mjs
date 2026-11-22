import test from 'node:test';
import assert from 'node:assert/strict';
import {
  NEW_PAGE_PREFIX, moveTitle, moveMenus, moveStart, moveForArtist, moveReady, moveBody, hasArtistPage, NEEDS_ARTIST_LINE,
  movePageTitle, pageMoveRoles, pageMoveBody, pageMoveMenus, pageMoveStart, pageMoveLines,
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

test('the menus: artist has no None, label and hosting start with None, each ends with a new page', () => {
  const menus = moveMenus(feeds, {});
  assert.deepEqual(menus.map((menu) => menu.role), ['artist', 'label', 'hosting'], owner);
  assert.deepEqual(menus[0].options.map((o) => o[1]), ['Amy', 'Zed', 'New Artist Page…'], `${owner} A page with no connector is not a value.`);
  assert.deepEqual(menus[1].options[0], ['', 'None'], owner);
  assert.deepEqual(menus[2].options[0], ['', 'None'], owner);
  for (const menu of menus) {
    assert.equal(menu.options.at(-1)[0], `${NEW_PAGE_PREFIX}${menu.role}`, owner);
  }
});

test('an artist menu with no artist page says so', () => {
  const menu = moveMenus([], {})[0];
  assert.deepEqual(menu.options[0], ['', 'No artist page yet'], owner);
});

test('the start values follow the links, then the pages above the artist (item 15)', () => {
  assert.deepEqual(moveStart(feeds, { publisher_ids: [] }), { artist: '2', label: '', hosting: '' }, owner);
  assert.deepEqual(moveStart(feeds, { publisher_ids: [1] }), { artist: '1', label: '10', hosting: '20' }, owner);
  assert.deepEqual(moveStart(feeds, { publisher_ids: [1, 20] }), { artist: '1', label: '10', hosting: '20' }, owner);
  assert.deepEqual(moveForArtist(feeds, 1), { artist: '1', label: '10', hosting: '20' }, owner);
  assert.deepEqual(moveForArtist(feeds, 2), { artist: '2', label: '', hosting: '' }, owner);
});

test('hosting above label above artist gives the hosting menu start value (ADR 0240 item 15)', () => {
  assert.deepEqual(moveForArtist(feeds, 1), { artist: '1', label: '10', hosting: '20' }, owner);
});

test('the body, the ready check and the title', () => {
  assert.deepEqual(moveBody({ artist: '1', label: '', hosting: '20' }), { artist_id: 1, label_id: null, hosting_id: 20 }, owner);
  assert.equal(moveReady({ artist: '' }), false, owner);
  assert.equal(moveReady({ artist: '3' }), true, owner);
  assert.equal(moveTitle({ title: 'Night' }), 'Move Night', owner);
});

test('a release needs an artist page', () => {
  assert.equal(hasArtistPage(feeds, { publisher_ids: [10] }), false, owner);
  assert.equal(hasArtistPage(feeds, { publisher_ids: [1] }), true, owner);
  assert.equal(NEEDS_ARTIST_LINE, 'A release needs an artist page.', owner);
});

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
