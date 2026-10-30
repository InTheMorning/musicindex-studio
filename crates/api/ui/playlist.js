// The playlist models (ADR 0053 items 1, 4, 6 and 7, ADR 0042). This module
// has no Preact and no browser state. The Node tests in
// `ui-tests/playlist.test.mjs` examine it.
//
// A remote item of the draft model has the fields `feed_guid`, `item_guid`,
// `feed_url`, `medium` and `title`, and the entry key `_cid`. A pick has the
// fields `feed_guid`, `item_guid`, `feed_url` and `title`. Each function
// keeps the other fields of each entry, such as `_cid`.

// The key of a track: the feed GUID and the item GUID. It works for a remote
// item and for a pick.
export function pickKey(track) {
  return `${track.feed_guid} ${track.item_guid}`;
}

// A pick from the response of `GET /api/v1/musicindex/feeds/{guid}/tracks`
// and one entry of its `tracks`. The item GUID is the `track_guid` of the
// entry.
export function pickFromIndex(feed, track) {
  return {
    feed_guid: feed.feed_guid,
    item_guid: track.track_guid,
    feed_url: feed.feed_url,
    title: track.title,
  };
}

// A remote item from a pick. A missing `feed_url` or `title` stays out of
// the object. It has no entry key. The page gives the key with `withCids`.
export function remoteItemFromPick(pick) {
  const item = { feed_guid: pick.feed_guid, item_guid: pick.item_guid };
  if (pick.feed_url) item.feed_url = pick.feed_url;
  item.medium = 'music';
  if (pick.title) item.title = pick.title;
  return item;
}

// Does the list have a track with the key of this pick?
export function hasTrack(remoteItems, pick) {
  const key = pickKey(pick);
  return remoteItems.some((item) => pickKey(item) === key);
}

// A new list with the new remote item last. When the list has the same key,
// the list stays as it is.
export function addTrack(remoteItems, item) {
  return hasTrack(remoteItems, item) ? remoteItems : [...remoteItems, item];
}

// A new list without the entry at `index`.
export function removeTrack(remoteItems, index) {
  return remoteItems.filter((_, i) => i !== index);
}

// The row of a track: its title, and the feed GUID and the item GUID.
export function trackRow(remoteItem) {
  return {
    title: remoteItem.title || 'Untitled track',
    sub: `${remoteItem.feed_guid} · ${remoteItem.item_guid}`,
  };
}

// The summary line of the tracks.
export function tracksSummary(remoteItems) {
  const n = remoteItems.length;
  if (n === 0) return 'No tracks';
  return n === 1 ? '1 track' : `${n} tracks`;
}
