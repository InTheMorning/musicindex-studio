// The model of the sheet "Move {title}" (ADR 0240 items 12 to 17). The
// functions have no Preact and no browser state. `editor.js` shows the sheet.
import { isPublisher, FIRST_CLASS_ROLES, roleWord } from './shell.js';

// The prefix of the value that opens the form of a new page (ADR 0240 item
// 14). The rest of the value is the role of the new page.
export const NEW_PAGE_PREFIX = 'new:';

// The writable pages of one role (ADR 0240 item 13), A to Z. A page that
// Studio cannot write has no connector.
export function writablePages(feeds, role) {
  return (feeds || []).filter((feed) => isPublisher(feed) && feed.connector_id && (feed.publisher_roles || []).includes(role));
}

function pagesOf(feeds, role) {
  return writablePages(feeds, role)
    .sort((a, b) => pageTitle(a).localeCompare(pageTitle(b)));
}

function pageTitle(feed) {
  return feed.title || feed.path || 'Untitled page';
}

// The title of the sheet (ADR 0240 item 12).
export function moveTitle(release) {
  return `Move ${release.title || release.path || 'Untitled release'}`;
}

// The title of the sheet for a publisher page (ADR 0240 item 21).
export function movePageTitle(page) {
  return `Move ${page.title || page.path || 'Untitled page'}`;
}

// The roles shown when an artist or label page moves (ADR 0240 item 21).
export function pageMoveRoles(page) {
  return (page.publisher_roles || []).includes('artist') ? ['label', 'hosting'] : ['hosting'];
}

// The body of a page move (ADR 0240 item 23).
export function pageMoveBody(page, values) {
  const id = (value) => (/^\d+$/.test(value || '') ? Number(value) : null);
  return Object.fromEntries(pageMoveRoles(page).map((role) => [`${role}_id`, id(values[role])]));
}

// The menus for an artist or label page move (ADR 0240 item 21).
export function pageMoveMenus(feeds, page, values) {
  return pageMoveRoles(page).map((role) => ({
    role,
    label: roleWord(role),
    value: values[role] ?? '',
    options: [['', 'None'], ...pagesOf(feeds, role).map((feed) => [String(feed.id), pageTitle(feed)]),
      [`${NEW_PAGE_PREFIX}${role}`, `New ${roleWord(role)} Page…`]],
  }));
}

// The current pages above the page that moves (ADR 0240 item 21).
export function pageMoveStart(feeds, page, roles = pageMoveRoles(page)) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const above = new Set();
  const pending = [...(page.parent_ids || [])];
  while (pending.length) {
    const id = pending.shift();
    if (above.has(id)) continue;
    above.add(id);
    pending.push(...((byId.get(id) && byId.get(id).parent_ids) || []));
  }
  return Object.fromEntries(roles.map((role) => [role, String(pagesOf(feeds, role).find((feed) => above.has(feed.id))?.id || '')]));
}

// The kept-link and release changes for the sheet (ADR 0240 item 22).
export function pageMoveLines(feeds, page, values) {
  const roleSet = new Set(pageMoveRoles(page));
  const releaseIds = page.release_ids || [];
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const current = new Set();
  const pending = [...(page.parent_ids || [])];
  while (pending.length) {
    const id = pending.shift();
    if (current.has(id)) continue;
    current.add(id);
    pending.push(...((byId.get(id) && byId.get(id).parent_ids) || []));
  }
  const selected = Object.entries(values).filter(([role]) => roleSet.has(role)).map(([, id]) => id).filter(Boolean);
  const pages = [...new Set([...current, ...selected])].map((id) => (feeds || []).find((feed) => String(feed.id) === String(id))).filter(Boolean);
  const changed = pages.filter((feed) => current.has(feed.id) !== selected.some((id) => String(id) === String(feed.id)));
  const lines = [
    ...changed.filter((feed) => selected.some((id) => String(id) === String(feed.id))).map((feed) => `+ ${pageTitle(feed)}`),
    ...changed.filter((feed) => !selected.some((id) => String(id) === String(feed.id))).map((feed) => `− ${pageTitle(feed)}`),
  ];
  return lines.length > 0
    ? [`Changes ${releaseIds.length} ${releaseIds.length === 1 ? 'release' : 'releases'}:`, ...lines]
    : ['Select a label or a publisher page.'];
}

function ancestors(feeds, page) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const found = new Set();
  const pending = [...(page.parent_ids || [])];
  while (pending.length) {
    const id = pending.shift();
    if (found.has(id)) continue;
    found.add(id);
    pending.push(...((byId.get(id) && byId.get(id).parent_ids) || []));
  }
  return found;
}

