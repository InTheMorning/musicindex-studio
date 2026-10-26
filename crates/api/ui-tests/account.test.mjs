// The account models (ADR 0042). Each test checks one rule of ADR 0055 or
// ADR 0056.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  bootView, signInEvent, normalEmail, authErrorText, unlockMethods,
  accountName, deleteConfirmation, recoveryAccount, addMethodChoices, recoveryErrorText,
  toBase64Url, fromBase64Url, creationOptions, requestOptions, registrationPayload,
  assertionPayload, prfOutputOf, passkeyBrowserCode, defaultPasskeyName,
  nip46State, nip46ErrorText, isBunkerAddress, qrSource,
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

test('bootView gives add_method for a session from a recovery sign-in', () => {
  const session = { mode: 'server', signed_in: true, recovery: true };
  assert.equal(bootView(session, null), 'add_method');
  assert.equal(bootView({ ...session, recovery: false }, null), 'shell');
});

test('recoveryAccount reads an email address or an npub and no other text', () => {
  assert.deepEqual(recoveryAccount('  A@X.org '), { kind: 'email', value: 'a@x.org' });
  assert.deepEqual(recoveryAccount(' npub1abc '), { kind: 'npub', value: 'npub1abc' });
  assert.equal(recoveryAccount('nobody'), null);
  assert.equal(recoveryAccount('a b@x.org'), null);
  assert.equal(recoveryAccount(''), null);
  assert.equal(recoveryAccount(undefined), null);
});

test('addMethodChoices lists the passphrase for an email and NIP-07 for a key with a signer', () => {
  assert.deepEqual(addMethodChoices({ email: 'a@x.org', pubkey: null }, true), ['passphrase']);
  assert.deepEqual(addMethodChoices({ email: null, pubkey: PUBKEY }, true), ['nip07']);
  assert.deepEqual(addMethodChoices({ email: null, pubkey: PUBKEY }, false), []);
  assert.deepEqual(addMethodChoices(null, true), []);
});

test('the text of method_required and of a failed recovery sign-in', () => {
  assert.equal(authErrorText('method_required'), 'Add a new sign-in method first.');
  assert.equal(authErrorText('method_not_usable'), 'This method cannot sign in to the account. Use a different method.');
  assert.match(recoveryErrorText('sign_in_failed'), /recovery key/);
  assert.equal(recoveryErrorText('too_many_attempts'), authErrorText('too_many_attempts'));
});

// ---- Passkeys (ADR 0056 items 20 to 24) ----

test('toBase64Url writes no padding and uses - and _', () => {
  assert.equal(toBase64Url(new Uint8Array([0xfb, 0xff, 0xfe])), '-__-');
  assert.equal(toBase64Url(new Uint8Array([1])), 'AQ');
  assert.equal(toBase64Url(new Uint8Array([1, 2])), 'AQI');
  assert.equal(toBase64Url(new Uint8Array(0)), '');
});

test('toBase64Url takes an ArrayBuffer and a view with an offset', () => {
  const bytes = new Uint8Array([9, 8, 7, 6]);
  assert.equal(toBase64Url(bytes.buffer), 'CQgHBg');
  assert.equal(toBase64Url(bytes.subarray(1, 3)), 'CAc');
});

test('fromBase64Url reverses toBase64Url for each length', () => {
  for (let length = 0; length < 70; length += 1) {
    const bytes = Uint8Array.from({ length }, (_, i) => (i * 37 + 11) % 256);
    assert.deepEqual(fromBase64Url(toBase64Url(bytes)), bytes);
  }
  assert.deepEqual(fromBase64Url('-__-'), new Uint8Array([0xfb, 0xff, 0xfe]));
});

test('toBase64Url handles a large buffer', () => {
  const bytes = new Uint8Array(100000).fill(255);
  assert.deepEqual(fromBase64Url(toBase64Url(bytes)), bytes);
});

const SALT = toBase64Url(new Uint8Array(32).fill(5));
const CREATE = {
  challenge: toBase64Url(new Uint8Array(32).fill(1)),
  assertion_challenge: toBase64Url(new Uint8Array(32).fill(2)),
  rp_id: 'studio.example.org',
  user_handle: toBase64Url(new Uint8Array(16).fill(3)),
  user_name: 'a@example.org',
  prf_salt: SALT,
  exclude: [toBase64Url(new Uint8Array([7, 7]))],
};

test('creationOptions asks for a resident key, user verification and ES256 only', () => {
  const options = creationOptions(CREATE);
  assert.equal(options.rp.id, 'studio.example.org');
  assert.deepEqual(options.challenge, new Uint8Array(32).fill(1));
  assert.deepEqual(options.user.id, new Uint8Array(16).fill(3));
  assert.equal(options.user.name, 'a@example.org');
  assert.deepEqual(options.pubKeyCredParams, [{ type: 'public-key', alg: -7 }]);
  assert.equal(options.authenticatorSelection.residentKey, 'required');
  assert.equal(options.authenticatorSelection.userVerification, 'required');
  assert.deepEqual(options.excludeCredentials, [{ type: 'public-key', id: new Uint8Array([7, 7]) }]);
});

