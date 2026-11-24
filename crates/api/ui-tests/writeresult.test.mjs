import { test } from 'node:test';
import assert from 'node:assert/strict';
import { writeResult, alsoRows, rowsToWrite, pageWriteFailed, writeInOrder, reviewRemoveLabel, discardFailureTitle } from '../ui/writeresult.js';

test('a feed with a public address gives its written line and address', () => {
  assert.deepEqual(writeResult(
    { title: 'Songs', public_url: 'https://example.test/feed.xml' },
    { kind: 's3', name: 'My bucket' },
  ), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    address: 'https://example.test/feed.xml',
  });
});

test('a linked page appears in the result', () => {
  assert.deepEqual(writeResult(
    { title: 'Songs', public_url: 'https://example.test/feed.xml' },
    { kind: 's3', name: 'My bucket' },
    { id: 7, title: 'Artist page' },
  ), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    changedPage: { id: 7, title: 'Artist page' },
    address: 'https://example.test/feed.xml',
  });
});

test('a local host without a public address gives the folder warning', () => {
  assert.deepEqual(writeResult({ title: 'Songs', public_url: null }, { kind: 'local', name: 'Music' }), {
    title: 'Written',
    line: 'Songs is written to Music.',
    warning: { title: 'Listeners cannot get it yet.', text: 'This folder has no public address.' },
  });
});

test('an S3 host without a public address gives the file host warning', () => {
  assert.deepEqual(writeResult({ title: 'Songs' }, { kind: 's3', name: 'My bucket' }), {
    title: 'Written',
    line: 'Songs is written to My bucket.',
    warning: { title: 'Listeners cannot get it yet.', text: 'This file host gives no public address.' },
  });
});

test('a linked page that Review wrote has no "also changed" line', () => {
  const connector = { kind: 's3', name: 'My bucket' };
  const feed = { title: 'Songs', public_url: 'https://example.test/feed.xml' };
  const page = { id: 7, title: 'Artist page' };
  assert.equal(writeResult(feed, connector, page, [7]).changedPage, undefined);
  assert.deepEqual(writeResult(feed, connector, page, [8]).changedPage, page);
});

const also = [
  { id: 7, title: 'Artist', artist: true, side: 'page', change: 'added' },
  { id: 8, title: 'Label', artist: false, side: 'page', change: 'added' },
];
const diffs = { 7: { id: 'd7', changed: true }, 8: { id: 'd8', changed: true } };

test('each page has a locked check box row, on at the start', () => {
  const rows = alsoRows(also, diffs, new Set(), 'Album');
  assert.deepEqual(rows.map((row) => [row.id, row.checked, row.label, row.note]), [
    [7, true, 'Also write Artist', 'Adds the link to Album on the artist page.'],
    [8, true, 'Also write Label', 'Adds the link to Album on Label.'],
  ]);
  assert.ok(rows.every((row) => row.locked));
  assert.deepEqual(rowsToWrite(rows).map((row) => row.id), [7, 8]);
});

test('page check boxes stay on, and a page with no diff has no row', () => {
  const rows = alsoRows(also, { 7: diffs[7] }, new Set([7]), '');
  assert.deepEqual(rows.map((row) => [row.id, row.checked, row.locked]), [[7, true, true]]);
  assert.equal(rows[0].note, 'Adds the link to this feed on the artist page.');
  assert.deepEqual(rowsToWrite(rows).map((row) => row.id), [7]);
  assert.deepEqual(rowsToWrite(alsoRows(also, diffs, new Set([8]), 'Album')).map((row) => row.id), [7, 8]);
  assert.deepEqual(alsoRows(undefined, diffs, new Set(), 'Album'), []);
});

test('Review removes a new release and discards other drafts', () => {
  assert.equal(reviewRemoveLabel('new'), 'Remove from Collection…');
  assert.equal(reviewRemoveLabel('changed'), 'Discard the Draft');
  assert.equal(discardFailureTitle(), 'Studio could not discard the draft.');
});

test('the release is written first and each page after it, in order', async () => {
  const calls = [];
  const step = (name) => async () => { calls.push(name); return { name }; };
  const result = await writeInOrder({
    release: step('release'),
    pages: [{ title: 'Artist', run: step('artist') }, { title: 'Label', run: step('label') }],
    onReleaseWritten: async (feed) => { calls.push(`done ${feed.name}`); },
  });
  assert.deepEqual(calls, ['release', 'done release', 'artist', 'label']);
  assert.deepEqual(result, { feed: { name: 'release' }, beforeFeeds: [], pageFeeds: [{ name: 'artist' }, { name: 'label' }] });
});

