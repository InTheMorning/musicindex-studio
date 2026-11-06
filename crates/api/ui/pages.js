// The sequence, the tiers and the fields of the sections of each page
// (ADR 0071, ADR 0068, ADR 0042). This list is the sequence of the pages. The
// functions have no Preact and no browser state. `editor.js` maps each ID to a
// render function.

// The file name for the messages. The text has no "ui" folder with slashes on
// both sides, because a Rust test reads such text as a served address.
const PAGES_FILE = 'crates/api/ui' + '/pages.js';

// The tiers, in their sequence (ADR 0071 item 1).
export const TIERS = ['identity', 'content', 'details', 'setup', 'limits'];

// The fields of the first section of the three feed pages.
const FEED_FIELDS = ['title', 'author', 'nostr', 'artwork', 'description', 'link', 'listing', 'explicit'];

const PAGES = {
  album: () => [
    { id: 'album', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'tracks', tier: 'content' },
    { id: 'publishers', tier: 'details' },
    { id: 'feed', tier: 'setup', rows: ['facts'] },
  ],
  publisher: () => [
    // The role says what the page is, so it is first (ADR 0075 item 1, as
    // amended on 2026-11-02).
    { id: 'role', tier: 'identity' },
    { id: 'channel', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'releases', tier: 'content' },
    { id: 'members', tier: 'details' },
    { id: 'feed', tier: 'setup', rows: ['facts'] },
  ],
  playlist: () => [
    { id: 'channel', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'publishers', tier: 'details' },
    { id: 'feed', tier: 'setup', rows: ['facts'] },
  ],
  settings: ({ album = false } = {}) => [
    { id: 'language', tier: 'setup' },
    { id: 'keywords', tier: 'setup' },
    { id: 'owner', tier: 'setup' },
    { id: 'locked', tier: 'setup' },
    ...(album ? [{ id: 'op3', tier: 'setup' }] : []),
    { id: 'editors', tier: 'setup' },
  ],
  track: () => [
    { id: 'track', tier: 'identity', fields: ['title', 'audio', 'alternates', 'artwork', 'description'] },
    { id: 'credits', tier: 'details' },
    { id: 'lyrics', tier: 'details' },
    { id: 'payments', tier: 'details' },
    { id: 'more', tier: 'setup', rows: ['pub_date', 'explicit', 'identity'] },
  ],
  hosts: () => [
    { id: 'cards', tier: 'identity' },
    { id: 'add', tier: 'content' },
  ],
  person: () => [
    { id: 'profile', tier: 'identity' },
    { id: 'check', tier: 'identity' },
    { id: 'more', tier: 'content' },
    { id: 'payment', tier: 'content' },
    { id: 'credits', tier: 'details' },
    { id: 'support', tier: 'details' },
    { id: 'member', tier: 'details' },
    { id: 'change', tier: 'setup' },
    { id: 'delete', tier: 'limits' },
  ],
  'studio-settings': ({ server, holdAssets = true }) => [
    { id: 'scale', tier: 'setup' },
    // Only the desktop app has a window to close (ADR 0124 item 29).
    ...(server ? [] : [{ id: 'window', tier: 'setup' }]),
    ...(server ? [{ id: 'account', tier: 'setup' }] : []),
    { id: 'apps', tier: 'setup' },
    { id: 'announcements', tier: 'setup' },
    { id: 'nostr', tier: 'setup' },
    // The desktop moves its data from here. A server has "Export Account" in
    // the Account panel (ADR 0107 item 3).
    ...(server ? [] : [{ id: 'move', tier: 'setup' }]),
    // A server with `--hold-assets off` has no downloads into the library, so
    // the section "Files on this server" takes the place of the library
    // limits (ADR 0105 items 14 and 17).
    server && !holdAssets ? { id: 'files', tier: 'limits' } : { id: 'library', tier: 'limits' },
  ],
};

// The sections of the page `name`, in their sequence. Each call gives a new
// copy, so a caller can change the copy. The page `studio-settings` leaves out
// `account` when `options.server` is false. It shows `files` in the place of
// `library` when `options.server` is true and `options.holdAssets` is false. The other pages ignore `options`.
export function pageSequence(name, options = {}) {
  return PAGES[name](options);
}

// The names of the pages that `pageSequence` knows.
export function pageNames() {
  return Object.keys(PAGES);
}

// The error message for a list of sections of the page `page`, or `null`. A
// section needs a tier of `TIERS`, and the tiers do not go down (ADR 0071).
export function tierError(page, sections) {
  let previous = null;
  for (const { id, tier } of sections) {
    const index = TIERS.indexOf(tier);
    if (index < 0) {
      return `ADR 0071 item 1: on the page ${page}, the section ${id} has the tier ${tier}, which is not a tier. Give the section a tier of ADR 0071 item 1 in ${PAGES_FILE}.`;
    }
    if (previous && index < TIERS.indexOf(previous)) {
      return `ADR 0071 item 6: on the page ${page}, the section ${id} (${tier}) comes after a section of the tier ${previous}. Give the section its tier of ADR 0071 item 1, and put it after each section of a lower tier in ${PAGES_FILE}.`;
    }
    previous = tier;
  }
  return null;
}
