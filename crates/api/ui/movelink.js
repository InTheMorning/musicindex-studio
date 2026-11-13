// The texts of a refused write and of the self link notice in Review
// (ADR 0126 items 6, 10 and 20, ADR 0042). This module has no Preact and no
// browser state. The Node tests in `ui-tests/movelink.test.mjs` examine it.
// The core owns each condition, each hop result, each stage and each repair
// (ADR 0065). This module only gives the text of what the core sends.

import { localTime } from './outsidecheck.js';

// The text of each condition. The keys are the variants of `Condition` in
// `crates/core/src/move_link.rs`.
export const CONDITION_TEXT = {
  loop: 'The new addresses come back to an address that they visited, or go through more than 10 addresses.',
  read_fails: 'That address does not give a correct feed, or Studio cannot read it.',
  guid_mismatch: 'The feed at that address has a different podcast:guid, so it is a different feed.',
  conflicting_names: 'The feed names two different new addresses.',
  self_link_differs: 'That address is not the address of this feed.',
  not_public: 'That address is not a public address, so apps cannot read it.',
};

// Where the first address came from, and what can occur. The keys are the
// variants of `LinkSource` (ADR 0126 item 28).
export const SOURCE_TEXT = {
  tag: 'This feed says that it moved to a new address, in the tag itunes:new-feed-url.',
  self_link: 'The draft changes the address that this feed gives for itself, in its self link.',
  move: 'You selected a move of this feed to a new address.',
};
export const DANGER_TEXT = {
  tag: 'An app or an index that follows the tag would lose your feed.',
  self_link: 'An app or an index that follows the self link would lose your feed.',
  move: 'After a move, apps and indexes go to that address, so they would lose your feed.',
};
// What the user can do. The repair buttons give the first choice.
export const NEXT_TEXT = {
  tag: 'To continue, remove the tag, or put a feed at that address.',
  self_link: 'To continue, use the public address of this feed.',
  move: 'To continue, correct the public address of the file host, or undo the move.',
};

// The reason of each hop. The keys are the variants of `HopResult`.
export const HOP_TEXT = {
  feed: 'Gives a feed that names the next address.',
  revisited: 'Already in this list. This is a loop.',
  too_many_hops: 'The list has 10 addresses already. Studio did not read this address.',
  not_a_feed: 'Does not give an RSS feed.',
  gone: 'Gives the status 404 or 410. The file is not there.',
  server_error: 'Gives a status that is not a success.',
  unreachable: 'Studio could not connect.',
  too_large: 'The file is too large.',
  invalid_address: 'This is not an http or https address.',
  not_public: 'This is not a public address.',
  guid_mismatch: 'Has a different podcast:guid, or none.',
  names_differ: 'Names other addresses than the draft.',
  conflicting_names: 'Names two different new addresses.',
  differs_from_draft: 'The self link differs from the address of the feed. Studio did not read it.',
};

// What the stage says about the state of the hosts (ADR 0126 items 17 and 18).
export const STAGE_TEXT = {
  1: 'Nothing changed on a file host.',
  2: 'The feed stays at its address. The new file stays on the new file host, and the next write replaces it.',
};

// The label and the route of each repair (ADR 0126 item 20). The keys are
// the variants of `Repair`.
export const REPAIRS = {
  use_public_address: { label: 'Use the Public Address', route: 'use-public-address' },
  remove_tag: { label: 'Remove the Tag', route: 'remove-tag' },
};

export const REFUSAL_TITLE = 'Studio refuses this write.';
export const NOTICE_TITLE = 'The self link names a different address.';
export const NOTICE_WHY = 'The self link is where this feed says that it lives. The draft did not change it. An app that follows it can read the file at that address, not this feed.';
export const NOTICE_END = 'Studio can still write the feed. To correct the self link, use the public address.';
export const REPLACES_POINTER_NOTE = 'The pointer file at the new address becomes the feed.';

