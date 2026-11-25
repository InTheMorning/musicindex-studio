import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { ERROR_STEPS, errorStep, retryHelps } from '../ui/errors.js';

test('each known code has its task step', () => {
  assert.equal(Object.keys(ERROR_STEPS).length, 15);
  for (const [code, row] of Object.entries(ERROR_STEPS)) {
    assert.equal(errorStep({ code }), row.step);
  }
  assert.equal(errorStep({ code: 'unknown' }), 'Try again. If the error stays, restart Studio.');
  assert.equal(errorStep({ code: 'in_change_set' }), 'Write or discard the changes first.',
    'ADR 0243 item 5: fix ERROR_STEPS in crates/api/ui/errors.js.');
});

test('retry appears only when a new attempt can help without user action', () => {
  assert.equal(retryHelps({ status: 0, code: 'network' }), true);
  assert.equal(retryHelps({ status: 503, code: 'host_unavailable' }), true);
  assert.equal(retryHelps({ status: 500, code: 'internal' }), true);
  assert.equal(retryHelps({ status: 503, code: 'host_unauthorized' }), false);
  assert.equal(retryHelps({ status: 503, code: 'vault_locked' }), false);
  assert.equal(retryHelps({ status: 422, code: 'network' }), false);
});

test('ErrorNotice puts raw text in a closed Details fold', async () => {
  const source = await readFile(new URL('../ui/app.js', import.meta.url), 'utf8');
  const notice = source.slice(source.indexOf('export function ErrorNotice'), source.indexOf('// The message of an API error'));
  assert.match(notice, /<details class="disclosure">/);
  assert.match(notice, /<span class="summary">Details<\/span>/);
  assert.doesNotMatch(notice, /<details[^>]*\bopen/);
});

test('the failed import row and each named list load have a retry state', async () => {
  const [editor, people, app] = await Promise.all([
    readFile(new URL('../ui/editor.js', import.meta.url), 'utf8'),
    readFile(new URL('../ui/peoplepages.js', import.meta.url), 'utf8'),
    readFile(new URL('../ui/app.js', import.meta.url), 'utf8'),
  ]);
  const importRow = editor.slice(editor.indexOf('function ImportRow'), editor.indexOf('// The next client-only ID'));
  assert.match(importRow, /Try Again/);
  assert.match(importRow, /Remove[\s\S]*Try Again/);
  // ADR 0222 item 5: each list load of task 183 shows LoadErrorNotice in the
  // place of an empty list.
  const loads = (source, list) => source.split(`<\${LoadErrorNotice} list="${list}"`).length - 1;
  for (const list of ['the snapshots', 'the moves', 'the members']) assert.equal(loads(editor, list), 1, `ADR 0222 item 5: editor.js must show LoadErrorNotice for ${list}.`);
  assert.equal(loads(people, 'the suggestions'), 3, 'ADR 0222 item 5: peoplepages.js must show LoadErrorNotice for the three suggestion loads.');
  assert.equal(loads(people, 'the wallets'), 1, 'ADR 0222 item 5: peoplepages.js must show LoadErrorNotice for the wallets.');
  assert.equal(loads(app, 'the feeds of this folder'), 1, 'ADR 0222 item 5: app.js must show LoadErrorNotice for the feeds of a folder.');
  assert.ok(app.includes('Studio could not load ${list}.'));
});

test('the step of bad_address is the message with the places (ADR 0245 item 8)', () => {
  assert.equal(errorStep({ code: 'bad_address', message: 'Credit · The Doerfels · Information address is not correct.' }),
    'Credit · The Doerfels · Information address is not correct.',
    'ADR 0245 item 8: the refusal names each place in words. Fix errorStep in crates/api/ui/errors.js.');
});

test('the step of role_taken is the message with the page (ADR 0257 item 10)', () => {
  const message = 'White Triangles already has the publisher page Harbor Records. Change it in the fields of the release.';
  assert.equal(errorStep({ code: 'role_taken', message }), message);
});
