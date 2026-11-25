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
// (ADR 0206 item 14).
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

// The left panel (ADR 0206 items 1, 6 and 7, ADR 0254 items 2 and 7). With
// no open feed, or with the page of a publisher feed, it gives the
// destinations and the Catalog tree. With an open release or playlist, or the
// outline view of one, it gives the outline of that feed. `model` is the
// typed model of the draft of the open feed, or null before it loads. `open`
// is the set of the keys of the open tree rows. `narrow` is true at 760 px or
// less. `vaultLocked` is true on a server with a locked vault: the row
// "Unlock Studio" then takes the place of "Lock Studio" (ADR 0206 item 1).
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
    const all = (view.name === 'releases' && view.group === undefined) || inReleases(view);
    return {
      kind: 'destinations',
      top: [
        // The Add feeds page belongs to Releases (ADR 0072 item 5). On that
        // page the row is current, and a click on it still opens Releases.
        {
          id: 'releases',
          label: 'Releases',
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
export function groupDestinations(feeds, view, open = []) {
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
    const isOpen = kids.length > 0 && openKeys.has(key);
    const roles = feed.publisher_roles || [];
    const releases = !roles.includes('hosting') || roles.some((role) => ['label', 'curator', 'artist'].includes(role))
      ? (feed.release_ids || []).length : null;
    const pages = roles.includes('hosting') ? (feed.child_ids || []).length : null;
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
        releases === null ? null : `${releases} ${releases === 1 ? 'release' : 'releases'}`,
        pages === null ? null : `${pages} ${pages === 1 ? 'page' : 'pages'} below`,
      ].filter(Boolean).join(', '),
      expandable: kids.length > 0,
      open: isOpen,
      also: parent === null || others.length === 0 ? null
        : `${labels.length > 0 ? 'Also on' : 'Also published by'} ${andNames(names)}`,
      current: key === selected,
      opens: key !== selected,
      view: openFeedView(feed.id, false, fromPath(path.slice(0, -1))),
      ...(heading ? { heading } : {}),
    });
    if (isOpen) kids.forEach((kid) => add(kid, [...path, kid.id]));
  };
  const top = ranked(list.filter((feed) => isPublisher(feed) && (feed.parent_ids || []).length === 0));
  let last = null;
  for (const feed of top) {
    const heading = headingOf(feed);
    add(feed, [feed.id], heading === last ? null : heading);
    last = heading;
  }
  const unlinked = unlinkedReleases(list).length;
  if (unlinked > 0) {
    rows.push({
      id: 'group-none', group: 'none', label: 'No artist yet', dot: null,
      count: unlinked, current: shown('none'), opens: !shown('none'),
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

// The releases with no link to an artist page of the collection (ADR 0254
// item 2).
export function unlinkedReleases(feeds) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  return (feeds || []).filter((feed) => isRelease(feed) && !(feed.publisher_ids || []).some((id) => {
    const page = byId.get(id);
    return isPublisher(page || {}) && (page.publisher_roles || []).includes('artist');
  }));
}

// The destination after a release move (ADR 0240 item 19).
export function movedReleaseView(artistId) {
  return { name: 'releases', group: artistId };
}

// The cover of a feed row. The row has the `itunes:image` as `image` and
// the address of the RSS image as `rss_image_url`. The cover rule is in
// `cover.js` (ADR 0060). A feed with no cover gives `null`.
export function feedCover(feed) {
  const rssImage = feed.rss_image_url ? { url: feed.rss_image_url } : null;
  return coverOf({ image: feed.image ?? null, rss_image: rssImage }) || null;
}

// The label of each draft state in the Releases list (ADR 0206 item 2).
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

// The first letter of each group of rows, in sequence (ADR 0206 item 3).
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

// The sorts of each list (ADR 0206 item 3, ADR 0085 item 4). The first
// sort is the default.
const ALL_SORTS = ['newest', 'az'];
const GROUP_SORTS = ['order', 'newest', 'az'];

// The names of each sort (ADR 0206 item 3, ADR 0085 item 4). A reversed list
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

// The Releases list (ADR 0206 items 2 and 3). `marked` is the ID of the feed
// that the last Back selection left marked. With no `group` it gives each
// album and playlist (ADR 0085 item 6). With the ID of a feed with roles as
// `group`, it gives the releases of that feed, and `head` gives the feed. The
// group 'none' gives the releases with no link (ADR 0085 items 3 and 4).
export function releasesList({ feeds, connectors, sort, view, reversed, marked, group, query = '' }) {
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
    head = { id: null, title: 'No artist yet', typeLabel: null, dot: null, note: 'These releases have no artist page.' };
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
  // Studio holds each row of this list, so the whole list reverses
  // (musicindex ADR 0043 R-LIST-7 item 9). The letters follow the rows.
  const isReversed = reversed === true;
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
    artistLine: artistLevel && typeof feed.author === 'string' && feed.author.trim().toLowerCase() !== displayTitle(owner).trim().toLowerCase()
      ? `Artist: ${feed.author}` : null,
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
  };
}

// The sort, the view and the reversed state of the Releases list from a query
// text, such as "sort=az". A value that is missing or not known gives the
// default.
export function listQuery(search) {
  const params = new URLSearchParams(typeof search === 'string' ? search : '');
  return {
    sort: params.get('sort') === 'az' ? 'az' : 'newest',
    view: params.get('view') === 'tiles' ? 'tiles' : 'rows',
    reversed: params.get('reversed') === '1',
  };
}

// The query text of a sort, a view and a reversed state. It holds only the
// values that are not the default. It is empty when each value is the default.
export function queryText({ sort, view, reversed }) {
  const params = new URLSearchParams();
  if (sort === 'az') params.set('sort', 'az');
  if (view === 'tiles') params.set('view', 'tiles');
  if (reversed === true) params.set('reversed', '1');
  return params.toString();
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

// ---- Narrow width (musicindex ADR 0080 R-SHELL-2, ADR 0206 items 11 to 14) ----

// The stack that a narrow window shows for one view. The entries go from the
// root page to the view, and the view is the last entry. The root page shows
// the Catalog tree (ADR 0254 item 10). `feeds` is the feed list, or null before
// it loads. The page of a publisher feed is directly above the root page, so
// its "‹ Catalog" goes back to the tree.
export function narrowStack(view, feeds = null) {
  const root = { name: 'root' };
  const releases = { name: 'releases' };
  switch (view.name) {
    case 'root': return [root];
    // A destination below Releases is a page of the root (ADR 0085 item 7).
    case 'releases': return view.group === undefined ? [root, releases] : [root, view];
    // The People area (ADR 0203 items 36 and 64).
    case 'person':
    case 'wallet':
    case 'change-set': return inHosts(view) ? [root, { name: 'hosts' }, view]
      : inReleases(view) ? [root, releases, view] : [root, { name: 'people' }, view];
    case 'check': return [root, { name: 'people' }, { name: 'person', id: view.id }, view];
    // The feed page is the page below Releases. The outline page is below the
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
        : [root, releases, view];
    }
    default: return [root, view];
  }
}

// The view of a feed on Releases. It is the feed view at each width
// (ADR 0223 item 8). The narrow page of the feed has the button Sections.
export function openFeedView(id, narrow, from) {
  const view = { name: 'feed', id };
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
