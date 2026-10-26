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
  return session.signed_in ? 'shell' : 'sign_in';
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
};

// The text for an error code of the auth routes. An unknown code gives a
// general text.
export function authErrorText(code) {
  return Object.hasOwn(AUTH_ERRORS, code) ? AUTH_ERRORS[code] : 'Something went wrong. Try again.';
}

// The methods of the unlock sheet, in order (ADR 0056 items 7 and 10).
// `session` is the answer of `GET /auth/v1/session`. `hasNip07` is true when
// the browser has `window.nostr`.
export function unlockMethods(session, hasNip07) {
  const methods = [];
  if (session && session.email) methods.push('passphrase');
  if (session && session.pubkey && hasNip07) methods.push('nip07');
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
