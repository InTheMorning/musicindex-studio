// The source of an image or of the audio player (ADR 0109, ADR 0042). The
// functions have no Preact and no browser state. `app.js` and `editor.js`
// do the requests. The core owns each feed rule (ADR 0065): this module only
// chooses where the browser loads a file from.

// The notice of the player for a file on an `http` address (ADR 0109 item 4).
export const HTTP_NOTICE = 'This file uses http. Studio cannot play it here.';

const OP3_PREFIX = 'https://op3.dev/e';

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

// Can the player play this source?
export function canPlay(source) {
  return source.kind === 'direct' || source.kind === 'held';
}
