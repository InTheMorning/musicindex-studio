// The model of the tab Check and its page (ADR 0174 items 17 and 19 to 21).
// The core gives each finding (ADR 0065). This module has no feed rule. It
// only turns the answer of the API into labels and rows.

import { grantActive } from './live.js';

// The text above the rows of findings (item 19).
export const CHECK_LEAD = 'Studio does not change the feed until you select a button. Each change goes to the draft, and you review it before the write.';

// The text of a page with no finding (item 21).
export const CHECK_NONE = 'Studio found no problem in this feed.';

// The text of a read-only feed (item 20). The feed has no file host, so no
// repair can reach a file.
export const CHECK_READ_ONLY = 'This feed has no file host, so Studio shows the findings and does not repair them.';

// The text of a feed with a show grant (item 20, ADR 0054 item 16).
export const CHECK_GRANT = 'This feed has a show grant. A granted change waits while the draft has changes. A repair adds a change to the draft.';

// The notice of an address that a read did not reach (item 16).
export const CHECK_UNREADABLE = 'Studio could not read this address.';

// The text of the page before the first answer.
export const CHECK_READING = 'Studio is reading the addresses of this feed.';

const LEVELS = { warning: 'Warning', notice: 'Notice', question: 'Question' };

// The texts of the sheet of **Show What to Do** (ADR 0175 item 8).
export const COPY_SHEET_TEXT = {
  title: 'A Copy Pays Other People',
  paragraphs: [
    'Music Index has a copy of this feed at a different address. The copy has the same GUID, and its payments go to other people.',
    'Only the operator of Music Index can remove the copy. Studio does not send this report. Copy it and send it to the operator.',
  ],
};

// The texts of the sheet of **Show the Ways** (ADR 0175 item 5).
export const WAYS_SHEET_TEXT = {
  title: 'Music Index Has Another Address',
  lead: 'Music Index reads this feed at',
  noPointer: 'Studio does not put a pointer file at that address. Use one of these ways.',
  ways: [
    { title: 'A redirect at the old host', text: 'Ask the old host for an HTTP 301 redirect from the old address to' },
    { title: 'A request to the operator of Music Index', text: 'Send the two addresses and the GUID to the operator of Music Index.' },
  ],
};

// The button of each repair, as the core names it. `move_not_confirmed` has
// no repair, so it has no button.
const REPAIRS = {
  add_the_link: 'Add the Link',
  use_the_public_address: 'Use the Public Address',
  use_the_https_address: 'Use the https Address',
  read_the_size: 'Read the Size',
  open_the_track: 'Open the Track',
  select_a_language: 'Select a Language',
  link_a_page: 'Link a Page',
  add_tracks: 'Add Tracks',
  add_a_cover: 'Add a Cover',
  add_a_recipient: 'Add a Recipient',
  use_lnurl_address: 'Use LNURL Address',
  use_node_address: 'Use Node Address',
};

// What a button does. A repair calls the repair route. The two other
// buttons only open a field, and they need no request.
const ACTIONS = {
  add_the_link: 'repair',
  use_the_public_address: 'repair',
  use_the_https_address: 'repair',
  read_the_size: 'repair',
  open_the_track: 'open-track',
  select_a_language: 'open-language',
  link_a_page: 'open-pages',
  add_tracks: 'open-tracks',
  add_a_cover: 'open-cover',
  add_a_recipient: 'open-recipient',
  use_lnurl_address: 'repair',
  use_node_address: 'repair',
};

export const PAGE_FINDING_TEXT = {
  no_artist_page: 'This release has no artist page.',
  no_publisher_page: 'This feed has no artist, label or hosting page.',
};

// The line, the menu label and the write line of each refusal (ADR 0218 item 2).
const PAGE_REFUSAL_TEXT = {
  no_artist_page: { line: 'Music Index lists each release under its artist.', label: 'Artist page', write: 'Link an artist page first.' },
  no_publisher_page: { line: 'Music Index lists each feed under its page.', label: 'Page', write: 'Link a page first.' },
};

