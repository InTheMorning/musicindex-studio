// The check for a change from outside Studio (ADR 0150). This module has no
// Preact and no browser state. The core makes the comparison and gives the
// result, and these functions give only the words.

// The Releases page reads at most this many feeds at one time (ADR 0150
// item 11).
export const OUTSIDE_LIMIT = 4;

export const OUTSIDE_SETTING_LABEL = 'Check for outside changes';
export const OUTSIDE_SETTING_HINT = 'When a page opens, Studio reads the public address of the feed, and a feed page also asks musicindex. The file hosts and musicindex can see these requests. Studio writes nothing because of this check.';

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
    strong: 'musicindex reads this feed from a different address.',
    index: check.index_url,
    own: check.own_url,
    after: 'A different collection or a different tool can hold this feed.',
  };
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
