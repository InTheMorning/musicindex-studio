// The model of the file picker (ADR 0062, ADR 0042). The functions have no
// Preact and no browser state. `editor.js` shows the sheet and does the
// requests.

// The kind of each file field of ADR 0062 item 1. A cover is the cover of an
// album, of a publisher feed or of a playlist. `recording` is the audio file
// of a live show.
const KINDS = {
  cover: 'image',
  'track-image': 'image',
  'live-image': 'image',
  artwork: 'image',
  'track-audio': 'audio',
  recording: 'audio',
  lyrics: 'lyrics',
};

// The kind of file that a field accepts: `image`, `audio` or `lyrics`. An unknown
// field gives `null`.
export function fieldKind(field) {
  return KINDS[field] || null;
}

// The file extensions of each kind, as the core reads them (ADR 0032 item
// 1). The WAV extension is an audio file, because Studio converts it
// (ADR 0033).
const EXTENSIONS = {
  image: ['jpg', 'jpeg', 'png', 'webp', 'gif'],
  audio: ['mp3', 'm4a', 'flac', 'ogg', 'opus', 'wav'],
  lyrics: ['srt', 'vtt', 'txt', 'json', 'html'],
};

// The accept list of the file input for each kind.
const ACCEPT = {
  image: 'image/*,.jpg,.jpeg,.png,.webp,.gif',
  audio: 'audio/*,.wav,.flac,.m4a,.ogg,.opus',
  lyrics: '.srt,.vtt,.txt,.json,.html',
};

// The text that an upload of the wrong kind shows (ADR 0062 item 13).
const WRONG_KIND_TEXT = {
  image: 'Select an image file',
  audio: 'Select an audio file',
  lyrics: 'Select a lyrics file',
};

// The text of each drop error (ADR 0062 item 20).
const TOO_MANY_TEXT = 'Drop one file only';

// The reason that a host file cannot be selected (ADR 0062 item 9).
export const NO_ADDRESS_TEXT = 'This file has no public address';

const SOURCE_RELEASE = { id: 'release', label: 'This Release' };
const SOURCE_COMPUTER = { id: 'computer', label: 'This Computer' };

// The three sources of the sheet, in the sequence of ADR 0062 item 2. The
// source "Hosts" has one entry for each connector. The source "This
// computer" has the accept list of the kind.
export function sources(field, connectors) {
  const kind = fieldKind(field);
  return [
    SOURCE_RELEASE,
    {
      id: 'hosts',
      label: 'File Hosts',
      hosts: (connectors || []).map((c) => ({ connectorId: c.id, name: c.name })),
    },
    { ...SOURCE_COMPUTER, accept: ACCEPT[kind] || '' },
  ];
}

// The last part of a path.
function baseName(path) {
  const cut = path.lastIndexOf('/');
  return cut === -1 ? path : path.slice(cut + 1);
}

// A tree of folders from the flat list of a host. Each file has a `path`
// with `/` between the folders. A node has `name`, `path`, `folders` and
// `files`. A folder comes before a file, and each list is in the sequence of
// the names. A host with no folders gives a root with no `folders`. Each
// file of the tree is the entry of the list, with `name` added.
export function hostTree(files) {
  const root = { name: '', path: '', folders: [], files: [] };
  for (const file of files || []) {
    const parts = file.path.split('/').filter((part) => part !== '');
    let node = root;
    for (const part of parts.slice(0, -1)) {
      const path = node.path ? `${node.path}/${part}` : part;
      let next = node.folders.find((folder) => folder.name === part);
      if (!next) {
        next = { name: part, path, folders: [], files: [] };
        node.folders.push(next);
      }
      node = next;
    }
    node.files.push({ ...file, name: baseName(file.path) });
  }
  const order = (node) => {
    node.folders.sort((a, b) => a.name.localeCompare(b.name));
    node.files.sort((a, b) => a.name.localeCompare(b.name));
    node.folders.forEach(order);
  };
  order(root);
  return root;
}

// Can the user select this file? A file with no public address cannot be
// selected (ADR 0062 item 9).
export function canSelect(file) {
  return file.address != null && file.address !== '';
}

// The reason text of a file that the user cannot select. A file that the
// user can select gives an empty text.
export function reasonText(file) {
  return canSelect(file) ? '' : NO_ADDRESS_TEXT;
}

// The kind of a file by its name, else by its media type. An unknown file
// gives `null`.
export function kindOfFile(file) {
  const name = file.name || '';
  const dot = name.lastIndexOf('.');
  const extension = dot === -1 ? '' : name.slice(dot + 1).toLowerCase();
  for (const kind of Object.keys(EXTENSIONS)) {
    if (EXTENSIONS[kind].includes(extension)) return kind;
  }
  const type = file.type || '';
  if (type.startsWith('image/')) return 'image';
  if (type.startsWith('audio/')) return 'audio';
  return null;
}

// Does the field accept the file that Studio received? `facts.kind` is the
// kind that the upload route found.
export function acceptsFile(field, facts) {
  const kind = fieldKind(field);
  return kind !== null && facts.kind === kind;
}

// The text of an upload that `acceptsFile` refused.
export function wrongKindText(field) {
  return WRONG_KIND_TEXT[fieldKind(field)] || 'Select a file of the right kind';
}

// The result of a drop on a field (ADR 0062 item 20). One file of the kind
// of the field gives `{ action: 'upload', file }`. Two files or more give
// `{ action: 'error', code: 'too_many', text }`. A file of a different kind
// gives the code `wrong_kind`. A drop with no file gives `{ action: 'none' }`.
export function dropResult(field, files) {
  if (!files || files.length === 0) return { action: 'none' };
  if (files.length > 1) return { action: 'error', code: 'too_many', text: TOO_MANY_TEXT };
  const [file] = files;
  if (kindOfFile(file) !== fieldKind(field)) {
    return { action: 'error', code: 'wrong_kind', text: wrongKindText(field) };
  }
  return { action: 'upload', file };
}

// Each address of the audio of a track: its enclosure and the sources of
// its alternative enclosures with an audio type.
export function audioAddresses(item) {
  const found = item.enclosure && item.enclosure.url ? [item.enclosure.url] : [];
  for (const enc of item.alternate_enclosures || []) {
    if (!(enc.media_type || '').startsWith('audio/')) continue;
    for (const source of enc.sources) if (source.uri) found.push(source.uri);
  }
  return found;
}

// The addresses of a field before the change, for `delete_unused` (ADR 0062
// item 17). `subject` is the channel model for a cover field, the transcript
// entry for a lyrics field, the image entry for an artwork field, else the
// item.
// The new address is not in the list, and no address is in it twice.
export function replacedAddresses(field, subject, newAddress) {
  let found;
  if (field === 'cover') {
    found = [subject.image, subject.rss_image && subject.rss_image.url];
  } else if (fieldKind(field) === 'audio') {
    found = audioAddresses(subject);
  } else if (fieldKind(field) === 'lyrics') {
    found = [subject.url];
  } else if (field === 'artwork') {
    found = [subject.href];
  } else {
    found = [subject.image];
  }
  const kept = found.filter((address) => typeof address === 'string' && address !== '' && address !== newAddress);
  return [...new Set(kept)];
}

// The facts of an audio file that the user selected from "This release" or
// "Hosts", in the form of an upload result. The size is `null` when the list
// does not know it. The duration comes from task 039c.
export function selectedFacts(file) {
  return {
    url: file.address,
    size: file.size > 0 ? file.size : null,
    media_type: file.media_type || null,
  };
}
