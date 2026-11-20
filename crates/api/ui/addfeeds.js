// The model of the Add One You Have page and of the form that makes a feed (ADR 0073
// items 1 to 10, ADR 0042). The functions have no Preact and no browser
// state. `app.js` shows the rows and the form.
import { compareNames, feedKind } from './shell.js';

// The rows of the page "Add One You Have" (ADR 0230 item 4). The page has one
// group. The row "From a file host folder" needs a file host, so it goes out
// when no host is connected. With no host, the flag `connect` shows the
// button that connects one (ADR 0230 item 3: each make entry needs a host).
export function addChoices({ hasHost }) {
  const have = [
    { id: 'find', title: 'By address or name', text: 'Search Music Index, or give an address' },
    { id: 'folder', title: 'From a file host folder', text: 'A feed file on a connected file host' },
    { id: 'msp', title: 'From MSP 2.0 backups', text: 'The .json files that MSP 2.0 saved' },
  ];
  return [
    { id: 'have', label: 'Add One You Have', rows: hasHost ? have : have.filter((row) => row.id !== 'folder'), connect: !hasHost },
  ];
}

// The title of the page "Add One You Have" (ADR 0230 item 4).
export const ADD_PAGE_TITLE = 'Add One You Have';

// The entries of the add menu of Releases and of an artist page (ADR 0230
// items 1 to 4). `release` is the add action of the artist of the page (from
// `releaseAdd`), or null on the top page, on a label page and on a hosting
// page. Then Release has no start parent. `open` is the form that the entry
// opens. `have` opens the page "Add One You Have".
export function addMenuEntries(release) {
  return [
    { id: 'release', label: 'Release', open: 'album', start: release ? release.start : {} },
    { id: 'artist', label: 'Artist', open: 'publisher', start: { role: 'artist' } },
    { id: 'label', label: 'Label', open: 'publisher', start: { role: 'label' } },
    { id: 'show', label: 'Show', open: 'show', start: {} },
    { id: 'playlist', label: 'Playlist', open: 'playlist', start: {} },
    { id: 'have', label: 'Add One You Have…', open: null, separatorBefore: true },
  ];
}

// The values of the select menu "Type" in the form that makes a feed.
export const MAKE_TYPES = [
  ['publisher', 'Artist or label'],
  ['show', 'Show'],
  ['playlist', 'Playlist'],
];

export const RELEASE_KINDS = [
  ['', 'From the tracks'],
  ['album', 'Album'],
  ['ep', 'EP'],
  ['single', 'Single'],
];

// The menu "Kind": the first option names the kind that the tracks give now.
// The label comes from feedKind, as the tag of the release does. With no track
// and no live item, the tracks give no kind yet.
export function releaseKindOptions(kindShown, trackCount, liveItemCount = 0) {
  const noTracks = !trackCount && !liveItemCount;
  const label = feedKind({ medium: 'music', kind_shown: kindShown }).label;
  const first = noTracks ? 'From the tracks (no tracks yet)' : `From the tracks (${label})`;
  return [['', first], ...RELEASE_KINDS.slice(1)];
}

const FIELDS = {
  album: { medium: 'music', titleLabel: 'Album title', authorLabel: 'Artist', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' },
  publisher: { medium: 'publisher', titleLabel: 'Name', authorLabel: null, role: true, button: 'Make the Page', busyLabel: 'Making the Page…' },
  show: { medium: 'music', titleLabel: 'Show title', authorLabel: 'Artist', role: false, button: 'Make the Show', busyLabel: 'Making the Show…' },
  playlist: { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the Playlist', busyLabel: 'Making the Playlist…' },
};

// The medium, the labels and the button of the form for one type. `role` is
// true when the form asks for the role. `authorLabel` is null when the name
// is also the author.
export function makeFields(type) {
  return FIELDS[type] || FIELDS.album;
}

// The title of the sheet that makes a feed (ADR 0164 item 7). It follows the
// type, and the role of a publisher feed. It does not use the word "feed".
const PAGE_WORDS = { artist: 'artist page', label: 'label page', hosting: 'hosting page' };

export function makeTitle(type, role) {
  if (type === 'publisher') return `Make a new ${PAGE_WORDS[role] || 'artist or label page'}`;
  if (type === 'show') return 'Make a new show';
  if (type === 'playlist') return 'Make a new playlist';
  return 'Make a new release';
}

// The start value of "Feed file" (ADR 0073 item 8). It is an edit aid, not a
// feed rule (ADR 0065): the core checks the path. The title goes to lower case
// with no accents. Each run of other characters becomes one hyphen, with no
// hyphen at the two ends. An empty result gives an empty path.
export function feedFileStart(title) {
  const stem = (title || '')
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return stem === '' ? '' : `${stem}.xml`;
}

// The values of the select menu "Start from" (ADR 0077 item 1). The first
// value is an empty album. Then come the music feeds of the list of releases, in
// the sequence of their titles. A feed with no title uses its address.
export function startChoices(feeds) {
  const albums = (feeds || [])
    .filter((feed) => (feed.medium || '').toLowerCase() === 'music')
    .map((feed) => [String(feed.id), feed.title || feed.path || 'Untitled feed']);
  albums.sort((a, b) => compareNames(a[1], b[1]));
  return [['', 'Empty album'], ...albums];
}

// A new album with no source needs its first track at once.
export function shouldAddTracksAfterNewRelease(type, from) {
  return type === 'album' && !from;
}
