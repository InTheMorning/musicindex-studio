// The models of the sheets of a page action (ADR 0254 items 22 to 28), of the
// gather sheet and of the release credit fields. The functions have no Preact
// and no browser state. `editor.js` shows the sheets.
import { isPublisher, roleWord, headingIndex, placeText, feedKind } from './shell.js';

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

// The IDs of the pages above `page`, at any depth.
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

// The sheets of a page action (ADR 0254 items 22 to 28). Each action has the
// question, the button, the role of the pages in the list, the list title and
// the hint. The `action` is the name that the route of packet 251c reads.
const PAGE_SHEETS = {
  sign_to_label: {
    question: (name) => `Which label releases ${name}?`, button: 'Change Label', pick: 'label', search: 'Search for a label',
    list: 'Releases on the new label', none: 'Select a label.', keeps: 'label',
  },
  change_label: {
    question: (name) => `Which label releases ${name}?`, button: 'Change Label', pick: 'label', search: 'Search for a label',
    list: 'Releases on the new label', none: 'Select a label.', keeps: 'label',
  },
  go_independent: {
    question: (name) => `Make ${name} independent`, button: 'Go Independent', pick: null, search: null,
    list: 'Releases with no label after the change', none: '', keeps: 'label',
  },
  change_publisher: {
    question: (name) => `Who publishes ${name}?`, button: 'Change Publisher', pick: 'hosting', search: 'Search for a publisher',
    list: 'Releases that change publisher', none: 'Select a publisher.', keeps: 'publisher',
  },
};

// The action names that a sheet can have.
export const PAGE_ACTION_NAMES = Object.keys(PAGE_SHEETS);

// The rank of a page, from `headingIndex`: 1 is a label and 3 is an artist.
const LABEL_RANK = 1;
const ARTIST_RANK = 3;

function hasRole(feed, role) {
  return (feed.publisher_roles || []).includes(role);
}

// The actions of the menu "More" of a page (ADR 0254 item 23), without
// "Open". A publisher and a curator have none. The menu shows an entry with
// `id` and `label`. "Choose a publisher…" and "Change publisher…" have the
// same action. The core decides each rule of the change (ADR 0065). This
// function reads only the pages that are above the page.
export function pageActions(page, feeds) {
  const rank = headingIndex(page);
  if (rank !== ARTIST_RANK && rank !== LABEL_RANK) return [];
  const above = [...ancestors(feeds, page)].map((id) => (feeds || []).find((feed) => feed.id === id)).filter(Boolean);
  const publisher = {
    id: 'change_publisher',
    label: above.some((feed) => hasRole(feed, 'hosting')) ? 'Change publisher…' : 'Choose a publisher…',
  };
  if (rank === LABEL_RANK) return [publisher];
  return (page.parent_ids || []).some((id) => hasRole((feeds || []).find((feed) => feed.id === id) || {}, 'label'))
    ? [{ id: 'change_label', label: 'Change label…' }, { id: 'go_independent', label: 'Go independent…' }]
    : [{ id: 'sign_to_label', label: 'Sign to a label…' }, publisher];
}

// The texts of the sheet of one action, or null for an unknown action.
export function pageSheet(page, action) {
  const sheet = PAGE_SHEETS[action];
  if (!sheet) return null;
  return {
    action,
    title: sheet.question(pageTitle(page)),
    button: sheet.button,
    role: sheet.pick,
    searchLabel: sheet.search,
    listTitle: sheet.list,
    noneHint: sheet.none,
    keeps: sheet.keeps,
  };
}

// The line "Now" of the sheet: the place of the page in music words. The
// preview route gives this line, and this text shows until its answer. A
// page with the role hosting and no publisher above it publishes its own
// releases, as in the place text of the core.
export function pageNow(page, feeds) {
  return placeText(page, feeds) || (hasRole(page, 'hosting') ? `Published by ${pageTitle(page)}` : 'Not published');
}

const countWords = (n, one, many) => `${n} ${n === 1 ? one : many}`;

