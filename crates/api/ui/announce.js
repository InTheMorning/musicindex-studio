// The announcement models (ADR 0052 items 5, 6, 11 and 20, ADR 0042). This
// module has no Preact and no browser state. The Node tests in
// `ui-tests/announce.test.mjs` examine it.

// The lines of the review sheet for the announcements of a write. `diff` is
// the diff of the API. An entry of `diff.announcements` gives one line. When
// `diff.announce_skip` is set, one line gives the reason for no announcement.
export function announceLines(diff) {
  const lines = [];
  const entries = diff && Array.isArray(diff.announcements) ? diff.announcements : [];
  for (const entry of entries) {
    lines.push(entry.podping
      ? `Tell Podcast Index and send a podping: ${entry.url}`
      : `Tell Podcast Index: ${entry.url}`);
  }
  const skip = diff ? diff.announce_skip : null;
  if (skip === 'off') {
    lines.push('No announcement: the announcements are off in Settings.');
  } else if (skip === 'no_address') {
    lines.push('No announcement: this feed has no public address.');
  }
  return lines;
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