test('ADR 0251 item 5: alsoRows gives each added and removed link text', () => {
  const rows = alsoRows([
    { id: 1, title: 'Artist', side: 'page', artist: true, change: 'added' },
    { id: 2, title: 'Label', side: 'page', artist: false, change: 'added' },
    { id: 3, title: 'Artist', side: 'page', artist: true, change: 'removed' },
    { id: 4, title: 'Label', side: 'page', artist: false, change: 'removed' },
  ], { 1: diffs[7], 2: diffs[7], 3: diffs[7], 4: diffs[7] }, new Set(), 'Album');
  assert.deepEqual(rows.map((row) => row.note), [
    'Adds the link to Album on the artist page.',
    'Adds the link to Album on Label.',
    'Removes the link to Album from the artist page.',
    'Removes the link to Album from Label.',
  ], 'ADR 0251 item 5: a page row is in Review of a release. Fix linkNote in writeresult.js.');
  const releaseRows = alsoRows([
    { id: 5, title: 'Album', side: 'release', artist: false, change: 'added' },
    { id: 6, title: 'Album', side: 'release', artist: false, change: 'removed' },
  ], { 5: diffs[7], 6: diffs[7] }, new Set(), 'Artist');
  assert.deepEqual(releaseRows.map((row) => row.note), [
    'Adds Artist as the artist or label of Album.',
    'Removes Artist from the artist or label of Album.',
  ], 'ADR 0251 item 5: a release row is in Review of a page. Fix linkNote in writeresult.js.');
  rows.push(...releaseRows);
  assert.ok(rows.every((row) => row.locked && row.checked));
});

test('ADR 0251 item 12: alsoRows names the release for a changed page link', () => {
  const rows = alsoRows([
    { id: 4, title: 'Release', side: 'release', artist: false, change: 'added', via: null },
    { id: 5, title: 'New page', side: 'page', artist: true, change: 'added', via: 4 },
  ], { 4: diffs[7], 5: diffs[7] }, new Set(), 'Artist');
  assert.deepEqual(rows[1], {
    id: 5, title: 'New page', side: 'page', artist: true, change: 'added', via: 4,
    diff: diffs[7], checked: true, locked: true, label: 'Also write New page',
    note: 'Adds the link to Release on New page.',
  }, 'ADR 0251 item 12: the linked page row is locked and names its release. Fix alsoRows in writeresult.js.');
});

test('ADR 0251 item 7: writeInOrder writes release rows before the page', async () => {
  const calls = [];
  const step = (name) => async () => { calls.push(name); return { name }; };
  const result = await writeInOrder({
    before: [{ title: 'Release', run: step('release') }],
    release: step('page'),
    pages: [],
  });
  assert.deepEqual(calls, ['release', 'page']);
  assert.deepEqual(result.beforeFeeds, [{ name: 'release' }]);
});

test('ADR 0251 item 8: writeInOrder stops after a failed release row', async () => {
  const calls = [];
  await assert.rejects(writeInOrder({
    before: [{ title: 'Release', run: async () => { calls.push('release'); throw new Error('refused'); } }],
    release: async () => { calls.push('page'); return {}; },
    pages: [{ title: 'Later page', run: async () => { calls.push('later'); } }],
  }), (error) => error.pageTitle === 'Release' && /refused/.test(error.message));
  assert.deepEqual(calls, ['release']);
});

test('a failed release write gives no page write', async () => {
  const calls = [];
  const failure = Object.assign(new Error('refused'), { code: 'conflict' });
  await assert.rejects(writeInOrder({
    release: async () => { throw failure; },
    pages: [{ title: 'Artist', run: async () => { calls.push('artist'); return {}; } }],
    onReleaseWritten: async () => { calls.push('done'); },
  }), (error) => error === failure && error.pageTitle === undefined);
  assert.deepEqual(calls, []);
});

test('a failed page write keeps the release written and names the page', async () => {
  const calls = [];
  const failure = Object.assign(new Error('refused'), { code: 'diff_not_approved' });
  await assert.rejects(writeInOrder({
    release: async () => { calls.push('release'); return { name: 'release' }; },
    pages: [
      { title: 'Artist', run: async () => { calls.push('artist'); throw failure; } },
      { title: 'Label', run: async () => { calls.push('label'); return {}; } },
    ],
    onReleaseWritten: async () => { calls.push('done'); },
  }), (error) => error === failure && error.pageTitle === 'Artist' && error.code === 'diff_not_approved');
  assert.deepEqual(calls, ['release', 'done', 'artist']);
});

test('the notice of a failed page write names the page', () => {
  assert.equal(pageWriteFailed('Artist'), 'Studio could not write Artist.');
});
