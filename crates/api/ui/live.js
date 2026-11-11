// The live item models (ADR 0054, ADR 0042). This module has no Preact and no
// browser state. The Node tests in `ui-tests/live.test.mjs` examine it.
//
// An entry of `live_items` in the draft model has `status`, `start`, `end`,
// `item` (the fields of an item, such as `title` and `guid`), `live_value`
// (`uri` and `protocol`, or null), `content_links` (each with `href` and
// `label`) and the entry key `_cid`.

// The key of a live item: its GUID, or its client ID when it has no GUID.
// The outline and the live item page use this key.
export function liveKey(live) {
  return live.item && live.item.guid ? live.item.guid : `cid-${live._cid}`;
}

// Is this live item the item of the key? The client ID stays the same when
// the GUID changes, so the old key continues to find the item.
export function isLiveItem(live, key) {
  return liveKey(live) === key || `cid-${live._cid}` === key;
}

// The class of the type dot of a live item. Red is only for live now. A
// show that is not live now uses the feed color (musicindex ADR 0012 item 3).
export function liveDot(status) {
  return status === 'live' ? 'live' : 'feed';
}

// The label of a status in the UI.
export function statusLabel(status) {
  switch (status) {
    case 'pending': return 'Planned';
    case 'live': return 'Live now';
    case 'ended': return 'Ended';
    default: return 'No status';
  }
}

// The next action of a status: the label of the button and the status that
// the status route sets (ADR 0054 item 5). An ended show has no action.
export function nextAction(status) {
  if (status === 'pending') return { label: 'Go Live', status: 'live' };
  if (status === 'live') return { label: 'End the Show', status: 'ended' };
  return null;
}

// Can the user add a recording? Only an ended show can have one (ADR 0054
// item 6).
export function canRecord(live) {
  return live.status === 'ended';
}

// Does the live item have a recording? A recording is an enclosure with an
// audio type. The enclosure of a stream has another type, such as
// application/x-mpegURL.
export function hasRecording(live) {
  const enclosure = live.item && live.item.enclosure;
  return Boolean(enclosure && enclosure.url && (enclosure.media_type || '').startsWith('audio/'));
}

// The summary line of a live item: the status and the start time.
// `timeText` is the start time in the language of the user. The caller gives
// it, so that this module has no date formatting that depends on the browser.
export function showSummary(live, timeText) {
  const label = statusLabel(live.status);
  return timeText ? `${label} · starts ${timeText}` : label;
}

// ---- Show grants (ADR 0054 items 11 to 17) ----

const DAY = 24 * 60 * 60;

// The time of an ISO 8601 text in seconds since the Unix epoch, or null.
// The text needs a zone, as in the core. A zone with no colon, such as
// `-0600`, gets a colon first.
function secondsOf(text) {
  if (typeof text !== 'string') return null;
  const value = text.trim();
  if (!/T.*(Z|[+-]\d{2}(:?\d{2})?)$/i.test(value)) return null;
  const fixed = value.replace(/([+-]\d{2})(\d{2})$/, '$1:$2').replace(/([+-]\d{2})$/, '$1:00');
  const ms = Date.parse(/[+-]\d{2}:\d{2}$|Z$/i.test(fixed) ? fixed : `${fixed}:00`);
  return Number.isNaN(ms) ? null : Math.floor(ms / 1000);
}

// The time when the grant of a live item ends: 24 hours after its `end`, or
// after its `start` when `end` is blank. It is null when a time cannot be
// read. The core has the same rule (ADR 0054 item 14).
export function grantLimit(liveItem) {
  const start = secondsOf(liveItem && liveItem.start);
  if (start === null) return null;
  const endText = liveItem.end && String(liveItem.end).trim();
  if (!endText) return start + DAY;
  const end = secondsOf(endText);
  return end === null ? null : end + DAY;
}

// Is the grant active at `now` (seconds)? It has no `ended_at`, and its
// limit is not past.
export function grantActive(grant, liveItem, now) {
  if (!grant || grant.ended_at !== null && grant.ended_at !== undefined) return false;
  const limit = grantLimit(liveItem);
  return limit === null || now < limit;
}

// The state of a grant in one sentence. `timeText` turns a time in seconds
// into text in the language of the user. The caller gives it, so that this
// module has no date formatting that depends on the browser.
export function grantSummary(grant, liveItem, now, timeText, appName = null) {
  if (!grant) return 'No grant.';
  const who = appName || 'The app';
  if (grant.ended_at !== null && grant.ended_at !== undefined) {
    return `The grant ended at ${timeText(grant.ended_at)}.`;
  }
  const limit = grantLimit(liveItem);
  if (limit !== null && limit <= now) return `The grant ended at ${timeText(limit)}.`;
  return limit === null
    ? `${who} can start and end this show.`
    : `${who} can start and end this show until ${timeText(limit)}.`;
}

const WRITE_TEXT = {
  go_live: 'Went live',
  end: 'Ended the show',
  set_live_value: 'Set the live value',
  remove_live_value: 'Removed the live value',
};

// What a refused change says: the change that did not happen.
const REFUSED_TEXT = {
  go_live: 'Not started',
  end: 'Not ended',
  set_live_value: 'Live value not set',
  remove_live_value: 'Live value not removed',
};

