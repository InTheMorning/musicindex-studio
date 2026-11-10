// The Welcome page of a new user and its first steps (ADR 0119, task 083).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import {
  firstStart, releaseAdd,
  WELCOME, HOST_STEP, showWelcome, welcomeSteps, pathTexts, hostStepModel, HOST_ROWS, welcomeBody, nextStep, FIRST_STEPS,
} from '../ui/welcome.js';
import { isTitleCase } from '../ui/casing.js';

const FEED_WORD = /\bfeeds?\b/i;

test('showWelcome is true for an empty feed list and false with one feed', () => {
  assert.equal(showWelcome([]), true);
  assert.equal(showWelcome(undefined), true);
  assert.equal(showWelcome([{ id: 1, medium: 'music' }]), false);
});

test('welcomeSteps gives the three steps of each path', () => {
  assert.deepEqual(welcomeSteps('music'), ['Your name', 'Where it lives', 'Your first album']);
  assert.deepEqual(welcomeSteps('label'), ['Label name', 'Where it lives', 'Your first artist']);
});

test('HOST_ROWS gives the desktop rows, with the folder as the first row', () => {
  const desktop = HOST_ROWS(false);
  assert.deepEqual(desktop.map((r) => r.title), ['Folder on this computer', 'Nostr site (nsite)', 'S3 or Cloudflare R2', 'SFTP server']);
  assert.deepEqual(desktop.map((r) => r.type), ['local', 'nsite', 's3', 'sftp']);
  assert.equal(desktop[0].primary, true);
  assert.equal(desktop[1].primary, undefined);
  assert.equal(desktop[1].line, 'A site on Nostr. Its pages and songs go to Blossom servers, signed with your Nostr key.');
  assert.equal(desktop[2].line, 'A storage bucket at Amazon S3, Cloudflare R2 or a similar provider.');
  assert.equal(desktop[0].line, 'A folder on this computer. The public address is optional.');
});

test('HOST_ROWS gives the server rows, with the temporary area as the first row', () => {
  const server = HOST_ROWS(true);
  assert.deepEqual(server.map((r) => r.title), ['Try it here (temporary)', 'Nostr site (nsite)', 'S3 or Cloudflare R2', 'SFTP server']);
  assert.deepEqual(server.map((r) => r.type), ['temporary', 'nsite', 's3', 'sftp']);
  assert.equal(server[0].primary, true);
});

test('hostStepModel selects the first row, or the menu with a connected file host', () => {
  const desk = hostStepModel([], false);
  assert.equal(desk.menu, null);
  assert.equal(desk.selected, 'local');
  assert.deepEqual(desk.rows.map((r) => r.type), ['local', 'nsite', 's3', 'sftp']);
  assert.equal(hostStepModel([], true).selected, 'temporary');
  const server = hostStepModel([{ id: 4, name: 'Bucket' }, { id: 9, name: 'Other' }], true);
  assert.equal(server.selected, null, 'the menu is selected, so no row form shows a second primary button');
  assert.deepEqual(server.rows.map((r) => r.type), ['temporary', 'nsite', 's3', 'sftp']);
  assert.deepEqual(server.menu, { value: '4', options: [['4', 'Bucket'], ['9', 'Other']] });
  assert.equal(hostStepModel(undefined, false).menu, null);
});

test('the text of ADR 0122 item 3 is as decided, for the temporary row', () => {
  assert.equal(HOST_ROWS(true)[0].line, 'Kept for 7 days. Pick a real file host to keep your pages.');
  assert.equal(HOST_STEP.temporary, 'Kept for 7 days. Pick a real file host to keep your pages.');
});

test('the texts of ADR 0122 items 6 and 10 are as decided', () => {
  assert.equal(HOST_STEP.placeholderGuid, 'Studio cannot make the GUID yet. A GUID comes from the public address of the feed, and this file host has none. Studio sets the GUID when the feed gets a public address.');
  assert.equal(HOST_STEP.expired, 'The temporary area has expired. Move this feed to a real file host.');
});

test('the button labels of step 2 are in title case', () => {
  for (const label of ['Continue', 'Make the Page', 'Add the Folder']) {
    assert.equal(isTitleCase(label), true, label);
  }
});

test('welcomeBody gives the role of the path, the name as title and author, and the file start', () => {
  assert.deepEqual(welcomeBody('music', 'Pixel Palace', 3), {
    connector_id: 3, path: 'pixel-palace.xml', title: 'Pixel Palace', author: 'Pixel Palace', medium: 'publisher', role: 'artist',
  });
  assert.equal(welcomeBody('label', 'Night Signals', 4).role, 'label');
});

test('nextStep gives album for an artist with no release, label for a label with no release', () => {
  assert.equal(nextStep({ id: 1, medium: 'publisher', publisher_roles: ['artist'], child_ids: [] }), 'album');
  assert.equal(nextStep({ id: 2, medium: 'publisher', publisher_roles: ['label'], child_ids: [] }), 'label');
});

