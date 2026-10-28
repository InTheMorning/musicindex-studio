// The sequence of the sections and the fields of each editor page (ADR 0068,
// ADR 0042). This list is the sequence of the pages. The functions have no
// Preact and no browser state. `editor.js` maps each ID to a render function.

// The fields of the first section of the three feed pages.
const FEED_FIELDS = ['title', 'author', 'artwork', 'description', 'link', 'listing', 'explicit'];

const PAGES = {
  album: () => [
    { id: 'album', fields: [...FEED_FIELDS] },
    { id: 'tracks' },
    { id: 'details', rows: ['recipients', 'credits'] },
    { id: 'publishers' },
    { id: 'feed', rows: ['settings', 'facts'] },
  ],
  publisher: () => [
    { id: 'channel', fields: [...FEED_FIELDS] },
    { id: 'releases' },
    { id: 'role' },
    { id: 'feed', rows: ['settings', 'facts'] },
  ],
  playlist: () => [
    { id: 'channel', fields: [...FEED_FIELDS] },
    { id: 'feed', rows: ['settings', 'facts'] },
  ],
  settings: () => [
    { id: 'language' },
    { id: 'keywords' },
    { id: 'owner' },
    { id: 'locked' },
    { id: 'editors' },
  ],
  track: () => [
    { id: 'track', fields: ['title', 'audio', 'duration', 'alternates', 'artwork', 'description'] },
    { id: 'credits' },
    { id: 'lyrics' },
    { id: 'payments' },
    { id: 'more', rows: ['pub_date', 'explicit', 'identity'] },
  ],
};

// The sections of the page `name`, in their sequence. Each call gives a new
// copy, so a caller can change the copy.
export function pageSequence(name) {
  return PAGES[name]();
}
