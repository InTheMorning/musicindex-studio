// ADR 0223 items 10 to 12: the label of a new release, the option "From the
// tracks" and the button "Choose Folder…" of the first-run library sheet.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { feedKind } from '../ui/shell.js';
import { releaseKindOptions } from '../ui/addfeeds.js';

test('feedKind gives Release for the kind release', () => {
  const owner = 'ADR 0223 item 11: a release with no track and no live item has the label Release. Fix feedKind in crates/api/ui/shell.js.';
  assert.equal(feedKind({ medium: 'music', kind_shown: 'release' }).label, 'Release', owner);
});

test('the option From the tracks names the kind that the tracks give', () => {
  const owner = 'ADR 0223 item 12: the option shows its kind. Fix releaseKindOptions in crates/api/ui/addfeeds.js.';
  assert.equal(releaseKindOptions('single', 2, 0)[0][1], 'From the tracks (Single)', owner);
  assert.equal(releaseKindOptions('release', 0, 0)[0][1], 'From the tracks (no tracks yet)', owner);
  assert.equal(releaseKindOptions('album', 0, 1)[0][1], 'From the tracks (Album)', owner);
  assert.deepEqual(releaseKindOptions('single', 2, 0).slice(1).map((o) => o[0]), ['album', 'ep', 'single'], owner);
});

test('the library sheet has the button Choose Folder and its route', () => {
  const owner = 'ADR 0223 item 10: the first-run sheet has Choose Folder. Fix LibrarySheet in crates/api/ui/app.js.';
  const source = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = source.indexOf('function LibrarySheet');
  const sheet = source.slice(start, source.indexOf('\nfunction ', start + 10));
  assert.ok(sheet.includes('Choose Folder…'), owner);
  assert.ok(sheet.includes('/choose-folder'), owner);
});
