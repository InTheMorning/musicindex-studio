// The picker model (ADR 0062, ADR 0042). Each test checks one rule.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  fieldKind, sources, hostTree, canSelect, reasonText, acceptsFile, dropResult,
  replacedAddresses, selectedFacts, kindOfFile, NO_ADDRESS_TEXT,
} from '../ui/picker.js';

const A = 'https://m.example/a.jpg';
const B = 'https://m.example/b.jpg';

test('fieldKind gives the kind of each file field', () => {
  assert.equal(fieldKind('cover'), 'image');
  assert.equal(fieldKind('track-image'), 'image');
  assert.equal(fieldKind('live-image'), 'image');
  assert.equal(fieldKind('artwork'), 'image');
  assert.equal(fieldKind('track-audio'), 'audio');
  assert.equal(fieldKind('recording'), 'audio');
  assert.equal(fieldKind('lyrics'), 'lyrics');
  assert.equal(fieldKind('title'), null);
});

test('sources gives the three sources in sequence, with one host entry for each connector', () => {
  const list = sources('cover', [{ id: 1, name: 'One' }, { id: 2, name: 'Two' }]);
  assert.deepEqual(list.map((s) => s.id), ['release', 'hosts', 'computer']);
  assert.deepEqual(list[1].hosts, [{ connectorId: 1, name: 'One' }, { connectorId: 2, name: 'Two' }]);
  assert.match(sources('cover', [])[2].accept, /\.png/);
  assert.match(sources('track-audio', [])[2].accept, /\.flac/);
  assert.deepEqual(sources('cover', [])[1].hosts, []);
});

test('hostTree makes folders from nested paths', () => {
  const tree = hostTree([
    { path: 'b/two.jpg', address: B },
    { path: 'a/deep/one.png', address: A },
    { path: 'a/x.png', address: A },
    { path: 'top.gif', address: A },
  ]);
  assert.deepEqual(tree.folders.map((f) => f.name), ['a', 'b']);
  assert.deepEqual(tree.files.map((f) => f.name), ['top.gif']);
  const a = tree.folders[0];
  assert.equal(a.path, 'a');
  assert.deepEqual(a.files.map((f) => f.name), ['x.png']);
  assert.equal(a.folders[0].path, 'a/deep');
  assert.equal(a.folders[0].files[0].path, 'a/deep/one.png');
});

test('hostTree gives one flat level for paths with no folder', () => {
  const tree = hostTree([{ path: 'b.jpg' }, { path: 'a.jpg' }]);
  assert.deepEqual(tree.folders, []);
  assert.deepEqual(tree.files.map((f) => f.name), ['a.jpg', 'b.jpg']);
  assert.deepEqual(hostTree([]).files, []);
});

test('canSelect refuses a file with no address and gives the reason', () => {
  assert.equal(canSelect({ address: A }), true);
  assert.equal(canSelect({ address: null }), false);
  assert.equal(canSelect({ address: '' }), false);
  assert.equal(canSelect({}), false);
  assert.equal(reasonText({ address: null }), NO_ADDRESS_TEXT);
  assert.equal(NO_ADDRESS_TEXT, 'This file has no public address');
  assert.equal(reasonText({ address: A }), '');
});

test('acceptsFile compares the kind of the upload with the kind of the field', () => {
  assert.equal(acceptsFile('cover', { kind: 'image' }), true);
  assert.equal(acceptsFile('cover', { kind: 'audio' }), false);
  assert.equal(acceptsFile('track-audio', { kind: 'audio' }), true);
  assert.equal(acceptsFile('track-audio', { kind: 'image' }), false);
});

test('dropResult uploads one file of the right kind', () => {
  const file = { name: 'cover.PNG', type: '' };
  assert.deepEqual(dropResult('cover', [file]), { action: 'upload', file });
  const song = { name: 'a.flac', type: '' };
  assert.equal(dropResult('track-audio', [song]).action, 'upload');
  assert.equal(dropResult('track-audio', [{ name: 'a.wav', type: 'audio/wav' }]).action, 'upload');
});

