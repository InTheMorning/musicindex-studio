// The account models (ADR 0055 items 9 and 12, ADR 0056 items 6 to 11, ADR
// 0042). This module has no Preact and no browser state. The Node tests in
// `ui-tests/account.test.mjs` examine it.
import { npubOf } from './signing.js';

// Which view does the page show at the start? `session` is the answer of
// `GET /auth/v1/session`. In the mode "server", a locked vault does not stop
// the boot: a route that needs the vault asks for the unlock (ADR 0056 item 9).
// `status` is the answer of `GET /api/v1/status`, or `null`.
export function bootView(session, status) {
  if (!session || session.mode === 'local') return 'local';
  // The vault state of `status` does not change the result on a server.
  void status;
  if (!session.signed_in) return 'sign_in';
  // A session from a recovery sign-in must add a sign-in method first
  // (ADR 0056 item 3).
  return session.recovery ? 'add_method' : 'shell';
}

// The kind of an HTTP authorization event (NIP-98).
export const KIND_HTTP_AUTH = 27235;

// The unsigned sign-in event (ADR 0056 item 6). `origin` is the public
// origin of the server, `challenge` comes from `POST /auth/v1/challenge`,
// and `createdAt` is in seconds.
export function signInEvent(origin, challenge, createdAt) {
  return {
    kind: KIND_HTTP_AUTH,
    created_at: createdAt,
    tags: [
      ['u', `${String(origin).replace(/\/+$/, '')}/auth/v1/nostr`],
      ['method', 'POST'],
      ['challenge', challenge],
    ],
    content: '',
  };
}

// An email address in lowercase with no space at the start or the end.
export function normalEmail(text) {
  return String(text ?? '').trim().toLowerCase();
}

const AUTH_ERRORS = {
  sign_in_failed: 'The email address or the passphrase is not correct.',
  email_in_use: 'Another account has this email address.',
  invite_required: 'Sign-up needs an invite code that nobody used.',
  challenge_invalid: 'The sign-in request is too old or was used already. Try again.',
  too_many_attempts: 'Too many sign-in attempts. Try again later.',
  busy: 'The server is busy. Try again in a short time.',
  session_ended: 'Your session ended. Sign in again.',
  wrong_key: 'The key does not open the vault.',
  no_signer: 'Studio found no signer extension in this browser.',
  method_required: 'Add a new sign-in method first.',
  method_not_usable: 'This method cannot sign in to the account. Use a different method.',
  passkey_no_prf: 'This passkey manager cannot open the vault. Use iCloud Keychain or Google Password Manager, or a different method.',
  passkey_invalid: 'The passkey was not accepted. Try again or use a different method.',
  passkey_cancelled: 'The passkey request was cancelled or timed out. Try again.',
  passkey_unsupported: 'This browser cannot give the passkey data that Studio needs. Use a different method.',
  passkey_exists: 'This passkey is already in use. Use a different passkey manager or device.',
  last_method: 'This is the last way to sign in to the account. Add a different method first.',
  invalid_name: 'Give the passkey a name of 1 to 64 characters.',
  vault_locked: 'Unlock the vault first.',
};

// The text for an error code of the auth routes. An unknown code gives a
// general text.
export function authErrorText(code) {
  return Object.hasOwn(AUTH_ERRORS, code) ? AUTH_ERRORS[code] : 'Something went wrong. Try again.';
}

// The methods of the unlock sheet, in order (ADR 0056 items 7, 10 and 24).
// `session` is the answer of `GET /auth/v1/session`. `hasNip07` is true when
// the browser has `window.nostr`. `hasPasskeys` is true when the browser has
// `PublicKeyCredential`.
export function unlockMethods(session, hasNip07, hasPasskeys = false) {
  const methods = [];
  if (session && session.email) methods.push('passphrase');
  if (session && session.pubkey && hasNip07) methods.push('nip07');
  if (session && session.passkeys > 0 && hasPasskeys) methods.push('passkey');
  return methods;
}

// The name of an account for the Account panel: the email address, or the
// `npub` of the Nostr key.
export function accountName(session) {
  if (session && session.email) return session.email;
  return (session && npubOf(session.pubkey)) || '';
}

// The text that `POST /auth/v1/account/delete` needs as `confirm`, or `null`
// when the typed text is not the email address or the `npub` of the account.
export function deleteConfirmation(session, typed) {
  const text = String(typed ?? '').trim();
  if (!text || !session) return null;
  if (session.email) return normalEmail(text) === session.email ? session.email : null;
  const npub = npubOf(session.pubkey);
  return npub && text === npub ? npub : null;
}

// The text for a failed recovery sign-in. The server gives `sign_in_failed`
// for an unknown account and for a key that is not correct.
export function recoveryErrorText(code) {
  if (code === 'sign_in_failed') return 'The account or the recovery key is not correct.';
  return authErrorText(code);
}

// The account text of the recovery page (ADR 0056 item 3). It gives an email
// address in lowercase, an `npub`, or `null` for other text. It does no
// bech32 check. The server does that check.
export function recoveryAccount(text) {
  const value = String(text ?? '').trim().toLowerCase();
  if (/^npub1\S+$/.test(value)) return { kind: 'npub', value };
  if (/^[^\s@]+@[^\s@]+$/.test(value)) return { kind: 'email', value };
  return null;
}

