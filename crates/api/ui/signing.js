// The signing models (ADR 0050 items 10 to 12, ADR 0042). This module has no
// Preact and no browser state. The Node tests in `ui-tests/signing.test.mjs`
// examine it. The `signer` argument is an object with `signEvent`, such as
// `window.nostr` of a NIP-07 signer.

// An error of the signer step. `code` is the code that the Review sheet and
// the tests read.
export class SignError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'SignError';
    this.code = code;
  }
}

// The events that the user signs. `diff.events` has one entry for each
// planned event. An entry with `user_signs` false is for a key in the vault.
// The sequence of the diff stays.
export function eventsToSign(diff) {
  const entries = diff && Array.isArray(diff.events) ? diff.events : [];
  return entries.filter((entry) => entry.user_signs).map((entry) => entry.event);
}

// Ask the signer to sign each event in sequence. `onProgress(done, total)`
// is called after each signed event. A signed event must keep the ID of
// the planned event. If the ID is different, the error code is
// `changed_event`. If the signer refuses or fails, the error code is
// `signer_refused`. The function gives the signed events in the same sequence.
export async function signAll(events, signer, onProgress) {
  const total = events.length;
  const signed = [];
  for (const event of events) {
    let result;
    try {
      result = await signer.signEvent({ ...event });
    } catch (_) {
      throw new SignError('signer_refused', 'The signer did not sign.');
    }
    if (!result || result.id !== event.id) {
      throw new SignError('changed_event', 'The signer changed an event.');
    }
    signed.push(result);
    if (onProgress) onProgress(signed.length, total);
  }
  return signed;
}

// A signer that works through the API: the NIP-46 signer of the workspace.
// `sign(event)` sends one unsigned event to `POST /api/v1/signer/sign` and
// gives the signed event. The object has the same `signEvent` as a NIP-07
// signer, so `signAll` uses either one (ADR 0056 item 17).
export function remoteSigner(sign) {
  return { signEvent: (event) => sign(event) };
}

// Which signer signs the events of a write (ADR 0056 items 17 and 18).
// `remote` is the answer of `GET /api/v1/signer`, or null while it loads.
// The result `kind` is:
// - `none`: no event needs a signature.
// - `nip07`: the browser has a NIP-07 signer.
// - `loading`: the answer of the API is not here yet.
// - `remote`: a NIP-46 signer is connected and has the key of the events.
// - `differs`: a NIP-46 signer is connected with a different key.
// - `connect`: no signer is connected.
export function signerChoice({ hasNip07, remote, events }) {
  if (!events || events.length === 0) return { kind: 'none' };
  if (hasNip07) return { kind: 'nip07' };
  if (!remote) return { kind: 'loading' };
  if (remote.state !== 'connected' || !remote.pubkey) return { kind: 'connect' };
  const same = events.every((event) => event.pubkey === remote.pubkey);
  return { kind: same ? 'remote' : 'differs', pubkey: remote.pubkey };
}

// The view of an answer of `GET /api/v1/signer` while the UI connects a
// signer. `phase` is `idle`, `waiting`, `connected` or `failed`.
export function signerView(answer) {
  const state = answer && typeof answer.state === 'string' ? answer.state : '';
  if (state === 'pending') return { phase: 'waiting', pubkey: null, error: null };
  if (state === 'connected' && typeof answer.pubkey === 'string') {
    return { phase: 'connected', pubkey: answer.pubkey, error: null };
  }
  if (typeof answer?.error === 'string' && answer.error) {
    return { phase: 'failed', pubkey: null, error: answer.error };
  }
  return { phase: 'idle', pubkey: null, error: null };
}

// The text for each error code of the signer step. A code with no entry
// shows in the general error notice.
const SIGN_MESSAGES = {
  no_signer: 'Studio found no signer. This write needs your signer. Connect a signer first.',
  changed_event: 'Your signer changed an event. Studio wrote nothing.',
  signer_refused: 'Your signer did not sign. Studio wrote nothing.',
  plan_expired: 'The plan of this write expired. Examine the changes again, then write.',
  signer_key_differs: 'The connected signer has a different key than this file host. Studio asked for no signature. Connect the signer of the file host.',
  signer_not_connected: 'No signer is connected. Connect a signer, then write.',
  signer_timeout: 'The signer gave no answer in time. Open the signer app and try again.',
  signer_unavailable: 'Studio cannot reach the relay of the signer. Try again later.',
  signer_rejected: 'The signer refused the request. Allow the request in the signer app and try again.',
  signer_invalid: 'The signer gave an answer that Studio does not accept. Use a different signer.',
};

