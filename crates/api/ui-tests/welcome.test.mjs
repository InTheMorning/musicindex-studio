// The Welcome page of a new user and its first steps (ADR 0119, task 083).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  firstStart,
  WELCOME, showWelcome, welcomeSteps, pathTexts, hostStep, HOST_ROWS, welcomeBody, nextStep, FIRST_STEPS,
} from '../ui/welcome.js';

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

test('hostStep gives pick, skip and menu for zero, one and two file hosts', () => {
  assert.equal(hostStep([]), 'pick');
  assert.equal(hostStep([{ id: 1, name: 'A' }]), 'skip');
  assert.equal(hostStep([{ id: 1, name: 'A' }, { id: 2, name: 'B' }]), 'menu');
});

test('HOST_ROWS gives the rows in the sequence of the operator, with the type of each form', () => {
  const desktop = HOST_ROWS(false);
  assert.deepEqual(desktop.map((r) => r.title), ['My own server', 'Nostr site (nsite)', 'S3 or Cloudflare R2']);
  assert.deepEqual(desktop.map((r) => r.type), ['local', 'nsite', 's3']);
  assert.equal(desktop[0].primary, true);
  assert.equal(desktop[1].primary, undefined);
  assert.deepEqual(HOST_ROWS(true).map((r) => r.type), ['sftp', 'nsite', 's3']);
  assert.equal(desktop[1].line, 'A site on Nostr. Its pages and songs go to Blossom servers, signed with your Nostr key.');
  assert.equal(desktop[2].line, 'A storage bucket at Amazon S3, Cloudflare R2 or a similar provider.');
});

test('the row My own server gives the two lines of its platform', () => {
  assert.equal(HOST_ROWS(false)[0].line, 'A folder on this computer, to try Studio first. Or your own SFTP server.');
  assert.equal(HOST_ROWS(true)[0].line, 'Your own SFTP server.');
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