test('creationOptions sets the PRF salt', () => {
  const options = creationOptions(CREATE);
  assert.deepEqual(options.extensions.prf.eval.first, new Uint8Array(32).fill(5));
});

test('requestOptions asks for user verification and sets the PRF salt', () => {
  const options = requestOptions({ challenge: CREATE.challenge, rp_id: 'studio.example.org', prf_salt: SALT, allow: [] });
  assert.equal(options.rpId, 'studio.example.org');
  assert.equal(options.userVerification, 'required');
  assert.deepEqual(options.allowCredentials, []);
  assert.deepEqual(options.challenge, new Uint8Array(32).fill(1));
  assert.deepEqual(options.extensions.prf.eval.first, new Uint8Array(32).fill(5));
});

test('requestOptions names the allowed credentials and accepts a missing list', () => {
  const allowed = requestOptions({ challenge: CREATE.challenge, rp_id: 'x', prf_salt: SALT, allow: ['AQI'] });
  assert.deepEqual(allowed.allowCredentials, [{ type: 'public-key', id: new Uint8Array([1, 2]) }]);
  const none = requestOptions({ challenge: CREATE.challenge, rp_id: 'x', prf_salt: SALT });
  assert.deepEqual(none.allowCredentials, []);
});

test('registrationPayload reads the key, the algorithm and the data', () => {
  const credential = {
    response: {
      clientDataJSON: new Uint8Array([1]).buffer,
      getAuthenticatorData: () => new Uint8Array([2]).buffer,
      getPublicKey: () => new Uint8Array([3]).buffer,
      getPublicKeyAlgorithm: () => -7,
    },
  };
  assert.deepEqual(registrationPayload(credential), {
    client_data: 'AQ', authenticator_data: 'Ag', public_key: 'Aw', algorithm: -7,
  });
});

test('registrationPayload gives null when the browser gives no public key', () => {
  assert.equal(registrationPayload({ response: {} }), null);
  assert.equal(registrationPayload(null), null);
  const none = {
    response: {
      clientDataJSON: new Uint8Array([1]).buffer,
      getAuthenticatorData: () => new Uint8Array([2]).buffer,
      getPublicKey: () => null,
      getPublicKeyAlgorithm: () => -7,
    },
  };
  assert.equal(registrationPayload(none), null);
});

test('assertionPayload and prfOutputOf read an assertion', () => {
  const credential = {
    rawId: new Uint8Array([1, 2]).buffer,
    response: {
      clientDataJSON: new Uint8Array([3]).buffer,
      authenticatorData: new Uint8Array([4]).buffer,
      signature: new Uint8Array([5]).buffer,
    },
    getClientExtensionResults: () => ({ prf: { results: { first: new Uint8Array(32).fill(255).buffer } } }),
  };
  assert.deepEqual(assertionPayload(credential), {
    credential_id: 'AQI', client_data: 'Aw', authenticator_data: 'BA', signature: 'BQ',
  });
  assert.equal(prfOutputOf(credential), toBase64Url(new Uint8Array(32).fill(255)));
});

test('prfOutputOf gives null when the passkey manager gives no PRF result', () => {
  assert.equal(prfOutputOf({ getClientExtensionResults: () => ({}) }), null);
  assert.equal(prfOutputOf({ getClientExtensionResults: () => ({ prf: { enabled: false } }) }), null);
  assert.equal(prfOutputOf({}), null);
  assert.equal(prfOutputOf(null), null);
});

test('passkey_no_prf has the text of the task', () => {
  assert.equal(
    authErrorText('passkey_no_prf'),
    'This passkey manager cannot open the vault. Use iCloud Keychain or Google Password Manager, or a different method.',
  );
});

test('passkeyBrowserCode separates a cancel from other failures', () => {
  assert.equal(passkeyBrowserCode({ name: 'NotAllowedError' }), 'passkey_cancelled');
  assert.equal(passkeyBrowserCode({ name: 'InvalidStateError' }), 'passkey_exists');
  assert.equal(passkeyBrowserCode({ name: 'NotSupportedError' }), 'passkey_unsupported');
  assert.equal(passkeyBrowserCode(null), 'passkey_unsupported');
});

test('defaultPasskeyName gives the next free number', () => {
  assert.equal(defaultPasskeyName([]), 'Passkey 1');
  assert.equal(defaultPasskeyName([{ name: 'Phone' }]), 'Passkey 2');
  assert.equal(defaultPasskeyName([{ name: 'Passkey 2' }, { name: 'Passkey 3' }]), 'Passkey 4');
});