// The rows and controls of the gather sheet (ADR 0243 items 8 to 10).
export function gatherPagesModel(feeds, level, role, selected = null) {
  const word = role === 'artist' ? 'Artist' : 'Label';
  const rows = (feeds || []).filter((feed) => isPublisher(feed) && feed.connector_id
    && ((feed.publisher_roles || []).length === 0 || (feed.publisher_roles || []).includes(role))
    && !ancestors(feeds, feed).has(level.id) && feed.id !== level.id)
    .sort((a, b) => pageTitle(a).localeCompare(pageTitle(b)))
    .map((feed) => ({
      id: feed.id,
      title: pageTitle(feed),
      releases: (feed.release_ids || []).length,
      role: (feed.publisher_roles || []).includes(role) ? word : 'No role',
      checked: selected === null ? !(feed.publisher_roles || []).includes(role) : selected.includes(feed.id),
      noRole: !(feed.publisher_roles || []).includes(role),
    }));
  const count = rows.filter((row) => row.checked).length;
  return {
    title: `Move ${word.toLowerCase()}s to ${pageTitle(level)}`,
    rows,
    count,
    button: `Move ${count} ${word}${count === 1 ? '' : 's'}`,
    ready: count > 0,
    note: rows.some((row) => row.checked && row.noRole) ? `A page with no role gets the role ${word}.` : null,
  };
}

// The three menus of the sheet (ADR 0240 items 13 and 14). `values` holds the
// selected value of each menu, as a string. The artist menu has no "None".
// Label and Hosting start with "None". Each menu ends with "New … Page…". An
// artist menu with no artist page starts with a line that says so, because
// no value can be selected.
export function moveMenus(feeds, values) {
  return FIRST_CLASS_ROLES.map((role) => {
    const pages = pagesOf(feeds, role).map((feed) => [String(feed.id), pageTitle(feed)]);
    const lead = role !== 'artist' ? [['', 'None']] : pages.length === 0 ? [['', 'No artist page yet']] : [];
    return {
      role,
      label: roleWord(role),
      options: [...lead, ...pages, [`${NEW_PAGE_PREFIX}${role}`, `New ${roleWord(role)} Page…`]],
      value: values[role] ?? '',
    };
  });
}

// The first page of `role` among the pages above an artist page (ADR 0240
// item 15, ADR 0215 item 2), or ''.
function parentOf(feeds, artistId, role) {
  const artist = (feeds || []).find((feed) => String(feed.id) === String(artistId));
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  const above = new Set();
  const pending = [...((artist && artist.parent_ids) || [])];
  while (pending.length > 0) {
    const id = pending.shift();
    if (above.has(id)) continue;
    above.add(id);
    pending.push(...((byId.get(id) && byId.get(id).parent_ids) || []));
  }
  const page = pagesOf(feeds, role).find((feed) => above.has(feed.id));
  return page ? String(page.id) : '';
}

// The start values of the menus (ADR 0240 item 15). Each menu starts with
// the page of its role that the release links to. Then the artist page gives
// the label page and the hosting page above it. With no linked artist page,
// the first artist page is selected.
export function moveStart(feeds, release) {
  const linked = new Set((release && release.publisher_ids) || []);
  const first = (role) => {
    const page = pagesOf(feeds, role).find((feed) => linked.has(feed.id));
    return page ? String(page.id) : '';
  };
  const artists = pagesOf(feeds, 'artist');
  const artist = first('artist') || (artists.length > 0 ? String(artists[0].id) : '');
  return {
    artist,
    label: first('label') || parentOf(feeds, artist, 'label'),
    hosting: first('hosting') || parentOf(feeds, artist, 'hosting'),
  };
}

// The values after the user selects an artist page (ADR 0240 item 15): the
// label page and the hosting page above that artist page, or "None".
export function moveForArtist(feeds, artistId) {
  return { artist: String(artistId), label: parentOf(feeds, artistId, 'label'), hosting: parentOf(feeds, artistId, 'hosting') };
}

// Can the user press Move? The artist page is necessary (ADR 0240 item 13).
export function moveReady(values) {
  return /^\d+$/.test(values.artist || '');
}

// The body of `POST /feeds/{id}/move-to` (ADR 0240 item 18).
export function moveBody(values) {
  const id = (value) => (/^\d+$/.test(value || '') ? Number(value) : null);
  return { artist_id: id(values.artist), label_id: id(values.label), hosting_id: id(values.hosting) };
}

// Does the release have a linked artist page? The section "Linked artist and
// label pages" says "A release needs an artist page." when it has none
// (ADR 0240 item 11).
export function hasArtistPage(feeds, release) {
  const linked = new Set((release && release.publisher_ids) || []);
  return (feeds || []).some((feed) => linked.has(feed.id) && (feed.publisher_roles || []).includes('artist'));
}

export const NEEDS_ARTIST_LINE = 'A release needs an artist page.';
