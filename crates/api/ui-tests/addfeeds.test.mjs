// The rules of the Add feeds page and of the form that makes a feed (ADR 0073).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { addMenuEntries, addSearchRows, addRowIsArtist, addSheetRows, addResultMark, addFoundValues, foundPageValues, foundPages, foundIndexPage, makeFields, makeTitle, feedFileStart, parentCard, MAKE_TYPES, RELEASE_KINDS } from '../ui/addfeeds.js';

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

test('the search sheet gives the two import rows and hides the folder row without a host', () => {
  assert.deepEqual(addSheetRows(true).map((row) => row.title), ['Add from a File Host Folder…', 'Add from MSP 2.0 Backups…']);
  assert.deepEqual(addSheetRows(false).map((row) => row.title), ['Add from MSP 2.0 Backups…']);
});

const artistPage = { id: 9, medium: 'publisher', connector_id: 1, publisher_roles: ['artist'] };

test('the add menu of Releases has the values of ADR 0240 item 1 in their sequence', () => {
  const owner = 'ADR 0240 item 1: the add menu has Release, Artist, Label, Hosting, Show and Playlist. Fix addMenuEntries in crates/api/ui/addfeeds.js.';
  const entries = addMenuEntries(null);
  assert.deepEqual(entries.map((e) => e.label), ['Release', 'Artist', 'Label', 'Hosting', 'Show', 'Playlist'], owner);
  assert.deepEqual(entries.map((e) => e.open), ['album', 'publisher', 'publisher', 'publisher', 'show', 'playlist'], owner);
  assert.deepEqual(entries[1].start, { role: 'artist' }, owner);
  assert.deepEqual(entries[2].start, { role: 'label' }, owner);
  assert.deepEqual(entries[3].start, { role: 'hosting' }, owner);
});

test('the add menu of a label page and a hosting page has no Release and no role of the same or a higher rank', () => {
  const owner = 'ADR 0240 item 4: a label page or a hosting page has no Release, and its roles make a page below it. Fix addMenuEntries in crates/api/ui/addfeeds.js.';
  const hosting = { id: 5, medium: 'publisher', connector_id: 1, publisher_roles: ['hosting'] };
  const onHosting = addMenuEntries(releaseAdd(hosting, 'Host'), hosting);
  assert.deepEqual(onHosting.map((e) => e.label), ['Artist', 'Label', 'Show', 'Playlist'], owner);
  assert.deepEqual(onHosting[0].start, { role: 'artist', below_id: 5 }, owner);
  assert.deepEqual(onHosting[1].start, { role: 'label', below_id: 5 }, owner);
  const label = { ...hosting, id: 6, publisher_roles: ['label'] };
  const onLabel = addMenuEntries(releaseAdd(label, 'Label'), label);
  assert.deepEqual(onLabel.map((e) => e.label), ['Artist', 'Show', 'Playlist'], owner);
  assert.deepEqual(onLabel[0].start, { role: 'artist', below_id: 6 }, owner);
});

test('the add menu of an artist page keeps Release and no publisher role actions', () => {
  const owner = 'ADR 0240 item 4: an artist page keeps the menu of Releases, and Release has the artist as its parent. Fix addMenuEntries.';
  const onArtist = addMenuEntries(releaseAdd(artistPage, 'Pixel Palace'), artistPage);
  assert.deepEqual(onArtist.map((e) => e.label), ['Release', 'Show', 'Playlist'], owner);
  assert.deepEqual(onArtist[0].start, { author: 'Pixel Palace', publisher_id: 9 }, owner);
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
  const release = form.slice(form.lastIndexOf('if (isRelease) {'));
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
  assert.doesNotMatch(source, /Start from|startChoices|body\.from(?!_found)\b/, owner);
  assert.doesNotMatch(addfeeds, /Start from|startChoices/, owner);
});

test('each add form shows the role rows of its kind and each row with no role (ADR 0233 item 2)', () => {
  const owner = 'ADR 0233 item 2: a form shows only the results of its kind. Fix addSearchRows in crates/api/ui/addfeeds.js.';
  const rows = [
    { feed_guid: 'a', kind: { type: 'role', roles: ['artist'] } },
    { feed_guid: 'l', kind: { type: 'role', roles: ['label'] } },
    { feed_guid: 'i', kind: { type: 'inferred_artist' } },
    { feed_guid: 'n', kind: { type: 'no_role' } },
  ];
  const ids = (type, role) => addSearchRows(rows, type, role).map((row) => row.feed_guid);
  assert.deepEqual(ids('album'), ['a', 'i', 'n'], owner);
  assert.deepEqual(ids('publisher', 'artist'), ['a', 'i', 'n'], owner);
  assert.deepEqual(ids('publisher', 'label'), ['l', 'n'], owner);
  assert.deepEqual(ids('show'), ['n'], owner);
  assert.deepEqual(ids('playlist'), ['n'], owner);
});

