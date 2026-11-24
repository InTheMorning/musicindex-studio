// The result of a write to one feed (ADR 0217 items 2 to 4). `writtenIds` are
// the IDs of the pages that Review wrote after the feed (ADR 0231 item 8). A
// page that is written has no "also changed" line.
export function writeResult(feed, connector, changedPage = null, writtenIds = []) {
  const result = {
    title: 'Written',
    line: `${feed.title} is written to ${connector.name}.`,
  };
  const showPage = changedPage && !writtenIds.includes(changedPage.id);
  const withChangedPage = showPage ? { ...result, changedPage } : result;
  if (feed.public_url) return { ...withChangedPage, address: feed.public_url };
  return {
    ...withChangedPage,
    warning: {
      title: 'Listeners cannot get it yet.',
      text: connector.kind === 'local'
        ? 'This folder has no public address.'
        : 'This file host gives no public address.',
    },
  };
}

// The pages that Review of a new release writes with it (ADR 0231 items 7 to
// 10). The diff route gives `also_write`: each publisher feed that has a draft
// with changes and a link to the release. These functions are pure, so a test
// can prove the rows and the order of the writes.

// The check box rows. `also` is `diff.also_write`. `diffs` maps the ID of a
// page to its diff. `off` is the set of IDs that the user turned off. A page
// with no loaded diff has no row, because Studio cannot write it without that
// diff.
export const ALSO_WRITE_HINT = 'A link is written on its two feeds.';

// The text under a row (ADR 0251 item 5). `side` is the side of the row.
// A page row is in Review of a release, and a release row is in Review of a
// page.
function linkNote(page, title, reviewed, viaTitle) {
  const added = page.change !== 'removed';
  if (page.side === 'release') {
    return added
      ? `Adds ${reviewed} as the artist or label of ${title}.`
      : `Removes ${reviewed} from the artist or label of ${title}.`;
  }
  const place = page.via ? title : page.artist ? 'the artist page' : title;
  const target = page.via ? viaTitle : reviewed;
  return added
    ? `Adds the link to ${target} on ${place}.`
    : `Removes the link to ${target} from ${place}.`;
}

export function alsoRows(also, diffs, off, reviewTitle) {
  const reviewed = reviewTitle || 'this feed';
  return (also || [])
    .filter((page) => diffs && diffs[page.id])
    .map((page) => {
      const title = page.title || 'Untitled feed';
      const viaTitle = page.via && (also || []).find((entry) => entry.id === page.via)?.title;
      return {
        id: page.id,
        title,
        side: page.side,
        artist: page.artist,
        change: page.change,
        via: page.via ?? null,
        diff: diffs[page.id],
        checked: true,
        locked: true,
        label: `Also write ${title}`,
        note: linkNote(page, title, reviewed, viaTitle || 'Untitled feed'),
      };
    });
}

// The rows with the check box on, in the order of the list.
export function rowsToWrite(rows) {
  return rows.filter((row) => row.checked);
}

export function reviewRemoveLabel(draftState) {
  return draftState === 'new' ? 'Remove from Collection…' : 'Discard the Draft';
}

export function discardFailureTitle() {
  return 'Studio could not discard the draft.';
}

// The title of the error notice when the write of a page fails.
export function pageWriteFailed(title) {
  return `Studio could not write ${title}.`;
}

// Write the release first. Write each page after it, one by one, in order.
// `release` is an async function that gives the written feed. A page is
// `{ title, run }`, and `run` is an async function that gives its written
// feed. `onReleaseWritten(feed)` runs after the release write and before the
// first page write.
//
// When the release write fails, no page is written and the error goes up as
// it is. When a page write fails, the release stays written and the next
// pages are not written. The error gets the field `pageTitle`, so that
// Review can name the page. The function gives the written release and the
// written pages.
export async function writeInOrder({ before = [], release, pages, onReleaseWritten, onLinkedWritten }) {
  const beforeFeeds = [];
  for (const item of before) {
    try {
      const feed = await item.run();
      beforeFeeds.push(feed);
      if (onLinkedWritten) await onLinkedWritten(feed);
    } catch (e) {
      const error = e && typeof e === 'object' ? e : new Error(String(e));
      error.pageTitle = item.title;
      throw error;
    }
  }
  const feed = await release();
  if (onReleaseWritten) await onReleaseWritten(feed);
  const pageFeeds = [];
  for (const page of pages) {
    try {
      const feed = await page.run();
      pageFeeds.push(feed);
      if (onLinkedWritten) await onLinkedWritten(feed);
    } catch (e) {
      const error = e && typeof e === 'object' ? e : new Error(String(e));
      error.pageTitle = page.title;
      throw error;
    }
  }
  return { feed, beforeFeeds, pageFeeds };
}
