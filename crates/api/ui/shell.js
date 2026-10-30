// The shell models (ADR 0042). This file imports only the live item model.
// It holds the logic
// of the left panel, of the Releases list and of the navigation. The
// components in `app.js` and `editor.js` use these functions. The Node tests
// in `crates/api/ui-tests/` examine them.
import { liveKey, isLiveItem, statusLabel } from './live.js';
import { coverOf } from './cover.js';

// The eight roles of a publisher feed, in the sequence of musicindex ADR
// 0036 R-ROLE-1 (ADR 0083 item 1).
export const ROLES = ['artist', 'label', 'hosting', 'producer', 'host', 'author', 'network', 'sponsor'];

// The first-class roles, which the Role menu and the form of a new
// publisher feed offer (musicindex ADR 0036 R-ROLE-2, ADR 0083 items 2
// and 3).
export const FIRST_CLASS_ROLES = ['artist', 'label', 'hosting'];

// The word of a role, for example "Artist".
export const roleWord = (role) => role.charAt(0).toUpperCase() + role.slice(1);

// The kind of a medium: its dot class and its label. A publisher feed lists
// each of its roles, and its dot has the color of the first one (musicindex
// ADR 0037 R-WORD-7, ADR 0084). The dot is hollow while nothing confirms the
// roles (musicindex ADR 0033 R-WORD-3 item 3, ADR 0082 item 6). A feed with
// no role shows "Feed" with a hollow dot.
export function mediumKind(medium, roles = [], confirmed = false) {
  switch ((medium || '').toLowerCase()) {
    case 'publisher': {
      const own = ROLES.filter((role) => (roles || []).includes(role));
      if (own.length === 0) return { dot: 'feed hollow', label: 'Feed' };
      return { dot: confirmed ? own[0] : `${own[0]} hollow`, label: own.map(roleWord).join(' · ') };
    }
    case 'musicl': return { dot: 'playlist', label: 'Playlist' };
    case 'music': return { dot: 'feed', label: 'Album' };
    default: return { dot: 'feed', label: 'Feed' };
  }
}

// The words of the roles in the `rel` of a link, joined as a type label
// (musicindex ADR 0037 R-WORD-7). A `rel` with no known role gives "No role".
export function relLabel(rel) {
  const words = String(rel || '').toLowerCase().split(/\s+/);
  const own = ROLES.filter((role) => words.includes(role));
  return own.length > 0 ? own.map(roleWord).join(' · ') : 'No role';
}

// The kind of a feed of the collection or of a discovery. The user confirms
// a set of roles, or the core finds the first role on the two sides of a
// link (ADR 0082 item 5, ADR 0087 item 2).
export function feedKind(feed) {
  return mediumKind(feed.medium, feed.publisher_roles, feed.roles_confirmed === true || feed.roles_linked === true);
}

// The choice of the Role menu for a set of roles: "artist", "label",
// "hosting", "custom", or "" for no role (ADR 0083 item 2).
export function roleChoice(roles) {
  const set = roles || [];
  if (set.length === 0) return '';
  if (set.length === 1 && FIRST_CLASS_ROLES.includes(set[0])) return set[0];
  return 'custom';
}

// The set after a change of one check box. The set keeps one or more
// roles, so a change that clears the last role gives the set as it is.
export function toggleRole(roles, role, checked) {
  const next = ROLES.filter((r) => (r === role ? checked : (roles || []).includes(r)));
  return next.length > 0 ? next : roles;
}

// The color of the logo shape of a feed with no cover: the entity color of
// its kind (musicindex ADR 0021 R-ART-2). A feed with no role uses the feed
// color.
export function kindGlow(kind) {
  const dot = kind.dot.split(' ')[0];
  return `var(--c-${['playlist', ...ROLES].includes(dot) ? dot : 'feed'})`;
}

// Is this feed a publisher feed (ADR 0041 item 1)?
export function isPublisher(feed) {
  return (feed.medium || '').toLowerCase() === 'publisher';
}

// Is this feed a playlist, a musicL feed (ADR 0053 item 1)?
export function isPlaylist(feed) {
  return (feed.medium || '').toLowerCase() === 'musicl';
}

// Is this feed an album, a publisher feed or a playlist that the editor can
// open? A feed with no connector opens too (ADR 0029 item 4, ADR 0041 item 3,
// ADR 0053 item 1, ADR 0067 item 1).
export function isEditable(feed) {
  return mediumKind(feed.medium).label === 'Album' || isPublisher(feed) || isPlaylist(feed);
}

