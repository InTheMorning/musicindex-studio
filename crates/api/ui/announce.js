// The announcement models (ADR 0255 items 19, 20, 25 and 34, ADR 0042). This
// module has no Preact and no browser state. The Node tests in
// `ui-tests/announce.test.mjs` examine it.

import { localTime } from './outsidecheck.js';

// The hint below the announcement choice of Review. `wait` is the answer of
// `GET /feeds/{id}/return-wait`, or `null`. With a wait of Music Index, the
// write sends the podping 30 seconds after the wait (ADR 0190 items 6 and 7).
export function announceHint(enabled, wait) {
  if (!enabled) return 'No index learns about this write until it reads the feed by itself.';
  if (wait && wait.state === 'waits' && Number.isFinite(wait.until) && !wait.too_far) {
    return `Studio sends it at ${localTime(wait.until + 30)}, after the wait of Music Index.`;
  }
  return 'Studio sends it 30 seconds after the write.';
}

// The label of the check box in the footer of the review sheet (ADR 0255
// item 19).
export const ANNOUNCE_FOOTER_LABEL = 'Send a podping';

// The cause of no podping, or null. The footer of the review sheet shows it
// in the position of the check box (ADR 0255 item 19). `diff` is the diff of
// the API.
export function announceSkipLine(diff) {
  const skip = diff ? diff.announce_skip : null;
  if (skip === 'no_address') return 'No podping: this feed has no public address.';
  if (skip === 'feed_deleted') return 'No podping: this write deletes the feed file.';
  if (skip === 'feed_unchanged') return 'No podping: this write does not change the feed.';
  return null;
}

// The text of one announcement. `timeText` turns seconds after the Unix
// epoch into a local time text.
function entryText(entry, timeText) {
  switch (entry.state) {
    case 'waiting':
    case 'sending':
      if (!entry.attempts) return `Waiting. Studio sends it at ${timeText(entry.next_at)}.`;
      return `Waiting. Studio tries again at ${timeText(entry.next_at)}. Last error: ${entry.last_error}`;
    case 'sent':
      return `Sent at ${timeText(entry.finished_at)}.`;
    case 'failed':
      return `Failed at ${timeText(entry.finished_at)}: ${entry.last_error}`;
    default:
      return '';
  }
}

// The text of the last announcement of a feed. `entries` is the list of
// `GET /feeds/{id}/announcements`, the newest first. A write can have two
// addresses. Then the text has one line for each address of the last write,
// the newest first, and each line starts with its address.
export function announceStatus(entries, timeText) {
  if (!Array.isArray(entries) || entries.length === 0) return 'No announcement yet.';
  // The entries of one write have the same creation time.
  const newest = entries[0];
  const same = entries.filter((entry) => entry.created_at === newest.created_at);
  if (same.length === 1) return entryText(newest, timeText);
  return same.map((entry) => `${entry.url}: ${entryText(entry, timeText)}`).join('\n');
}

// True when the newest announcement is not final. The feed page then loads
// the list again.
export function isWaiting(entries) {
  if (!Array.isArray(entries) || entries.length === 0) return false;
  return entries[0].state === 'waiting' || entries[0].state === 'sending';
}