// The refusal panel of Review (ADR 0218 items 1 to 6). `selected` is true
// after a selection in the menu. With no candidate page, the panel has the
// button that makes an artist page in place of the menu.
export function pageRefusalView(reason, selected = false, candidates = 1) {
  const title = PAGE_FINDING_TEXT[reason];
  if (!title) return null;
  return {
    title, ...PAGE_REFUSAL_TEXT[reason], placeholder: 'Select a page',
    button: candidates > 0 ? 'Link and Continue' : 'Make an Artist Page…',
    menu: candidates > 0, linkDisabled: candidates > 0 && !selected,
  };
}

// The label of the tab: "Check 5", or "Check" for no finding and before the
// first answer (ADR 0174 item 17).
export function tabLabel(count) {
  return Number.isInteger(count) && count > 0 ? `Check ${count}` : 'Check';
}

// The count of the tab from an answer of the API, or 0 before the answer.
export function checkCount(check) {
  return check && Array.isArray(check.findings) ? check.findings.length : 0;
}

// The button label of a repair, or null.
export function repairLabel(repair) {
  return REPAIRS[repair] || null;
}

// The key of a finding. The repair route finds the finding by its kind, its
// entry and its address.
export function findingKey(finding) {
  return [finding.kind, finding.entry ?? '', finding.url ?? ''].join('|');
}

// The rows of the findings. A row has no button when the feed is read-only
// (the core then gives no repair), when the finding has no repair, or when
// **Open the Track** has no track to open. `action` tells the page what the
// button does. `body` is what the repair route needs, without the revision.
export function checkRows(check) {
  if (!check || !Array.isArray(check.findings)) return [];
  return check.findings.map((finding, index) => ({ finding, index })).sort((a, b) => {
    const rank = { warning: 0, notice: 1, question: 2 };
    return (rank[a.finding.level] ?? 1) - (rank[b.finding.level] ?? 1) || a.index - b.index;
  }).map(({ finding }) => {
    if (finding.kind === 'copy_other_recipients') return {
      key: findingKey(finding), level: LEVELS[finding.level] || finding.level,
      warning: finding.level === 'warning', title: finding.title, reason: finding.reason,
      button: 'Show What to Do', action: 'show-copy', report: finding.report || '',
    };
    if (finding.kind === 'index_address_differs') return {
      key: findingKey(finding), level: LEVELS[finding.level] || finding.level,
      warning: false, title: finding.title, reason: finding.reason,
      button: 'Show the Ways', action: 'show-ways', indexAddress: finding.url || '',
    };
    const label = repairLabel(finding.repair);
    const action = label ? ACTIONS[finding.repair] : null;
    const usable = action && !(action === 'open-track' && !finding.entry);
    return {
      key: findingKey(finding),
      level: LEVELS[finding.level] || finding.level,
      warning: finding.level === 'warning',
      title: finding.title,
      reason: finding.reason,
      button: usable ? label : null,
      action: usable ? action : null,
      track: usable && action === 'open-track' ? finding.entry : null,
      body: usable && action === 'repair'
        ? { kind: finding.kind, entry: finding.entry ?? null, url: finding.url ?? null }
        : null,
    };
  });
}

// The notices of the addresses that a read did not reach (item 16).
export function unreadableRows(check) {
  if (!check || !Array.isArray(check.unreadable)) return [];
  return check.unreadable.map((url) => ({ key: url, level: 'Notice', title: url, reason: CHECK_UNREADABLE }));
}

// Does the feed have an active show grant? `grants` is the list of the
// route of the grants of a feed. `liveItems` is `model.live_items`. `now` is
// in seconds.
export function hasShowGrant(grants, liveItems, now) {
  return (grants || []).some((grant) => {
    const live = (liveItems || []).find((l) => l.item && l.item.guid === grant.item_guid);
    return grantActive(grant, live, now);
  });
}

// The texts of the page, in the order that the page shows them. `lead` is
// null when no row follows. `none` is the text of item 21. `reading` is true
// before the first answer.
export function checkTexts(check, { grant = false } = {}) {
  const rows = checkRows(check);
  const answered = Boolean(check);
  return {
    lead: rows.length > 0 && !check.read_only ? CHECK_LEAD : null,
    readOnly: answered && check.read_only ? CHECK_READ_ONLY : null,
    grant: grant && !(answered && check.read_only) ? CHECK_GRANT : null,
    none: answered && rows.length === 0 ? CHECK_NONE : null,
    reading: !answered ? CHECK_READING : null,
  };
}