// The rows of the list of pages (ADR 0254 item 25), A to Z. `query` is the
// text of the search field. A publisher question starts with the row "Not
// published", which has `id` null. The page above the page has `now`. The
// pages below the page and the page itself are not in the list.
export function pageChoiceRows(feeds, page, action, query = '') {
  const sheet = PAGE_SHEETS[action];
  if (!sheet || sheet.pick === null) return [];
  const above = ancestors(feeds, page);
  const text = query.trim().toLowerCase();
  const rows = pagesOf(feeds, sheet.pick)
    .filter((feed) => feed.id !== page.id && !ancestors(feeds, feed).has(page.id))
    .filter((feed) => !text || pageTitle(feed).toLowerCase().includes(text))
    .map((feed) => ({
      id: feed.id,
      title: pageTitle(feed),
      dot: feedKind(feed).dot,
      sub: sheet.pick === 'label' ? pageNow(feed, feeds) : countWords((feed.child_ids || []).length, 'page', 'pages'),
      now: above.has(feed.id),
    }));
  if (sheet.pick !== 'hosting') return rows;
  // A page with the role hosting publishes its own releases.
  const none = !hasRole(page, 'hosting') && !(feeds || []).some((feed) => above.has(feed.id) && hasRole(feed, 'hosting'));
  const label = 'Not published';
  return !text || label.toLowerCase().includes(text) ? [{ id: null, title: label, dot: null, sub: '', now: none }, ...rows] : rows;
}

// The releases of the page, with their check marks (ADR 0254 item 25).
export function pageReleaseRows(feeds, page, checked) {
  const byId = new Map((feeds || []).map((feed) => [feed.id, feed]));
  return (page.release_ids || []).map((id) => ({
    id,
    title: byId.has(id) ? pageTitle(byId.get(id)) : 'Untitled release',
    checked: checked.includes(id),
  }));
}

// The path and the query of the preview route (packet 251c). `releases` is
// `undefined` for the start selection, or a list of IDs.
export function previewPath(page, action, { selected = null, above = null, releases = undefined } = {}) {
  const query = [`action=${action}`];
  if (selected !== null && selected !== undefined) query.push(`page=${selected}`);
  if (above !== null && above !== undefined) query.push(`above=${above}`);
  if (releases !== undefined) query.push(`releases=${releases.join(',')}`);
  return `/feeds/${page.id}/page-change?${query.join('&')}`;
}

// The body of the change route (packet 251c). `selected` is `undefined` when
// the user selected no row, and `null` for "Not published".
export function pageChangeBody(action, selected, releases) {
  return { action, page: selected ?? null, releases };
}

// The start selection that the preview gives (ADR 0254 item 28).
export function startSelection(preview) {
  return [...(preview?.start_selection || [])];
}

// The drop targets follow the row rank (ADR 0254 items 38 and 39).
// The parent row and each page below the dragged page take no drop.
export function pageDropRows(feeds, page, above = null) {
  const rank = headingIndex(page);
  if (!isPublisher(page) || ![ARTIST_RANK, LABEL_RANK].includes(rank)) return [];
  const labelAction = pageActions(page, feeds).find((entry) => ['change_label', 'sign_to_label'].includes(entry.id))?.id;
  const actions = [...(labelAction ? [labelAction] : []), 'change_publisher'];
  return actions.flatMap((action) => pageChoiceRows(feeds, page, action)
    .filter((row) => row.id !== null && row.id !== above)
    .filter((row) => headingIndex(feeds.find((feed) => feed.id === row.id)) === (action === 'change_publisher' ? 0 : LABEL_RANK))
    .map((row) => ({ id: row.id, action })));
}

// A drop changes immediately only if the preview selects each release and shows a change.
// The core gives the selection and decides if anything changes (ADR 0065).
export function pageDropDecision(page, preview) {
  const selected = new Set(startSelection(preview));
  return preview?.result?.nothing_changes === false && (page.release_ids || []).every((id) => selected.has(id))
    ? 'change' : 'sheet';
}

// The top area removes the label and publisher with each release selected.
export function leaveAllBody(page) {
  return pageChangeBody('leave_all', null, [...(page.release_ids || [])]);
}

function lowerFirst(text) {
  return text ? `${text[0].toLowerCase()}${text.slice(1)}` : text;
}

