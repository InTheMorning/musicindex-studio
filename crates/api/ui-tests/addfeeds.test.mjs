// The rules of the Add feeds page and of the form that makes a feed (ADR 0073).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { addChoices, addMenuEntries, ADD_PAGE_TITLE, makeFields, makeTitle, feedFileStart, parentCard, releaseFindChoices, kindOfTrackCount, MAKE_TYPES, RELEASE_KINDS } from '../ui/addfeeds.js';

import { releaseAdd } from '../ui/welcome.js';

test('feedFileStart gives the file name from the title', () => {
  assert.equal(feedFileStart('Pixel Palace'), 'pixel-palace.xml');
  assert.equal(feedFileStart('Ça va — Live!'), 'ca-va-live.xml');
  assert.equal(feedFileStart('2024: Best Of'), '2024-best-of.xml');
});

test('feedFileStart gives an empty path when the title has no usable character', () => {
  assert.equal(feedFileStart('  '), '');
  assert.equal(feedFileStart(''), '');
  assert.equal(feedFileStart('!!!'), '');
});

test('addChoices gives the one group of the page Add One You Have', () => {
  const owner = 'ADR 0230 item 4: the page has the three rows of "Add One You Have". Fix addChoices in crates/api/ui/addfeeds.js.';
  const groups = addChoices({ hasHost: true });
  assert.deepEqual(groups.map((g) => [g.id, g.label]), [['have', 'Add One You Have']], owner);
  assert.deepEqual(groups[0].rows.map((r) => r.id), ['find', 'folder', 'msp'], owner);
  assert.equal(groups[0].connect, false, owner);
  assert.equal(ADD_PAGE_TITLE, 'Add One You Have', owner);
  for (const row of groups[0].rows) {
    assert.ok(row.title && row.text, `the row ${row.id} needs a title and a text`);
  }
});

test('addChoices leaves out the folder row and asks for a host when no host exists', () => {
  const groups = addChoices({ hasHost: false });
  assert.deepEqual(groups[0].rows.map((r) => r.id), ['find', 'msp']);
  assert.equal(groups[0].connect, true);
});

const artistPage = { id: 9, medium: 'publisher', connector_id: 1, publisher_roles: ['artist'] };

test('the add menu has the values of ADR 0230 item 1 in their sequence', () => {
  const owner = 'ADR 0230 item 1: the add menu has Release, Artist, Label, Show, Playlist, a separator, then Add One You Have. Fix addMenuEntries in crates/api/ui/addfeeds.js.';
  const entries = addMenuEntries(null);
  assert.deepEqual(entries.map((e) => e.label), ['Release', 'Artist', 'Label', 'Show', 'Playlist', 'Add One You Have…'], owner);
  assert.deepEqual(entries.map((e) => Boolean(e.separatorBefore)), [false, false, false, false, false, true], owner);
  assert.deepEqual(entries.map((e) => e.open), ['album', 'publisher', 'publisher', 'show', 'playlist', null], owner);
  assert.deepEqual(entries[1].start, { role: 'artist' }, owner);
  assert.deepEqual(entries[2].start, { role: 'label' }, owner);
});

test('the add menu of an artist page gives that artist as the start parent of Release', () => {
  const owner = 'ADR 0230 items 2 and 3: Release has the artist page as its parent, and no parent on the top page. Fix addMenuEntries and releaseAdd.';
  const onArtist = addMenuEntries(releaseAdd(artistPage, 'Pixel Palace'));
  assert.equal(onArtist[0].start.publisher_id, 9, owner);
  assert.equal(onArtist[0].start.author, 'Pixel Palace', owner);
  assert.deepEqual(addMenuEntries(releaseAdd(null, '')) [0].start, {}, owner);
  const label = { ...artistPage, publisher_roles: ['label'] };
  assert.deepEqual(addMenuEntries(releaseAdd(label, 'Label'))[0].start, {}, owner);
});