// The methods that the page "Add a sign-in method" offers, in order.
// `session` is the answer of `GET /auth/v1/session`. `hasNip07` is true when
// the browser has `window.nostr`. `hasPasskeys` is true when the browser has
// `PublicKeyCredential`.
export function addMethodChoices(session, hasNip07, hasPasskeys = false) {
  const choices = [];
  if (session && session.email) choices.push('passphrase');
  if (session && session.pubkey && hasNip07) choices.push('nip07');
  if (session && hasPasskeys) choices.push('passkey');
  return choices;
}

// ---- Passkeys with PRF (ADR 0056 items 20 to 24) ----

// The text of a byte array in base64url, with no padding. `bytes` is an
// `ArrayBuffer` or a typed array.
export function toBase64Url(bytes) {
  const view = bytes instanceof ArrayBuffer
    ? new Uint8Array(bytes)
    : new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let binary = '';
  for (let i = 0; i < view.length; i += 0x8000) {
    binary += String.fromCharCode(...view.subarray(i, i + 0x8000));
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// The bytes of a base64url text, as a `Uint8Array`.
export function fromBase64Url(text) {
  const base = String(text ?? '').replace(/-/g, '+').replace(/_/g, '/');
  const binary = atob(base + '='.repeat((4 - (base.length % 4)) % 4));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

// The COSE number of ES256. Studio accepts this algorithm only.
export const ES256 = -7;

const PASSKEY_TIMEOUT = 300000;

function prfExtension(options) {
  return { prf: { eval: { first: fromBase64Url(options.prf_salt) } } };
}

// The `publicKey` object for `navigator.credentials.create`. `options` is
// the answer of `POST /auth/v1/passkeys/options`.
export function creationOptions(options) {
  return {
    challenge: fromBase64Url(options.challenge),
    rp: { id: options.rp_id, name: 'Music Index Studio' },
    user: {
      id: fromBase64Url(options.user_handle),
      name: options.user_name,
      displayName: options.user_name,
    },
    pubKeyCredParams: [{ type: 'public-key', alg: ES256 }],
    authenticatorSelection: { residentKey: 'required', requireResidentKey: true, userVerification: 'required' },
    excludeCredentials: (options.exclude || []).map((id) => ({ type: 'public-key', id: fromBase64Url(id) })),
    attestation: 'none',
    timeout: PASSKEY_TIMEOUT,
    extensions: prfExtension(options),
  };
}

// The `publicKey` object for `navigator.credentials.get`. `options` is the
// answer of `POST /auth/v1/passkey/options`. For the assertion after a
// registration, the caller sets `challenge` to `assertion_challenge` and
// `allow` to the new credential.
export function requestOptions(options) {
  return {
    challenge: fromBase64Url(options.challenge),
    rpId: options.rp_id,
    userVerification: 'required',
    allowCredentials: (options.allow || []).map((id) => ({ type: 'public-key', id: fromBase64Url(id) })),
    timeout: PASSKEY_TIMEOUT,
    extensions: prfExtension(options),
  };
}

// The part of a registration that `POST /auth/v1/passkeys` needs, or `null`
// when the browser cannot give the public key or the algorithm.
export function registrationPayload(credential) {
  const response = credential && credential.response;
  if (!response || typeof response.getPublicKey !== 'function'
    || typeof response.getAuthenticatorData !== 'function'
    || typeof response.getPublicKeyAlgorithm !== 'function') return null;
  const key = response.getPublicKey();
  if (!key) return null;
  return {
    client_data: toBase64Url(response.clientDataJSON),
    authenticator_data: toBase64Url(response.getAuthenticatorData()),
    public_key: toBase64Url(key),
    algorithm: response.getPublicKeyAlgorithm(),
  };
}

// The part of an assertion that the passkey routes need.
export function assertionPayload(credential) {
  const response = credential.response;
  return {
    credential_id: toBase64Url(credential.rawId),
    client_data: toBase64Url(response.clientDataJSON),
    authenticator_data: toBase64Url(response.authenticatorData),
    signature: toBase64Url(response.signature),
  };
}

// The PRF output of an assertion in base64url, or `null` when the passkey
// manager gave none.
export function prfOutputOf(credential) {
  const results = credential && typeof credential.getClientExtensionResults === 'function'
    ? credential.getClientExtensionResults()
    : null;
  const first = results && results.prf && results.prf.results && results.prf.results.first;
  return first ? toBase64Url(first) : null;
}

// The error code for a failure of `navigator.credentials`. The user can
// cancel the request, and the request can time out.
export function passkeyBrowserCode(error) {
  if (error && (error.name === 'NotAllowedError' || error.name === 'AbortError')) return 'passkey_cancelled';
  if (error && error.name === 'InvalidStateError') return 'passkey_exists';
  return 'passkey_unsupported';
}

// A name for a new passkey, such as "Passkey 2". `list` is the answer of
// `GET /auth/v1/passkeys`.
export function defaultPasskeyName(list) {
  const names = new Set((list || []).map((item) => item.name));
  for (let n = (list || []).length + 1; ; n += 1) {
    if (!names.has(`Passkey ${n}`)) return `Passkey ${n}`;
  }
}
