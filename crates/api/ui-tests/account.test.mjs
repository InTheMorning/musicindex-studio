// The account models (ADR 0042). Each test checks one rule of ADR 0055 or
// ADR 0056.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  bootView, signInEvent, normalEmail, authErrorText, unlockMethods,
  accountName, deleteConfirmation,
} from '../ui/account.js';

const PUBKEY = '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';

test('bootView gives local for the desktop', () => {
  assert.equal(bootView({ mode: 'local' }, null), 'local');
});

test('bootView gives sign_in for a server with no session', () => {
  assert.equal(bootView({ mode: 'server', signed_in: false, signup: 'open' }, null), 'sign_in');
});

test('bootView gives shell for a signed-in user, also with a locked vault', () => {
  assert.equal(bootView({ mode: 'server', signed_in: true, vault: 'open' }, { vault: 'open' }), 'shell');
  assert.equal(bootView({ mode: 'server', signed_in: true, vault: 'locked' }, { vault: 'locked' }), 'shell');
  assert.equal(bootView({ mode: 'server', signed_in: true, vault: 'locked' }, null), 'shell');
});

test('signInEvent has kind 27235, the three tags and an empty content', () => {
  const event = signInEvent('https://studio.example.org', 'abc123', 1700000000);
  assert.equal(event.kind, 27235);
  assert.equal(event.created_at, 1700000000);
  assert.equal(event.content, '');
  assert.deepEqual(event.tags, [
    ['u', 'https://studio.example.org/auth/v1/nostr'],
    ['method', 'POST'],
    ['challenge', 'abc123'],
  ]);
});

test('signInEvent does not double the slash at the end of the origin', () => {
  const event = signInEvent('http://127.0.0.1:47140/', 'c', 1);
  assert.deepEqual(event.tags[0], ['u', 'http://127.0.0.1:47140/auth/v1/nostr']);
});

test('normalEmail gives lowercase text with no space at the ends', () => {
  assert.equal(normalEmail('  Ann@Example.ORG \n'), 'ann@example.org');
  assert.equal(normalEmail('a b@x.org'), 'a b@x.org');
  assert.equal(normalEmail(null), '');
});

test('authErrorText gives one text for each code', () => {
  const codes = ['sign_in_failed', 'email_in_use', 'invite_required', 'challenge_invalid',
    'too_many_attempts', 'busy', 'session_ended', 'wrong_key', 'no_signer'];
  const texts = codes.map(authErrorText);
  for (const text of texts) assert.ok(text.length > 0);
  assert.equal(new Set(texts).size, codes.length);
  assert.match(authErrorText('sign_in_failed'), /email address or the passphrase/);
});

test('authErrorText gives a general text for an unknown code', () => {
  const general = authErrorText('no_such_code');
  assert.match(general, /Something went wrong/);
  assert.equal(authErrorText(undefined), general);
  assert.equal(authErrorText('constructor'), general);
});

test('unlockMethods lists the passphrase for an account with an email', () => {
  assert.deepEqual(unlockMethods({ email: 'a@x.org', pubkey: null }, false), ['passphrase']);
  assert.deepEqual(unlockMethods({ email: 'a@x.org', pubkey: null }, true), ['passphrase']);
});

test('unlockMethods lists NIP-07 for a Nostr account with a signer in the browser', () => {
  assert.deepEqual(unlockMethods({ email: null, pubkey: PUBKEY }, true), ['nip07']);
  assert.deepEqual(unlockMethods({ email: null, pubkey: PUBKEY }, false), []);
  assert.deepEqual(unlockMethods({ email: 'a@x.org', pubkey: PUBKEY }, true), ['passphrase', 'nip07']);
});

test('accountName gives the email address, or the npub', () => {
  assert.equal(accountName({ email: 'a@x.org', pubkey: null }), 'a@x.org');
  assert.match(accountName({ email: null, pubkey: PUBKEY }), /^npub1/);
});

test('deleteConfirmation accepts only the email address or the npub of the account', () => {
  assert.equal(deleteConfirmation({ email: 'a@x.org', pubkey: null }, ' A@X.org '), 'a@x.org');
  assert.equal(deleteConfirmation({ email: 'a@x.org', pubkey: null }, 'b@x.org'), null);
  assert.equal(deleteConfirmation({ email: 'a@x.org', pubkey: null }, ''), null);
  const npub = accountName({ email: null, pubkey: PUBKEY });
  assert.equal(deleteConfirmation({ email: null, pubkey: PUBKEY }, npub), npub);
  assert.equal(deleteConfirmation({ email: null, pubkey: PUBKEY }, PUBKEY), null);
});