// Is this feed a release: an album or a playlist (ADR 0085 item 6, ADR 0086
// item 1)?
export function isRelease(feed) {
  return mediumKind(feed.medium).label === 'Album' || isPlaylist(feed);
}

// The hint of a feed that the editor does not open (ADR 0090 item 1).
export function otherFeedText(feed) {
  const medium = feed.medium ? `the type “${feed.medium}”` : 'no type';
  return `This feed has ${medium}. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed.`;
}

// Is this feed read-only? A feed with no connector has no host that Studio
// can write to (ADR 0067 item 1).
export function isReadOnly(feed) {
  return !feed.connector_id;
}

// The second line of the feed move row: what the address from before gets
// (ADR 0038 items 16 to 18).
export function moveNote(method) {
  if (method === 'none') return 'Studio writes nothing at the address before the move.';
  return method === 'redirect'
    ? 'The address from before gets a redirect to the new address.'
    : 'The address from before gets a pointer file with the new address.';
}

// What Review shows for the host of a feed (ADR 0067 items 5 and 9). `host`:
// the feed has a connector. `choose`: the feed is read-only and has no host
// choice. `move`: the feed is read-only and has a host choice. `feedMove` is
// the move of the diff, or null.
export function reviewHost({ hasHost, isReadOnly: readOnly, feedMove }) {
  if (!readOnly) return { kind: 'host' };
  if (feedMove) return { kind: 'move', connectorId: feedMove.connector_id };
  if (hasHost) return { kind: 'move', connectorId: null };
  return { kind: 'choose' };
}

// The key of a track: its GUID, or its client ID when it has no GUID yet
// (task 014c). The outline and the album page use this key to open the
// track page of one track.
export function trackKey(item) {
  return item.guid ? item.guid : `cid-${item._cid}`;
}

// Is this track the track of the key? A new track has a key from its client
// ID. After the first save, the server gives it a GUID and its key changes.
// The client ID stays, so the old key continues to find the track.
export function isTrack(item, key) {
  return trackKey(item) === key || `cid-${item._cid}` === key;
}

// The title of a feed in the lists. A feed with no title shows its path.
function displayTitle(feed) {
  return feed.title || feed.path || 'Untitled feed';
}

// Is the page of an outline entry the page that the view shows now? Only a
// feed view has a page. The outline view marks no entry.
function isCurrentPage(view, page, track) {
  if (view.name !== 'feed') return false;
  if ((view.page || 'album') !== page) return false;
  return page !== 'track' || view.track === track;
}

// The entries of the outline of an open feed (ADR 0039 item 6). The head of
// the outline opens the first page of the feed, so no entry is for that page
// (ADR 0069). At narrow width there is no "Releases" entry, because Back
// opens Releases (ADR 0039 item 11a).
function outlineItems(feed, view, model, narrow, feeds) {
  // A feed opened from a level goes back to that level (ADR 0088 item 6).
  const from = view.from;
  const items = narrow ? [] : [from
    ? { id: 'releases', label: titleOf(feeds, from.group), target: upView([...from.trail, from.group], feed.id) }
    : { id: 'releases', label: 'Releases' }];
  const page = (id, label, pageName, track) => ({
    id,
    label,
    page: pageName,
    ...(track === undefined ? {} : { track }),
    current: isCurrentPage(view, pageName, track),
  });
  if (isPublisher(feed)) {
    // A publisher feed has two pages and no tracks (ADR 0041 item 3).
    items.push(page('settings', 'Feed settings', 'settings'));
    items.push(page('files', 'Files', 'files'));
    return items;
  }
  if (isEditable(feed) && isPlaylist(feed)) {
    // A playlist has no track pages and no import (ADR 0053 item 4).
    items.push(page('tracks', 'Tracks', 'tracks'));
    items.push(page('recipients', 'Value routes', 'recipients'));
    items.push(page('settings', 'Feed settings', 'settings'));
    items.push(page('files', 'Files', 'files'));
    return items;
  }
  if (!isEditable(feed)) {
    // A feed that the editor cannot open shows one page.
    return items;
  }
  if (model) {
    model.items.forEach((item, i) => {
      const key = trackKey(item);
      items.push({
        id: `track:${key}`,
        page: 'track',
        track: key,
        number: i + 1,
        label: item.title || 'Untitled track',
        current: isCurrentPage(view, 'track', key),
      });
    });
    items.push({ id: 'add-tracks', label: 'Add tracks', action: 'add-tracks' });
    // The live shows come after the tracks (ADR 0054 item 3). An entry of
    // a live show uses `track` for the key of the live item.
    items.push({ id: 'live-shows', label: 'Live shows', heading: true });
    (model.live_items || []).forEach((live) => {
      const key = liveKey(live);
      items.push({
        id: `live:${key}`,
        page: 'live',
        track: key,
        label: live.item.title || 'Untitled show',
        status: statusLabel(live.status),
        current: view.name === 'feed' && view.page === 'live' && isLiveItem(live, view.track),
      });
    });
    items.push({ id: 'add-live', label: 'Add a live show', action: 'add-live' });
    items.push(page('recipients', 'Value routes', 'recipients'));
    items.push(page('credits', 'Credits and links', 'credits'));
    items.push(page('settings', 'Feed settings', 'settings'));
  }
  items.push(page('files', 'Files', 'files'));
  return items;
}

