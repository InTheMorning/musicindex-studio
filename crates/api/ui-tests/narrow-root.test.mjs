// Task 188, ADR 0223 item 8, and task 193, ADR 0226 items 7 and 8: the narrow
// root page and one tap to a release.
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

test('ADR 0226 items 7 and 8: RootPage gives the full panel to LeftPanelFooter, as the wide panel does', () => {
  // rc.27 check finding 12: the narrow root page put People, File hosts and
  // Settings above the feed groups.
  const start = app.indexOf('function RootPage');
  const body = app.slice(start, app.indexOf('\n}\n', start));
  const fix = 'Render destinationRows(panel.top, go), then LeftPanelFooter with panel=${panel}, in RootPage of app.js.';
  assert.ok(body.indexOf('destinationRows(panel.top, go)') > -1, `ADR 0226 item 7: ${fix}`);
  assert.ok(body.indexOf('destinationRows(') < body.indexOf('<${LeftPanelFooter} panel=${panel}'), `ADR 0226 item 7: ${fix}`);
  assert.ok(!body.includes('panel.bottom'), `ADR 0226 item 8: RootPage must not split panel.bottom. ${fix}`);
});

test('ADR 0228 item 11: the button "‹ Overview" is at the left of the menu of the tabs, not in the bar. Fix PageTabs and EditableFeed in app.js.', () => {
  const tabs = app.indexOf('export function PageTabs');
  const button = app.indexOf('‹ Overview</button>', tabs);
  const menu = app.indexOf('${menu}</div></div>', button);
  assert.ok(button > tabs && menu > button);
  assert.ok(!app.includes('>Sections</button>'));
});
