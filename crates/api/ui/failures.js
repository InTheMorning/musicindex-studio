// The words for a feed that Studio could not read. The core gives the kind,
// and the UI gives the words, because a message never shows the raw text of
// a library (musicindex ADR 0023 R-MSG-1). It has no Preact and no browser
// state, so that the Node tests can examine it (ADR 0042).

// The text of each kind. It says what failed and what the user can do
// (R-MSG-1 item 2). `site` is the host of the address.
export const FAIL_TEXT = {
  not_a_feed: () => 'The address gives a page that is not a feed. The site may have moved the feed. Ask the owner of the site for its new address.',
  gone: () => 'The address gives no file. The site removed or moved the feed.',
  server_error: () => 'The site gave an error. Try again later.',
  unreachable: () => 'Studio could not connect to the site. Try again later.',
  not_public: (site) => `Studio on a server cannot connect to ${site || 'this address'}, because it is not a public address.`,
  too_large: () => 'The file is too large to be a feed.',
  invalid_address: () => 'The address is not a web address. It must start with https:// or http://.',
  unknown_guid: () => 'musicindex does not know this feed.',
  index_unreachable: () => 'Studio could not get the address of this feed from musicindex. Try again later.',
  bad_backup: () => 'The file is not an MSP 2.0 backup.',
};

// The host of an address, or null.
function hostOf(address) {
  try {
    return new URL(address).host || null;
  } catch {
    return null;
  }
}

// One line for a failure: the name of the feed, its site and the text.
// The name is the title, or else the address, or else the source.
export function failedLine(failed) {
  const site = hostOf(failed.url || failed.source);
  const text = (FAIL_TEXT[failed.kind] || (() => 'Studio could not read the feed.'))(site);
  return { name: failed.title || failed.url || failed.source, site, text };
}
