import test from 'node:test';
import assert from 'node:assert/strict';
import { HTTP_NOTICE, canPlay, heldMap, mediaSource, playerNotice, plainAddress } from '../ui/media.js';

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
