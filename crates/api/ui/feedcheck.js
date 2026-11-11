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

const LEVELS = { notice: 'Notice', question: 'Question' };

// The button of each repair, as the core names it. `move_not_confirmed` has
// no repair, so it has no button.
const REPAIRS = {
  add_the_link: 'Add the Link',
  use_the_public_address: 'Use the Public Address',
  use_the_https_address: 'Use the https Address',
  read_the_size: 'Read the Size',
  open_the_track: 'Open the Track',
  select_a_language: 'Select a Language',
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
};

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
  return check.findings.map((finding) => {
    const label = repairLabel(finding.repair);
    const action = label ? ACTIONS[finding.repair] : null;
    const usable = action && !(action === 'open-track' && !finding.entry);
    return {
      key: findingKey(finding),
      level: LEVELS[finding.level] || finding.level,
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
