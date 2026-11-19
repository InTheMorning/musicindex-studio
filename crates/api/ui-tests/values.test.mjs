// Payment block creation waits for the first saved recipient (task 180).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../ui/values.js', import.meta.url), 'utf8');
const section = (name, next) => source.slice(source.indexOf(`export function ${name}`), source.indexOf(`export function ${next}`));

test('the album open paths show a temporary block and update only after a recipient save', () => {
  const album = section('AlbumPayments', 'TrackPayments');
  const recipients = source.slice(source.indexOf('function Recipients('), source.indexOf('function RecipientSheet('));
  assert.match(album, /const showTemporary = model\.values\.length === 0 && \(adding \|\| openAdd\)/);
  assert.match(album, /onClick=\$\{\(\) => setAdding\(true\)\}/);
  assert.match(album, /onChange=\$\{saveTemporary\}/);
  assert.match(album, /const saveTemporary = \(values\) => \{ setAdding\(false\); onChange\(values\); \}/);
  assert.match(recipients, /const added = withCids\(\[\{ \.\.\.blank, \.\.\.values \}\]\)\[0\];[\s\S]*onChange\(\[\.\.\.recipients, added\]\)/);
  assert.match(album, /blocks=\$\{temporary\} onChange=\$\{saveTemporary\} openAdd=\$\{openAdd \|\| adding\}/);
  assert.match(album, /onEmptyClose=\$\{\(\) => setAdding\(false\)\}/);
});

test('the track open path waits for the first recipient when the album has none', () => {
  const track = source.slice(source.indexOf('export function TrackPayments'));
  assert.match(track, /if \(albumBlocks\.length === 0\) \{ setAdding\(true\); return; \}/);
  assert.match(track, /blocks=\$\{temporary\} onChange=\$\{saveTemporary\} openAdd=\$\{true\}/);
  assert.match(track, /const saveTemporary = \(values\) => \{ setAdding\(false\); onChange\(values\); \}/);
  assert.match(track, /onEmptyClose=\$\{\(\) => setAdding\(false\)\}/);
  assert.match(track, /if \(albumBlocks\.length === 0\) \{ setAdding\(true\); return; \}[\s\S]*onChange\(withCids\(fresh\(\[albumBlocks\[0\]\]\)/);
});
