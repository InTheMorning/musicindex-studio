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
const FEED_FIELDS = ['title', 'author', 'artwork', 'description', 'link', 'listing', 'explicit'];

const PAGES = {
  album: () => [
    { id: 'album', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'tracks', tier: 'content' },
    { id: 'details', tier: 'details', rows: ['recipients', 'credits'] },
    { id: 'publishers', tier: 'details' },
    { id: 'feed', tier: 'setup', rows: ['settings', 'facts'] },
  ],
  publisher: () => [
    { id: 'channel', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'releases', tier: 'content' },
    { id: 'details', tier: 'details', rows: ['recipients', 'credits'] },
    { id: 'role', tier: 'setup' },
    { id: 'feed', tier: 'setup', rows: ['settings', 'facts'] },
  ],
  playlist: () => [
    { id: 'channel', tier: 'identity', fields: [...FEED_FIELDS] },
    { id: 'feed', tier: 'setup', rows: ['settings', 'facts'] },
  ],
  settings: () => [
    { id: 'language', tier: 'setup' },
    { id: 'keywords', tier: 'setup' },
    { id: 'owner', tier: 'setup' },
    { id: 'locked', tier: 'setup' },
    { id: 'editors', tier: 'setup' },
  ],
  track: () => [
    { id: 'track', tier: 'identity', fields: ['title', 'audio', 'duration', 'alternates', 'artwork', 'description'] },
    { id: 'credits', tier: 'details' },
    { id: 'lyrics', tier: 'details' },
    { id: 'payments', tier: 'details' },
    { id: 'more', tier: 'setup', rows: ['pub_date', 'explicit', 'identity'] },
  ],
  hosts: () => [
    { id: 'cards', tier: 'identity' },
    { id: 'add', tier: 'content' },
  ],
  'studio-settings': ({ server }) => [
    ...(server ? [{ id: 'account', tier: 'setup' }] : []),
    { id: 'apps', tier: 'setup' },
    { id: 'announcements', tier: 'setup' },
    { id: 'library', tier: 'limits' },
  ],
};

// The sections of the page `name`, in their sequence. Each call gives a new
// copy, so a caller can change the copy. The page `studio-settings` leaves out
// `account` when `options.server` is false. The other pages ignore `options`.
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