function andNames(names) {
  return names.length < 2 ? names.join('') : `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}

// The result part of the sheet (ADR 0254 item 25). The texts come from the
// preview that the route gives. No link rule runs here (ADR 0065).
// `selected` is `undefined` when the user selected no row. The result has
// `hint` or the lines `after`, `changes` and `keeps`, and `ready` for the
// button.
export function pageSheetResult(page, action, selected, preview) {
  const sheet = PAGE_SHEETS[action];
  const none = { hint: null, after: null, changes: null, keeps: null, ready: false };
  if (!sheet) return none;
  if (sheet.pick !== null && selected === undefined) return { ...none, hint: sheet.none };
  if (!preview) return none;
  const result = preview.result;
  if (result.nothing_changes) return { ...none, hint: 'Nothing changes. Select a different one or a release.' };
  const kept = result.kept_release_titles || [];
  return {
    hint: null,
    after: `After: ${pageTitle(page)} is ${lowerFirst(result.place)}.`,
    changes: (page.release_ids || []).length > 0 ? `Changes ${countWords(result.changed_releases, 'release', 'releases')}.` : null,
    keeps: kept.length > 0 ? `${andNames(kept)} ${kept.length === 1 ? 'keeps its' : 'keep their'} ${sheet.keeps}.` : null,
    ready: true,
  };
}

// The text of the notice after a change (ADR 0254 item 35).
export function pageChangeNotice(page, change) {
  return `${pageTitle(page)} is now ${lowerFirst(change.place)}. Not written yet.`;
}

// The Add sheet shows writable pages and starts with no selection (ADR 0254 items 18 to 21).
// Each page below the sheet page has no row, because the core refuses it.
export function gatherPagesModel(feeds, level, role, selected = [], query = '') {
  const word = role === 'artist' ? 'Artist' : 'Label';
  const signing = headingIndex(level) === LABEL_RANK;
  const text = query.trim();
  const rows = (feeds || []).filter((feed) => isPublisher(feed) && feed.connector_id
    && ((feed.publisher_roles || []).length === 0 || (feed.publisher_roles || []).includes(role))
    && !ancestors(feeds, feed).has(level.id) && !(level.child_ids || []).includes(feed.id) && feed.id !== level.id)
    .sort((a, b) => pageTitle(a).localeCompare(pageTitle(b)))
    .map((feed) => ({
      id: feed.id,
      title: pageTitle(feed),
      releases: (feed.release_ids || []).length,
      role: (feed.publisher_roles || []).length ? feedKind(feed).label : 'No role',
      place: pageNow(feed, feeds),
      checked: (selected || []).includes(feed.id),
      noRole: (feed.publisher_roles || []).length === 0,
    }));
  const count = rows.filter((row) => row.checked).length;
  return {
    title: `${signing ? 'Sign an artist to' : `Publish ${role === 'artist' ? 'an artist' : 'a label'} on`} ${pageTitle(level)}`,
    rows: rows.filter((row) => row.title.toLowerCase().includes(text.toLowerCase())),
    selectedIds: rows.filter((row) => row.checked).map((row) => row.id),
    count,
    button: signing ? 'Sign' : 'Publish',
    ready: count > 0,
    note: rows.some((row) => row.checked && row.noRole) ? `A page with no role gets the role ${word}.` : null,
    result: count ? `All releases of the selected ${role === 'artist' ? 'artists' : 'labels'} ${signing ? 'then come out on' : 'are then published by'} ${pageTitle(level)}.` : null,
    hint: count ? `For only some releases, use ${signing ? 'Change label…' : 'Change publisher…'} in the ⋯ menu of the ${role}.` : null,
    newLabel: `＋ Make a new ${role} page${text ? ` "${text}"` : ''}`,
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

// The Catalog count and Review lines use only the values of the API.
export function catalogChangeCount(set) {
  return set?.open && set.operation?.kind === 'catalog' ? `${countWords(set.catalog_steps, 'change', 'changes')} not written` : null;
}

export function catalogReview(set) {
  if (set?.operation?.kind !== 'catalog') return null;
  return {
    title: 'What changes',
    feedsTitle: 'Feeds that Studio writes',
    rows: (set.catalog_changes || []).map((row) => ({
      id: row.feed_id, title: row.title || 'Untitled feed',
      // A page that is only a publisher has no place (ADR 0254 item 36).
      now: row.now == null ? null : `Now: ${row.now}`, after: row.after == null ? null : `After: ${row.after}`,
    })),
  };
}

// The Add notice uses the places of the API for the selected pages.
export function gatherChangeNotice(answer, ids) {
  return (answer.change_set?.catalog_changes || []).filter((row) => ids.includes(row.feed_id))
    .map((row) => pageChangeNotice({ title: row.title }, { place: row.after })).join(' ');
}