// Is the view a feed, or the outline of a feed? Both open one feed.
export function isFeedView(view) {
  return view.name === 'feed' || view.name === 'outline';
}

// The left panel (ADR 0039 items 1, 5 and 6). With no open feed it gives the
// destinations. With an open feed, or the outline view, it gives the outline
// of that feed. `model` is the typed model of the draft of the open feed, or
// null before it loads. `narrow` is true at 760 px or less. `vaultLocked` is
// true on a server with a locked vault: the row "Unlock Studio" then takes
// the place of "Lock Studio" (ADR 0039 item 1).
export function leftPanel({ view, feeds, model, vaultOpen, vaultLocked = false, narrow = false }) {
  const bottom = [
    { id: 'hosts', label: 'Hosts', icon: 'hard-drive', current: view.name === 'hosts' },
    { id: 'settings', label: 'Settings', icon: 'settings', current: view.name === 'settings' },
  ];
  if (vaultOpen) bottom.push({ id: 'lock', label: 'Lock Studio', icon: 'lock', current: false });
  else if (vaultLocked) bottom.push({ id: 'unlock', label: 'Unlock Studio', icon: 'lock-open', current: false });
  const feed = isFeedView(view) && Array.isArray(feeds)
    ? feeds.find((f) => f.id === view.id)
    : undefined;
  const owner = levelFeed(view, feeds);
  if (!feed && owner) return { ...feedLevel(owner, view, feeds), bottom };
  if (!feed) {
    const all = view.name === 'releases' && view.group === undefined;
    return {
      kind: 'destinations',
      top: [
        // The Add feeds page belongs to Releases (ADR 0072 item 5). On that
        // page the row is current, and a click on it still opens Releases.
        {
          id: 'releases',
          label: 'Releases',
          icon: 'library',
          current: all || view.name === 'add-feed',
          opens: !all,
        },
        ...groupDestinations(feeds, view),
      ],
      bottom,
    };
  }
  return {
    kind: 'outline',
    feed: {
      id: feed.id,
      title: displayTitle(feed),
      image: feedCover(feed),
      glow: kindGlow(feedKind(feed)),
      typeLabel: feedKind(feed).label,
      // The head opens the first page (ADR 0069).
      page: 'album',
      current: isCurrentPage(view, 'album'),
      label: `${displayTitle(feed)}, ${feedKind(feed).label}`,
    },
    items: outlineItems(feed, view, model, narrow, feeds),
    bottom,
  };
}

// ---- The levels of the hierarchy (ADR 0088) ----

// The role headings of the levels, in their sequence (ADR 0088 item 4). A
// feed is below the heading of its first role, so a feed with the roles
// artist and label is one feed in Artists.
const HEADINGS = [
  ['hosting', 'Hosting'], ['label', 'Labels'], ['artist', 'Artists'], ['producer', 'Producers'],
  ['host', 'Hosts'], ['author', 'Authors'], ['network', 'Networks'], ['sponsor', 'Sponsors'],
];

function headingIndex(feed) {
  const roles = feed.publisher_roles || [];
  const first = ROLES.find((role) => roles.includes(role));
  return first ? HEADINGS.findIndex(([role]) => role === first) : HEADINGS.length;
}

