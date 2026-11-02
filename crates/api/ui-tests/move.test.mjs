// The model of the export and the import of an account (ADR 0107, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  exportTitle, exportView, importedNotice, showsImportNotice, sendingText, importFacts, importMethods,
  importOpenView, importErrorText, canRetryOpen, fullDay, aboutSize, EXPORT_NOTICE, EXPORT_WARNING,
} from '../ui/move.js';

const MB = 1024 * 1024;
const facts = (extra) => ({
  feeds: 12, drafts: 3, versions: 41, people: 18, movable: true, methods: ['passphrase', 'recovery'], passkeys: 0,
  holds_assets: false, asset_bytes: 0, archive_bytes: Math.round(3.4 * MB), archive_bytes_without_assets: Math.round(3.4 * MB),
  ...extra,
});

test('screen 1: a server that keeps no audio shows the facts and no check box', () => {
  const view = exportView(facts(), { server: true });
  assert.equal(view.screen, 'plain');
  assert.equal(view.title, 'Export Account');
  assert.equal(view.primary, 'Export Account');
  assert.equal(view.checkbox, null, 'ADR 0107 item 6: a server with --hold-assets off gives the user no choice. Fix exportView in crates/api/ui/move.js.');
  assert.deepEqual(view.notice, EXPORT_NOTICE);
  assert.deepEqual(view.rows.map((r) => r.summary), [
    '12 feeds · 3 drafts · 41 versions',
    '18 people',
    'Opens with your passphrase and your recovery key',
    'This server keeps no audio and no images. Your file hosts keep them.',
  ]);
  assert.equal(view.size, 'The file is about 3.4 MB.');
  assert.equal(view.hint, 'The export changes nothing. This account stays on this server until you delete it.');
});

test('screen 2: a desktop, or a server that keeps audio, offers the audio and images', () => {
  const data = facts({ holds_assets: true, asset_bytes: 2 * 1024 * MB, archive_bytes: 2.1 * 1024 * MB, archive_bytes_without_assets: Math.round(0.1 * 1024 * MB) });
  const on = exportView(data, { server: false, includeAssets: true });
  assert.equal(on.screen, 'audio');
  assert.equal(on.title, 'Export Studio');
  assert.equal(on.primary, 'Export Studio');
  assert.equal(on.checkbox, 'Include the audio and images');
  assert.equal(on.rows[3].summary, 'about 2.0 GB');
  assert.equal(on.size, 'The file is about 2.1 GB.');
  assert.equal(on.hint, 'The export changes nothing. Your data stays in this Studio.');
  const none = exportView({ ...data, asset_bytes: 0 }, { server: false });
  assert.equal(none.rows[3].summary, 'No audio and no images in the library');
  const off = exportView(data, { server: false, includeAssets: false });
  assert.equal(off.size, 'The file is about 102.4 MB.', 'The size follows the check box.');
  assert.equal(exportView(data, { server: true }).title, 'Export Account');
});

test('screen 3: a vault that cannot move gives only the recovery key', () => {
  const view = exportView(facts({ movable: false, methods: ['passkey'], passkeys: 1 }), { server: true });
  assert.equal(view.screen, 'warn');
  assert.equal(view.primary, 'Make a Recovery Key');
  assert.equal(view.error, true);
  assert.deepEqual(view.notice, EXPORT_WARNING);
  assert.deepEqual(view.rows, []);
  assert.equal(view.checkbox, null);
  assert.equal(view.hint, 'Studio shows the recovery key one time, then makes the file.');
  // After the key shows, the facts say movable and the sheet continues.
  assert.equal(exportView(facts({ methods: ['passkey', 'recovery'] }), { server: true }).screen, 'plain');
});

test('the vault row names each way that opens the vault', () => {
  const row = (methods) => exportView(facts({ methods }), { server: true }).rows[2].summary;
  assert.equal(row(['recovery']), 'Opens with your recovery key');
  assert.equal(row(['passphrase', 'recovery', 'nostr', 'passkey']), 'Opens with your passphrase, your recovery key and your Nostr signer');
});

test('one feed, one draft and one person use the singular', () => {
  const view = exportView(facts({ feeds: 1, drafts: 1, versions: 1, people: 1 }), { server: true });
  assert.equal(view.rows[0].summary, '1 feed · 1 draft · 1 version');
  assert.equal(view.rows[1].summary, '1 person');
  assert.equal(aboutSize(3 * MB), 'about 3.0 MB');
  assert.equal(exportTitle(false), 'Export Studio');
});

