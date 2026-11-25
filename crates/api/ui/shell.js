// The shell models (ADR 0042). This file imports only the live item model.
// It holds the logic
// of the left panel, of the Releases list and of the navigation. The
// components in `app.js` and `editor.js` use these functions. The Node tests
// in `crates/api/ui-tests/` examine them.
import { liveKey, isLiveItem, statusLabel } from './live.js';
import { coverOf } from './cover.js';
import { tabLabel } from './feedcheck.js';

// The nine roles of a publisher feed, in the sequence of musicindex ADR
// 0081 R-CURATOR-1 item 1 (ADR 0083 item 1).
export const ROLES = ['artist', 'label', 'hosting', 'curator', 'producer', 'host', 'author', 'network', 'sponsor'];

// The first-class roles, which the Role menu and the form of a new
// publisher feed offer (musicindex ADR 0036 R-ROLE-2, ADR 0083 items 2
// and 3).
export const FIRST_CLASS_ROLES = ['artist', 'label', 'hosting'];

// The word of a role, for example "Artist".
export const roleWord = (role) => role === 'hosting' ? 'Publisher' : role.charAt(0).toUpperCase() + role.slice(1);

// The Role hint uses the states from the feed list (ADR 0253 item 18).
export function roleHint(feed, readOnly = false) {
  const roles = feed.publisher_roles || [];
  if (roles.length === 0) return null;
  if (feed.roles_linked) return 'The feed and its releases give the same role.';
  if (feed.roles_confirmed) return null;
  return `Studio found ${roles.length === 1 ? 'this role' : 'these roles'} in the feed: ${roles.map(roleWord).join(', ')}.${readOnly ? '' : ' Select the roles to confirm them.'}`;
}

// The folds after a change of the Role field: each role that some releases
// keep on a different page, with the titles of those releases (ADR 0257
// item 10a). `kept` is `kept_role_releases` of the response.
export function keptRoleFolds(kept) {
  return ROLES.filter((role) => (kept?.[role] || []).length > 0).map((role) => {
    const releases = kept[role];
    const page = `${roleWord(role).toLowerCase()} page`;
    return {
      summary: releases.length === 1 ? `1 release keeps its ${page}.` : `${releases.length} releases keep their ${page}.`,
      names: releases.map((release) => release.title || 'Untitled release'),
    };
  });
}

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

// The check boxes show the roles of the page in the sequence of ROLES.
export function linkRoleChoices(pageRoles, rel) {
  const words = String(rel || '').toLowerCase().split(/\s+/);
  return ROLES.filter((role) => (pageRoles || []).includes(role))
    .map((role) => ({ role, label: roleWord(role), checked: words.includes(role) }));
}

// The kind of a feed of the collection or of a discovery. The user confirms
// a set of roles, or the core finds the first role on the two sides of a
// link (ADR 0082 item 5, ADR 0087 item 2).
export function feedKind(feed) {
  const hasLinks = ['release_ids', 'parent_ids', 'child_ids'].some((key) => (feed[key] || []).length > 0);
  const confirmed = feed.roles_linked === true || (feed.roles_confirmed === true && !hasLinks);
  const kind = mediumKind(feed.medium, feed.publisher_roles, confirmed);
  if ((feed.medium || '').toLowerCase() !== 'music') return kind;
  const labels = { release: 'Release', album: 'Album', ep: 'EP', single: 'Single' };
  return { ...kind, label: labels[feed.kind_shown] || 'Album' };
}