// The feeds of `list` under their role headings, A to Z in each heading.
// `item` makes the row of each feed.
function headed(list, item) {
  const sections = [];
  for (const feed of list.slice().sort((a, b) => headingIndex(a) - headingIndex(b) || compareAz(a, b))) {
    const index = headingIndex(feed);
    // A feed with no role is in "No role" (ADR 0090 item 2).
    const heading = index < HEADINGS.length ? HEADINGS[index][1] : 'No role';
    if (sections.length === 0 || sections[sections.length - 1].heading !== heading) {
      sections.push({ heading, items: [] });
    }
    sections[sections.length - 1].items.push(item(feed));
  }
  return sections;
}

// The art of a publisher feed (musicindex ADR 0021 R-ART-3 item 2): its own
// cover, or else the different covers of its releases for the mosaic. An
// empty list gives the logo shape.
export function publisherArt(feed, feeds) {
  const own = feedCover(feed);
  if (own) return [own];
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  const covers = (feed.release_ids || []).map((id) => byId.get(id)).filter(Boolean).map(feedCover).filter(Boolean);
  return [...new Set(covers)];
}

// The cells of a mosaic of 2 by 2, as musicindex.org search.html gives them.
// One image is not a mosaic.
export function mosaicCells(images) {
  if (images.length >= 4) return images.slice(0, 4);
  if (images.length === 3) return [images[0], images[1], images[2], images[0]];
  if (images.length === 2) return [images[0], images[1], images[1], images[0]];
  return images;
}

// The publisher feed of a level view, or null. A level view is Releases
// with the ID of a publisher feed as its group.
function levelFeed(view, feeds) {
  if (view.name !== 'releases' || typeof view.group !== 'number' || !Array.isArray(feeds)) return null;
  return feeds.find((feed) => feed.id === view.group && isPublisher(feed)) || null;
}

// The view that "‹" opens from a level, or from a feed opened from a level.
// `trail` is the list of the levels above, from the top. With no trail it
// is Releases (ADR 0088 item 6).
function upView(trail, marked) {
  if (!trail || trail.length === 0) return { name: 'releases', marked };
  return { name: 'releases', group: trail[trail.length - 1], trail: trail.slice(0, -1), marked };
}

function titleOf(feeds, id) {
  const feed = (feeds || []).find((f) => f.id === id);
  return feed ? displayTitle(feed) : 'Releases';
}

// The left panel of a level (ADR 0088 item 5): "‹" to the level above, the
// head of the feed, the feeds directly below it under their role headings,
// then its releases in feed order.
function feedLevel(owner, view, feeds) {
  const trail = Array.isArray(view.trail) ? view.trail : [];
  const byId = new Map(feeds.map((feed) => [feed.id, feed]));
  const from = { group: owner.id, trail };
  const children = (owner.child_ids || []).map((id) => byId.get(id)).filter(Boolean);
  return {
    kind: 'level',
    back: { label: trail.length > 0 ? titleOf(feeds, trail[trail.length - 1]) : 'Releases', view: upView(trail, owner.id) },
    head: {
      id: owner.id,
      title: displayTitle(owner),
      art: publisherArt(owner, feeds),
      glow: kindGlow(feedKind(owner)),
      typeLabel: feedKind(owner).label,
      from,
    },
    sections: headed(children, (feed) => ({
      id: feed.id,
      label: displayTitle(feed),
      dot: feedKind(feed).dot,
      glow: kindGlow(feedKind(feed)),
      art: publisherArt(feed, feeds),
      count: (feed.release_ids || []).length,
      view: { name: 'releases', group: feed.id, trail: [...trail, owner.id] },
    })),
    releases: (owner.release_ids || []).map((id) => byId.get(id)).filter(Boolean).map((feed) => ({
      id: feed.id,
      title: displayTitle(feed),
      from,
    })),
  };
}

