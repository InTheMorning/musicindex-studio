// The model of the add forms (ADR 0233, ADR 0042). These functions have no
// Preact and no browser state. `app.js` shows their rows and forms.
import { feedKind, kindGlow, publisherArt, headingIndex } from './shell.js';

// The entries of the add menu of Releases and of a page head (ADR 0230 items
// 1 to 4, ADR 0240 items 1 and 4). `release` is the add action of the artist
// of the page, or null when Release has no start parent. `head` is the feed of
// the page head, or null on the page Releases. On a hosting page or a label
// page, the menu has no Release and no role of the same rank or a higher rank,
// and each remaining role makes a page below the head.
export function addMenuEntries(release, head = null) {
  const roles = [['artist', 'Artist'], ['label', 'Label'], ['hosting', 'Publisher']];
  const rank = head ? headingIndex(head) : null;
  const publisherHead = head && (head.publisher_roles || []).some((role) => ['artist', 'label', 'hosting'].includes(role));
  const appliesRank = head && (head.publisher_roles || []).some((role) => role === 'label' || role === 'hosting');
  const below = Boolean(appliesRank && rank !== null && rank <= 1);
  const roleEntries = roles
    .filter(([role]) => !publisherHead || (appliesRank && (!below || headingIndex({ publisher_roles: [role] }) > rank)))
    .map(([role, label]) => ({ id: role, label, open: 'publisher', start: below ? { role, below_id: head.id } : { role } }));
  return [
    ...(below ? [] : [{ id: 'release', label: 'Release', open: 'album', start: release ? release.start : {} }]),
    ...roleEntries,
    { id: 'show', label: 'Show', open: 'show', start: {} },
    { id: 'playlist', label: 'Playlist', open: 'playlist', start: {} },
  ];
}

// True when a row of the search is an artist: a role row with the role
// artist, or an inferred artist (ADR 0233 item 13).
export function addRowIsArtist(row) {
  const kind = row?.kind || {};
  return (kind.type === 'role' && (kind.roles || []).includes('artist')) || kind.type === 'inferred_artist';
}

// The artist and label rows of the search that a form shows (ADR 0233 item
// 2). A release form and an artist form show artists, and a label form shows
// labels. A show form and a playlist form show no role row. A row with no
// role shows in each form. The feed results of Music Index have no kind, so
// each form shows them.
export function addSearchRows(rows, type, role) {
  const wanted = type === 'album' ? ['artist'] : type === 'publisher' ? [role || 'artist'] : [];
  return (rows || []).filter((row) => {
    const kind = row.kind || {};
    if (kind.type === 'role') return (kind.roles || []).some((r) => wanted.includes(r));
    if (kind.type === 'inferred_artist') return wanted.includes('artist');
    return true;
  });
}

// The two import rows in the Music Index sheet (ADR 0233 item 3).
export function addSheetRows(hasHost) {
  return [
    ...(hasHost ? [{ id: 'folder', title: 'Add from a File Host Folder…' }] : []),
    { id: 'msp', title: 'Add from MSP 2.0 Backups…' },
  ];
}

// The mark of a result in the add sheet (ADR 0233 items 6 and 7).
export function addResultMark(row) {
  if (row?.yours && row?.in_studio_id != null) return 'Yours · In Studio';
  if (row?.yours) return 'Yours';
  if (row?.in_studio_id != null) return 'In Studio';
  return null;
}

// The form start values of a found feed (ADR 0233 items 8 to 14).
export function addFoundValues(row, preview = {}, values = {}) {
  const found = {
    title: values.title || row?.title || '',
    author: preview.author || values.author || row?.artist || '',
    description: values.description || '',
    image: values.image || values.image_url || row?.image_url || '',
    website: values.website || '',
  };
  return Object.fromEntries(Object.entries(found).filter(([, value]) => value !== ''));
}

