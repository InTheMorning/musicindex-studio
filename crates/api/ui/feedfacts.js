// The display of the facts of a feed (ADR 0042). This module has no Preact and
// no browser state. The Node tests in `ui-tests/feedfacts.test.mjs` examine it.

export const LOCK_HELP = 'This tells other podcast platforms not to import this feed. A platform can use the lock owner email to confirm who owns the feed.';

// A version marker of a file host, in a short form. An S3 or R2 ETag has
// quote marks, and a weak ETag also has a prefix.
export function versionLabel(version) {
  return String(version).replace(/^W\//, '').replaceAll('"', '').slice(0, 19);
}