// The destinations below Releases (ADR 0085 items 1 and 3, ADR 0088 item
// 3): each top feed, with no feed above it, under its role heading, then
// "Not linked" when a release has no link. `count` is the number of
// releases of the destination. The first row of a heading has `heading`.
export function groupDestinations(feeds, view) {
  const list = Array.isArray(feeds) ? feeds : [];
  const shown = (group) => view.name === 'releases' && view.group === group;
  const top = list.filter((feed) => isPublisher(feed) && (feed.parent_ids || []).length === 0);
  const groups = headed(top, (feed) => ({
    id: `group-${feed.id}`,
    group: feed.id,
    label: displayTitle(feed),
    dot: feedKind(feed).dot,
    glow: kindGlow(feedKind(feed)),
    art: publisherArt(feed, list),
    count: (feed.release_ids || []).length,
    current: shown(feed.id),
    opens: !shown(feed.id),
  })).flatMap((section) => section.items.map((item, i) => (i === 0 ? { ...item, heading: section.heading } : item)));
  const unlinked = unlinkedReleases(list).length;
  if (unlinked > 0) {
    groups.push({
      id: 'group-none', group: 'none', label: 'Not linked', dot: null,
      count: unlinked, current: shown('none'), opens: !shown('none'),
    });
  }
  // The feeds that the editor does not open (ADR 0090 item 2).
  const other = otherFeeds(list).length;
  if (other > 0) {
    groups.push({
      id: 'group-other', group: 'other', label: 'Other feeds', dot: null,
      count: other, current: shown('other'), opens: !shown('other'),
    });
  }
  return groups;
}

// The feeds that are not a publisher feed and not a release (ADR 0090
// item 2).
function otherFeeds(feeds) {
  return feeds.filter((feed) => !isPublisher(feed) && !isRelease(feed));
}

// The IDs of each feed above and below `owner`, from the levels that the
// core gives (ADR 0088 item 8).
function relatedLevels(feeds, owner) {
  const byId = new Map(feeds.map((feed) => [feed.id, feed]));
  const found = new Set();
  const walk = (feed, key) => {
    for (const id of (feed && feed[key]) || []) {
      if (found.has(id)) continue;
      found.add(id);
      walk(byId.get(id), key);
    }
  };
  walk(owner, 'parent_ids');
  walk(owner, 'child_ids');
  return found;
}

// The releases with no artist or label in the collection (ADR 0085 item 3).
function unlinkedReleases(feeds) {
  return feeds.filter((feed) => isRelease(feed) && (feed.publisher_ids || []).length === 0);
}

// The cover of a feed row. The row has the `itunes:image` as `image` and
// the address of the RSS image as `rss_image_url`. The cover rule is in
// `cover.js` (ADR 0060). A feed with no cover gives `null`.
export function feedCover(feed) {
  const rssImage = feed.rss_image_url ? { url: feed.rss_image_url } : null;
  return coverOf({ image: feed.image ?? null, rss_image: rssImage }) || null;
}

// The label of each draft state in the Releases list (ADR 0039 item 2).
const DRAFT_LABELS = {
  new: 'New, not written',
  changed: 'Saved, not written',
  unchanged: 'No changes',
  none: 'No changes',
};

// The name of the host of a feed. A feed with no connector is read-only.
function hostOf(feed, connectors) {
  if (!feed.connector_id) return 'Read-only';
  const connector = (connectors || []).find((c) => c.id === feed.connector_id);
  return connector ? connector.name : 'Unknown host';
}

// The letter of a row in the A to Z sort. A title that does not start with
// A to Z gives "#".
function letterOf(title) {
  return /^[A-Z]/i.test(title) ? title[0].toUpperCase() : '#';
}

// Newest first. The feed that joined the collection last comes first. Two
// feeds that joined at the same time keep the greater ID first.
function compareNewest(a, b) {
  return (b.added_at ?? 0) - (a.added_at ?? 0) || Number(b.id) - Number(a.id);
}

// A to Z. A title with a letter from A to Z comes before a title with any
// other first character, so that "#" comes after Z.
function compareAz(a, b) {
  const bucket = (feed) => (letterOf(displayTitle(feed)) === '#' ? 1 : 0);
  return bucket(a) - bucket(b)
    || displayTitle(a).localeCompare(displayTitle(b), 'en', { sensitivity: 'base' })
    || Number(a.id) - Number(b.id);
}

// The first letter of each group of rows, in sequence (ADR 0039 item 2a).
function letterGroups(rows) {
  const groups = [];
  for (const row of rows) {
    const letter = letterOf(row.title);
    if (groups.length === 0 || groups[groups.length - 1].letter !== letter) {
      groups.push({ letter, firstId: row.id });
    }
  }
  return groups;
}

// The sorts of each list (ADR 0039 item 2a, ADR 0085 item 4). The first
// sort is the default.
const ALL_SORTS = ['newest', 'az'];
const GROUP_SORTS = ['order', 'newest', 'az'];

