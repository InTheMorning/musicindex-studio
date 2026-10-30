// The model of the Add feeds page and of the form that makes a feed (ADR 0073
// items 1 to 10, ADR 0042). The functions have no Preact and no browser
// state. `app.js` shows the rows and the form.
import { feedKind } from './shell.js';

// The rows of the Add feeds page, in two groups. With no connected host, the
// group "Make a new feed" has no rows and the flag `connect`. The row
// "From a host folder" needs a host, so it goes out too.
export function addChoices({ hasHost }) {
  const make = [
    { id: 'album', title: 'Album', text: 'A release with its tracks' },
    { id: 'publisher', title: 'Artist or label', text: 'A list of your releases' },
    { id: 'show', title: 'Show', text: 'A feed for live shows' },
    { id: 'playlist', title: 'Playlist', text: 'Tracks from other feeds' },
  ];
  const have = [
    { id: 'find', title: 'By address or name', text: 'Search musicindex, or give a feed address' },
    { id: 'folder', title: 'From a host folder', text: 'A feed file on a connected host' },
    { id: 'msp', title: 'From MSP 2.0 backups', text: 'The .json files that MSP 2.0 saved' },
  ];
  return [
    hasHost
      ? { id: 'make', label: 'Make a new feed', rows: make }
      : { id: 'make', label: 'Make a new feed', rows: [], connect: true },
    { id: 'have', label: 'Add a feed you have', rows: hasHost ? have : have.filter((row) => row.id !== 'folder') },
  ];
}

// The values of the select menu "Type" in the form that makes a feed.
export const MAKE_TYPES = [
  ['album', 'Album'],
  ['publisher', 'Artist or label'],
  ['show', 'Show'],
  ['playlist', 'Playlist'],
];

const FIELDS = {
  album: { medium: 'music', titleLabel: 'Album title', authorLabel: 'Artist', role: false, button: 'Make the release', busyLabel: 'Making the release…' },
  publisher: { medium: 'publisher', titleLabel: 'Name', authorLabel: null, role: true, button: 'Make the feed', busyLabel: 'Making the feed…' },
  show: { medium: 'music', titleLabel: 'Show feed title', authorLabel: 'Artist', role: false, button: 'Make the show feed', busyLabel: 'Making the show feed…' },
  playlist: { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the playlist', busyLabel: 'Making the playlist…' },
};

// The medium, the labels and the button of the form for one type. `role` is
// true when the form asks for the role. `authorLabel` is null when the name
// is also the author.
export function makeFields(type) {
  return FIELDS[type] || FIELDS.album;
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
// value is an empty album. Then come the albums of the list of releases, in
// the sequence of their titles. A feed with no title uses its address.
export function startChoices(feeds) {
  const albums = (feeds || [])
    .filter((feed) => feedKind(feed).label === 'Album')
    .map((feed) => [String(feed.id), feed.title || feed.path || 'Untitled feed']);
  albums.sort((a, b) => a[1].localeCompare(b[1]));
  return [['', 'Empty album'], ...albums];
}