test('dropResult refuses two files and a file of the wrong kind', () => {
  const image = { name: 'a.jpg', type: 'image/jpeg' };
  const two = dropResult('cover', [image, image]);
  assert.equal(two.action, 'error');
  assert.equal(two.code, 'too_many');
  const wrong = dropResult('cover', [{ name: 'a.mp3', type: 'audio/mpeg' }]);
  assert.equal(wrong.code, 'wrong_kind');
  assert.equal(wrong.text, 'Select an image file');
  assert.equal(dropResult('track-audio', [image]).text, 'Select an audio file');
  assert.equal(dropResult('cover', [{ name: 'notes.txt', type: 'text/plain' }]).code, 'wrong_kind');
  assert.equal(dropResult('cover', []).action, 'none');
});

test('dropResult accepts a lyrics file on a lyrics field and refuses an image', () => {
  const lyrics = { name: 'words.SRT', type: '' };
  assert.deepEqual(dropResult('lyrics', [lyrics]), { action: 'upload', file: lyrics });
  for (const name of ['a.vtt', 'a.txt', 'a.json', 'a.html']) {
    assert.equal(dropResult('lyrics', [{ name, type: '' }]).action, 'upload', name);
  }
  const wrong = dropResult('lyrics', [{ name: 'a.png', type: 'image/png' }]);
  assert.equal(wrong.code, 'wrong_kind');
  assert.equal(wrong.text, 'Select a lyrics file');
});

test('kindOfFile reads the extension first, then the media type', () => {
  assert.equal(kindOfFile({ name: 'x.opus', type: '' }), 'audio');
  assert.equal(kindOfFile({ name: 'noext', type: 'image/avif' }), 'image');
  assert.equal(kindOfFile({ name: 'noext', type: '' }), null);
});

test('replacedAddresses gives each cover address except the new one, once', () => {
  const model = { image: A, rss_image: { url: B } };
  assert.deepEqual(replacedAddresses('cover', model, 'https://m.example/c.jpg'), [A, B]);
  assert.deepEqual(replacedAddresses('cover', model, A), [B]);
  assert.deepEqual(replacedAddresses('cover', { image: A, rss_image: { url: A } }, B), [A]);
  assert.deepEqual(replacedAddresses('cover', { image: null, rss_image: null }, B), []);
});

test('replacedAddresses gives the image of a track and of a live show', () => {
  assert.deepEqual(replacedAddresses('track-image', { image: A }, B), [A]);
  assert.deepEqual(replacedAddresses('live-image', { image: null }, B), []);
});

test('replacedAddresses gives the address of an image entry and not the track image', () => {
  assert.deepEqual(replacedAddresses('artwork', { href: A, image: B }, 'https://m.example/c.jpg'), [A]);
  assert.deepEqual(replacedAddresses('artwork', { href: '' }, B), []);
});

test('replacedAddresses gives the address of a lyrics entry and not the track image', () => {
  assert.deepEqual(replacedAddresses('lyrics', { url: A, image: B }, 'https://m.example/new.srt'), [A]);
  assert.deepEqual(replacedAddresses('lyrics', { url: '' }, B), []);
});

test('replacedAddresses gives the enclosure and the audio sources of an audio field', () => {
  const item = {
    enclosure: { url: 'https://m.example/a.mp3' },
    alternate_enclosures: [
      { media_type: 'audio/flac', sources: [{ uri: 'https://m.example/a.flac' }] },
      { media_type: 'video/mp4', sources: [{ uri: 'https://m.example/v.mp4' }] },
    ],
  };
  assert.deepEqual(replacedAddresses('track-audio', item, 'https://m.example/n.mp3'),
    ['https://m.example/a.mp3', 'https://m.example/a.flac']);
  assert.deepEqual(replacedAddresses('track-audio', { enclosure: null, alternate_enclosures: [] }, A), []);
});

test('selectedFacts gives the address, the size and the media type of a listed file', () => {
  assert.deepEqual(selectedFacts({ address: A, size: 12, media_type: 'audio/mpeg', name: 'x' }),
    { url: A, size: 12, media_type: 'audio/mpeg' });
  assert.equal(selectedFacts({ address: A, size: 0, media_type: null }).size, null);
});

test('editor.js has no old file button', () => {
  const source = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.equal(source.includes('Replace the Audio File'), false);
  assert.equal(source.includes('Import Image'), false);
});