test('no label of an add control has the word "feed" except "feed file"', () => {
  const owner = 'ADR 0230 item 6: a user action does not use the word "feed". Change the label in addfeeds.js or app.js.';
  const labels = [
    ...addMenuEntries(null).map((e) => e.label),
    ...addChoices({ hasHost: true }).flatMap((g) => [g.label, ...g.rows.map((r) => r.title)]),
    ADD_PAGE_TITLE,
    ...['album', 'publisher', 'show', 'playlist'].flatMap((t) => [makeFields(t).button, makeFields(t).busyLabel, makeFields(t).titleLabel]),
    ...['artist', 'label', 'hosting', undefined].map((r) => makeTitle('publisher', r)),
    makeTitle('show'), makeTitle('playlist'), makeTitle('album'), makeTitle('album', null, 'Pixel Palace'),
  ];
  for (const label of labels) assert.doesNotMatch(label.replace(/feed file/gi, ''), /feed/i, `${owner} Label: ${label}`);
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /＋ Add a Feed|>Add a Feed<|Open the Feed/, owner);
  assert.match(source, /\$\{head\.typeLabel\} Page/, 'ADR 0230 item 7: the head button is {Type} Page.');
});

test('makeFields gives the fields of each type', () => {
  assert.deepEqual(makeFields('album'), { medium: 'music', titleLabel: 'Title', authorLabel: 'Artist name', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' });
  assert.deepEqual(makeFields('publisher'), { medium: 'publisher', titleLabel: 'Name', authorLabel: null, role: true, button: 'Make the Page', busyLabel: 'Making the Page…' });
  assert.deepEqual(makeFields('show'), { medium: 'music', titleLabel: 'Show title', authorLabel: 'Artist', role: false, button: 'Make the Show', busyLabel: 'Making the Show…' });
  assert.deepEqual(makeFields('playlist'), { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the Playlist', busyLabel: 'Making the Playlist…' });
});

test('MAKE_TYPES leaves out album, and release fields have a kind menu', () => {
  const owner = 'ADR 0213 items 2, 6 and 12: keep Album out of Type and give the release fields and Kind values. Fix addfeeds.js and app.js.';
  assert.deepEqual(MAKE_TYPES.map(([id]) => id), ['publisher', 'show', 'playlist'], owner);
  for (const [id] of MAKE_TYPES) assert.ok(makeFields(id).button, owner);
  assert.deepEqual(RELEASE_KINDS, [['', 'From the tracks'], ['album', 'Album'], ['ep', 'EP'], ['single', 'Single']], owner);
  assert.deepEqual(makeFields('album'), { medium: 'music', titleLabel: 'Title', authorLabel: 'Artist name', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' }, owner);
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(source, /body\.kind = releaseKind \|\| null/, owner);
  assert.match(source, /body\.publisher_id = parentId/, owner);
  assert.match(source, /label="Kind" value=\$\{releaseKind\}/, owner);
  const form = source.slice(source.indexOf('function MakeFeedForm'), source.indexOf('// Choose a feed file'));
  // ADR 0230 item 12: the fields follow the parent card and the row of Music Index.
  const release = form.slice(form.indexOf('if (isRelease) {'));
  const fieldOrder = [release.indexOf('<${ParentCard'), release.indexOf('Find on Music Index'), release.indexOf('${titleField}'),
    release.indexOf('label="Kind"'), release.indexOf('${authorField}'), release.indexOf('${hostField}'), release.indexOf('${feedFile}')];
  assert.ok(fieldOrder.every((position, index) => position >= 0 && (index === 0 || fieldOrder[index - 1] < position)),
    'ADR 0230 items 9, 11 and 12: the release form shows the parent card, the row Find on Music Index, then Title, Kind, Artist name, File host and Feed file. Fix MakeFeedForm in crates/api/ui/app.js.');
});

test('the sheet title names the parent, or has no parent (ADR 0230 item 8)', () => {
  const owner = 'ADR 0230 item 8: the title is "New release for {parent}", or "New release" with no parent. Fix makeTitle in crates/api/ui/addfeeds.js.';
  assert.equal(makeTitle('album', null, 'Pixel Palace'), 'New release for Pixel Palace', owner);
  assert.equal(makeTitle('album', null, null), 'New release', owner);
  assert.equal(makeTitle('album'), 'New release', owner);
});

test('the parent card shows the page, and no card when there is no parent (ADR 0230 items 9 and 10)', () => {
  const owner = 'ADR 0230 items 9 and 10: the parent card gives the name and the type of the page, or null with no parent. Fix parentCard in crates/api/ui/addfeeds.js.';
  const feeds = [{ id: 9, medium: 'publisher', connector_id: 1, publisher_roles: ['artist'], roles_confirmed: true, title: 'Pixel Palace' }];
  const card = parentCard(feeds, 9);
  assert.equal(card.title, 'Pixel Palace', owner);
  assert.equal(card.typeLabel, 'Artist', owner);
  assert.equal(parentCard(feeds, null), null, owner);
  assert.equal(parentCard(feeds, undefined), null, owner);
  assert.equal(parentCard(feeds, 77), null, owner);
});

test('the release form cannot make a release with no parent (ADR 0230 item 10)', () => {
  const owner = 'ADR 0230 item 10: with no parent the card says "No artist page", has Select…, and Make the Release stays off. Fix MakeFeedForm in crates/api/ui/app.js.';
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const form = source.slice(source.indexOf('function ParentCard'), source.indexOf('// Choose a feed file'));
  assert.match(form, /'No artist page'/, owner);
  assert.match(form, /'Change…' : 'Select…'/, owner);
  assert.match(form, /\(!isRelease \|\| parentId !== null\)/, owner);
  assert.match(form, /if \(isRelease && parentId === null\) return;/, owner);
});

test('the release form has no "Start from" and no route field from (ADR 0230 item 13)', () => {
  const owner = 'ADR 0230 item 13: the copy of a release is gone. Remove "Start from", startChoices and the field from. Do not bring them back.';
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const addfeeds = readFileSync(new URL('../ui/addfeeds.js', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /Start from|startChoices|body\.from/, owner);
  assert.doesNotMatch(addfeeds, /Start from|startChoices/, owner);
});

test('the choices of a found release give Title, Artist name and Kind (ADR 0230 items 11 and 14)', () => {
  const owner = 'ADR 0230 items 11 and 14: a pick fills Title and Artist name, and Kind with a track count. Fix releaseFindChoices in crates/api/ui/addfeeds.js.';
  const hit = { feed_guid: 'g1', title: 'Hit Title', artist: 'Ann' };
  const choices = releaseFindChoices({ title: 'Found', image_url: 'https://a.example/a.jpg', website: 'https://a.example/', track_count: 5 }, hit);
  assert.equal(choices.length, 1, owner);
  assert.deepEqual(choices[0].values, { title: 'Found', author: 'Ann', kind: 'ep', image: 'https://a.example/a.jpg', website: 'https://a.example/' }, owner);
  // With no track count, the kind is not set. The title falls back to the row.
  assert.deepEqual(releaseFindChoices({}, hit)[0].values, { title: 'Hit Title', author: 'Ann' }, owner);
  // An artist page or a label page is not a release.
  assert.deepEqual(releaseFindChoices({ title: 'Band' }, null), [], owner);
});

test('the kind follows the track count (ADR 0230 item 14)', () => {
  const owner = 'ADR 0230 item 14 and ADR 0213 item 11: Kind comes from a track count only when Music Index gives one. Fix kindOfTrackCount in crates/api/ui/addfeeds.js.';
  assert.deepEqual([1, 3, 4, 6, 7, 20].map(kindOfTrackCount), ['single', 'single', 'ep', 'ep', 'album', 'album'], owner);
  assert.deepEqual([0, -1, 2.5, undefined, null, '5'].map(kindOfTrackCount), [null, null, null, null, null, null], owner);
});
