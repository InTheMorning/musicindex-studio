// The display of the facts of a feed (ADR 0042). This module has no Preact and
// no browser state. The Node tests in `ui-tests/feedfacts.test.mjs` examine it.

export const LOCK_HELP = 'This asks other podcast platforms not to import this feed. If you move the feed to a platform that imports it, turn this off first. A platform can ignore this request, and apps can still read the feed.';

// A version marker of a file host, in a short form. An S3 or R2 ETag has
// quote marks, and a weak ETag also has a prefix.
export function versionLabel(version) {
  return String(version).replace(/^W\//, '').replaceAll('"', '').slice(0, 19);
}

// An address with a space (ADR 0023 item 9). The core makes the correction.
export const hasSpace = (url) => typeof url === 'string' && url.includes(' ');
export const ROW_SPACE_NOTE = 'This address has a space.';

// The notice at the top of a list of releases, for `count` such addresses.
export function spaceNotice(count) {
  return count === 1
    ? { text: '1 address has a space. Apps can fail on it.', action: 'Correct the Address' }
    : { text: `${count} addresses have a space. Apps can fail on them.`, action: `Correct All ${count}` };
}

// The tag `itunes:new-feed-url` of the feed file, for the Feed facts and the
// Files page (ADR 0021 item 4). The core compares the tag with the public
// address, and `address_differences` gives the result.
export function newFeedAddress(feed) {
  if (!feed.new_feed_url) return null;
  const elsewhere = (feed.address_differences || []).some((d) => d.source === 'new_feed_url');
  return {
    url: feed.new_feed_url,
    note: !elsewhere && feed.public_url ? 'This is the address of this feed.' : null,
  };
}

// The row "Library folder" of the Files page (ADR 0167 items 5 to 7, 11).
// `files` is the answer of the Files route. The row always has "Change…".
// It gives the default path and the relative path for "Move to Default" only
// when the folder is not at its default path. A server that holds no assets
// gives no folder, so no row.
export function libraryFolderRow(files) {
  if (!files || files.library_folder == null || files.default_library_folder == null || files.library_folder_is_default == null) return null;
  const moves = !files.library_folder_is_default;
  return {
    path: files.library_folder,
    defaultPath: moves ? files.default_library_folder : null,
    moveToDefault: moves ? files.default_library_folder_path : null,
  };
}
