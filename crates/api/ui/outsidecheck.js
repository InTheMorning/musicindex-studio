import { unlinkedReleases, otherFeeds } from './shell.js';
import { INDEX_READ_AGAIN_MS, INDEX_READ_AGAIN_MAX } from './releasemove.js';
import { musicindexFeedUrl } from './people.js';

// The check for a change from outside Studio (ADR 0150). This module has no
// Preact and no browser state. The core makes the comparison and gives the
// result, and these functions give only the words.

// The Releases page reads at most this many feeds at one time (ADR 0150
// item 11).
export const OUTSIDE_LIMIT = 4;

export const OUTSIDE_SETTING_LABEL = 'Check for outside changes';
export const OUTSIDE_SETTING_HINT = 'When a page opens, Studio reads the public address of the feed, and a feed page also asks Music Index. The file hosts and Music Index can see these requests. Studio writes nothing because of this check.';

// The mark on a tile or a row of Releases, or null (ADR 0150 items 2, 3 and 5).
// A different address at musicindex has no mark on the tile.
export function outsideMark(check) {
  if (!check || !check.state) return null;
  return check.state === 'moved'
    ? { text: 'Moved outside Studio', tone: 'warning' }
    : { text: 'Changed outside Studio', tone: 'warning' };
}

// The notice of the hosted file on a feed page, or null. A draft with edits
// keeps them: Read Again moves only a draft with no edits (ADR 0007 item 6).
export function outsideNotice(check) {
  if (!check || !check.state) return null;
  if (check.state === 'moved') {
    return {
      tone: 'warning',
      strong: 'A different Studio moved this feed.',
      text: 'New address',
      address: check.new_url,
      host: check.new_host,
      after: 'The old address has a pointer file.',
      difference: check.connected === true,
      follow: true,
      // A return writes a pointer file at the new address, so it needs a
      // connected file host there and here (ADR 0211 item 18).
      back: Boolean(check.new_host) && check.connected === true,
      keep: true,
      again: false,
    };
  }
  return {
    tone: 'warning',
    strong: 'The hosted file changed outside Studio.',
    text: 'Read Again takes the new file, and it keeps a draft with edits.',
    address: null,
    after: null,
    difference: false,
    keep: false,
    again: true,
  };
}

// The notice of a read-only feed that followed a move (ADR 0211 items 15
// and 16), or null. The API gives `followed_move` and `host_for_address`.
export function readOnlyHostNotice(feed) {
  if (!feed || feed.connector_id) return null;
  if (feed.host_for_address === true) {
    return {
      text: 'A connected file host gives the address of this feed. Use it to edit the feed here.',
      button: 'Use This File Host',
      use: true,
    };
  }
  if (feed.followed_move !== true) return null;
  return {
    text: 'This Studio is not connected to the file host of this address. Connect it to edit the feed here.',
    button: 'Connect a File Host…',
    use: false,
  };
}

// The notice of a different address at musicindex, or null (ADR 0150 item 5).
export function indexNotice(check) {
  if (!check || !check.index_url) return null;
  return {
    strong: 'Music Index reads this feed from a different address.',
    index: check.index_url,
    own: check.own_url,
    after: 'A different collection or a different tool can hold this feed.',
  };
}

// The release page link to Music Index (ADR 0220 items 1 to 3).
export function releaseMusicIndexLink(feed, check) {
  if (!feed || !feed.podcast_guid || !check || check.held !== true) return null;
  return { href: musicindexFeedUrl(feed.podcast_guid), text: 'Open on musicindex.org' };
}

// The local time of a Unix time in seconds. Review uses it too (ADR 0190
// item 2).
export const localTime = (seconds) => new Date(seconds * 1000).toLocaleString();

// The notice after a move by Studio (ADR 0184 items 3 to 7). A sent line can
// send again (ADR 0191 item 4). A feed with no public address has no send
// button (ADR 0184 item 11).
function podpingLines(announcements, ownUrl) {
  return (announcements || []).map((announcement) => {
    const line = { url: announcement.url, state: announcement.state, tag: '', time: null, text: '' };
    if (announcement.state === 'waiting') {
      line.tag = 'Waiting';
      line.time = localTime(announcement.send_at);
    } else if (announcement.state === 'sent') {
      line.tag = 'Sent';
      line.time = localTime(announcement.finished_at);
      if (ownUrl) line.action = 'Send the Podping Again';
    } else if (announcement.state === 'failed') {
      line.tag = 'Failed';
      line.time = localTime(announcement.finished_at);
      line.text = announcement.error || '';
      if (ownUrl) line.action = 'Send the Podping Again';
    } else if (announcement.state === 'not_sent') {
      line.tag = 'Not sent';
      if (ownUrl) line.action = 'Send the Podping';
    }
    return line;
  });
}

export function moveNotice(check) {
  if (!check || check.index_is_move_source !== true || !check.last_move) return null;
  const lines = podpingLines(check.last_move.announcements, check.own_url);
  return {
    tone: 'plain',
    strong: `You moved this feed at ${localTime(check.last_move.moved_at)}. Music Index reads the new address after it gets the podping.`,
    lines,
  };
}

// The age of a write, in seconds, after which the notice asks Music Index
// only one time for each load of the page (task 155b). The age starts at the
// write, or at the last Check Again (`since`) when that time is later.
export const WRITE_NOTICE_AGE = 300;
const writeAge = (write, nowSeconds, since = 0) => nowSeconds - Math.max(write.written_at, since);

