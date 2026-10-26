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
  if (status === 'pending') return { label: 'Go live', status: 'live' };
  if (status === 'live') return { label: 'End the show', status: 'ended' };
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

// The problems that stop a write (ADR 0054 item 4). The texts are the texts
// of `live::problems` in the core.
export function liveProblems(live) {
  const found = [];
  if (!live.start || !live.start.trim()) found.push('This show has no start time.');
  if (!live.content_links || live.content_links.length === 0) found.push('This show has no content link.');
  if (!['pending', 'live', 'ended'].includes(live.status)) {
    found.push('This show has a status that is not pending, live or ended.');
  }
  return found;
}

// The start time that the route needs, as ISO 8601 text, from the value of a
// `datetime-local` field. An empty or incorrect value gives null.
export function startToIso(value) {
  if (!value) return null;
  const time = new Date(value);
  return Number.isNaN(time.getTime()) ? null : time.toISOString();
}

// The value of a `datetime-local` field from ISO 8601 text, in local time.
// A text that is not a time gives an empty text.
export function isoToLocalInput(text) {
  if (!text) return '';
  const time = new Date(text);
  if (Number.isNaN(time.getTime())) return '';
  const pad = (n) => String(n).padStart(2, '0');
  return `${time.getFullYear()}-${pad(time.getMonth() + 1)}-${pad(time.getDate())}T${pad(time.getHours())}:${pad(time.getMinutes())}`;
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
export function grantSummary(grant, liveItem, now, timeText) {
  if (!grant) return 'No grant.';
  if (grant.ended_at !== null && grant.ended_at !== undefined) {
    return `The grant ended at ${timeText(grant.ended_at)}.`;
  }
  const limit = grantLimit(liveItem);
  if (limit !== null && limit <= now) return `The grant ended at ${timeText(limit)}.`;
  return limit === null
    ? 'Connected apps can start and end this show.'
    : `Connected apps can start and end this show until ${timeText(limit)}.`;
}

const WRITE_TEXT = {
  go_live: 'Went live',
  end: 'Ended the show',
  set_live_value: 'Set the live value',
  remove_live_value: 'Removed the live value',
};

// One granted write as a line: the app, the change and the time.
export function grantWriteText(write, timeText) {
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
      note: 'Connected apps cannot write to a host with My signer, because no person is present to sign.',
    };
  }
  return { disabled: false, note: null };
}
