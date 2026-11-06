// The source of an image or of the audio player (ADR 0109, ADR 0042). The
// functions have no Preact and no browser state. `app.js` and `editor.js`
// do the requests. The core owns each feed rule (ADR 0065): this module only
// chooses where the browser loads a file from.

// The notice of the player for a file on an `http` address (ADR 0109 item 4).
export const HTTP_NOTICE = 'This file uses http. Studio cannot play it here.';

const OP3_PREFIX = 'https://op3.dev/e';

// True for a placeholder address of a file on a file host with no public
// address (ADR 0122 item 12). The core gives the same text in the draft.
export function isPlaceholderAddress(url) {
  return typeof url === 'string' && url.startsWith('pending:');
}

// The address with no OP3 prefix, as `plain_address` of the core gives it
// (ADR 0076 item 1). It is an aid for the source only: the player must not
// make a download for OP3, and a held file has a plain planned address.
export function plainAddress(url) {
  if (typeof url !== 'string' || !url.startsWith(OP3_PREFIX)) return url;
  const rest = url.slice(OP3_PREFIX.length);
  let tail;
  if (rest.startsWith(',')) {
    const slash = rest.indexOf('/');
    if (slash < 0) return url;
    tail = rest.slice(slash + 1);
  } else if (rest.startsWith('/')) {
    tail = rest.slice(1);
  } else {
    return url;
  }
  return /^https?:\/\//.test(tail) ? tail : `https://${tail}`;
}

// The map from a planned address to a library file ID, from the answer of
// `GET /media/held`: `{ files: [{ file, url }] }`.
export function heldMap(answer) {
  const map = new Map();
  for (const row of (answer && answer.files) || []) {
    if (row && typeof row.url === 'string' && Number.isInteger(row.file)) map.set(row.url, row.file);
  }
  return map;
}

// Where the browser loads the file at `url` from. `held` is the map of
// `heldMap`, or `null` while the list loads. The answer is one of:
//   { kind: 'direct', src }          an https address, loaded from its host
//   { kind: 'held', file }           a file that only Studio holds, loaded through Studio
//   { kind: 'none', reason }         nothing to load: `empty`, `http`, `other` or `loading`
export function mediaSource(url, held) {
  if (typeof url !== 'string' || url.trim() === '') return { kind: 'none', reason: 'empty' };
  // A placeholder has no public address, so only Studio can load it (ADR
  // 0122 item 14).
  if (isPlaceholderAddress(url.trim())) {
    if (held === null || held === undefined) return { kind: 'none', reason: 'loading' };
    if (held.has(url.trim())) return { kind: 'held', file: held.get(url.trim()) };
    return { kind: 'none', reason: 'other' };
  }
  const plain = plainAddress(url.trim());
  if (/^http:\/\/\S+$/i.test(plain)) return { kind: 'none', reason: 'http' };
  if (!/^https:\/\/\S+$/i.test(plain)) return { kind: 'none', reason: 'other' };
  if (held === null || held === undefined) return { kind: 'none', reason: 'loading' };
  if (held.has(plain)) return { kind: 'held', file: held.get(plain) };
  return { kind: 'direct', src: plain };
}

// The notice below the player: the http text for an `http` address, else
// nothing.
export function playerNotice(source) {
  return source.kind === 'none' && source.reason === 'http' ? HTTP_NOTICE : null;
}

// The error of the player when the browser does not play the file. A file
// from its host is a load from a different site, and a browser setting can
// block it, so the text names that cause. A file that Studio holds loads
// from Studio, so the setting has no effect on it.
export const PLAYER_ERROR = 'Studio cannot play this file.';
// The text below the player from a press of Play until the audio starts. A
// slow start is not an error (human test of 0.1.0-rc.18).
export const PLAYER_STARTING = 'Starting…';

// The part of the position bar that is done, from 0 to 1. `styles.css` draws
// the fill and the cursor from this one value, so that they agree.
export function playerProgress(position, length) {
  if (!Number.isFinite(length) || length <= 0 || !Number.isFinite(position)) return 0;
  return Math.min(1, Math.max(0, position / length));
}

// True when `play()` stopped only because a later pause or load replaced it.
// The audio can still start, so this is not an error.
export function playWasReplaced(error) {
  return Boolean(error) && error.name === 'AbortError';
}

export const DIRECT_PLAYER_ERROR = 'Studio cannot play this file. Your browser can block audio from other sites.';

export function playerError(source) {
  return source && source.kind === 'direct' ? DIRECT_PLAYER_ERROR : PLAYER_ERROR;
}

// Can the player play this source?
export function canPlay(source) {
  return source.kind === 'direct' || source.kind === 'held';
}
