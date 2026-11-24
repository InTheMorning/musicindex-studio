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

// The text of a feed with a show grant (item 20, ADR 0255 item 51).
export const CHECK_GRANT = 'This feed has a show grant. A granted change waits while the draft has changes. A repair adds a change to the draft.';

// The notice of an address that a read did not reach (item 16).
export const CHECK_UNREADABLE = 'Studio could not read this address.';

// The text of the page before the first answer.
export const CHECK_READING = 'Studio is reading the addresses of this feed.';

const LEVELS = { stop: 'Stops the write', warning: 'Warning', notice: 'Notice', question: 'Question' };

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
  add_https: 'Add https://',
  read_the_size: 'Read the Size',
  open_the_track: 'Open the Track',
  select_a_language: 'Select a Language',
  link_a_page: 'Link a Page',
  make_it_the_artist_link: 'Make It the Artist Link',
  add_tracks: 'Add Tracks',
  add_a_cover: 'Add a Cover',
  add_a_recipient: 'Add a Recipient',
  use_lnurl_address: 'Use LNURL Address',
  use_node_address: 'Use Node Address',
  put_back_guid: 'Put Back',
  keep_new_guid: 'Keep the New GUID',
};

// What a button does. A repair calls the repair route. The two other
// buttons only open a field, and they need no request.
const ACTIONS = {
  add_the_link: 'repair',
  use_the_public_address: 'repair',
  use_the_https_address: 'repair',
  add_https: 'repair',
  read_the_size: 'repair',
  open_the_track: 'open-track',
  select_a_language: 'open-language',
  link_a_page: 'open-pages',
  make_it_the_artist_link: 'repair',
  add_tracks: 'open-tracks',
  add_a_cover: 'open-cover',
  add_a_recipient: 'open-recipient',
  use_lnurl_address: 'repair',
  use_node_address: 'repair',
  use_the_guid_of: 'repair',
  put_back_guid: 'repair',
  keep_new_guid: 'repair',
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
export const NEW_PAGE_CHOICE = 'new';

// The refusal panel of Review (ADR 0218 items 1 to 6). `selected` is true
// after a selection in the menu. The last choice of the menu makes an artist
// page (ADR 0251 item 13).
export function pageRefusalView(reason, selected = false, noRoleTitle = null) {
  const title = PAGE_FINDING_TEXT[reason];
  if (!title) return null;
  return {
    title, ...PAGE_REFUSAL_TEXT[reason], placeholder: 'Select a page',
    button: 'Link and Continue', menu: true, linkDisabled: !selected,
    newChoice: 'New artist page…',
    noRole: noRoleTitle ? {
      line: `The link to ${noRoleTitle} has no role.`,
      button: REPAIRS.make_it_the_artist_link,
    } : null,
  };
}

// The Stop panel for a linked release (ADR 0251 items 10 and 11).
export function linkedRefusalView(row) {
  const text = {
    no_artist_page: {
      line: 'A release needs an artist page.', label: 'Artist page',
      title: `${row.title} has no other artist page.`,
    },
    no_publisher_page: {
      line: 'A release needs a page.', label: 'Page',
      title: `${row.title} has no other artist, label or hosting page.`,
    },
  }[row.refusal];
  if (!text) return null;
  return {
    ...text, placeholder: 'Select a page', menu: true,
    newChoice: 'New artist page…',
    link: 'Link and Continue', keep: 'Keep the Release',
    write: `Link a page to ${row.title} first.`,
  };
}

// The GUID change panel model (ADR 0249 item 3).
export function guidChangeView(change, typed = '') {
  if (!change) return null;
  const title = change.title || '';
  const expected = title.trim() ? title : 'CHANGE';
  const copy = {
    feed: {
      title: 'This write changes the GUID',
      lines: [`Payments from other music feeds that name ${change.known} stop.`, 'Apps can show the feed as a new show.'],
      keep: 'Keep the Old GUID',
    },
    replaced_file: {
      title: 'This move replaces a file with a different GUID',
      lines: [`The address served the GUID ${change.known}. It then serves ${change.new}.`, `Payments that name ${change.known} stop.`],
    },
    index_address: {
      title: 'Music Index has a different feed at this address',
      lines: [`Music Index holds ${change.known} at this address. This feed has ${change.new}.`, `Payments that name ${change.known} stop.`],
    },
  }[change.kind || 'feed'];
  return {
    title: copy.title,
    ...(change.kind === 'feed' || !change.kind ? { guids: `Known: ${change.known} → New: ${change.new}` } : {}),
    lines: copy.lines,
    ...(copy.keep ? { keep: copy.keep } : {}),
    prompt: `Type ${expected} to change the GUID`,
    confirmed: String(typed).trim() === expected,
  };
}

// The write line of the address panel (ADR 0245 item 7).
export const ADDRESS_WRITE_LINE = 'Repair these addresses first.';

const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;

// The address panel of a review (ADR 0245 items 6 and 7). `feeds` lists
// `{ feedId, title, check }` for each feed of the review. `check` is the
// answer of the check route. The core finds each address and gives the place
// in words. The function only reads the findings `short_address`. It gives
// null when no feed has one. `feeds` of the result has the data that
// `repairAddresses` needs.
export function addressPanelView(feeds) {
  const shown = [];
  for (const feed of feeds || []) {
    const findings = ((feed.check && feed.check.findings) || []).filter((finding) => finding.kind === 'short_address');
    if (findings.length === 0) continue;
    shown.push({
      feedId: feed.feedId,
      title: feed.title,
      revision: feed.check.revision,
      rows: findings.map((finding) => ({
        key: `${feed.feedId}|${findingKey(finding)}`,
        feedId: feed.feedId,
        feedTitle: feed.title,
        field: finding.field ?? null,
        place: finding.place,
        address: finding.url,
        newAddress: `https://${String(finding.url ?? '').trim()}`,
        body: { kind: finding.kind, entry: finding.entry ?? null, url: finding.url ?? null, ...(finding.field ? { field: finding.field } : {}) },
      })),
    });
  }
  if (shown.length === 0) return null;
  const n = shown.length;
  return {
    title: n === 1 ? '1 feed has addresses that are not full' : `${n} feeds have addresses that are not full`,
    button: `Add https:// in ${plural(n, 'Feed', 'Feeds')}`,
    write: ADDRESS_WRITE_LINE,
    feeds: shown,
    rows: shown.flatMap((feed) => feed.rows),
  };
}

// The repair of each listed address (ADR 0245 items 4 and 6). `call(feedId,
// body)` asks the repair route of the feed and gives its answer. Each answer
// has the check with the new revision for the next repair of the same feed.
// The result maps each feed ID to its last answer.
export async function repairAddresses(view, call) {
  const answers = new Map();
  for (const feed of view.feeds) {
    let revision = feed.revision;
    for (const row of feed.rows) {
      const answer = await call(feed.feedId, { ...row.body, revision });
      revision = answer.check.revision;
      answers.set(feed.feedId, answer);
    }
  }
  return answers;
}

// The People panel of a review (ADR 0245 items 11 and 13). `answer` is the
// answer of the People route of the review findings. `titleOf(feedId)` gives
// the title of a feed. A person with no finding is left out. The panel does
// not stop the write. The core finds each difference.
export function peoplePanelView(answer, titleOf = () => '') {
  const groups = ((answer && answer.people) || []).map((person) => {
    const entries = (person.feeds || []).filter((feed) => (feed.findings || []).length > 0)
      .map((feed) => ({ feedId: feed.feed_id, title: titleOf(feed.feed_id), entry: feed }));
    if (entries.length === 0) return null;
    return {
      personId: person.person_id,
      name: person.name,
      entries,
      feedIds: entries.map((entry) => entry.feedId),
      button: `Use the People Values in ${plural(entries.length, 'Feed', 'Feeds')}`,
    };
  }).filter(Boolean);
  if (groups.length === 0) return null;
  const n = new Set(groups.flatMap((group) => group.feedIds)).size;
  return { title: n === 1 ? '1 feed differs from People' : `${n} feeds differ from People`, groups };
}

// The label of the tab: "Issues (5)", or "Issues" for no finding and before
// the first answer (ADR 0228 item 13).
export function tabLabel(count) {
  return Number.isInteger(count) && count > 0 ? `Issues (${count})` : 'Issues';
}

// The count of the tab from an answer of the API, or 0 before the answer.
export function checkCount(check) {
  return check && Array.isArray(check.findings) ? check.findings.length : 0;
}

// The actions that open a field, a page or a sheet of the release (ADR 0219
// item 3). The shell does them for the checklist and for the Check page.
export const OPEN_ACTIONS = ['open-track', 'open-tracks', 'open-cover', 'open-recipient'];

// The four release checklist rows come from the findings of the draft
// (ADR 0219 items 6 to 8).
export function releaseChecklist(check) {
  if (!check || !Array.isArray(check.findings)) return null;
  const kinds = ['no_tracks', 'no_cover', 'no_audio_file', 'no_recipient', 'no_artist_page'];
  const findings = check.findings.filter((finding) => kinds.includes(finding.kind));
  if (findings.length === 0) return null;
  const first = (kind) => findings.find((finding) => finding.kind === kind);
  const noAudio = findings.filter((finding) => finding.kind === 'no_audio_file').length;
  const rows = [
    { label: 'Tracks', finding: first('no_tracks') },
    { label: 'Cover image', finding: first('no_cover') },
    { label: noAudio > 1 ? `${noAudio} tracks have no audio file` : 'Audio file of each track', finding: first('no_audio_file') },
    { label: 'Payment recipient', finding: first('no_recipient') },
    { label: 'Artist page', finding: first('no_artist_page') },
  ].map(({ label, finding }) => ({
    label,
    mark: finding ? 'cross' : 'check',
    // The row of the Check page for this finding, with its button and its
    // action, or null (ADR 0219 item 7).
    open: finding ? checkRows({ findings: [finding] })[0] : null,
  }));
  // The title counts the rows with a cross, not the findings: three tracks
  // with no audio file are one thing that the release needs.
  const count = rows.filter((row) => row.open).length;
  return { title: `This release needs ${count} ${count === 1 ? 'thing' : 'things'}`, rows };
}

// Addresses with a type finding get a warning mark in the payment page.
export function wrongAddressSet(check) {
  if (!check || !Array.isArray(check.findings)) return new Set();
  return new Set(check.findings.filter((finding) => finding.kind === 'wrong_address_type').map((finding) => finding.url));
}

// The button label of a repair, or null.
export function repairLabel(repair) {
  return REPAIRS[repair] || null;
}

// The key of a finding. The repair route finds the finding by its kind, its
// entry, its address and its field path (ADR 0245 item 2).
export function findingKey(finding) {
  return [finding.kind, finding.entry ?? '', finding.url ?? '', finding.field ?? ''].join('|');
}

// The rows of the findings. A row has no button when the feed is read-only
// (the core then gives no repair), when the finding has no repair, or when
// **Open the Track** has no track to open. `action` tells the page what the
// button does. `body` is what the repair route needs, without the revision.
export function checkRows(check) {
  if (!check || !Array.isArray(check.findings)) return [];
  return check.findings.map((finding, index) => ({ finding, index })).sort((a, b) => {
    const rank = { stop: 0, warning: 1, notice: 2, question: 3 };
    return (rank[a.finding.level] ?? 1) - (rank[b.finding.level] ?? 1) || a.index - b.index;
  }).map(({ finding }) => {
    if (finding.kind === 'unlisted_by_page') return {
      key: findingKey(finding), level: LEVELS[finding.level] || finding.level,
      warning: false, title: finding.title, reason: finding.reason,
      button: `Write ${finding.page_title}…`, action: 'write-page', pageId: finding.page_id,
    };
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
    const label = finding.repair === 'put_back_guid'
      ? `${REPAIRS.put_back_guid} ${finding.known_short || ''}`.trim()
      : finding.repair === 'use_the_guid_of' && finding.target_title
      ? `Use the GUID of ${finding.target_title}`
      : repairLabel(finding.repair);
    const action = label ? ACTIONS[finding.repair] : null;
    const usable = action && !(action === 'open-track' && !finding.entry);
    const secondLabel = finding.second_repair === 'put_back_guid'
      ? `${REPAIRS.put_back_guid} ${finding.known_short || ''}`.trim()
      : repairLabel(finding.second_repair);
    return {
      key: findingKey(finding),
      level: LEVELS[finding.level] || finding.level,
      warning: finding.level === 'warning',
      stop: finding.level === 'stop',
      title: finding.title,
      reason: finding.reason,
      button: usable ? label : null,
      action: usable ? action : null,
      track: usable && action === 'open-track' ? finding.entry : null,
      body: usable && action === 'repair'
        ? { kind: finding.kind, entry: finding.entry ?? null, url: finding.url ?? null, ...(finding.field ? { field: finding.field } : {}) }
        : null,
      secondButton: secondLabel ? secondLabel : null,
      secondAction: secondLabel ? ACTIONS[finding.second_repair] : null,
      secondBody: secondLabel ? { kind: finding.kind, repair: finding.second_repair, entry: finding.entry ?? null, url: finding.url ?? null, ...(finding.field ? { field: finding.field } : {}) } : null,
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