test('nextStep gives null for an artist with a release, and for a feed that is not a publisher feed', () => {
  assert.equal(nextStep({ id: 1, medium: 'publisher', publisher_roles: ['artist'], release_ids: [7], child_ids: [] }), null);
  assert.equal(nextStep({ id: 1, medium: 'publisher', publisher_roles: ['artist'], child_ids: [7] }), null);
  assert.equal(nextStep({ id: 3, medium: 'music', publisher_roles: [], child_ids: [] }), null);
  assert.equal(nextStep(null), null);
});

test('the texts of ADR 0119 items 2, 5, 6 and 8 are as decided', () => {
  assert.equal(WELCOME.intro, 'Studio publishes your music so that every music app can play it. What do you want to start with?');
  assert.equal(WELCOME.rows.music.title, 'I make music');
  assert.equal(WELCOME.rows.music.line, 'Start with your artist page. Your albums go on it.');
  assert.equal(WELCOME.rows.label.line, 'Start with your label page. Your artists and releases go on it.');
  assert.equal(WELCOME.rows.online.title, 'My music is online already');
  assert.equal(WELCOME.rows.online.line, 'Find your artist or albums, and bring them into Studio.');
  assert.equal(pathTexts('music').hint, 'A band is one artist. Add the members later, in People.');
  assert.equal(pathTexts('music').where, 'Studio does not keep your music. It puts your artist page and your songs on a file host that you choose. You can move them later.');
  assert.equal(pathTexts('label').where, 'Studio does not keep your music. It puts your label page and your songs on a file host that you choose. You can move them later.');
  assert.equal(FIRST_STEPS.album.title, 'Add your first album');
  assert.equal(FIRST_STEPS.album.line, 'Drop the songs in. Studio reads their names and order, and makes the album page.');
  assert.deepEqual(FIRST_STEPS.label.map((s) => s.title), ['Add an artist', 'Add a release', 'Find an artist that is online']);
});

test('the texts of the Welcome page, the host rows and the steps do not use the word feed', () => {
  const texts = [
    WELCOME, HOST_ROWS(false), HOST_ROWS(true), welcomeSteps('music'), welcomeSteps('label'),
    pathTexts('music'), pathTexts('label'), FIRST_STEPS,
  ];
  for (const text of texts) {
    assert.doesNotMatch(JSON.stringify(text), FEED_WORD);
  }
});

test('a first-step row gives the start values of its form (ADR 0119 item 8)', () => {
  assert.deepEqual(firstStart('album', 'album', { title: 'HeyCitizen' }), { author: 'HeyCitizen' });
  assert.deepEqual(firstStart('label', 'publisher', { title: 'Night Signals Records' }), { role: 'artist' });
  assert.deepEqual(firstStart('label', 'album', { title: 'Night Signals Records' }), {});
});

test('no UI file names the practice pages or the button Publish for Real (ADR 0122 item 11)', () => {
  const uiDir = new URL('../ui/', import.meta.url);
  const files = readdirSync(uiDir).filter((name) => /\.(js|css|html)$/.test(name));
  assert.ok(files.length > 0);
  for (const name of files) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    assert.doesNotMatch(text, /practice|publish for real/i, name);
  }
});

test('the File hosts row of a temporary area and the expired notice use the texts of ADR 0122 items 3 and 10', () => {
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  // The row of the temporary area shows its text and has no Edit button (item 9).
  assert.match(app, /isTemporary && html`<p class="hint">\$\{HOST_STEP\.temporary\}<\/p>`/);
  assert.match(app, /!isTemporary && html`<button class="button" type="button" disabled=\$\{removeBusy\} onClick=\$\{startEdit\}>Edit<\/button>`/);
  // A feed of an expired area shows the notice of item 10 in the place of the no-file notice.
  assert.match(app, /feed\.temporary_expired\s*\? html`<div class="notice" role="status">\$\{HOST_STEP\.expired\}<\/div>`/);
  assert.equal(HOST_STEP.expired, 'The temporary area has expired. Move this feed to a real file host.');
});

test('the track page shows the text of ADR 0122 item 14 under a placeholder address', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.equal(HOST_STEP.placeholderAddress, 'Studio cannot make the address of this file yet. The address comes from the public address of the feed, and this file host has none. Studio sets the address when the feed gets a public address.');
  assert.match(editor, /isPlaceholderAddress\(value\.url\) \? HOST_STEP\.placeholderAddress : null/);
});

test('releaseAdd gives the add row of the Releases section (ADR 0164 items 1 and 2)', () => {
  const artist = { id: 1, medium: 'publisher', connector_id: 4, publisher_roles: ['artist'], release_ids: [7], child_ids: [] };
  assert.deepEqual(releaseAdd(artist, { title: 'HeyCitizen' }), { label: 'Add an Album', start: { author: 'HeyCitizen' } });
  const label = { ...artist, publisher_roles: ['label'] };
  assert.deepEqual(releaseAdd(label, { title: 'A Label' }), { label: 'Add a Release', start: {} });
  // The first-step row shows in its place.
  assert.equal(releaseAdd({ ...artist, release_ids: [] }, { title: 'HeyCitizen' }), null);
  // No file host, so the feed is read only.
  assert.equal(releaseAdd({ ...artist, connector_id: null }, { title: 'HeyCitizen' }), null);
  assert.equal(releaseAdd({ ...artist, medium: 'music' }, { title: 'HeyCitizen' }), null);
});