test('unlockMethods and addMethodChoices add the passkey only with a passkey and a browser that supports it', () => {
  const email = { email: 'a@example.org', passkeys: 1 };
  assert.deepEqual(unlockMethods(email, false, true), ['passphrase', 'passkey']);
  assert.deepEqual(unlockMethods(email, false, false), ['passphrase']);
  assert.deepEqual(unlockMethods({ email: 'a@example.org', passkeys: 0 }, false, true), ['passphrase']);
  assert.deepEqual(addMethodChoices(email, false, true), ['passphrase', 'passkey']);
  assert.deepEqual(addMethodChoices(email, false, false), ['passphrase']);
});

// ---- NIP-46 remote signer (ADR 0056 items 12 to 16, task 035c) ----

test('nip46State gives waiting while the signer has not answered', () => {
  assert.deepEqual(nip46State({ state: 'pending', pubkey: null, error: null }), {
    phase: 'waiting', pubkey: null, npub: null, error: null,
  });
});

test('nip46State gives the public key and the npub of a connected signer', () => {
  const view = nip46State({ state: 'connected', pubkey: PUBKEY, error: null });
  assert.equal(view.phase, 'connected');
  assert.equal(view.pubkey, PUBKEY);
  assert.match(view.npub, /^npub1[02-9ac-hj-np-z]+$/);
  assert.equal(view.error, null);
});

test('nip46State gives the error code of a failed connection', () => {
  const view = nip46State({ state: 'failed', pubkey: null, error: 'signer_timeout' });
  assert.equal(view.phase, 'failed');
  assert.equal(view.error, 'signer_timeout');
});

test('nip46State treats an answer that it does not know as a failure', () => {
  for (const answer of [null, undefined, {}, { state: 'other' }, { state: 'failed' }, { state: 7 }]) {
    const view = nip46State(answer);
    assert.equal(view.phase, 'failed');
    assert.equal(view.error, 'signer_invalid');
  }
});

test('nip46ErrorText has a text for each code of the NIP-46 routes', () => {
  const codes = [
    'address_not_public', 'signer_timeout', 'signer_unavailable', 'bunker_invalid', 'signer_rejected',
    'signer_invalid', 'connection_unknown', 'connection_not_ready', 'connection_busy',
    'signer_not_connected', 'no_nostr_wrapper',
  ];
  const general = authErrorText('no_such_code');
  const seen = new Set();
  for (const code of codes) {
    const text = nip46ErrorText(code);
    assert.ok(text.length > 10 && text !== general, code);
    seen.add(text);
  }
  assert.equal(seen.size, codes.length);
  assert.equal(nip46ErrorText('signer_timeout'), 'The signer gave no answer in time. Open the signer app and try again.');
});

test('nip46ErrorText gives the text of the other auth routes for their codes', () => {
  assert.equal(nip46ErrorText('invite_required'), authErrorText('invite_required'));
  assert.equal(nip46ErrorText('too_many_attempts'), authErrorText('too_many_attempts'));
  assert.equal(nip46ErrorText('no_such_code'), authErrorText('no_such_code'));
  assert.equal(nip46ErrorText(undefined), authErrorText(undefined));
});

test('unlockMethods adds the remote signer when the session has one', () => {
  const nostr = { pubkey: PUBKEY, signer: true };
  assert.deepEqual(unlockMethods(nostr, false), ['nip46']);
  assert.deepEqual(unlockMethods(nostr, true), ['nip07', 'nip46']);
  assert.deepEqual(unlockMethods({ pubkey: PUBKEY, signer: false }, false), []);
  assert.deepEqual(unlockMethods({ pubkey: PUBKEY }, false), []);
  assert.deepEqual(unlockMethods({ pubkey: PUBKEY, signer: true, passkeys: 1 }, false, true), ['nip46', 'passkey']);
});

test('isBunkerAddress accepts a bunker address and refuses other text', () => {
  assert.equal(isBunkerAddress(`bunker://${PUBKEY}?relay=wss%3A%2F%2Frelay.example&secret=abc`), true);
  assert.equal(isBunkerAddress(`  bunker://${PUBKEY}?relay=wss://relay.example `), true);
  assert.equal(isBunkerAddress(`bunker://${PUBKEY}`), false);
  assert.equal(isBunkerAddress('bunker://abc?relay=wss://relay.example'), false);
  assert.equal(isBunkerAddress(`nostrconnect://${PUBKEY}?relay=wss://relay.example`), false);
  assert.equal(isBunkerAddress(''), false);
  assert.equal(isBunkerAddress(null), false);
});

test('qrSource gives a data address for SVG text and nothing for no text', () => {
  const source = qrSource('<svg xmlns="http://www.w3.org/2000/svg"></svg>');
  assert.ok(source.startsWith('data:image/svg+xml;utf8,'));
  assert.ok(!source.includes('<'));
  assert.equal(qrSource(''), '');
  assert.equal(qrSource(undefined), '');
});