// The warning of a return to an earlier address (ADR 0128 items 7 and 8, ADR
// 0190 items 2 to 5). The text comes from the state of the route
// `GET /feeds/{id}/return-wait`. Stophammer owns the values of the wait, so
// the text gives no table of waits.
export const RETURN_TITLE = 'This is a return to an earlier address.';
export const RETURN_FOLLOWS = 'Music Index follows this return at once. Each return in one day makes the wait for the next return longer.';
export const RETURN_UNKNOWN = 'Music Index can wait before it follows a return, and the wait grows with each return. After the wait, send the podping again.';
export const RETURN_TOO_FAR = 'Send the podping again after that time.';

// The text of a wait until a Unix time (ADR 0190 item 2).
export function returnWaitsText(until) {
  return `Music Index waits until ${localTime(until)} before it follows this return. Studio sends the podping after that time.`;
}

// The warning for a feed move of the core and the state of the wait, or
// `null` when the move is not a return. With no state, or a state that this
// module does not know, the warning gives the text of ADR 0190 item 4. Only
// "follows at once" has the plain tone (ADR 0191 items 2 and 3).
export function returnWarning(move, wait) {
  if (!move || !move.replaces_pointer) return null;
  if (wait && wait.state === 'waits' && Number.isFinite(wait.until)) {
    const text = returnWaitsText(wait.until);
    return { title: RETURN_TITLE, text: wait.too_far ? `${text} ${RETURN_TOO_FAR}` : text, tone: 'warning' };
  }
  if (wait && wait.state === 'follows') return { title: RETURN_TITLE, text: RETURN_FOLLOWS, tone: 'plain' };
  return { title: RETURN_TITLE, text: RETURN_UNKNOWN, tone: 'warning' };
}

const UNKNOWN_CONDITION = 'A link of the draft is not correct.';
const UNKNOWN_HOP = 'Studio cannot use this address.';

// The repair buttons for a list of repair names from the core. A name that
// this module does not know gives no button.
export function repairButtons(repairs) {
  return (repairs || []).filter((id) => Object.hasOwn(REPAIRS, id)).map((id) => ({ id, label: REPAIRS[id].label }));
}

// The path of the route of a repair, under `/feeds/{id}/draft/`.
export function repairRoute(id) {
  return REPAIRS[id].route;
}

// The text of a refusal of the core: the result of the check, or the
// `details` of the error `bad_move_link`. `null` when `refusal` is empty.
export function refusalView(refusal) {
  if (!refusal) return null;
  return {
    title: REFUSAL_TITLE,
    source: Object.hasOwn(SOURCE_TEXT, refusal.source) ? SOURCE_TEXT[refusal.source] : '',
    text: Object.hasOwn(CONDITION_TEXT, refusal.condition) ? CONDITION_TEXT[refusal.condition] : UNKNOWN_CONDITION,
    danger: Object.hasOwn(DANGER_TEXT, refusal.source) ? DANGER_TEXT[refusal.source] : '',
    next: Object.hasOwn(NEXT_TEXT, refusal.source) ? NEXT_TEXT[refusal.source] : '',
    state: Object.hasOwn(STAGE_TEXT, refusal.stage) ? STAGE_TEXT[refusal.stage] : '',
    hops: (refusal.hops || []).map((hop) => ({
      url: hop.url,
      finalUrl: hop.final_url || null,
      why: Object.hasOwn(HOP_TEXT, hop.result) ? HOP_TEXT[hop.result] : UNKNOWN_HOP,
    })),
    repairs: repairButtons(refusal.repairs),
  };
}

// The refusal in the error of a write, or `null` for any other error.
export function refusalOfError(error) {
  return error && error.code === 'bad_move_link' && error.details ? error.details : null;
}

// The text of each notice of the check (ADR 0126 item 6). A notice that
// this module does not know gives no text.
export function noticeViews(report) {
  if (!report) return [];
  return (report.notices || [])
    .filter((notice) => notice.notice === 'self_link_differs')
    .map((notice) => ({
      title: NOTICE_TITLE,
      selfUrl: notice.self_url,
      draftUrl: notice.draft_url,
      why: NOTICE_WHY,
      end: NOTICE_END,
      repairs: repairButtons((report.repairs || []).filter((id) => id === 'use_public_address')),
    }));
}

// True when the check of the core gives a refusal. The write button waits.
export function blocksWrite(report) {
  return !!(report && report.refusal);
}
