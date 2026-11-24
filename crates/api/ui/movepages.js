// The model of the sheet "Move {title}" (ADR 0240 items 12 to 17). The
// functions have no Preact and no browser state. `editor.js` shows the sheet.
import { isPublisher, roleWord } from './shell.js';

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

// Map the core credits to field words (ADR 0257). No link rule runs here.
export function releaseCreditFields(credits, feeds) {
  const followsLabel = credits.label !== null;
  const source = followsLabel ? 'label' : 'artist page';
  const followLabel = followsLabel ? 'Follow the Label Again' : 'Follow the Artist Page Again';
  const publisherHint = 'A release has one publisher. A music file calls the label "Publisher". Here the label goes in Label.';
  const menus = ['artist', 'label', 'hosting'].map((role) => {
    const key = role === 'hosting' ? 'publisher' : role;
    const selected = credits[key];
    const pages = pagesOf(feeds, role).map((page) => [String(page.id), pageTitle(page)]);
    const value = selected ? (selected.id === null ? 'external' : String(selected.id)) : '';
    if (selected && !pages.some(([id]) => id === value)) pages.unshift([value, selected.title, true]);
    const others = credits[`other_${key}s`] || [];
    return {
      role, label: roleWord(role), value,
      placeholder: role === 'artist' && !selected ? 'Select an artist page' : null,
      options: [...pages, ...(role === 'artist' ? [] : [['', role === 'label' ? 'Independent (no label)' : 'Not published']]),
        [`${NEW_PAGE_PREFIX}${role}`, `New ${roleWord(role)} Page…`]],
      hint: role === 'artist' ? 'One artist page. A second artist goes in the release alias, for now.'
        : role === 'hosting' ? (credits.publisher_follows
          ? `Follows the ${source}. ${publisherHint}`
          : 'You selected the publisher.') : null,
      otherHint: others.length ? `Also linked in the file: ${others.map((page) => page.title).join(', ')}.` : null,
    };
  });
  return {
    menus,
    follows: credits.follows?.id ?? null,
    followLabel: credits.publisher_follows ? null : followLabel,
    followDisabled: credits.follows !== null && credits.follows.id === null,
    alias: credits.alias || '',
    aliasOpen: credits.alias !== null,
    aliasPlaceholder: credits.artist?.title || '',
    aliasHint: `The artist name of this release in podcast apps and music files: “${credits.alias ?? credits.artist?.title ?? ''}”. The release stays in the catalog of the artist page above.`,
  };
}
