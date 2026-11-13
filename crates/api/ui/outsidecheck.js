import { unlinkedReleases, otherFeeds } from './shell.js';

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
    ? { text: 'Moved outside Studio', tone: 'error' }
    : { text: 'Changed outside Studio', tone: 'accent' };
}

// The notice of the hosted file on a feed page, or null. A draft with edits
// keeps them: Read Again moves only a draft with no edits (ADR 0007 item 6).
export function outsideNotice(check) {
  if (!check || !check.state) return null;
  if (check.state === 'moved') {
    return {
      tone: 'error',
      strong: 'This feed moved outside Studio.',
      text: 'The hosted file names a new address:',
      address: check.new_url,
      after: 'A write from here can put the feed back on the old file host.',
      difference: check.connected === true,
      keep: true,
      again: check.connected !== true,
    };
  }
  return {
    tone: 'plain',
    strong: 'The hosted file changed outside Studio.',
    text: 'Read Again takes the new file, and it keeps a draft with edits.',
    address: null,
    after: null,
    difference: false,
    keep: false,
    again: true,
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

// The local time of a Unix time in seconds. Review uses it too (ADR 0190
// item 2).
export const localTime = (seconds) => new Date(seconds * 1000).toLocaleString();

// The notice after a move by Studio (ADR 0184 items 3 to 7). A sent line can
// send again (ADR 0191 item 4). A feed with no public address has no send
// button (ADR 0184 item 11).
export function moveNotice(check) {
  if (!check || check.index_is_move_source !== true || !check.last_move) return null;
  const lines = (check.last_move.announcements || []).map((announcement) => {
    const line = { url: announcement.url, state: announcement.state, tag: '', time: null, text: '' };
    if (announcement.state === 'waiting') {
      line.tag = 'Waiting';
      line.time = localTime(announcement.send_at);
    } else if (announcement.state === 'sent') {
      line.tag = 'Sent';
      line.time = localTime(announcement.finished_at);
      if (check.own_url) line.action = 'Send the Podping Again';
    } else if (announcement.state === 'failed') {
      line.tag = 'Failed';
      line.time = localTime(announcement.finished_at);
      line.text = announcement.error || '';
      if (check.own_url) line.action = 'Send the Podping Again';
    } else if (announcement.state === 'not_sent') {
      line.tag = 'Not sent';
      if (check.own_url) line.action = 'Send the Podping';
    }
    return line;
  });
  return {
    tone: 'plain',
    strong: `You moved this feed at ${localTime(check.last_move.moved_at)}. Music Index reads the new address after it gets the podping.`,
    lines,
  };
}

// The delay before the notice reads a waiting podping again (ADR 0189 item 6).
// A podping that is late, for example with no network, gets a load each 30
// seconds, not each second.
export const LATE_NOTICE_LOAD = 30000;
export function nextNoticeLoad(check, nowSeconds) {
  if (!check || !check.last_move) return null;
  const waiting = (check.last_move.announcements || [])
    .filter((announcement) => announcement.state === 'waiting')
    .map((announcement) => announcement.send_at);
  if (waiting.length === 0) return null;
  const delay = (Math.min(...waiting) + 5 - nowSeconds) * 1000;
  return delay < 1000 ? LATE_NOTICE_LOAD : delay;
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
  outside: { icon: 'cloud-alert', tone: 'error' },
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
