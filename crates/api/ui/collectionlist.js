// The collections of the desktop app (ADR 0149). This module has no Preact
// and no browser state. The core keeps the list and applies each rule.

// The text of the row at the bottom of the left panel, or null. The row shows
// only when the open collection is not "Main collection" (ADR 0149 item 17).
export function collectionMark(collections) {
  const open = (collections || []).find((c) => c.open);
  return open && !open.main ? `Collection: ${open.name}` : null;
}

// One row of the section "Collections" (ADR 0149 items 6 and 12).
export function collectionRow(collection) {
  const count = collection.feeds === 1 ? '1 feed' : `${collection.feeds} feeds`;
  return {
    name: collection.name,
    line: count,
    canOpen: !collection.open,
    canDelete: !collection.open && !collection.main,
  };
}

// The words of a sheet that makes a collection (ADR 0149 items 7 to 11).
// `open` is the name of the open collection, and `size` is the text of the
// size of a copy.
export function collectionSheet(kind, open, size) {
  switch (kind) {
    case 'copy': return {
      title: `Copy ${open}`,
      start: `${open} copy`,
      lines: [`The copy needs ${size} on this computer. It holds the feeds, the drafts, the versions, the people, the library and the vault. It holds no connected app and no queued announcement.`],
      warning: {
        strong: 'This copy can write to the same file hosts.',
        text: `A write from the copy changes the real feed. The passphrase of the copy is the passphrase of ${open}.`,
      },
      primary: 'Copy and Open',
      busy: 'Copying…',
    };
    case 'import': return {
      title: 'Import an Account File',
      start: '',
      lines: ['Studio makes a new collection and opens it. Then select Import an Account on its first page. No import replaces data.'],
      warning: null,
      primary: 'Make and Open',
      busy: 'Making…',
    };
    default: return {
      title: 'New Collection',
      start: '',
      lines: ['A new collection has its own feeds, its own library and its own vault, with its own passphrase.'],
      warning: null,
      primary: 'Make and Open',
      busy: 'Making…',
    };
  }
}

export const COLLECTIONS_HINT = 'Each collection has its own feeds, library and vault. Only the open collection runs, so a connected app can write only to it. Open restarts Studio.';
export const OPENING_TEXT = 'Studio restarts on this collection…';