test('an artist row of a release form opens the releases of that artist (ADR 0233 item 13)', () => {
  const owner = 'ADR 0233 item 13: an artist row gives the releases of the artist. Fix addRowIsArtist in crates/api/ui/addfeeds.js.';
  assert.equal(addRowIsArtist({ kind: { type: 'role', roles: ['artist', 'label'] } }), true, owner);
  assert.equal(addRowIsArtist({ kind: { type: 'inferred_artist' } }), true, owner);
  assert.equal(addRowIsArtist({ kind: { type: 'role', roles: ['label'] } }), false, owner);
  assert.equal(addRowIsArtist({ feed_guid: 'f' }), false, owner);
});

test('found-feed marks and copy values use the route fields', () => {
  assert.equal(addResultMark({ yours: true }), 'Yours');
  assert.equal(addResultMark({ in_studio_id: 8 }), 'In Studio');
  assert.equal(addResultMark({ yours: true, in_studio_id: 8 }), 'Yours · In Studio');
  assert.deepEqual(addFoundValues({ title: 'Release', artist: 'Ann' }, { author: 'Ann' }), { title: 'Release', author: 'Ann' });
});

test('the add sheet uses the Music Index routes and removes the old row message', () => {
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(source, /\/musicindex\/releases\//);
  assert.match(source, /\/feeds\/copy-preview\?url=/);
  assert.match(source, /body\.copy_from = copyFrom/);
  assert.doesNotMatch(source, /Select a release\. This row is not a release\./);
  assert.doesNotMatch(source, /name === 'add-feed'/);
});


const releasePreview = { author: 'Ann', link: 'https://ann.example', recipients: 2, title: 'Album B' };

test('a found release gives the artist form the artist, the website and the recipients', () => {
  const owner = 'ADR 0238 item 4: the Artist form takes author, link and recipients. Fix foundPageValues in crates/api/ui/addfeeds.js.';
  const values = foundPageValues('artist', releasePreview, {});
  assert.equal(values.title, 'Ann', owner);
  assert.equal(values.website, 'https://ann.example', owner);
  assert.equal(values.copiesRecipients, true, owner);
  assert.equal(values.line, 'Takes the artist of Album B. Links Album B to this page.', owner);
  assert.equal(values.emptyLine, '', owner);
});

test('a found release gives the label form no value and asks for the name', () => {
  const owner = 'ADR 0238 items 4 and 5: the Label form has no start value. Fix foundPageValues in crates/api/ui/addfeeds.js.';
  const values = foundPageValues('label', releasePreview, {});
  assert.deepEqual([values.title, values.website, values.copiesRecipients], ['', '', false], owner);
  assert.equal(values.emptyLine, 'Album B does not name its label. Type the name.', owner);
});

test('a found release gives the hosting form no value and asks for the host', () => {
  const owner = 'ADR 0238 items 4 and 5: the host is a person, not the owner of the feed. Fix foundPageValues in crates/api/ui/addfeeds.js.';
  const values = foundPageValues('hosting', releasePreview, {});
  assert.deepEqual([values.title, values.website, values.copiesRecipients], ['', '', false], owner);
  assert.equal(values.emptyLine, 'Type the name of the person who hosts Album B.', owner);
});

test('the page of Music Index is an artist page when its title is the artist', () => {
  const owner = 'ADR 0238 item 2: the page that Music Index gives for a found feed. Fix foundIndexPage in crates/api/ui/addfeeds.js.';
  const values = { publisher_feed_guid: 'p1', publisher_feed_title: 'Brandon James Seyferth' };
  assert.deepEqual(foundIndexPage(values, 'brandon james  seyferth'), { feed_guid: 'p1', feed_url: null, title: 'Brandon James Seyferth', rel: 'artist' }, owner);
  assert.equal(foundIndexPage(values, 'Someone Else').rel, null, owner);
  assert.equal(foundIndexPage({}, 'Ann'), null, owner);
  assert.equal(foundIndexPage(null, 'Ann'), null, owner);
});

test('an unread feed gives only the artist name to the artist form', () => {
  const owner = 'ADR 0238 item 7: with no feed file only the Artist form gets a value. Fix foundPageValues in crates/api/ui/addfeeds.js.';
  const failed = { failed: true, title: 'Album B', author: '' };
  const artist = foundPageValues('artist', failed, { artist: 'Ann' });
  assert.equal(artist.title, 'Ann', owner);
  assert.equal(artist.website, '', owner);
  assert.equal(artist.line, 'Studio could not read the feed file of Album B. Only the name is copied.', owner);
  assert.equal(foundPageValues('hosting', failed, { artist: 'Ann' }).title, '', owner);
  assert.equal(foundPageValues('label', failed, { artist: 'Ann' }).title, '', owner);
});

test('the page list keeps the pages of the role and the pages with no role', () => {
  const owner = 'ADR 0238 item 2: keep the pages with the role or with no rel. Fix foundPages in crates/api/ui/addfeeds.js.';
  const pages = [
    { feed_guid: 'a', rel: 'artist' },
    { feed_guid: 'b', rel: 'label' },
    { feed_guid: 'c' },
    { feed_guid: 'd', rel: 'artist label' },
    { rel: 'artist' },
  ];
  assert.deepEqual(foundPages(pages, 'artist').map((p) => p.feed_guid), ['a', 'c', 'd'], owner);
  assert.deepEqual(foundPages(pages, 'hosting').map((p) => p.feed_guid), ['c'], owner);
  assert.deepEqual(foundPages(null, 'artist'), [], owner);
});
