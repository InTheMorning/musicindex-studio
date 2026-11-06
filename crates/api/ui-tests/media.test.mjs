import test from 'node:test';
import assert from 'node:assert/strict';
import { playerProgress, HTTP_NOTICE, PLAYER_ERROR, DIRECT_PLAYER_ERROR, canPlay, heldMap, isPlaceholderAddress, mediaSource, playerError, playerNotice, plainAddress, playWasReplaced } from '../ui/media.js';

// ADR 0109 item 5 and the acceptance criteria: the source of an image and of
// the player. Fix a failure in crates/api/ui/media.js, not in the test.

const held = heldMap({ files: [{ file: 7, url: 'https://music.example/a.mp3', media_type: 'audio/mpeg' }] });

test('ADR 0109: an https address of a file on its host loads from the host', () => {
  assert.deepEqual(mediaSource('https://host.example/cover.jpg', held), { kind: 'direct', src: 'https://host.example/cover.jpg' });
  assert.deepEqual(mediaSource('HTTPS://host.example/a.mp3', held).kind, 'direct');
});

test('ADR 0109 item 5: a file that only Studio holds loads through Studio by its ID', () => {
  assert.deepEqual(mediaSource('https://music.example/a.mp3', held), { kind: 'held', file: 7 });
});

test('ADR 0109 item 4: an http address gives no source and the player notice', () => {
  const source = mediaSource('http://host.example/a.mp3', held);
  assert.deepEqual(source, { kind: 'none', reason: 'http' });
  assert.equal(playerNotice(source), 'This file uses http. Studio cannot play it here.');
  assert.equal(HTTP_NOTICE, 'This file uses http. Studio cannot play it here.');
  assert.equal(canPlay(source), false);
});

test('ADR 0109: an empty, a wrong and a not yet known address give no source and no notice', () => {
  for (const url of [null, undefined, '', '  ', 'ftp://host.example/a', 'data:image/png;base64,AA', 'cover.jpg']) {
    const source = mediaSource(url, held);
    assert.equal(source.kind, 'none', String(url));
    assert.equal(playerNotice(source), null, String(url));
  }
  const loading = mediaSource('https://host.example/a.mp3', null);
  assert.deepEqual(loading, { kind: 'none', reason: 'loading' });
  assert.equal(canPlay(loading), false);
});

test('ADR 0076: the OP3 prefix does not change the source of the player', () => {
  assert.equal(plainAddress('https://op3.dev/e,pg=abc/music.example/a.mp3'), 'https://music.example/a.mp3');
  assert.equal(plainAddress('https://op3.dev/e/music.example/a.mp3'), 'https://music.example/a.mp3');
  assert.equal(plainAddress('https://op3.dev/e,pg=abc/http://music.example/a.mp3'), 'http://music.example/a.mp3');
  assert.equal(plainAddress('https://op3.dev/show/abc'), 'https://op3.dev/show/abc');
  assert.deepEqual(mediaSource('https://op3.dev/e,pg=abc/music.example/a.mp3', held), { kind: 'held', file: 7 });
  assert.equal(mediaSource('https://op3.dev/e,pg=abc/http://music.example/a.mp3', held).reason, 'http');
});

test('heldMap ignores a bad row', () => {
  assert.equal(heldMap(null).size, 0);
  assert.equal(heldMap({ files: [{ file: 'x', url: 'https://a.example/' }, { file: 1 }, null] }).size, 0);
});

test('a file from its host names the browser setting in the player error, and a held file does not', () => {
  const held = new Map([['https://music.example/held.mp3', 7]]);
  const direct = mediaSource('https://files.example/a.mp3', held);
  assert.equal(playerError(direct), DIRECT_PLAYER_ERROR);
  assert.equal(DIRECT_PLAYER_ERROR, 'Studio cannot play this file. Your browser can block audio from other sites.');
  assert.equal(playerError(mediaSource('https://music.example/held.mp3', held)), PLAYER_ERROR);
  assert.equal(PLAYER_ERROR, 'Studio cannot play this file.');
});

test('ADR 0122 item 14: a placeholder address plays only from the file that Studio holds', () => {
  const placeholder = 'pending:my-album/01 Song.mp3';
  const holding = heldMap({ files: [{ file: 9, url: placeholder }] });
  assert.equal(isPlaceholderAddress(placeholder), true);
  assert.equal(isPlaceholderAddress('https://music.example/a.mp3'), false);
  assert.deepEqual(mediaSource(placeholder, holding), { kind: 'held', file: 9 });
  assert.deepEqual(mediaSource(placeholder, null), { kind: 'none', reason: 'loading' });
  assert.deepEqual(mediaSource('pending:other.mp3', holding), { kind: 'none', reason: 'other' });
});

test('a play that a later load replaced is not an error', () => {
  assert.equal(playWasReplaced({ name: 'AbortError' }), true, 'The human test of 0.1.0-rc.18 showed a false error while the audio started. Fix: keep the AbortError out of the player error.');
  assert.equal(playWasReplaced({ name: 'NotSupportedError' }), false);
  assert.equal(playWasReplaced(null), false);
});

test('the position bar gives one value for its fill and its cursor', () => {
  assert.equal(playerProgress(10, 200), 0.05);
  assert.equal(playerProgress(0, NaN), 0);
  assert.equal(playerProgress(10, 0), 0);
  assert.equal(playerProgress(300, 200), 1);
  assert.equal(playerProgress(-1, 200), 0);
});
