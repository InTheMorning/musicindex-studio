// The rules of the Add feeds page and of the form that makes a feed (ADR 0073).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { addChoices, addMenuEntries, ADD_PAGE_TITLE, makeFields, makeTitle, feedFileStart, startChoices, MAKE_TYPES, RELEASE_KINDS } from '../ui/addfeeds.js';

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
    makeTitle('show'), makeTitle('playlist'), makeTitle('album'),
  ];
  for (const label of labels) assert.doesNotMatch(label.replace(/feed file/gi, ''), /feed/i, `${owner} Label: ${label}`);
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /＋ Add a Feed|>Add a Feed<|Open the Feed/, owner);
  assert.match(source, /\$\{head\.typeLabel\} Page/, 'ADR 0230 item 7: the head button is {Type} Page.');
});

test('makeFields gives the fields of each type', () => {
  assert.deepEqual(makeFields('album'), { medium: 'music', titleLabel: 'Album title', authorLabel: 'Artist', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' });
  assert.deepEqual(makeFields('publisher'), { medium: 'publisher', titleLabel: 'Name', authorLabel: null, role: true, button: 'Make the Page', busyLabel: 'Making the Page…' });
  assert.deepEqual(makeFields('show'), { medium: 'music', titleLabel: 'Show title', authorLabel: 'Artist', role: false, button: 'Make the Show', busyLabel: 'Making the Show…' });
  assert.deepEqual(makeFields('playlist'), { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the Playlist', busyLabel: 'Making the Playlist…' });
});

test('MAKE_TYPES leaves out album, and release fields have a kind menu', () => {
  const owner = 'ADR 0213 items 2, 6 and 12: keep Album out of Type and give the release fields and Kind values. Fix addfeeds.js and app.js.';
  assert.deepEqual(MAKE_TYPES.map(([id]) => id), ['publisher', 'show', 'playlist'], owner);
  for (const [id] of MAKE_TYPES) assert.ok(makeFields(id).button, owner);
  assert.deepEqual(RELEASE_KINDS, [['', 'From the tracks'], ['album', 'Album'], ['ep', 'EP'], ['single', 'Single']], owner);
  assert.deepEqual(makeFields('album'), { medium: 'music', titleLabel: 'Album title', authorLabel: 'Artist', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' }, owner);
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(source, /body\.kind = releaseKind \|\| null/, owner);
  assert.match(source, /body\.publisher_id = start\.publisher_id/, owner);
  assert.match(source, /label="Kind" value=\$\{releaseKind\}/, owner);
  const form = source.slice(source.indexOf('function MakeFeedForm'), source.indexOf('// Choose a feed file'));
  const fieldOrder = [form.indexOf('label="Kind"'), form.indexOf('id="make-title"'), form.indexOf('id="make-author"'),
    form.indexOf('id="make-from"'), form.lastIndexOf('for="make-host"'), form.indexOf('label">Feed file')];
  assert.ok(fieldOrder.every((position, index) => position >= 0 && (index === 0 || fieldOrder[index - 1] < position)), owner);
});

test('startChoices gives only the empty album when no album exists', () => {
  assert.deepEqual(startChoices([]), [['', 'Empty album']]);
  assert.deepEqual(startChoices(undefined), [['', 'Empty album']]);
});

test('startChoices gives the albums in the sequence of their titles', () => {
  const feeds = [
    { id: 7, medium: 'music', title: 'Zebra' },
    { id: 3, medium: 'music', title: 'Apple' },
  ];
  assert.deepEqual(startChoices(feeds), [['', 'Empty album'], ['3', 'Apple'], ['7', 'Zebra']]);
});

test('startChoices uses the address of a feed with no title', () => {
  const feeds = [{ id: 4, medium: 'music', title: '', path: 'a/b.xml' }];
  assert.deepEqual(startChoices(feeds), [['', 'Empty album'], ['4', 'a/b.xml']]);
});

test('startChoices leaves out a publisher feed and a playlist', () => {
  const feeds = [
    { id: 1, medium: 'publisher', publisher_roles: ['artist'], roles_confirmed: true, title: 'Band' },
    { id: 2, medium: 'musicL', title: 'Mix' },
    { id: 3, medium: 'music', title: 'Album' },
  ];
  assert.deepEqual(startChoices(feeds), [['', 'Empty album'], ['3', 'Album']]);
});

test('startChoices includes releases with a kept EP or single kind', () => {
  const feeds = [
    { id: 1, medium: 'music', kind_shown: 'ep', title: 'EP' },
    { id: 2, medium: 'music', kind_shown: 'single', title: 'Single' },
  ];
  assert.deepEqual(startChoices(feeds), [['', 'Empty album'], ['1', 'EP'], ['2', 'Single']],
    'ADR 0077 item 1: Start from must list each music release. Fix startChoices in crates/api/ui/addfeeds.js.');
});

test('makeTitle names the page that the sheet makes (ADR 0164 item 7)', () => {
  assert.equal(makeTitle('album', null), 'Make a new release');
  assert.equal(makeTitle('publisher', 'artist'), 'Make a new artist page');
  assert.equal(makeTitle('publisher', 'label'), 'Make a new label page');
  assert.equal(makeTitle('publisher', 'hosting'), 'Make a new hosting page');
  assert.equal(makeTitle('publisher', null), 'Make a new artist or label page');
  assert.equal(makeTitle('show', null), 'Make a new show');
  assert.equal(makeTitle('playlist', null), 'Make a new playlist');
});