export function trackCountText(count) {
  if (typeof count !== 'number') return null;
  return `${count} ${count === 1 ? 'track' : 'tracks'}`;
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

// Is this feed a publisher feed (ADR 0255 item 1)?
export function isPublisher(feed) {
  return (feed.medium || '').toLowerCase() === 'publisher';
}

// Is this feed a playlist, a musicL feed (ADR 0053 item 1)?
export function isPlaylist(feed) {
  return (feed.medium || '').toLowerCase() === 'musicl';
}

// Is this feed an album, a publisher feed or a playlist that the editor can
// open? A feed with no connector opens too (ADR 0029 item 4, ADR 0255 item 3,
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

// The notice for a correctable medium problem (ADR 0250 items 4, 5 and 9).
export function mediumProblemView(problem, hasHost) {
  const title = 'Studio does not know the type of this feed';
  let text;
  if (problem.kind === 'no_tag') text = 'This feed has no <podcast:medium> tag.';
  else text = problem.line == null
    ? `This feed has ${problem.tag}. Studio reads only <podcast:medium>.`
    : `Line ${problem.line} has ${problem.tag}. Studio reads only <podcast:medium>.`;
  return {
    title,
    text,
    menu: hasHost,
    start: problem.suggested || '',
    hostLine: hasHost ? null : 'Connect the folder that holds this file in File hosts to correct it.',
  };
}

// The cause shown after a found feed type (ADR 0250 item 10).
export function mediumProblemCause(problem) {
  return problem.kind === 'no_tag' ? 'no medium tag' : `misspelled tag ${problem.tag}`;
}

// Is this feed read-only? A feed with no connector has no host that Studio
// can write to (ADR 0067 item 1).
export function isReadOnly(feed) {
  return !feed.connector_id;
}

// The second line of the feed move row: what the address from before gets
// (ADR 0204 items 16 to 18).
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

// The entries of the outline of an open release or playlist (ADR 0239 item
// 1). A publisher feed has no outline (ADR 0254 item 7). The first row is
// "Catalog". It goes to the page that opened the feed, or else it shows the
// tree again (ADR 0254 items 7 and 43). The head of the outline opens the
// first page of the feed, so no entry is for that page (ADR 0069). At narrow
// width there is no "Catalog" row, because Back opens the page before it
// (ADR 0258 item 14).
function outlineItems(feed, view, model, narrow) {
  const above = view.from ? openFeedView(view.from.group, narrow, fromPath(pathOfFrom(view.from).slice(0, -1))) : null;
  const items = narrow ? [] : [{ id: 'catalog', label: 'Catalog', ...(above ? { target: above } : {}) }];
  // A playlist has no track pages, so its tracks are a tab (ADR 0053 item 4).
  if (!isEditable(feed) || isPlaylist(feed) || !model) return items;
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
  return items;
}

// The tabs of the page of a feed (ADR 0239 items 2 to 6). A track page has no
// tabs. The page of one live show keeps the tab "Live Shows". The labels use
// title case (musicindex ADR 0085 R-CASE-1). The tab Check comes before
// Settings, and its label has the count of the findings (ADR 0174 item 17).
export function pageTabs(feed, view, checkCount = 0) {
  if (!isEditable(feed) || view.name !== 'feed') return [];
  const page = view.page || 'album';
  if (page === 'track') return [];
  // The first tab of a release is "Release". Its first section names the
  // kind (ADR 0225 item 1).
  const release = (feed.medium || '').toLowerCase() === 'music';
  const first = { page: 'album', label: release ? 'Release' : feedKind(feed).label };
  const routes = { page: 'recipients', label: 'Value Routes' };
  const credits = { page: 'credits', label: 'Credits' };
  const files = { page: 'files', label: 'Files' };
  const check = { page: 'check', label: tabLabel(checkCount) };
  const settings = { page: 'settings', label: 'Settings' };
  const list = isPublisher(feed)
    ? [first, routes, credits, files, check, settings]
    : isPlaylist(feed)
      ? [first, { page: 'tracks', label: 'Tracks' }, routes, files, check, settings]
      : [first, { page: 'lives', label: 'Live Shows' }, routes, credits, files, check, settings];
  const shown = page === 'live' ? 'lives' : page;
  return list.map((tab) => ({ ...tab, current: tab.page === shown }));
}

// Do the tabs fit as a segmented control in their column (ADR 0197 item 1)?
// Each segment has the width of the longest label, plus 4 px at each side
// (musicindex ADR 0051 R-SEG-1 items 4 and 5). `available` is the width of the
// tray. `labelWidths` are the widths of the labels on one line, with the mark
// of a tab that is not saved. `gap` is the space between two segments.
// `padding` is the space between the tray and a segment, at one side.
export function tabsFit({ available, labelWidths, gap, padding }) {
  if (labelWidths.length === 0) return true;
  const segment = Math.max(...labelWidths) + 8;
  const needed = labelWidths.length * segment + (labelWidths.length - 1) * gap + 2 * padding;
  return needed <= available;
}

// Is the view a feed, or the outline of a feed? Both open one feed.
export function isFeedView(view) {
  return view.name === 'feed' || view.name === 'outline';
}

// The views of the People area. People is current on each of them (ADR 0203
// item 1), except a change set of a move, which has the area `hosts` (ADR 0148
// item 17).
export const PEOPLE_VIEWS = ['people', 'person', 'check', 'wallet', 'change-set'];

// Is the move from `current` to `target` only a change of the tab of one
// page with tabs in its view? A tab change keeps the draft and does not ask
// the leave question (ADR 0166 item 10, ADR 0226 item 12).
export function isTabChange(current, target) {
  if (!current || !target || target.name !== current.name || target.id !== current.id) return false;
  return ['settings', 'person'].includes(current.name) && (current.tab ?? null) !== (target.tab ?? null);
}

const inHosts = (view) => view.name === 'hosts' || (view.name === 'change-set' && view.area === 'hosts');
const inReleases = (view) => view.name === 'change-set' && view.area === 'releases';

// The left panel (ADR 0258 items 1, 6 and 7, ADR 0254 items 2 and 7). With
// no open feed, or with the page of a publisher feed, it gives the
// destinations and the Catalog tree. With an open release or playlist, or the
// outline view of one, it gives the outline of that feed. `model` is the
// typed model of the draft of the open feed, or null before it loads. `open`
// is the set of the keys of the open tree rows. `narrow` is true at 760 px or
// less. `vaultLocked` is true on a server with a locked vault: the row
// "Unlock Studio" then takes the place of "Lock Studio" (ADR 0258 item 1).
export function leftPanel({ view, feeds, model, vaultOpen, vaultLocked = false, narrow = false, collection = null,
  serverMode = false, collectionCount = 0, open = [] }) {
  const bottom = [
    // The People destination holds the page of a person, the page of a
    // wallet and the review of a change set (ADR 0203 items 36 and 79).
    {
      id: 'people', label: 'People', icon: 'users',
      current: PEOPLE_VIEWS.includes(view.name) && !inHosts(view) && !inReleases(view),
      // A click on the row opens the list from a page of the area.
      opens: view.name !== 'people',
    },
    { id: 'hosts', label: 'File hosts', icon: 'hard-drive', current: inHosts(view), opens: view.name !== 'hosts' },
    { id: 'settings', label: 'Settings', icon: 'settings', current: view.name === 'settings' },
  ];
  // ADR 0242 items 1 and 6: one collection row on a desktop with a list of
  // collections. With two or more, the row is the accent mark of the open
  // collection. With one, the row is "Switch Collection…".
  if (!serverMode && collectionCount >= 2 && collection) bottom.push({ id: 'collection', label: collection, icon: 'folder', mark: true, current: false });
  else if (!serverMode && collectionCount >= 1) bottom.push({ id: 'collection', label: 'Switch Collection…', icon: 'folder-open', mark: false, current: false });
  if (vaultOpen) bottom.push({ id: 'lock', label: 'Lock Studio', icon: 'lock', current: false });
  else if (vaultLocked) bottom.push({ id: 'unlock', label: 'Unlock Studio', icon: 'lock-open', current: false });
  // ADR 0241 item 1: put Sign Out after the vault row.
  if (serverMode) bottom.push({ id: 'signout', label: 'Sign Out', icon: 'user', current: false });
  const feed = isFeedView(view) && Array.isArray(feeds)
    ? feeds.find((f) => f.id === view.id)
    : undefined;
  if (!feed || isPublisher(feed)) {
    const all = view.name === 'overview' || inReleases(view);
    return {
      kind: 'destinations',
      top: [
        // Overview is the first row, with the icon of Releases (ADR 0263
        // item 1). The Add feeds page belongs to it (ADR 0072 item 5). On
        // that page the row is current, and a click on it still opens
        // Overview.
        {
          id: 'overview',
          label: 'Overview',
          icon: 'library',
          current: all,
          opens: !all,
        },
        ...groupDestinations(feeds, view, open),
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
      dot: feedKind(feed).dot,
      // The head opens the first page (ADR 0069). It stays selected on each
      // tab of the feed (ADR 0239 item 3).
      page: 'album',
      first: isCurrentPage(view, 'album'),
      current: view.name === 'feed' && view.page !== 'track',
      label: `${displayTitle(feed)}, ${feedKind(feed).label}`,
    },
    items: outlineItems(feed, view, model, narrow),
    bottom,
  };
}

// ---- The Catalog tree (ADR 0254) ----

// The four role headings follow ADR 0253 item 16. A feed with no level role
// goes below "No role" (ADR 0254 item 3).
const HEADINGS = [
  ['hosting', 'Publishers'], ['label', 'Labels'], ['curator', 'Curators'], ['artist', 'Artists'],
];

// The rank of a feed in the headings: 0 is the highest. A feed with no level
// role has the rank after the last heading (ADR 0253 item 15).
export function headingIndex(feed) {
  const roles = feed.publisher_roles || [];
  const first = ROLES.find((role) => roles.includes(role));
  const index = HEADINGS.findIndex(([role]) => role === first);
  return index < 0 ? HEADINGS.length : index;
}

// The heading of a feed. A feed with no role is in "No role" (ADR 0090 item
// 2).
const headingOf = (feed) => {
  const index = headingIndex(feed);
  return index < HEADINGS.length ? HEADINGS[index][1] : 'No role';
};

// A copy of `list` in the rank of the headings, then A to Z (ADR 0254 items 3
// and 5). The function sorts rows for display. It does not decide which feed
// is below which feed (ADR 0065).
function ranked(list) {
  return list.slice().sort((a, b) => headingIndex(a) - headingIndex(b) || compareAz(a, b));
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

// The names in one line: "A", "A and B", "A, B and C".
function andNames(names) {
  return names.length < 2 ? names.join('') : `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}

// The IDs of the feeds above a row, from the top, as the `from` of a view
// gives them. `from` is `{ group, trail }`: `group` is the feed directly
// above, and `trail` holds the feeds above that one, from the top.
export function pathOfFrom(from) {
  return from ? [...(from.trail || []), from.group] : [];
}

// The `from` of a view for a list of the IDs above a row. An empty list gives
// no `from`.
export function fromPath(path) {
  return path.length === 0 ? undefined : { group: path[path.length - 1], trail: path.slice(0, -1) };
}

// The key of a tree row. A feed below two feeds has two rows, so the key is
// the IDs from the top to the row (ADR 0254 item 5).
export const rowKey = (path) => path.join('/');

// The path of the tree row of feed `id`, from the top, with `id` last: the
// row that the user selected when `from` names a row of the tree, or else the
// feed directly above with the highest rank, then A to Z (ADR 0254 item 12a).
export function rowPath(feeds, id, from = undefined) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const given = pathOfFrom(from);
  const below = (upper, lower) => (byId.get(upper)?.child_ids || []).includes(lower);
  const holds = given.length > 0 && given.every((up, i) => byId.has(up) && (i === 0
    ? (byId.get(up).parent_ids || []).length === 0
    : below(given[i - 1], up))) && below(given[given.length - 1], id);
  if (holds) return [...given, id];
  const path = [id];
  for (;;) {
    const feed = byId.get(path[0]);
    const above = ranked((feed?.parent_ids || []).map((up) => byId.get(up)).filter((up) => up && !path.includes(up.id)));
    if (above.length === 0) return path;
    path.unshift(above[0].id);
  }
}

// The path and the page above of the page of feed `id` (ADR 0254 items 12 and
// 12a). `names` holds each feed above, from the top, with the view that opens
// its page. `above` is the ID of the page above, or null. A feed with no feed
// above it has the text "Top level".
export function pagePath(feeds, id, from = undefined) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const up = rowPath(feeds, id, from).slice(0, -1);
  return {
    names: up.map((upId, i) => ({
      id: upId,
      title: byId.has(upId) ? displayTitle(byId.get(upId)) : 'Untitled feed',
      view: openFeedView(upId, false, fromPath(up.slice(0, i))),
    })),
    above: up.length > 0 ? up[up.length - 1] : null,
    top: up.length === 0,
  };
}

// ---- The page of a publisher feed (ADR 0254 items 11 to 17) ----

// The sorts of the group Releases, the default first (ADR 0254 item 13a).
export const PAGE_SORTS = ['order', 'newest', 'az'];

// The name of a sort of the group Releases. "Feed order" is the sequence of
// ADR 0254 item 4a.
export const pageSortLabel = (sort) => (sort === 'order' ? 'Feed order' : sortLabel(sort, false));

// The sort of the page in a view. A view with no sort, or with a sort that
// is not known, gives the default. The view is the navigation record of the
// session (ADR 0254 item 13a).
export const pageSort = (view) => (view && PAGE_SORTS.includes(view.sort) ? view.sort : PAGE_SORTS[0]);

// The state text of the feed of a page, as a release tile gives it (ADR 0254
// item 11a). A release with a draft does not change it.
export const draftLabel = (feed) => DRAFT_LABELS[feed.draft_state] || 'No changes';

// The place of a feed in music words (ADR 0254 item 11), for display only.
// The text reads `parent_ids` and `publisher_roles` of the feed list. It
// reads the parents of the labels above the feed, to name a publisher above a
// label. It does not decide which feed is below which feed (ADR 0065). An
// artist is "On {labels}" or "Independent", then "published by {names}" or
// "not published". A feed at another rank gives "Published by {names}" or
// "Not published". A feed with the role `hosting` and no publisher above
// it gives `null`.
export function placeText(feed, feeds) {
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  const has = (f, role) => (f.publisher_roles || []).includes(role);
  const above = (f) => (f.parent_ids || []).map((id) => byId.get(id)).filter(Boolean);
  const names = (list) => andNames(ranked(list).map(displayTitle));
  const parents = above(feed);
  const labels = parents.filter((p) => has(p, 'label'));
  const publishers = new Map();
  [...parents, ...labels.flatMap(above)].filter((p) => has(p, 'hosting')).forEach((p) => publishers.set(p.id, p));
  const by = publishers.size > 0 ? `published by ${names([...publishers.values()])}` : null;
  const rank = headingIndex(feed);
  if (rank === HEADINGS.findIndex(([role]) => role === 'artist')) {
    return `${labels.length > 0 ? `On ${names(labels)}` : 'Independent'}, ${by || 'not published'}`;
  }
  if (by) return `P${by.slice(1)}`;
  // A publisher with no publisher above it has no place line.
  return has(feed, 'hosting') ? null : 'Not published';
}

// The text of a count: "1 release", "3 pages below".
const countWords = (n, one, many) => `${n} ${n === 1 ? one : many}`;

// The groups of the page of publisher feed `feed` (ADR 0254 items 6b and 13
// to 16). `from` is the `from` of the view of the page. `sort` is a sort of
// `PAGE_SORTS`. The groups go from the highest rank of the level roles of the
// feed. A group of a different role heading shows only when it has rows, and
// it has no add row. Each group gives:
// - `id`, `heading` and `empty`: the title and the text of an empty group.
// - `add`: the add rows. A row has `id`, `label` and `ready`. A row with
//   `ready` false does nothing until task 254 (ADR 0254 items 18 to 21).
// - `rows`: the rows of the pages below or of the releases, each with the
//   `view` that a selection opens.
// - `sorts` and `sort`: only the group Releases has them (item 13a).
// The function lists the feed list fields as they are (ADR 0065).
export function publisherGroups(feed, feeds, { sort = PAGE_SORTS[0], from = undefined } = {}) {
  const list = Array.isArray(feeds) ? feeds : [];
  const byId = new Map(list.map((f) => [f.id, f]));
  const roles = feed.publisher_roles || [];
  const childFrom = fromPath(rowPath(list, feed.id, from));
  const pageRow = (page) => {
    const kind = feedKind(page);
    const { releases, pages } = levelCounts(page);
    return {
      id: page.id,
      title: displayTitle(page),
      typeLabel: kind.label,
      dot: kind.dot,
      countText: [
        releases === null ? null : countWords(releases, 'release', 'releases'),
        pages === null ? null : countWords(pages, 'page', 'pages'),
      ].filter(Boolean).join(' · '),
      view: openFeedView(page.id, false, childFrom),
    };
  };
  const kids = ranked((feed.child_ids || []).map((id) => byId.get(id)).filter((kid) => kid && isPublisher(kid) && kid.id !== feed.id));
  const claimed = new Set();
  const claim = (ranks) => kids.filter((kid) => !claimed.has(kid.id) && ranks.includes(headingIndex(kid)))
    .map((kid) => { claimed.add(kid.id); return pageRow(kid); });
  const rankOf = (role) => HEADINGS.findIndex(([r]) => r === role);
  const groups = [];
  const levels = HEADINGS.map(([role]) => role).filter((role) => roles.includes(role));
  // A label claims its artists before a publisher does (item 13).
  const claimedRows = {};
  if (levels.includes('label')) claimedRows.label = claim([rankOf('artist')]);
  if (levels.includes('hosting')) {
    claimedRows.hosting = claim(levels.includes('label') ? [rankOf('label')] : [rankOf('label'), rankOf('artist')]);
  }
  for (const role of levels) {
    if (role === 'hosting') {
      groups.push({
        id: 'pages', heading: 'Labels and artists published here', empty: 'Nothing published here yet.',
        add: [{ id: 'publish-label', label: '＋ Publish a label', ready: true }, { id: 'publish-artist', label: '＋ Publish an artist', ready: true }],
        rows: claimedRows.hosting,
      });
    } else if (role === 'label') {
      groups.push({
        id: 'artists', heading: 'Artists on this label', empty: 'No artists on this label yet.',
        add: [{ id: 'sign-artist', label: '＋ Sign an artist', ready: true }],
        rows: claimedRows.label,
      });
    }
  }
  const hasReleases = levels.some((role) => ['label', 'curator', 'artist'].includes(role)) || levels.length === 0;
  if (hasReleases) {
    const key = PAGE_SORTS.includes(sort) ? sort : PAGE_SORTS[0];
    const releases = (feed.release_ids || []).map((id) => byId.get(id)).filter(Boolean);
    const sorted = key === 'order' ? releases : releases.slice().sort(key === 'az' ? compareAz : compareNewest);
    const start = roles.includes('artist') ? releaseStart(feed, displayTitle(feed)) : null;
    groups.push({
      id: 'releases', heading: 'Releases', empty: 'No releases on this page.',
      add: roles.includes('artist') ? [{ id: 'add-release', label: '＋ Add release', ready: start !== null }] : [],
      sorts: PAGE_SORTS, sort: key,
      rows: sorted.map((release) => {
        const kind = feedKind(release);
        return {
          id: release.id,
          title: displayTitle(release),
          typeLabel: kind.label,
          dot: kind.dot,
          countText: trackCountText(release.track_count),
          view: openFeedView(release.id, false, childFrom),
        };
      }),
    });
  }
  // A child of a different role heading shows only when it has rows.
  const rest = kids.filter((kid) => !claimed.has(kid.id));
  for (let rank = 0; rank <= HEADINGS.length; rank += 1) {
    const rows = rest.filter((kid) => headingIndex(kid) === rank).map(pageRow);
    if (rows.length > 0) {
      groups.push({ id: `other-${rank}`, heading: rank < HEADINGS.length ? HEADINGS[rank][1] : 'No role', empty: '', add: [], rows });
    }
  }
  return groups;
}

// The set of open rows after a click on the button of the row with `key`.
export function toggleRow(open, key) {
  const next = new Set(open || []);
  if (!next.delete(key)) next.add(key);
  return next;
}

// The set of open rows with each row above a new feed open (ADR 0254 item
// 43). `from` is the `from` of the view of the new feed.
export function revealRows(open, from) {
  const next = new Set(open || []);
  const up = pathOfFrom(from);
  up.forEach((_, i) => next.add(rowKey(up.slice(0, i + 1))));
  return next;
}

// The start values of a new release of a publisher feed (ADR 0164 items 1
// and 2, ADR 0232 item 4). Only a writable artist page is the start parent.
// Other feeds give null.
export function releaseStart(feed, name) {
  if (!feed || !isPublisher(feed) || !feed.connector_id || !(feed.publisher_roles || []).includes('artist')) return null;
  return { author: name || '', publisher_id: feed.id };
}

// The `from` that the feeds below publisher feed `id` get when the page of
// `id` opened with `from`. A release row of a page and a new feed of the page
// go back to this page (ADR 0254 item 43).
export function pageLevel(id, from) {
  return { group: id, trail: from ? pathOfFrom(from) : [] };
}

// The page of a new feed keeps the tree row that opened its add form.
export function newFeedView(id, from) {
  return { name: 'feed', id, ...(from ? { from } : {}) };
}

// The numbers of a row (ADR 0254 item 6): the releases of the feed, the pages
// below it, or `null` for a number that the row does not show. A feed with
// only the role `hosting` shows the pages. A feed with no level role shows its
// releases.
function levelCounts(feed) {
  const roles = feed.publisher_roles || [];
  const releases = !roles.includes('hosting') || roles.some((role) => ['label', 'curator', 'artist'].includes(role))
    ? (feed.release_ids || []).length : null;
  const pages = roles.includes('hosting') ? (feed.child_ids || []).length : null;
  return { releases, pages };
}

// The rows of the panel below Releases (ADR 0254 items 2 to 8): the rows of
// the Catalog tree, then "No artist yet" when a release has no artist page,
// then "Other feeds". `open` is the set of the keys of the open rows. The top
// rows are the feeds with no feed above them, under their role headings. The
// first top row of a heading has `heading`. Each row gives:
// - `key` and `depth`: the place of the row in the tree.
// - `expandable` and `open`: the state of the button before the title.
// - `counts`, `count`, `countText` and `countLabel`: the releases of the feed,
//   the pages below it, or the two numbers (item 6).
// - `also`: the second line of a feed below two or more feeds (item 8).
// - `current` and `view`: the page that a selection opens.
// With `all` true, each row that has rows below it is open. Overview uses
// this to read the whole sequence of the tree (ADR 0264 item 1).
export function groupDestinations(feeds, view, open = [], { all = false } = {}) {
  const list = Array.isArray(feeds) ? feeds : [];
  const byId = new Map(list.map((feed) => [feed.id, feed]));
  const shown = (group) => view.name === 'releases' && view.group === group;
  const openKeys = new Set(open || []);
  const selected = isFeedView(view) && isPublisher(byId.get(view.id) || {})
    ? rowKey(rowPath(list, view.id, view.from)) : null;
  const rows = [];
  const add = (feed, path, heading) => {
    const key = rowKey(path);
    const kids = ranked((feed.child_ids || []).map((id) => byId.get(id)).filter((kid) => kid && isPublisher(kid) && !path.includes(kid.id)));
    const counted = levelCounts(feed);
    // The playlists of a curator page, in the sequence of its feed (ADR 0254
    // item 4a, ADR 0263 item 10).
    const playlists = (feed.release_ids || []).map((id) => byId.get(id)).filter((item) => item && isPlaylist(item));
    const hasPlaylists = (feed.publisher_roles || []).includes('curator') && playlists.length > 0;
    // The count of a row below the heading "Curators" is the number of its
    // playlists (ADR 0263 item 10). The first level role of the feed gives
    // its heading (ADR 0253 item 15).
    const curatorCount = hasPlaylists && headingOf(feed) === 'Curators';
    const isOpen = (kids.length > 0 || hasPlaylists) && (all || openKeys.has(key));
    const releases = curatorCount ? playlists.length : counted.releases;
    const pages = counted.pages;
    const releaseNoun = curatorCount ? ['playlist', 'playlists'] : ['release', 'releases'];
    const parent = path.length > 1 ? path[path.length - 2] : null;
    const others = ranked((feed.parent_ids || []).filter((id) => id !== parent).map((id) => byId.get(id)).filter(Boolean));
    const labels = others.filter((other) => (other.publisher_roles || []).includes('label'));
    const names = (labels.length > 0 ? labels : others).map(displayTitle);
    const kind = feedKind(feed);
    rows.push({
      id: `tree-${key}`,
      key,
      group: feed.id,
      depth: path.length - 1,
      label: displayTitle(feed),
      dot: kind.dot,
      glow: kindGlow(kind),
      art: publisherArt(feed, list),
      counts: { releases, pages },
      count: releases ?? pages,
      countText: [releases, pages].filter((n) => n !== null).join(' · '),
      countLabel: [
        releases === null ? null : `${releases} ${releases === 1 ? releaseNoun[0] : releaseNoun[1]}`,
        pages === null ? null : `${pages} ${pages === 1 ? 'page' : 'pages'} below`,
      ].filter(Boolean).join(', '),
      expandable: kids.length > 0 || hasPlaylists,
      open: isOpen,
      also: parent === null || others.length === 0 ? null
        : `${labels.length > 0 ? 'Also on' : 'Also published by'} ${andNames(names)}`,
      current: key === selected,
      opens: key !== selected,
      view: openFeedView(feed.id, false, fromPath(path.slice(0, -1))),
      ...(heading ? { heading } : {}),
    });
    if (isOpen) {
      // One row for each playlist of a curator page, then the pages below it
      // (ADR 0263 item 10). A playlist row has no key, no count and no menu.
      playlists.forEach((playlist) => {
        const playlistKind = feedKind(playlist);
        rows.push({
          id: `tree-${key}-playlist-${playlist.id}`,
          group: playlist.id,
          depth: path.length,
          label: displayTitle(playlist),
          typeLabel: playlistKind.label,
          dot: playlistKind.dot,
          glow: kindGlow(playlistKind),
          art: [feedCover(playlist)].filter(Boolean),
          count: null,
          current: false,
          opens: true,
          view: openFeedView(playlist.id, false, fromPath(path)),
        });
      });
      kids.forEach((kid) => add(kid, [...path, kid.id]));
    }
  };
  const top = ranked(list.filter((feed) => isPublisher(feed) && (feed.parent_ids || []).length === 0));
  let last = null;
  for (const feed of top) {
    const heading = headingOf(feed);
    add(feed, [feed.id], heading === last ? null : heading);
    last = heading;
  }
  const unlinked = unlinkedReleases(list).length;
  // "No artist yet", "No curator yet" and "Other feeds" follow the tree (ADR
  // 0263 item 9).
  if (unlinked > 0) {
    rows.push({
      id: 'group-none', group: 'none', label: 'No artist yet', dot: null,
      count: unlinked, current: shown('none'), opens: !shown('none'),
    });
  }
  // The playlists with no curator page (ADR 0263 item 11).
  const noCurator = unlinkedPlaylists(list).length;
  if (noCurator > 0) {
    rows.push({
      id: 'group-no-curator', group: 'no-curator', label: 'No curator yet', dot: null,
      count: noCurator, current: shown('no-curator'), opens: !shown('no-curator'),
    });
  }
  // The feeds that the editor does not open (ADR 0090 item 2).
  const other = otherFeeds(list).length;
  if (other > 0) {
    rows.push({
      id: 'group-other', group: 'other', label: 'Other feeds', dot: null,
      count: other, current: shown('other'), opens: !shown('other'),
    });
  }
  return rows;
}

// The feeds that are not a publisher feed and not a release (ADR 0090
// item 2).
export function otherFeeds(feeds) {
  return feeds.filter((feed) => !isPublisher(feed) && !isRelease(feed));
}

// The IDs of each feed above and below `owner`, from the levels that the
// core gives (ADR 0254 item 9).
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

// The albums with no link to an artist page of the collection (ADR 0254
// item 2, ADR 0263 item 11). A playlist is never in this list.
export function unlinkedReleases(feeds) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  return (feeds || []).filter((feed) => isAlbum(feed) && !(feed.publisher_ids || []).some((id) => {
    const page = byId.get(id);
    return isPublisher(page || {}) && (page.publisher_roles || []).includes('artist');
  }));
}

// The cover of a feed row. The row has the `itunes:image` as `image` and
// the address of the RSS image as `rss_image_url`. The cover rule is in
// `cover.js` (ADR 0060). A feed with no cover gives `null`.
export function feedCover(feed) {
  const rssImage = feed.rss_image_url ? { url: feed.rss_image_url } : null;
  return coverOf({ image: feed.image ?? null, rss_image: rssImage }) || null;
}

// The label of each draft state in the Releases list (ADR 0258 item 2).
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
  return connector ? connector.name : 'Unknown file host';
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

// The order of two names in the language of the reader. A number in a name
// counts as a number, so "2.mp3" comes before "15.mp3" (musicindex ADR 0053
// R-LIST-12 item 1).
const NAMES = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
export function compareNames(a, b) {
  return NAMES.compare(String(a ?? ''), String(b ?? ''));
}

// A copy of `list` in the name order of `compareNames`. `nameOf` gives the
// name of an entry.
export function byName(list, nameOf) {
  return (list || []).slice().sort((a, b) => compareNames(nameOf(a), nameOf(b)));
}

// A to Z. A title with a letter from A to Z comes before a title with any
// other first character, so that "#" comes after Z.
function compareAz(a, b) {
  const bucket = (feed) => (letterOf(displayTitle(feed)) === '#' ? 1 : 0);
  return bucket(a) - bucket(b)
    || compareNames(displayTitle(a), displayTitle(b))
    || Number(a.id) - Number(b.id);
}

// The first letter of each group of rows, in sequence (ADR 0258 item 3).
// Each row has a `title` and an `id`.
export function letterGroups(rows) {
  const groups = [];
  for (const row of rows) {
    const letter = letterOf(row.title);
    if (groups.length === 0 || groups[groups.length - 1].letter !== letter) {
      groups.push({ letter, firstId: row.id });
    }
  }
  return groups;
}

// The sorts of each list (ADR 0258 item 3, ADR 0085 item 4). The first
// sort is the default.
const ALL_SORTS = ['newest', 'az'];
const GROUP_SORTS = ['order', 'newest', 'az'];

// The names of each sort (ADR 0258 item 3, ADR 0085 item 4). A reversed list
// names its reversed order (musicindex ADR 0043 R-LIST-7 item 5).
const SORT_LABELS = { order: 'Page Order', newest: 'Newest', az: 'A to Z' };
const REVERSED_SORT_LABELS = { order: 'Page Order, Reversed', newest: 'Oldest', az: 'Z to A' };

// The name of a sort, in the usual order or in the reversed order.
export function sortLabel(sort, reversed) {
  return (reversed ? REVERSED_SORT_LABELS : SORT_LABELS)[sort];
}

// The index of the segment that gets the focus after a key in a segmented
// control (musicindex ADR 0051 R-SEG-1 item 7, ADR 0157). Left and Right move
// one segment and wrap, as on musicindex.org. Home and End go to the first
// and the last segment. Each other key gives null. A key moves only the focus:
// Enter or Space selects the segment, as on musicindex.org.
export function segmentKey(index, count, key) {
  if (count === 0) return null;
  switch (key) {
    case 'ArrowRight': return (index + 1) % count;
    case 'ArrowLeft': return (index - 1 + count) % count;
    case 'Home': return 0;
    case 'End': return count - 1;
    default: return null;
  }
}

// The entries of a sort menu: each sort, then its reversed direction, as
// words (musicindex ADR 0255 R-LIST-13 item 2, ADR 0155).
export function sortEntries(sorts) {
  return sorts.flatMap((sort) => [false, true].map((reversed) => ({ sort, reversed, label: sortLabel(sort, reversed) })));
}

// ---- Overview (ADR 0263 items 3 to 5, ADR 0264) ----

// The filters of Overview, in sequence (ADR 0263 item 3). The first one is the
// default.
export const OVERVIEW_FILTERS = [['all', 'All'], ['releases', 'Releases'], ['playlists', 'Playlists']];

// The filter of an Overview view. A value that is not known gives "all".
export const overviewFilter = (view) => (view && OVERVIEW_FILTERS.some(([id]) => id === view.filter) ? view.filter : 'all');

// An album is a release in the words of Overview. A playlist is not a release
// (ADR 0263 item 3).
const isAlbum = (feed) => (feed.medium || '').toLowerCase() === 'music';

// The pages of `feed` with `role` in its `publisher_ids`, as feeds.
function pagesWithRole(feed, byId, role) {
  return (feed.publisher_ids || []).map((id) => byId.get(id))
    .filter((page) => page && isPublisher(page) && (page.publisher_roles || []).includes(role));
}

// The playlists with no link to a curator page of the collection (ADR 0263
// item 11).
export function unlinkedPlaylists(feeds) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  return (feeds || []).filter((feed) => isPlaylist(feed) && pagesWithRole(feed, byId, 'curator').length === 0);
}

// The release date of a tile as a date text, such as "2026-10-29", from the
// RFC 3339 text of the core. Studio does not change the zone of the date. A
// value that is not a date gives null.
const dateText = (value) => (typeof value === 'string' && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : null);

// The instant of a release date in milliseconds, or null (ADR 0264 item 4).
function instantOf(feed) {
  const at = Date.parse(feed.release_date);
  return Number.isNaN(at) ? null : at;
}

// The order of two tiles below a heading (ADR 0264 item 4): the newest
// instant first, then the tiles with no date, then A to Z for equal values.
function compareTiles(a, b) {
  const x = instantOf(a);
  const y = instantOf(b);
  if (x !== y && x !== null && y !== null) return y - x;
  if ((x === null) !== (y === null)) return x === null ? 1 : -1;
  return compareAz(a, b);
}

const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;

// The model of Overview (ADR 0263 items 3 to 5, ADR 0264). `filter` is
// "all", "releases" or "playlists". `query` is the text of the search, and
// `marked` is the ID of the feed that the last Back selection left marked.
// The function lists the feed list fields as they are (ADR 0065). It gives:
// - `filters`: the segments of the filter, each with `id`, `label` and `current`.
// - `countsText`: the counts line, with no count of zero.
// - `notices`: the notice rows, each with `id`, `text` and the `view` that it opens.
// - `parts`: the release part, then the playlist part. A part with no tile that
//   shows is not in the list. A part has `id` and `headings`.
// - `empty` and `emptyText`: true when no tile shows, and the text for it.
// A heading has `id`, `level` (1 or 2), `title`, `feedId` (null for a heading
// that opens nothing), `view`, `count`, `countLabel`, `tiles` and `children`.
// A tile has `key`, `id`, `title`, `date`, `dot`, `glow`, `image`, `marked`
// and `view`.
export function overviewModel({ feeds, filter = 'all', query = '', marked = null }) {
  const list = Array.isArray(feeds) ? feeds : [];
  const byId = new Map(list.map((feed) => [feed.id, feed]));
  // The sequence of the tree is the sequence of the Catalog rows, with each
  // row open. A page below two pages has the place of its first row.
  const place = new Map();
  groupDestinations(list, { name: 'overview' }, [], { all: true })
    .filter((row) => row.key !== undefined)
    .forEach((row) => { if (!place.has(row.group)) place.set(row.group, place.size); });
  const byTree = (a, b) => (place.has(a.id) ? place.get(a.id) : place.size) - (place.has(b.id) ? place.get(b.id) : place.size)
    || compareAz(a, b);
  const first = (pages) => pages.slice().sort(byTree)[0] || null;
  const shown = { all: ['releases', 'playlists'], releases: ['releases'], playlists: ['playlists'] }[filter] || ['releases', 'playlists'];

  const search = typeof query === 'string' ? query.trim() : '';
  const needle = search.toLowerCase();
  const matches = (feed) => {
    if (!needle) return true;
    const pages = ['artist', 'label', 'curator'].flatMap((role) => pagesWithRole(feed, byId, role));
    return [feed, ...pages].some((item) => displayTitle(item).toLowerCase().includes(needle));
  };
  const tile = (feed, scope) => {
    const kind = feedKind(feed);
    const date = dateText(feed.release_date);
    return {
      key: `${scope}-${feed.id}`,
      id: feed.id,
      title: displayTitle(feed),
      date,
      dot: kind.dot,
      glow: kindGlow(kind),
      image: feedCover(feed),
      marked: feed.id === marked,
      view: openFeedView(feed.id, false, undefined),
    };
  };
  const heading = (id, level, page, title, feedsBelow, children = []) => {
    const tiles = children.length > 0 ? [] : feedsBelow.slice().sort(compareTiles).map((feed) => tile(feed, id));
    const count = children.length > 0 ? children.reduce((sum, child) => sum + child.count, 0) : tiles.length;
    return {
      id,
      level,
      title,
      feedId: page ? page.id : null,
      view: page ? openFeedView(page.id, false, undefined) : null,
      count,
      tiles,
      children,
    };
  };

  // The release part (ADR 0264 items 1 to 3). Each album goes below the first
  // artist page and the first label page of its links. A page that is the
  // artist page of the album is not also its label page.
  const albums = shown.includes('releases') ? list.filter((feed) => isAlbum(feed) && matches(feed)) : [];
  const labels = new Map();
  const independent = new Map();
  const noArtist = [];
  for (const album of albums) {
    const artist = first(pagesWithRole(album, byId, 'artist'));
    const label = first(pagesWithRole(album, byId, 'label').filter((page) => !artist || page.id !== artist.id));
    const slot = artist ? artist.id : 'none';
    if (label) {
      if (!labels.has(label.id)) labels.set(label.id, new Map());
      const artists = labels.get(label.id);
      if (!artists.has(slot)) artists.set(slot, []);
      artists.get(slot).push(album);
    } else if (artist) {
      if (!independent.has(slot)) independent.set(slot, []);
      independent.get(slot).push(album);
    } else {
      noArtist.push(album);
    }
  }
  const artistHeadings = (scope, artists) => {
    const rows = [...artists.keys()].filter((key) => key !== 'none').map((id) => byId.get(id)).sort(byTree)
      .map((page) => heading(`${scope}-artist-${page.id}`, 2, page, displayTitle(page), artists.get(page.id)));
    if (artists.has('none')) rows.push(heading(`${scope}-no-artist`, 2, null, 'No artist yet', artists.get('none')));
    return rows;
  };
  const releaseHeadings = [...labels.keys()].map((id) => byId.get(id)).sort(byTree)
    .map((page) => heading(`label-${page.id}`, 1, page, displayTitle(page), [], artistHeadings(`label-${page.id}`, labels.get(page.id))));
  if (independent.size > 0) releaseHeadings.push(heading('independent', 1, null, 'Independent', [], artistHeadings('independent', independent)));
  if (noArtist.length > 0) releaseHeadings.push(heading('no-artist', 1, null, 'No artist yet', noArtist));

  // The playlist part (ADR 0264 item 5). A playlist below two curator pages
  // shows below each one.
  const playlists = shown.includes('playlists') ? list.filter((feed) => isPlaylist(feed) && matches(feed)) : [];
  const curators = new Map();
  const noCurator = [];
  for (const playlist of playlists) {
    const pages = pagesWithRole(playlist, byId, 'curator');
    if (pages.length === 0) noCurator.push(playlist);
    for (const page of pages) {
      if (!curators.has(page.id)) curators.set(page.id, []);
      curators.get(page.id).push(playlist);
    }
  }
  const playlistHeadings = [...curators.keys()].map((id) => byId.get(id)).sort(byTree)
    .map((page) => heading(`curator-${page.id}`, 1, page, displayTitle(page), curators.get(page.id)));
  if (noCurator.length > 0) playlistHeadings.push(heading('no-curator', 1, null, 'No curator yet', noCurator));

  const parts = [{ id: 'releases', headings: releaseHeadings }, { id: 'playlists', headings: playlistHeadings }]
    .filter((part) => part.headings.length > 0);

  // The counts line and the notice rows (ADR 0263 item 3). The counts cover
  // the whole collection, not the filter or the search.
  const pages = list.filter(isPublisher);
  const roleCount = (role) => pages.filter((feed) => (feed.publisher_roles || []).includes(role)).length;
  const countsText = [
    [roleCount('hosting'), 'publisher', 'publishers'],
    [roleCount('label'), 'label', 'labels'],
    [roleCount('artist'), 'artist', 'artists'],
    [roleCount('curator'), 'curator', 'curators'],
    [list.filter(isAlbum).length, 'release', 'releases'],
    [list.filter(isPlaylist).length, 'playlist', 'playlists'],
  ].filter(([n]) => n > 0).map(([n, one, many]) => plural(n, one, many)).join(' · ');
  const notices = [];
  const noArtistCount = unlinkedReleases(list).length;
  if (noArtistCount > 0) {
    notices.push({
      id: 'no-artist',
      text: `${noArtistCount} ${noArtistCount === 1 ? 'release has' : 'releases have'} no artist yet.`,
      view: { name: 'releases', group: 'none' },
    });
  }
  const noCuratorCount = unlinkedPlaylists(list).length;
  if (noCuratorCount > 0) {
    notices.push({
      id: 'no-curator',
      text: `${noCuratorCount} ${noCuratorCount === 1 ? 'playlist has' : 'playlists have'} no curator yet.`,
      view: { name: 'releases', group: 'no-curator' },
    });
  }
  // A feed that Studio has not written has the draft state "new" or "changed"
  // (ADR 0259 item 5). The notice opens the first one: the pages in the
  // sequence of the tree, then the rows in the sequence of the list.
  const unwritten = list.filter((feed) => feed.draft_state === 'new' || feed.draft_state === 'changed');
  if (unwritten.length > 0) {
    const target = unwritten.filter(isPublisher).sort(byTree)[0]
      || unwritten.filter((feed) => !isPublisher(feed)).sort(compareTiles)[0];
    notices.push({
      id: 'not-written',
      text: `${unwritten.length} ${unwritten.length === 1 ? 'page or release is' : 'pages and releases are'} not written.`,
      view: openFeedView(target.id, false, undefined),
    });
  }

  const empty = parts.length === 0;
  return {
    filter,
    filters: OVERVIEW_FILTERS.map(([id, label]) => ({ id, label, current: id === filter })),
    query,
    countsText,
    notices,
    parts,
    empty,
    emptyText: search ? `Nothing has “${search}”.` : filter === 'playlists' ? 'No playlists yet.' : 'No releases yet.',
  };
}

// The list of one group below the Catalog (ADR 0258 items 2 and 3).
// `marked` is the ID of the feed that the last Back selection left marked.
// `group` is the ID of a feed with roles, and the list is the releases of that
// feed. Then `head` gives the feed. The group 'none' gives the releases with no
// link (ADR 0085 items 3 and 4). The group 'no-curator' gives the playlists
// with no curator page (ADR 0263 item 11). The group 'other' gives the feeds
// that Studio does not edit. These three groups are list pages (ADR 0263 item
// 9a): the sequence of Overview, the search, no sort and no letters, and a row
// with the release date and the artist or curator. Overview has its own
// model, `overviewModel`. Packet 256d replaces the sort of the group of a page.
export function releasesList({ feeds, connectors, sort, view, reversed, marked, group, query = '' }) {
  const list = Array.isArray(feeds) ? feeds : [];
  const listPage = group === 'none' || group === 'no-curator' || group === 'other';
  const owner = group === undefined || listPage
    ? null
    : list.find((feed) => feed.id === group && isPublisher(feed)) || null;
  let chosen = [];
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
    head = { id: null, title: 'No artist yet', typeLabel: null, dot: null, note: 'These releases have no artist page.' };
  } else if (group === 'no-curator') {
    chosen = unlinkedPlaylists(list);
    head = { id: null, title: 'No curator yet', typeLabel: null, dot: null, note: 'These playlists have no curator page.' };
  } else if (group === 'other') {
    chosen = otherFeeds(list);
    head = { id: null, title: 'Other feeds', typeLabel: null, dot: null, note: 'Studio edits albums, playlists, and artist and label feeds. It shows only the facts of these feeds.' };
  }
  // A list page has one sequence and no sort (ADR 0263 items 5 and 9a).
  const sorts = listPage ? [] : owner ? GROUP_SORTS : ALL_SORTS;
  const sortKey = listPage ? null : sorts.includes(sort) ? sort : sorts[0];
  const viewKey = view === 'tiles' && !listPage ? 'tiles' : 'rows';
  const sorted = listPage
    ? chosen.slice().sort(compareTiles)
    : sortKey === 'order'
      ? chosen.slice()
      : chosen.slice().sort(sortKey === 'az' ? compareAz : compareNewest);
  // Studio holds each row of this list, so the whole list reverses
  // (musicindex ADR 0043 R-LIST-7 item 9). The letters follow the rows.
  const isReversed = reversed === true && !listPage;
  if (isReversed) sorted.reverse();
  const search = typeof query === 'string' ? query.trim().toLowerCase() : '';
  const matches = (feed) => {
    if (!search) return true;
    const titles = [displayTitle(feed), ...(feed.publisher_ids || []).map((id) => list.find((item) => item.id === id)).filter(Boolean).map(displayTitle)];
    return titles.some((title) => title.toLowerCase().includes(search));
  };
  const filtered = search ? sorted.filter(matches) : sorted;
  const titleOf = (id) => {
    const feed = list.find((f) => f.id === id);
    return feed ? displayTitle(feed) : null;
  };
  // A release of two feeds names the other feeds (ADR 0085 item 5). A feed
  // above or below the owner is in the tree already, so the note leaves it
  // out (ADR 0254 item 9).
  const related = owner ? relatedLevels(list, owner) : new Set();
  const noteOf = (feed) => {
    if (!owner) return null;
    const others = (feed.publisher_ids || []).filter((id) => id !== owner.id && !related.has(id)).map(titleOf).filter(Boolean);
    return others.length > 0 ? `Also on ${others.join(', ')}` : null;
  };
  const artistLevel = owner && (owner.publisher_roles || []).includes('artist');
  const rows = filtered.map((feed) => ({
    id: feed.id,
    title: displayTitle(feed),
    typeLabel: feedKind(feed).label,
    countText: trackCountText(feed.track_count),
    dot: feedKind(feed).dot,
    glow: kindGlow(feedKind(feed)),
    host: hostOf(feed, connectors),
    image: feedCover(feed),
    draftLabel: DRAFT_LABELS[feed.draft_state] || 'No changes',
    // A feed that apps do not list (ADR 0063).
    hidden: feed.hidden === true,
    marked: feed.id === marked,
    note: noteOf(feed),
    artistLine: listPage ? (typeof feed.author === 'string' && feed.author.trim()
      ? `${isPlaylist(feed) ? 'Curator' : 'Artist'}: ${feed.author.trim()}` : null) : artistLevel && typeof feed.author === 'string' && feed.author.trim().toLowerCase() !== displayTitle(owner).trim().toLowerCase()
      ? `Artist: ${feed.author}` : null,
    // A list page row shows the release date and the artist or curator (ADR
    // 0263 item 9a with item 7).
    date: listPage ? dateText(feed.release_date) : null,
    unlistedLine: owner && (feed.unlisted_by || []).includes(owner.id)
      ? `⚠ ${displayTitle(owner)} does not list it yet` : null,
  }));
  const levelTile = head && head.id !== null ? {
    id: head.id,
    title: head.title,
    typeLabel: head.typeLabel,
    dot: head.dot,
    glow: kindGlow(feedKind(owner)),
    image: feedCover(owner),
    draftLabel: DRAFT_LABELS[owner.draft_state] || 'No changes',
  } : null;
  return {
    sort: sortKey,
    sorts,
    reversed: isReversed,
    view: viewKey,
    head,
    levelTile,
    // The release rows have an indent below the level tile (ADR 0228 item 19).
    nested: levelTile !== null && viewKey === 'rows',
    rows,
    letters: search ? null : sortKey === 'az' ? letterGroups(rows) : null,
    listPage,
  };
}

// ---- Navigation (musicindex ADR 0080 R-NAV-2) ----
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

// Back. From a feed or its outline, the Overview entry before it keeps its
// scroll and gets the feed as its marked row. From a feed or an outline at
// index 0, an Overview entry with the marked feed goes before it.
export function backNav(nav) {
  const { entries, index } = nav;
  const current = entries[index];
  if (current && isFeedView(current)) {
    if (index === 0) {
      return { entries: [{ name: 'overview', marked: current.id }, ...entries], index: 0 };
    }
    const next = entries.slice();
    const before = entries[index - 1];
    if (before.name === 'overview' || before.name === 'releases') next[index - 1] = { ...before, marked: current.id };
    return { entries: next, index: index - 1 };
  }
  return { entries: entries.slice(), index: Math.max(0, index - 1) };
}

// The view that the link to Overview opens (ADR 0069, ADR 0263 item 1). The
// nearest Overview entry at or before the index gives its scroll, its filter
// and its search. The function does not change `nav`.
export function overviewView(nav, feedId) {
  const view = { name: 'overview', marked: feedId };
  for (let i = nav.index; i >= 0; i -= 1) {
    const entry = nav.entries[i];
    if (entry && entry.name === 'overview') {
      for (const key of ['filter', 'query']) if (entry[key] !== undefined) view[key] = entry[key];
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
// The root page has no back control, also at a later entry (musicindex ADR
// 0080 R-SHELL-2, R-TITLE-1 item 4). The browser Back still leaves it.
export function canBack(nav) {
  const current = nav.entries[nav.index];
  if (current && current.name === 'root') return false;
  return nav.index > 0 || isFeedView(current);
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

// ---- Narrow width (musicindex ADR 0080 R-SHELL-2, ADR 0258 items 11 to 14) ----

// The stack that a narrow window shows for one view. The entries go from the
// root page to the view, and the view is the last entry. The root page shows
// the Catalog tree (ADR 0254 item 10). `feeds` is the feed list, or null before
// it loads. The page of a publisher feed is directly above the root page, so
// its "‹ Catalog" goes back to the tree.
export function narrowStack(view, feeds = null) {
  const root = { name: 'root' };
  const overview = { name: 'overview' };
  switch (view.name) {
    case 'root': return [root];
    // Overview keeps its filter and its search (ADR 0263 item 8).
    case 'overview': return [root, view];
    // A list page below the Catalog is a page of the root (ADR 0085 item 7).
    case 'releases': return view.group === undefined ? [root, overview] : [root, view];
    // The People area (ADR 0203 items 36 and 64).
    case 'person':
    case 'wallet':
    case 'change-set': return inHosts(view) ? [root, { name: 'hosts' }, view]
      : inReleases(view) ? [root, overview, view] : [root, { name: 'people' }, view];
    case 'check': return [root, { name: 'people' }, { name: 'person', id: view.id }, view];
    // The feed page is the page below Overview. The outline page is below the
    // feed page (ADR 0223 item 8).
    case 'outline': {
      const feed = { name: 'feed', id: view.id, ...(view.from ? { from: view.from } : {}) };
      return [...narrowStack(feed, feeds), view];
    }
    case 'feed': {
      const own = (feeds || []).find((feed) => feed.id === view.id);
      if (own && isPublisher(own)) return [root, view];
      // A release that opened from a page of a publisher feed goes back to
      // that page.
      return view.from
        ? [root, openFeedView(view.from.group, true, fromPath(pathOfFrom(view.from).slice(0, -1))), view]
        : [root, overview, view];
    }
    default: return [root, view];
  }
}

// The view of a feed on Overview. It is the feed view at each width
// (ADR 0223 item 8). The narrow page of the feed has the button Sections.
export function openFeedView(id, narrow, from) {
  const view = { name: 'feed', id };
  return from ? { ...view, from } : view;
}

// The view at a wide width. The outline has no wide view, so it becomes the
// feed, and the root page becomes Overview.
export function wideView(view) {
  if (view.name === 'outline') return { name: 'feed', id: view.id };
  if (view.name === 'root') return { name: 'overview' };
  return view;
}

// The view that an old record gives. A view `releases` with no group was the
// page Releases, and it opens Overview now. It keeps its marked feed and its
// scroll, and it drops `sort`, `reversed` and `view` (ADR 0263 item 8).
export function legacyView(view) {
  if (!view || view.name !== 'releases' || view.group !== undefined) return view;
  const next = { name: 'overview' };
  if (view.marked !== undefined) next.marked = view.marked;
  if (view.scroll !== undefined) next.scroll = view.scroll;
  return next;
}

// The navigation state of an old record, with `legacyView` for each entry.
export function legacyNav(nav) {
  return { ...nav, entries: nav.entries.map(legacyView) };
}

// The query text of an address without the keys that the page Releases used.
// An old address with `sort`, `reversed` or `view` opens Overview, and the
// address loses these keys (ADR 0263 item 8). Other keys stay.
export function addressSearch(search) {
  const params = new URLSearchParams(typeof search === 'string' ? search : '');
  ['sort', 'reversed', 'view'].forEach((key) => params.delete(key));
  const text = params.toString();
  return text ? `?${text}` : '';
}

// The navigation state for a new width of the window. A narrow window gets
// the stack of the view. A wide window gets the view alone.
export function widthNav(view, narrow, feeds = null) {
  if (!narrow) return startNav(wideView(view));
  const entries = narrowStack(view, feeds);
  return { entries, index: entries.length - 1 };
}

// The home button and the title button of musicindex ADR 0080 R-TITLE-1 show
// in the narrow layout on each page other than the root page.
export function showsRootButtons(nav, narrow) {
  if (!narrow || !nav) return false;
  const view = nav.entries[nav.index];
  return Boolean(view) && view.name !== 'root';
}

// The state after the home button or the title button (musicindex ADR 0080
// R-TITLE-1 item 4): the root page is a new entry after the current entry,
// and the entries after the current entry go. Back then opens the page that
// the person left, and Forward has no page. On the root page nothing moves.
export function rootNav(nav) {
  const current = nav.entries[nav.index];
  if (current && current.name === 'root') return nav;
  return pushNav(nav, { name: 'root' });
}

// The navigation state that fits a width. A state that already has the form
// of the width is kept, so that a reload keeps its history. Any other state
// becomes `widthNav` of its current view.
export function navAtWidth(nav, narrow, feeds = null) {
  const narrowForm = nav.entries[0].name === 'root';
  const wideForm = nav.entries.every((e) => e.name !== 'root' && e.name !== 'outline');
  if (narrow ? narrowForm : wideForm) return nav;
  return widthNav(nav.entries[nav.index], narrow, feeds);
}