// True when Music Index has not read a write with a podping (ADR 0175 item
// 1). The route gives `index_read` null also when the index does not hold the
// feed yet. After a podping, that is a write that Music Index did not read.
const writeNotRead = (write) => Boolean(write) && (write.index_read === false
  || (write.index_read === null && (write.announcements || []).length > 0));

// The notice after a write with a podping (ADR 0175 item 1). `kind` is
// 'waiting', 'confirmed' or 'no_reply'. The confirmation shows only when the
// page showed a different kind before (`confirmedHere`).
export function writeNotice(check, { reads, nowSeconds, confirmedHere, since = 0 }) {
  const write = check && check.last_write;
  if (!write || moveNotice(check)) return null;
  if (write.index_read !== true && !writeNotRead(write)) return null;
  const time = localTime(write.written_at);
  if (write.index_read === true) {
    return confirmedHere ? { kind: 'confirmed', tone: 'plain', strong: `Music Index has your write of ${time}.`, lines: [] } : null;
  }
  const lines = podpingLines(write.announcements, check.own_url);
  if (writeNotRead(write) && (reads >= INDEX_READ_AGAIN_MAX || writeAge(write, nowSeconds, since) > WRITE_NOTICE_AGE)) {
    return { kind: 'no_reply', tone: 'plain', strong: `Music Index has not read your write of ${time} yet.`, lines, action: 'Check Again' };
  }
  return {
    kind: 'waiting',
    tone: 'plain',
    strong: `You wrote this feed at ${time}. Music Index reads it after it gets the podping.`,
    lines,
  };
}

// The delay before the notice reads a waiting podping again (ADR 0189 item 6).
// A podping that is late, for example with no network, gets a load each 30
// seconds, not each second. After the podping, a write that Music Index did
// not read gets a load each 20 seconds, until the limit of the reads or the
// age limit of the write (ADR 0175 item 1).
export const LATE_NOTICE_LOAD = 30000;
export function nextNoticeLoad(check, nowSeconds, reads = 0, since = 0) {
  if (!check) return null;
  const waiting = [...(check.last_move?.announcements || []), ...(check.last_write?.announcements || [])]
    .filter((announcement) => announcement.state === 'waiting')
    .map((announcement) => announcement.send_at);
  if (waiting.length > 0) {
    const delay = (Math.min(...waiting) + 5 - nowSeconds) * 1000;
    return delay < 1000 ? LATE_NOTICE_LOAD : delay;
  }
  if (writeNotRead(check.last_write) && reads < INDEX_READ_AGAIN_MAX) {
    return writeAge(check.last_write, nowSeconds, since) > WRITE_NOTICE_AGE ? null : INDEX_READ_AGAIN_MS;
  }
  return null;
}

// Call `work` for each item, with at most `limit` calls at one time. A failed
// call does not stop the others.
export async function eachLimited(items, limit, work) {
  let next = 0;
  const runner = async () => {
    while (next < items.length) {
      const item = items[next];
      next += 1;
      try { await work(item); } catch (_) { /* A check never blocks a page. */ }
    }
  };
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, runner));
}

// The feeds that a row of the left panel covers (ADR 0165 item 3). A number
// is a publisher feed: the feed and each release of it. 'none' is "Not
// linked" and 'other' is "Other feeds". A feed that is not in the list is
// left out.
export function panelSet(feeds, group) {
  const list = Array.isArray(feeds) ? feeds : [];
  if (group === 'none') return unlinkedReleases(list);
  if (group === 'other') return otherFeeds(list);
  const byId = new Map(list.map((feed) => [feed.id, feed]));
  const owner = byId.get(group);
  if (!owner) return [];
  return [owner, ...(owner.release_ids || []).map((id) => byId.get(id)).filter(Boolean)];
}

// The marks of the left panel, from the most urgent (ADR 0165 item 2).
const STATE_MARKS = {
  outside: { icon: 'cloud-alert', tone: 'warning' },
  pending: { icon: 'pencil', tone: 'accent', text: 'Changes not written' },
  unchecked: { icon: 'circle-dashed', tone: 'muted', text: 'Not checked' },
};

// A feed is not checked when it has no result, or the read did not succeed
// (ADR 0165 item 2).
function isUnchecked(results, feed) {
  const check = results && results[feed.id];
  return !check || check.read !== true;
}

// The mark of a set of feeds, or null when each feed is up to date (ADR 0165
// items 2 and 3). `results` maps a feed ID to the answer of the check. The
// answer has `text`, `icon` and `tone`. A set with a moved feed says "Moved".
export function stateMark(feeds, results) {
  const list = Array.isArray(feeds) ? feeds : [];
  const states = list.map((feed) => results && results[feed.id] && results[feed.id].state);
  if (states.some((state) => state === 'moved' || state === 'changed')) {
    const moved = states.includes('moved');
    return {
      kind: 'outside',
      ...STATE_MARKS.outside,
      text: moved ? 'Moved outside Studio' : 'Changed outside Studio',
    };
  }
  const draft = (feed) => feed.draft_state === 'new' || feed.draft_state === 'changed';
  if (list.some(draft)) return { kind: 'pending', ...STATE_MARKS.pending };
  if (list.some((feed) => isUnchecked(results, feed))) return { kind: 'unchecked', ...STATE_MARKS.unchecked };
  return null;
}
