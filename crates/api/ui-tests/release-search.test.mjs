import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { releasesList, overviewModel } from '../ui/shell.js';

const feeds = [
  { id: 1, title: 'Night Drive', medium: 'music', publisher_ids: [], added_at: 2 },
  { id: 2, title: 'Album 2', medium: 'music', publisher_ids: [], added_at: 1 },
  { id: 10, title: 'Artist X', medium: 'publisher', role_set: ['artist'], publisher_roles: ['artist'], release_ids: [3, 4] },
  { id: 3, title: 'First Release', medium: 'music', publisher_ids: [10], added_at: 4 },
  { id: 4, title: 'Second Release', medium: 'music', publisher_ids: [10], added_at: 3 },
];

const list = (query, extra = {}) => releasesList({
  feeds,
  connectors: [],
  group: 'none',
  query,
  ...extra,
});

test('release search matches a release title', () => {
  assert.deepEqual(list('night').rows.map((row) => row.title), ['Night Drive']);
  assert.deepEqual(list('night').rows.map((row) => row.id), [1]);
});

test('release search matches the title of a publisher feed', () => {
  const model = overviewModel({ feeds, filter: 'all', query: 'artist x', marked: null });
  const ids = JSON.stringify(model.parts);
  assert.ok(ids.includes('"id":3') && ids.includes('"id":4'));
  assert.ok(!ids.includes('"id":1,'));
});

test('release search ignores letter case', () => {
  assert.deepEqual(list('NIGHT').rows, list('night').rows);
});

test('an unmatched release search gives no rows', () => {
  const result = list('no match');
  assert.deepEqual(result.rows, []);
});

test('People and Releases use the shared SearchField', () => {
  const people = readFileSync(new URL('../ui/peoplepages.js', import.meta.url), 'utf8');
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const page = app.slice(app.indexOf('function OverviewPage('), app.indexOf('// The add menu of Overview'));
  assert.match(people, /<\$\{SearchField/);
  assert.doesNotMatch(people, /class="field search-field"/);
  assert.match(page, /<\$\{SearchField/);
});
