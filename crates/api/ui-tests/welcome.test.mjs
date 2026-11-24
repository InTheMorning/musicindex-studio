// The Welcome page of a new user and its first steps (ADR 0221, task 083).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import {
  firstStart, releaseAdd, releaseArtistChoices, libraryResult,
  WELCOME, HOST_STEP, showWelcome, welcomeSteps, pathTexts, hostStepModel, HOST_ROWS, hostTypeOptions, welcomeBody, nextStep, FIRST_STEPS,
} from '../ui/welcome.js';
import { isTitleCase } from '../ui/casing.js';

const FEED_WORD = /\bfeeds?\b/i;

test('showWelcome is true for an empty feed list and false with one feed', () => {
  assert.equal(showWelcome([]), true);
  assert.equal(showWelcome(undefined), true);
  assert.equal(showWelcome([{ id: 1, medium: 'music' }]), false);
});

test('welcomeSteps gives the two stages of each path', () => {
  assert.deepEqual(welcomeSteps('music'), ['Your name', 'Where it lives']);
  assert.deepEqual(welcomeSteps('label'), ['Label name', 'Where it lives']);
});

test('the Welcome page has the hosting row and the hosting path has three steps (ADR 0240 items 2 and 3)', () => {
  const owner = 'ADR 0240 items 2 and 3: the third row and its steps. Fix WELCOME and PATHS in crates/api/ui/welcome.js.';
  assert.deepEqual(Object.keys(WELCOME.rows), ['music', 'label', 'hosting', 'online'], owner);
  assert.equal(WELCOME.rows.hosting.title, "I'm hosting for other artists", owner);
  assert.equal(WELCOME.rows.hosting.line, 'Start with your hosting page. The artists and labels that you host go on it.', owner);
  assert.deepEqual(welcomeSteps('hosting'), ['Your name', 'Where it lives', 'Your first artist'], owner);
  assert.equal(welcomeBody('hosting', 'Wrathful Test', 4).role, 'hosting', owner);
  assert.deepEqual(FIRST_STEPS.hosting.map((s) => s.title), ['Add an artist', 'Add a label', 'Find an artist that is online'], owner);
  assert.equal(nextStep({ id: 3, medium: 'publisher', publisher_roles: ['hosting'], child_ids: [] }), 'hosting', owner);
  assert.equal(nextStep({ id: 3, medium: 'publisher', publisher_roles: ['hosting'], child_ids: [8] }), null, owner);
  assert.deepEqual(firstStart('hosting', 'publisher', { title: 'Host' }, 3, 'label'), { role: 'label', below_id: 3 }, owner);
  assert.deepEqual(firstStart('hosting', 'publisher', { title: 'Host' }, 3, 'artist'), { role: 'artist', below_id: 3 }, owner);
  assert.deepEqual(firstStart('hosting', 'find', { title: 'Host' }, 3), {}, owner);
});

test('HOST_ROWS gives the desktop rows, with the folder as the first row', () => {
  const desktop = HOST_ROWS(false);
  assert.deepEqual(desktop.map((r) => r.title), ['Folder on this computer', 'Nostr site (nsite)', 'S3 or Cloudflare R2', 'SFTP server']);
  assert.deepEqual(desktop.map((r) => r.type), ['local', 'nsite', 's3', 'sftp']);
  assert.equal(desktop[0].primary, true);
  assert.equal(desktop[1].primary, undefined);
  assert.equal(desktop[1].line, 'A site on Nostr. Its pages and songs go to Blossom servers, the file servers of Nostr, signed with your Nostr key.');
  assert.equal(desktop[2].line, 'A storage bucket, an online folder at Amazon S3, Cloudflare R2 or a similar provider.');
  assert.equal(desktop[3].line, 'Your own server or web hosting account, which you reach with SFTP.');
  assert.equal(desktop[0].line, 'A folder on this computer. The public address is optional.');
});

test('HOST_ROWS gives the server rows, with the temporary area as the first row', () => {
  const server = HOST_ROWS(true);
  assert.deepEqual(server.map((r) => r.title), ['Try it here (temporary)', 'Nostr site (nsite)', 'S3 or Cloudflare R2', 'SFTP server']);
  assert.deepEqual(server.map((r) => r.type), ['temporary', 'nsite', 's3', 'sftp']);
  assert.equal(server[0].primary, true);
});