// The start values and the lines of a page form for a found feed (ADR 0238
// items 4, 5 and 7). `found` is the copy preview with the `title` of the feed.
// `row` is the row of the search. Only the Artist form gets a value. A feed
// does not name its label. The host is the person who hosts the feed, and the
// owner of a feed is frequently a hosting service, so the Hosting form gets no
// name either.
export function foundPageValues(role, found = {}, row = {}) {
  const read = !found.failed;
  const title = role === 'artist' ? ((read ? found.author : '') || row.artist || '') : '';
  const name = found.title || '';
  const ask = role === 'hosting' ? `Type the name of the person who hosts ${name}.` : `${name} does not name its ${role}. Type the name.`;
  return {
    title,
    website: role === 'artist' && read ? found.link || '' : '',
    copiesRecipients: role === 'artist' && read && found.recipients > 0,
    line: read ? `Takes the ${role} of ${name}. Links ${name} to this page.` : `Studio could not read the feed file of ${name}. Only the name is copied.`,
    emptyLine: title === '' ? ask : '',
  };
}

// The page that Music Index gives for a found feed, or null (ADR 0238 item 2).
// `values` is the answer of `/musicindex/values/{guid}`. The page is an artist
// page when its title is the artist of the feed. Otherwise it has no role.
export function foundIndexPage(values, artist) {
  if (!values || !values.publisher_feed_guid) return null;
  const words = (text) => String(text || '').toLowerCase().split(/[^\p{L}\p{N}]+/u).filter(Boolean).join(' ');
  const title = values.publisher_feed_title || '';
  const same = title !== '' && words(title) === words(artist);
  return { feed_guid: values.publisher_feed_guid, feed_url: null, title, rel: same ? 'artist' : null };
}

// The pages that a found feed names in `podcast:publisher` and that a page form
// can take (ADR 0238 item 2): the pages with the role of the form, and the
// pages with no role.
export function foundPages(pages, role) {
  return (pages || []).filter((page) => {
    if (!page.feed_guid && !page.feed_url) return false;
    const words = String(page.rel || '').split(/[\s,]+/).filter(Boolean).map((word) => word.toLowerCase());
    return words.length === 0 || words.includes(role);
  });
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
  show: { medium: 'music', titleLabel: 'Show title', authorLabel: 'Artist name', role: false, button: 'Make the Show', busyLabel: 'Making the Show…' },
  playlist: { medium: 'musicL', titleLabel: 'Playlist name', authorLabel: 'Author', role: false, button: 'Make the Playlist', busyLabel: 'Making the Playlist…' },
};

// The medium, the labels and the button of the form for one type. `role` is
// true when the form asks for the role. `authorLabel` is null when the name
// is also the author.
export function makeFields(type) {
  return FIELDS[type] || FIELDS.album;
}

// A publisher form with a start role does not need the Type menu (ADR 0243 item 15).
export function makeTypeVisible(type, startRole = null) {
  return !(type === 'publisher' && startRole);
}

// The report after a folder match (ADR 0243 items 14 and 15).
export function folderMatchReport(result = {}) {
  const notFound = result.not_found || [];
  return {
    heading: notFound.length === 0 ? null
      : notFound.length === 1 ? '1 feed stays read-only. Studio did not find it in this folder.'
        : `${notFound.length} feeds stay read-only. Studio did not find them in this folder.`,
    notFound,
    truncated: result.truncated === true,
    truncatedText: result.truncated === true
      ? 'Studio stopped at 5000 files. Connect a smaller folder, for example the folder that holds your feeds.'
      : null,
  };
}

// The title of the sheet that makes a page or a release (ADR 0164 item 7, ADR
// 0230 item 8). It follows the type, and the role of a publisher page. A
// release names its parent, when it has one. It does not use the word "feed".
const PAGE_WORDS = { artist: 'artist page', label: 'label page', hosting: 'publisher page' };

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

// The start value of "Feed file" (ADR 0212 item 10). It is an edit aid, not a
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
