// Task 188, ADR 0223 items 7 and 8: the narrow root page and one tap to a release.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { openFeedView, widthNav } from '../ui/shell.js';

const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
const root = { name: 'root' };
const releases = { name: 'releases' };

test('a feed opens as the feed view at each width', () => {
  assert.deepEqual(openFeedView(5, true), { name: 'feed', id: 5 });
  assert.deepEqual(openFeedView(5, false), { name: 'feed', id: 5 });
});

test('the narrow stack of a feed is root, Releases, feed', () => {
  assert.deepEqual(widthNav({ name: 'feed', id: 5 }, true).entries, [root, releases, { name: 'feed', id: 5 }]);
});

test('the narrow stack of an outline is root, Releases, feed, outline', () => {
  const nav = widthNav({ name: 'outline', id: 5 }, true);
  assert.deepEqual(nav.entries, [root, releases, { name: 'feed', id: 5 }, { name: 'outline', id: 5 }]);
  assert.equal(nav.index, 3);
});

test('the outline view keeps its wide mapping to the feed view', () => {
  assert.deepEqual(widthNav({ name: 'outline', id: 5 }, false).entries, [{ name: 'feed', id: 5 }]);
});

test('ADR 0223 item 7: RootPage renders People, File hosts and Settings before destinationRows. Move the fixed rows in RootPage of app.js.', () => {
  const start = app.indexOf('function RootPage');
  const body = app.slice(start, app.indexOf('\n}\n', start));
  const fixed = body.indexOf("['people', 'hosts', 'settings']");
  assert.ok(fixed > -1 && fixed < body.indexOf('destinationRows('));
});

test('ADR 0223 item 8: the bar of the feed page has the button Sections. Add it in EditableFeed of app.js.', () => {
  const start = app.indexOf('function EditableFeed');
  const bar = app.indexOf('<${Bar}>', start);
  const sections = app.indexOf('>Sections</button>', start);
  assert.ok(sections > bar && sections < app.indexOf('<${DraftToolbar}', bar));
});