// The cause of a refused granted write (ADR 0110 item 43). The core gives
// the cause name.
const CAUSE_TEXT = {
  stream_not_live: 'the stream was not live',
  draft_with_changes: 'the feed had a draft with changes',
  grant_ended: 'the grant had ended',
  live_value_invalid: 'Studio refused the live value',
  vault_not_open: 'the grant vault did not open',
};

// One granted write as a line: the app, the change and the time. A refused
// write has a `cause`, and the line gives it (screen 11).
export function grantWriteText(write, timeText) {
  if (write.cause) {
    const what = REFUSED_TEXT[write.change] || 'Change not made';
    const why = CAUSE_TEXT[write.cause] || 'Studio refused the change';
    return `${write.app_name} · ${what}: ${why} · ${timeText(write.written_at)}`;
  }
  return `${write.app_name} · ${WRITE_TEXT[write.change] || write.change} · ${timeText(write.written_at)}`;
}

// ---- The grant check box and the host (ADR 0058 items 14 and 15) ----

// Does the host use "My signer"? An nsite or Blossom host signs with the
// signer of the user, or with a key in the vault. `connector` is a row of
// `/connectors`, and its `config` is JSON text. A host of another kind, a
// host without a row and a text that is not JSON give false.
export function hostUsesSigner(connector) {
  if (!connector || !['nsite', 'blossom'].includes(connector.kind)) return false;
  let config = connector.config;
  if (typeof config === 'string') {
    try { config = JSON.parse(config); } catch { return false; }
  }
  return Boolean(config) && config.signing !== 'vault';
}

// The state of the grant check box. A host with "My signer" cannot do a
// granted write, because no person is present to sign. The box is then
// disabled, and `note` gives the reason.
export function grantBoxState(feedHostSigner) {
  if (feedHostSigner) {
    return {
      disabled: true,
      note: 'Connected apps cannot write to a file host with My signer, because no person is present to sign.',
    };
  }
  return { disabled: false, note: null };
}

// The group of the live show page that shows a problem of the core (ADR 0173
// item 2): `show`, `stream`, `links` or `status`. A show with a recording has
// no Stream section, so its stream problem shows in the Status section
// (item 3).
const PROBLEM_GROUPS = { start: 'show', end: 'show', stream: 'stream', contentLink: 'links' };
export function problemGroup(problem, hasStreamSection) {
  const group = PROBLEM_GROUPS[problem.field] || 'status';
  return group === 'stream' && !hasStreamSection ? 'status' : group;
}

// ---- The stream type and the stream check (ADR 0162) ----

// The value of the "Other…" entry of the stream type menu. It is not a type.
export const OTHER_TYPE = '__other__';

// A type counts as selected by the user when it is not blank at page load
// (ADR 0162 item 5).
export function typeIsSelected(mediaType) {
  return Boolean(mediaType && mediaType.trim());
}

const sameType = (a, b) => a.trim().toLowerCase() === b.trim().toLowerCase();

// What the stream type menu shows. `menu` is the list that the core gives:
// `{ label, media_type }`. A blank type shows no choice. A type that is not
// in the menu shows as "Other…", and the text field shows its text. After
// the user picks "Other…" (`otherPicked`), the text field also shows when
// the text is blank. `value` is the value of the select.
export function typeMenuState(mediaType, menu, otherPicked = false) {
  const text = mediaType || '';
  if (!text.trim() && !otherPicked) return { value: '', showText: false, text };
  const entry = menu.find((option) => sameType(option.media_type, text));
  if (entry && !otherPicked) return { value: entry.media_type, showText: false, text };
  return { value: OTHER_TYPE, showText: true, text };
}

// The type after an address change or a check. The page fills the type only
// until the user selects one (ADR 0162 item 5). An empty type is not a
// selection, so it gets the fill also after an earlier selection. A
// candidate that is not there (an address with no known extension, a check
// with no type) leaves the type as it is.
export function filledType(current, selected, candidate) {
  if (!candidate || (selected && typeIsSelected(current))) return current;
  return candidate;
}

// The result line of a stream check (ADR 0162 item 1). `result` is the
// answer of the core: `{ live, server_type, fill_label, cause }`. The line
// names the type with the menu label that the core gives (item 10).
// `checkedAt` is the time of the check as text, such as "14:05". The warning
// line uses the cause that the core gives.
export function checkLine(result, checkedAt) {
  if (!result) return null;
  if (result.live) {
    const type = result.fill_label || result.server_type || 'type not given';
    return { warning: false, text: `Live · ${type} · checked ${checkedAt}` };
  }
  return { warning: true, text: result.cause || 'The stream is not live.' };
}

// Whether the core marked an address field as invalid.
export function hasBadAddress(bad, path) {
  return bad.some((entry) => entry.field === path);
}

// The type warning of a stream check (ADR 0162 item 9). The core gives the
// text when the type of the stream server does not agree with the stream
// type. A type that the user did not select gets the type of the server,
// so it gets no warning.
export function checkTypeWarning(result, selected) {
  if (!selected || !result || !result.type_warning || !result.fill_type) return null;
  return { text: result.type_warning, fillType: result.fill_type, fillLabel: result.fill_label || result.fill_type };
}