// The Releases list (ADR 0039 items 2 and 2a). `marked` is the ID of the feed
// that the last Back selection left marked. With no `group` it gives each
// album and playlist (ADR 0085 item 6). With the ID of a feed with roles as
// `group`, it gives the releases of that feed, and `head` gives the feed. The
// group 'none' gives the releases with no link (ADR 0085 items 3 and 4).
export function releasesList({ feeds, connectors, sort, view, marked, group }) {
  const list = Array.isArray(feeds) ? feeds : [];
  const owner = group === undefined || group === 'none'
    ? null
    : list.find((feed) => feed.id === group && isPublisher(feed)) || null;
  let chosen;
  let head = null;
  if (owner) {
    const byId = new Map(list.map((feed) => [feed.id, feed]));
    chosen = (owner.release_ids || []).map((id) => byId.get(id)).filter(Boolean);
    head = {
      id: owner.id,
      title: displayTitle(owner),
      typeLabel: feedKind(owner).label,
      dot: feedKind(owner).dot,
    };
  } else if (group === 'none') {
    chosen = unlinkedReleases(list);
    head = { id: null, title: 'Not linked', typeLabel: null, dot: null, note: 'These releases have no link to an artist or a label.' };
  } else if (group === 'other') {
    chosen = otherFeeds(list);
    head = { id: null, title: 'Other feeds', typeLabel: null, dot: null, note: 'Studio edits albums, playlists, and artist and label feeds. It shows only the facts of these feeds.' };
  } else {
    chosen = list.filter(isRelease);
  }
  const sorts = owner ? GROUP_SORTS : ALL_SORTS;
  const sortKey = sorts.includes(sort) ? sort : sorts[0];
  const viewKey = view === 'tiles' ? 'tiles' : 'rows';
  const sorted = sortKey === 'order'
    ? chosen.slice()
    : chosen.slice().sort(sortKey === 'az' ? compareAz : compareNewest);
  const titleOf = (id) => {
    const feed = list.find((f) => f.id === id);
    return feed ? displayTitle(feed) : null;
  };
  // A release of two feeds names the other feeds (ADR 0085 item 5). A feed
  // above or below the owner is in the tree already, so the note leaves it
  // out (ADR 0088 item 8).
  const related = owner ? relatedLevels(list, owner) : new Set();
  const noteOf = (feed) => {
    if (!owner) return null;
    const others = (feed.publisher_ids || []).filter((id) => id !== owner.id && !related.has(id)).map(titleOf).filter(Boolean);
    return others.length > 0 ? `Also on ${others.join(', ')}` : null;
  };
  const rows = sorted.map((feed) => ({
    id: feed.id,
    title: displayTitle(feed),
    typeLabel: feedKind(feed).label,
    glow: kindGlow(feedKind(feed)),
    host: hostOf(feed, connectors),
    image: feedCover(feed),
    draftLabel: DRAFT_LABELS[feed.draft_state] || 'No changes',
    // A feed that apps do not list (ADR 0063).
    hidden: feed.hidden === true,
    marked: feed.id === marked,
    note: noteOf(feed),
  }));
  return {
    sort: sortKey,
    sorts,
    view: viewKey,
    head,
    rows,
    letters: sortKey === 'az' ? letterGroups(rows) : null,
  };
}

// The sort and the view of the Releases list from a query text, such as
// "sort=az". A value that is missing or not known gives the default.
export function listQuery(search) {
  const params = new URLSearchParams(typeof search === 'string' ? search : '');
  return {
    sort: params.get('sort') === 'az' ? 'az' : 'newest',
    view: params.get('view') === 'tiles' ? 'tiles' : 'rows',
  };
}

// The query text of a sort and a view. It holds only the values that are not
// the default. It is empty when both values are the default.
export function queryText({ sort, view }) {
  const params = new URLSearchParams();
  if (sort === 'az') params.set('sort', 'az');
  if (view === 'tiles') params.set('view', 'tiles');
  return params.toString();
}

// ---- Navigation (musicindex ADR 0027 R-NAV-2) ----
// The state is { entries, index }. Each function gives a new state and does
// not change its argument.

// The state with one entry, the view at the start.
export function startNav(view) {
  return { entries: [view], index: 0 };
}

// A new view. It drops each entry after the current one.
export function pushNav(nav, view) {
  const entries = [...nav.entries.slice(0, nav.index + 1), view];
  return { entries, index: entries.length - 1 };
}

