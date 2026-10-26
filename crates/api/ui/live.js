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