test('screen 8: the notice after an import', () => {
  assert.deepEqual(importedNotice({ origin: 'https://studio.example.org', passkeys: true }), {
    strong: 'This account came from a file.',
    text: 'Add a passkey for this server. Each connected app must use the address of this server: https://studio.example.org.',
  });
  assert.ok(!importedNotice({ origin: 'https://a.example', passkeys: false }).text.includes('passkey'));
  assert.equal(showsImportNotice({ imported: true }), true);
  assert.equal(showsImportNotice({ imported: false }), false);
  assert.equal(showsImportNotice({}), false, 'ADR 0107 item 14: the notice shows only while the session says imported.');
});

const check = (extra) => ({
  upload: 'u1',
  account: { email: 'heycitizen@example.com', nostr_keys: [] },
  needs_email: false,
  studio_version: '0.1.0-rc.10',
  exported_at: Math.floor(new Date(2026, 10, 1, 12).getTime() / 1000),
  counts: { feeds: 12, drafts: 3, versions: 41, people: 18 },
  methods: ['passphrase', 'recovery'],
  nostr_wrappers: [],
  ...extra,
});

test('screen 5: the progress text of the upload', () => {
  assert.equal(sendingText(2.1 * MB, 3.4 * MB), 'Sending… 2.1 MB of 3.4 MB');
});

test('screen 6: the facts of the file', () => {
  assert.deepEqual(importFacts(check()), [
    { label: 'Account', summary: 'heycitizen@example.com' },
    { label: 'Exported', summary: '1 November 2026 from Studio 0.1.0-rc.10' },
    { label: 'Content', summary: '12 feeds · 3 drafts · 18 people' },
  ]);
  assert.equal(fullDay(check().exported_at), '1 November 2026');
  const key = 'a'.repeat(64);
  const nostr = importFacts(check({ account: { email: null, nostr_keys: [key] } }));
  assert.match(nostr[0].summary, /^npub1/);
});

test('screen 6: the passphrase shows first, and the other ways follow', () => {
  const view = importOpenView(check(), { server: true, hasNip07: true });
  assert.equal(view.secret, 'passphrase');
  assert.equal(view.primary, 'Open and Make the Account');
  assert.equal(view.intro, 'Studio makes the account when the vault opens.');
  const desktop = importOpenView(check(), { server: false, hasNip07: false });
  assert.equal(desktop.primary, 'Open and Import', 'The desktop makes no account. Fix importOpenView in crates/api/ui/move.js.');
  assert.equal(desktop.intro, 'Studio imports the file when the vault opens.');
  assert.deepEqual(view.others, [{ id: 'recovery', label: 'Use the Recovery Key' }]);
  assert.equal(view.needsEmail, false);
  const recovery = importOpenView(check(), { server: true, hasNip07: true, mode: 'recovery' });
  assert.equal(recovery.secret, 'recovery');
  assert.deepEqual(recovery.others, [{ id: 'passphrase', label: 'Use the Passphrase' }]);
});

test('a Nostr file offers the extension and, on a server, the remote signer', () => {
  const data = check({ methods: ['nostr'], nostr_wrappers: [{ pubkey: 'b'.repeat(64), ciphertext: 'x' }] });
  assert.deepEqual(importMethods(data, { server: true, hasNip07: true }), ['nip07', 'nip46']);
  assert.deepEqual(importMethods(data, { server: true, hasNip07: false }), ['nip46']);
  assert.deepEqual(importMethods(data, { server: false, hasNip07: true }), ['nip07']);
  const view = importOpenView(data, { server: true, hasNip07: true });
  assert.equal(view.secret, null);
  assert.equal(view.primary, null);
  assert.equal(importOpenView(data, { server: false, hasNip07: false }).noMethod, true);
});

test('the email field shows on a server only when the file has no email and the user types a secret', () => {
  const none = check({ account: { email: null, nostr_keys: ['a'.repeat(64)] }, needs_email: true });
  assert.equal(importOpenView(none, { server: true, hasNip07: false }).needsEmail, true);
  assert.equal(importOpenView(none, { server: false, hasNip07: false }).needsEmail, false);
  assert.equal(importOpenView(check(), { server: true, hasNip07: false }).needsEmail, false);
});

test('the error texts name the cause of a refused file', () => {
  assert.match(importErrorText('archive_invalid', 'newer_schema: the collection is newer'), /newer Studio/);
  assert.match(importErrorText('archive_invalid', 'no_portable_method: ADR 0107 item 4'), /only passkeys/);
  assert.match(importErrorText('archive_invalid', 'something new'), /cannot use this file/);
  assert.match(importErrorText('account_exists'), /Sign in/);
  assert.equal(importErrorText('nothing_like_this'), null);
  assert.ok(canRetryOpen('wrong_key'));
  assert.ok(!canRetryOpen('account_exists'));
});