// Back. From a feed or its outline, the Releases entry before it keeps its
// scroll and gets the feed as its marked row. From a feed or an outline at
// index 0, a Releases entry with the marked feed goes before it.
export function backNav(nav) {
  const { entries, index } = nav;
  const current = entries[index];
  if (current && isFeedView(current)) {
    if (index === 0) {
      return { entries: [{ name: 'releases', marked: current.id }, ...entries], index: 0 };
    }
    const next = entries.slice();
    const before = entries[index - 1];
    if (before.name === 'releases') next[index - 1] = { ...before, marked: current.id };
    return { entries: next, index: index - 1 };
  }
  return { entries: entries.slice(), index: Math.max(0, index - 1) };
}

// The view that the link "Releases" opens (ADR 0069). The nearest Releases
// entry at or before the index gives its scroll. The function does not
// change `nav`.
export function releasesView(nav, feedId) {
  const view = { name: 'releases', marked: feedId };
  for (let i = nav.index; i >= 0; i -= 1) {
    const entry = nav.entries[i];
    if (entry && entry.name === 'releases') {
      if (entry.scroll !== undefined) view.scroll = entry.scroll;
      break;
    }
  }
  return view;
}

// Forward. It moves to the next entry, when an entry is after the current one.
export function forwardNav(nav) {
  return { entries: nav.entries.slice(), index: canForward(nav) ? nav.index + 1 : nav.index };
}

// Back is possible above index 0, and from a feed or an outline at index 0.
// The root page has no Back.
export function canBack(nav) {
  return nav.index > 0 || isFeedView(nav.entries[nav.index]);
}

// Forward is possible when an entry is after the current one.
export function canForward(nav) {
  return nav.index < nav.entries.length - 1;
}

// The scroll position of the current entry.
export function setScroll(nav, y) {
  const entries = nav.entries.slice();
  entries[nav.index] = { ...entries[nav.index], scroll: y };
  return { entries, index: nav.index };
}

// ---- Narrow width (musicindex ADR 0030 R-SHELL-2, ADR 0039 items 10 to 11a) ----

// The stack that a narrow window shows for one view. The entries go from the
// root page to the view, and the view is the last entry.
export function narrowStack(view) {
  const root = { name: 'root' };
  const releases = { name: 'releases' };
  switch (view.name) {
    case 'root': return [root];
    // A destination below Releases is a page of the root (ADR 0085 item 7).
    // Each level above a level is a page below it (ADR 0088 item 7).
    case 'releases': return view.group === undefined ? [root, releases] : [root, ...levelPages(view.trail), view];
    case 'add-feed': return [root, releases, view];
    case 'outline': return view.from ? [root, ...levelPages([...view.from.trail, view.from.group]), view] : [root, releases, view];
    case 'feed': return view.from
      ? [root, ...levelPages([...view.from.trail, view.from.group]), { name: 'outline', id: view.id, from: view.from }, view]
      : [root, releases, { name: 'outline', id: view.id }, view];
    default: return [root, view];
  }
}

// The level pages of a trail, from the top (ADR 0088 item 7).
function levelPages(trail) {
  const list = Array.isArray(trail) ? trail : [];
  return list.map((group, i) => ({ name: 'releases', group, trail: list.slice(0, i) }));
}

// The view of a feed on Releases. At narrow width the selection opens the
// outline of the feed first (ADR 0039 item 11a).
export function openFeedView(id, narrow, from) {
  const view = narrow ? { name: 'outline', id } : { name: 'feed', id };
  return from ? { ...view, from } : view;
}

// The view at a wide width. The outline has no wide view, so it becomes the
// feed, and the root page becomes Releases.
export function wideView(view) {
  if (view.name === 'outline') return { name: 'feed', id: view.id };
  if (view.name === 'root') return { name: 'releases' };
  return view;
}

// The navigation state for a new width of the window. A narrow window gets
// the stack of the view. A wide window gets the view alone.
export function widthNav(view, narrow) {
  if (!narrow) return startNav(wideView(view));
  const entries = narrowStack(view);
  return { entries, index: entries.length - 1 };
}

// The navigation state that fits a width. A state that already has the form
// of the width is kept, so that a reload keeps its history. Any other state
// becomes `widthNav` of its current view.
export function navAtWidth(nav, narrow) {
  const narrowForm = nav.entries[0].name === 'root';
  const wideForm = nav.entries.every((e) => e.name !== 'root' && e.name !== 'outline');
  if (narrow ? narrowForm : wideForm) return nav;
  return widthNav(nav.entries[nav.index], narrow);
}
