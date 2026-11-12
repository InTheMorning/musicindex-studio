// The collections of the desktop app (ADR 0149). This module has no Preact
// and no browser state. The core keeps the list and applies each rule.

// The text of the row at the bottom of the left panel, or null. The row shows
// when the list has more than one collection (ADR 0176 item 12).
export function collectionMark(collections) {
  const list = collections || [];
  const open = list.find((c) => c.open);
  return open && list.length > 1 ? `Collection: ${open.name}` : null;
}

// The count of feeds of a collection, such as "1 feed".
function feedsText(collection) {
  return collection.feeds === 1 ? '1 feed' : `${collection.feeds} feeds`;
}

// The menu entries of one row of "Collections" (ADR 0183 items 2 and 3).
export function collectionRow(collection) {
  const actions = collection.open
    ? [
        { label: 'Rename…', action: 'rename' },
        { label: 'Copy…', action: 'copy' },
        { label: 'Export…', action: 'export' },
      ]
    : [
        { label: 'Open', action: 'open' },
        { label: 'Rename…', action: 'rename' },
        { label: 'Export…', action: 'export' },
        { label: 'Replace with a File…', action: 'replace' },
        { label: 'Delete…', action: 'delete', separatorBefore: true, destructive: true },
      ];
  return {
    name: collection.name,
    line: feedsText(collection),
    badge: collection.open ? 'Current' : null,
    actions,
  };
}

// The actions below the collection list (ADR 0183 item 5).
export function collectionSectionActions() {
  return [
    { label: 'New Collection…', action: 'new' },
    { label: 'Import an Account File…', action: 'import' },
  ];
}

const GOES = 'the feeds, the drafts, the versions, the people and the vault';

// The words of the sheets of a row (ADR 0176 items 5 to 9). `kind` is
// `rename`, `replace` or `delete`. The sheets of replace and delete give the
// name, the count of feeds and the parts that go, and they offer the export.
export function collectionAction(kind, collection) {
  const name = collection.name;
  switch (kind) {
    case 'rename': return {
      title: 'Rename Collection',
      start: name,
      lines: ['Only the name changes. The files stay where they are.'],
      primary: 'Rename',
      busy: 'Renaming…',
      destructive: false,
      exportFirst: false,
      file: false,
    };
    case 'replace': return {
      title: `Replace “${name}” with a File`,
      start: null,
      lines: [
        `${name} holds ${feedsText(collection)} now. Studio reads the file first, and a file that is not good changes nothing.`,
        `After a good read, ${GOES} of the file take the place of the current ones. Nothing changes on a file host.`,
        'The vault of the file opens with a method of the file, not with the method of the vault that goes.',
      ],
      primary: 'Replace Collection',
      busy: 'Replacing…',
      destructive: true,
      exportFirst: true,
      file: true,
    };
    default: return {
      title: `Delete “${name}”?`,
      start: null,
      lines: [
        `${name} holds ${feedsText(collection)}. Studio removes ${GOES} from this computer. You cannot get them back.`,
        'Nothing changes on a file host.',
      ],
      primary: 'Delete Collection',
      busy: 'Deleting…',
      destructive: true,
      exportFirst: true,
      file: false,
    };
  }
}

// The text of a closed collection whose vault no different Studio can open.
// The recovery key belongs to the vault that is open.
export const CLOSED_EXPORT_HINT = 'Open this collection, make a recovery key there, and export it again.';

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
