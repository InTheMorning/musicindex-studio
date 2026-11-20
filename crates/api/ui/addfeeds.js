// The model of the Add One You Have page and of the form that makes a feed (ADR 0073
// items 1 to 10, ADR 0042). The functions have no Preact and no browser
// state. `app.js` shows the rows and the form.
import { feedKind, kindGlow, publisherArt } from './shell.js';

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
  album: { medium: 'music', titleLabel: 'Title', authorLabel: 'Artist name', role: false, button: 'Make the Release', busyLabel: 'Making the Release…' },
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

// The title of the sheet that makes a page or a release (ADR 0164 item 7, ADR
// 0230 item 8). It follows the type, and the role of a publisher page. A
// release names its parent, when it has one. It does not use the word "feed".
const PAGE_WORDS = { artist: 'artist page', label: 'label page', hosting: 'hosting page' };

export function makeTitle(type, role, parent) {
  if (type === 'publisher') return `Make a new ${PAGE_WORDS[role] || 'artist or label page'}`;
  if (type === 'show') return 'Make a new show';
  if (type === 'playlist') return 'Make a new playlist';
  return parent ? `New release for ${parent}` : 'New release';
}

// The parent card of the form that makes a release (ADR 0230 items 9 and 10):
// the art, the name and the type of the parent page. It gives `null` when
// `publisherId` names no page of the list.
export function parentCard(feeds, publisherId) {
  const page = (feeds || []).find((feed) => feed.id === publisherId);
  if (!page) return null;
  return {
    id: page.id,
    title: page.title || page.path || 'Untitled page',
    typeLabel: feedKind(page).label,
    dot: feedKind(page).dot,
    glow: kindGlow(feedKind(page)),
    art: publisherArt(page, feeds),
  };
}

// The kind that a track count gives (ADR 0230 item 14). This is the
// customary split of the music stores. Music Index does not give a count
// in the values of a feed yet, so a pick leaves the kind as it is.
export function kindOfTrackCount(count) {
  if (!Number.isInteger(count) || count < 1) return null;
  if (count <= 3) return 'single';
  return count <= 6 ? 'ep' : 'album';
}

const hasText = (value) => typeof value === 'string' && value.trim() !== '';

// The choices of a found release (ADR 0230 items 11, 14 and 15). `values` is
// the answer of `GET /musicindex/values/{guid}`, and `hit` is the row of a
// release. A row of an artist page or a label page gives no choice. The one
// choice has only the values that Music Index gives: `title`, `author`,
// `kind`, `description`, `image` and `website`.
export function releaseFindChoices(values, hit) {
  if (!hit) return [];
  const kind = kindOfTrackCount(values.track_count);
  const found = {
    title: hasText(values.title) ? values.title : (hasText(hit.title) ? hit.title : null),
    author: hasText(hit.artist) ? hit.artist : null,
    kind,
    description: hasText(values.description) ? values.description : null,
    image: hasText(values.image_url) ? values.image_url : null,
    website: hasText(values.website) ? values.website : null,
  };
  const present = Object.fromEntries(Object.entries(found).filter(([, value]) => value !== null));
  return Object.keys(present).length === 0 ? [] : [{ id: 'release', values: present }];
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

// A new release has no track, so it needs its first track at once.
export function shouldAddTracksAfterNewRelease(type) {
  return type === 'album';
}
