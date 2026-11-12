// The model of a server that holds no assets (ADR 0105, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  dayText, canCopyFile, fileState, fileStatus, canDownloadToDevice, missingCount, missingNotice,
  reviewNotice, serverFilesRows, sizeText, SERVER_NOTICE, ADD_TRACKS_NOTICE,
} from '../ui/assets.js';

// The browser time is the time of the process. A local noon avoids a change
// of the day at the edge.
const at = (year, month, day) => new Date(year, month - 1, day, 12, 0, 0);
const sec = (date) => Math.floor(date.getTime() / 1000);
const NOW = at(2026, 11, 1);
const OFF = { holdAssets: false, now: NOW };

const row = (extra) => ({
  local: false, copies: [], in_use: true, removed_at: null, expires_at: null, ...extra,
});
const copy = { connector_id: 1, path: 'a/b.mp3' };

test('dayText gives the local day and month, and the year when it differs', () => {
  assert.equal(dayText(sec(at(2026, 11, 8)), NOW), '8 November');
  assert.equal(dayText(sec(at(2025, 12, 30)), NOW), '30 December 2025');
});

test('a file on its file host', () => {
  const file = row({ local: false, copies: [copy] });
  assert.equal(fileState(file, OFF), 'on_host');
  assert.equal(fileStatus(file, OFF), 'On its file host');
});

test('a pending file waits for the write and gives the day of the removal', () => {
  const file = row({ local: true, expires_at: sec(at(2026, 11, 8)) });
  assert.equal(fileState(file, OFF), 'pending');
  assert.equal(fileStatus(file, OFF), 'Waits for the write · Studio removes it on 8 November');
});

test('a file on this server only', () => {
  const file = row({ local: true });
  assert.equal(fileState(file, OFF), 'server_only');
  assert.equal(fileStatus(file, OFF), 'On this server only');
});

test('a removed file gives the day of the removal', () => {
  const file = row({ local: false, removed_at: sec(at(2026, 10, 25)) });
  assert.equal(fileState(file, OFF), 'removed');
  assert.equal(fileStatus(file, OFF), 'Removed from this server on 25 October');
});

test('the removed state wins over the pending state', () => {
  const file = row({ removed_at: sec(at(2026, 10, 25)), expires_at: sec(at(2026, 11, 8)) });
  assert.equal(fileState(file, OFF), 'removed');
});

test('a server that holds assets gives no state', () => {
  const file = row({ local: true, expires_at: sec(at(2026, 11, 8)) });
  assert.equal(fileState(file, { holdAssets: true }), null);
  assert.equal(fileStatus(file), null);
});

test('a remote file with no copy gives no state', () => {
  assert.equal(fileStatus(row({ source_url: 'https://example.org/a.mp3' }), OFF), null);
});

test('Download to This Device needs a server and bytes or a copy', () => {
  assert.equal(canDownloadToDevice(row({ local: true }), { server: true }), true);
  assert.equal(canDownloadToDevice(row({ copies: [copy] }), { server: true }), true);
  assert.equal(canDownloadToDevice(row({ local: true }), { server: false }), false);
  assert.equal(canDownloadToDevice(row({}), { server: true }), false);
  assert.equal(canDownloadToDevice(row({ removed_at: 5 }), { server: true }), false);
});

test('the notice of missing files counts the removed files that the draft uses', () => {
  const files = [row({ removed_at: 5 }), row({ removed_at: 6, in_use: false }), row({ local: true })];
  assert.equal(missingCount(files), 1);
  assert.equal(missingNotice(1).strong, '1 file of this draft is missing.');
  assert.equal(missingNotice(2).strong, '2 files of this draft are missing.');
});

test('the texts of the notices', () => {
  assert.equal(SERVER_NOTICE.strong, 'This server keeps no audio and no images.');
  assert.match(ADD_TRACKS_NOTICE, /maximum of 7 days/);
  assert.equal(reviewNotice(2), 'After the write, Studio removes its copies of these 2 files from this server. Keep your own copies.');
  assert.equal(reviewNotice(1), 'After the write, Studio removes its copy of this file from this server. Keep your own copy.');
});

test('the rows of the section Files on this server', () => {
  const size = (n) => `${n} B`;
  assert.deepEqual(serverFilesRows({ used: 5, limit: 10 }, size).map((r) => r.label),
    ['Audio and images', 'Feed XML, drafts and versions', 'Space in use']);
  assert.equal(serverFilesRows({ used: 5, limit: 10 }, size)[2].summary, '5 B of 10 B');
  assert.equal(serverFilesRows(null, size).length, 2);
});

test('sizeText gives a short size', () => {
  assert.equal(sizeText(512), '512 B');
  assert.equal(sizeText(7.8 * 1024 * 1024), '7.8 MB');
  assert.equal(sizeText(10 * 1024 ** 3), '10 GB');
  assert.equal(sizeText(1536), '1.5 KB');
});

test('a removed file cannot be copied', () => {
  assert.equal(canCopyFile(row({ removed_at: 5 })), false);
  assert.equal(canCopyFile(row({ local: true })), true);
});