test('hostTypeOptions gives the shell default and offers only one temporary area (ADR 0122 items 3 and 9)', () => {
  assert.deepEqual(hostTypeOptions(false, []), [
    ['local', 'Folder on this computer'], ['s3', 'S3 or R2 bucket'], ['sftp', 'SFTP server'],
    ['nsite', 'nsite site'], ['blossom', 'Blossom server'],
  ]);
  assert.deepEqual(hostTypeOptions(true, [])[0], ['temporary', 'Temporary area (7 days)']);
  assert.equal(hostTypeOptions(true, [{ kind: 'temporary' }]).some(([kind]) => kind === 'temporary'), false);
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

test('hostStepModel removes the temporary row when the account has the temporary area (ADR 0122 item 9)', () => {
  const server = hostStepModel([{ id: 3, name: 'Temporary area', kind: 'temporary' }], true);
  assert.deepEqual(server.rows.map((r) => r.type), ['nsite', 's3', 'sftp'],
    'ADR 0122 item 9: an account has one temporary area only. Select the area in the menu, not in a second row.');
  assert.deepEqual(server.menu, { value: '3', options: [['3', 'Temporary area']] });
  assert.equal(server.selected, null);
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

test('libraryResult lists positive counts and gives the empty folder line', () => {
  assert.deepEqual(libraryResult({ release_folders: 2, feed_files: 0, moved_files: 1 }), {
    rows: [{ label: 'Release folders', count: 2 }, { label: 'Files moved', count: 1 }], empty: false,
  });
  assert.deepEqual(libraryResult({ release_folders: 0, feed_files: 0, moved_files: 0 }), { rows: [], empty: true });
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(app, /The folder has no releases yet\./);
  assert.match(app, /libraryResult\(report\)/);
});

test('the first-run page action uses Make the Page labels in step 2', () => {
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(app, /busy \? 'Making the Page…' : 'Make the Page'/);
  assert.match(app, /submitLabel="Make the Page" busyLabel="Making the Page…"/);
  assert.match(app, /type === 's3'[\s\S]*submitLabel=\$\{submitLabel \|\| 'Add the Bucket'\}[\s\S]*busyLabel=\$\{busyLabel \|\| 'Testing the Bucket…'\}/);
  assert.match(app, /type === 'sftp'[\s\S]*submitLabel=\$\{submitLabel \|\| 'Accept and Add the Server'\}/);
  assert.match(app, /type === 'nsite'[\s\S]*submitLabel=\$\{submitLabel \|\| 'Add the Site'\}/);
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

test('the texts of ADR 0221 items 2, 5, 6 and 8 are as decided', () => {
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
  assert.deepEqual(FIRST_STEPS.label.map((s) => s.title), ['Add an artist', 'Find an artist that is online']);
});

test('the texts of the Welcome page, the host rows and the steps do not use the word feed', () => {
  const texts = [
    WELCOME, HOST_ROWS(false), HOST_ROWS(true), welcomeSteps('music'), welcomeSteps('label'), welcomeSteps('hosting'),
    pathTexts('music'), pathTexts('label'), pathTexts('hosting'), FIRST_STEPS,
  ];
  for (const text of texts) {
    assert.doesNotMatch(JSON.stringify(text), FEED_WORD);
  }
});

test('a first-step row gives the start values of its form (ADR 0221 item 8)', () => {
  assert.deepEqual(firstStart('album', 'album', { title: 'HeyCitizen' }, 8), { author: 'HeyCitizen', publisher_id: 8 },
    'ADR 0256 item 7: the artist first step must pass its publisher ID. Fix firstStart in crates/api/ui/welcome.js.');
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

test('releaseAdd gives the add action only on an artist page (ADR 0256 item 4)', () => {
  const artist = { id: 1, medium: 'publisher', connector_id: 4, publisher_roles: ['artist'], release_ids: [7], child_ids: [] };
  const owner = 'ADR 0256 item 4: only a writable artist page gets Add a Release. Fix releaseAdd in crates/api/ui/welcome.js.';
  assert.deepEqual(releaseAdd(artist, 'HeyCitizen'), { label: 'Add a Release', start: { author: 'HeyCitizen', publisher_id: 1 } }, owner);
  const label = { ...artist, publisher_roles: ['label'] };
  assert.equal(releaseAdd(label, 'A Label'), null, owner);
  const hosting = { ...artist, publisher_roles: ['hosting'] };
  assert.equal(releaseAdd(hosting, 'A Host'), null, owner);
  assert.equal(releaseAdd({ ...artist, connector_id: null }, 'HeyCitizen'), null, owner);
  // The list view of a feed with no release also has the action.
  assert.equal(releaseAdd({ ...artist, release_ids: [] }, 'HeyCitizen').label, 'Add a Release', owner);
  assert.equal(releaseAdd({ ...artist, medium: 'music' }, 'HeyCitizen'), null, owner);
  assert.equal(releaseAdd(null, ''), null, owner);
});

test('a release start lists writable artist pages only (ADR 0256 items 4 and 5)', () => {
  const feeds = [
    { id: 1, medium: 'publisher', connector_id: 4, publisher_roles: ['artist'], title: 'Artist' },
    { id: 2, medium: 'publisher', connector_id: null, publisher_roles: ['artist'], title: 'Read-only artist' },
    { id: 3, medium: 'publisher', connector_id: 4, publisher_roles: ['label'], title: 'Label' },
  ];
  assert.deepEqual(releaseArtistChoices(feeds), [['1', 'Artist']],
    'ADR 0256 items 4 and 5: a release needs a writable artist page. Fix releaseArtistChoices in crates/api/ui/welcome.js.');
});

test('ADR 0231 item 6: the parent choices of a new release have artist pages only', () => {
  const feeds = [
    { id: 1, medium: 'publisher', connector_id: 4, publisher_roles: ['artist'], title: 'Artist' },
    { id: 3, medium: 'publisher', connector_id: 4, publisher_roles: ['label'], title: 'Label' },
    { id: 5, medium: 'publisher', connector_id: 4, publisher_roles: ['artist', 'label'], title: 'Both' },
    { id: 6, medium: 'music', connector_id: 4, publisher_roles: [], title: 'Album' },
  ];
  const ids = releaseArtistChoices(feeds).map(([id]) => id);
  assert.ok(!ids.includes('3'),
    'ADR 0231 item 6: the parent list has no label page. Fix releaseArtistChoices in crates/api/ui/welcome.js.');
  assert.ok(!ids.includes('6'),
    'ADR 0231 item 6: the parent list has no release. Fix releaseArtistChoices in crates/api/ui/welcome.js.');
});