export function signErrorText(code) {
  return SIGN_MESSAGES[code] || null;
}

// The summary line of the Review sheet.
export function signSummary(count) {
  return `This write signs ${count} ${count === 1 ? 'event' : 'events'} with your signer.`;
}

// The label of the write button while the signer works.
export function signProgressLabel(progress) {
  return `Signing ${progress.done} of ${progress.total}…`;
}

// The addresses of a text area: one address on each line. Empty lines go out.
export function addressLines(text) {
  return String(text || '').split('\n').map((line) => line.trim()).filter(Boolean);
}

// The text area value of a list of addresses.
export function linesText(list) {
  return (list || []).join('\n');
}

// The field of an nsite or Blossom form that a message of `invalid_config`
// names. The result is null when the message names no field.
export function invalidConfigField(message) {
  const text = String(message || '').toLowerCase();
  if (/secret key/.test(text)) return 'secret_key';
  if (/relay/.test(text)) return 'relays';
  if (/gateway/.test(text)) return 'gateway';
  if (/server/.test(text)) return 'servers';
  if (/site name|feed/.test(text)) return 'd';
  return null;
}

// The name of a Blossom host when the user gives none: the host name of the
// first server.
export function blossomDefaultName(servers) {
  const first = servers[0];
  if (!first) return '';
  try {
    return new URL(first).host || first;
  } catch (_) {
    return first;
  }
}

// The site name of a new site from the GUID of a feed, as the core makes
// it: the text before the first `-`, in lowercase, with a maximum of 8
// characters (ADR 0048 item 4).
export function siteNameFromGuid(guid) {
  return String(guid || '').trim().split('-')[0].toLowerCase().slice(0, 8);
}

// True for a pasted private key in the `nsec` form. The lookup of the lists
// gets only this form, because the API reads 64 hexadecimal characters as a
// public key.
export function isNsec(text) {
  return /^nsec1[02-9ac-hj-np-z]+$/.test(String(text || '').trim());
}

// ---- The npub of a public key (NIP-19) ----

const CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
const GENERATORS = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];

function polymod(values) {
  let check = 1;
  for (const value of values) {
    const top = check >>> 25;
    check = ((check & 0x1ffffff) << 5) ^ value;
    for (let i = 0; i < 5; i += 1) {
      if ((top >>> i) & 1) check ^= GENERATORS[i];
    }
  }
  return check >>> 0;
}

// The `npub` text of a public key of 64 hexadecimal characters. A text that
// is not a public key gives null.
export function npubOf(hex) {
  if (typeof hex !== 'string' || !/^[0-9a-fA-F]{64}$/.test(hex)) return null;
  const bytes = hex.match(/../g).map((pair) => parseInt(pair, 16));
  const words = [];
  let accumulator = 0;
  let bits = 0;
  for (const byte of bytes) {
    accumulator = (accumulator << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      bits -= 5;
      words.push((accumulator >> bits) & 31);
    }
  }
  if (bits > 0) words.push((accumulator << (5 - bits)) & 31);
  const prefix = 'npub';
  const expanded = [
    ...[...prefix].map((c) => c.charCodeAt(0) >> 5),
    0,
    ...[...prefix].map((c) => c.charCodeAt(0) & 31),
  ];
  const mod = polymod([...expanded, ...words, 0, 0, 0, 0, 0, 0]) ^ 1;
  const checksum = [0, 1, 2, 3, 4, 5].map((i) => (mod >>> (5 * (5 - i))) & 31);
  return `${prefix}1${[...words, ...checksum].map((w) => CHARSET[w]).join('')}`;
}

// The gateway address of an nsite site, as the core makes it: the public key
// in 50 base36 characters, the site name, and the gateway.
export function siteAddress(config) {
  if (!config || !/^[0-9a-fA-F]{64}$/.test(config.public_key || '')) return null;
  const digits = BigInt(`0x${config.public_key}`).toString(36).padStart(50, '0');
  return `https://${digits}${config.d}.${config.gateway || 'nsite.lol'}`;
}
