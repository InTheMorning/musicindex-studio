// The model of a server that holds no assets (ADR 0105, ADR 0042). This
// module has no Preact and no browser state. The Node tests in
// `ui-tests/assets.test.mjs` examine it. Each function gives a display
// value of the row of a library file. The core owns the rules: it sets
// `local`, `copies`, `expires_at` and `removed_at`, and the UI only names
// the state (ADR 0065).

const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July',
  'August', 'September', 'October', 'November', 'December'];

// The date of a time in seconds, in the local time of the browser (ADR 0093),
// such as "8 November". The year follows when it differs from the year of
// `now`.
export function dayText(seconds, now = new Date()) {
  const time = new Date(seconds * 1000);
  const text = `${time.getDate()} ${MONTHS[time.getMonth()]}`;
  return time.getFullYear() === now.getFullYear() ? text : `${text} ${time.getFullYear()}`;
}

// The state of a file on a server that holds no assets: `removed`, `pending`,
// `server_only` or `on_host`. The result is `null` for a file with none of
// these states, and always when the server holds assets.
export function fileState(file, { holdAssets = true } = {}) {
  if (holdAssets) return null;
  if (file.removed_at != null) return 'removed';
  if (file.expires_at != null) return 'pending';
  if (file.local && file.copies.length === 0) return 'server_only';
  if (file.copies.length > 0) return 'on_host';
  return null;
}

// The status line of a file, or `null` when `fileState` gives no state.
export function fileStatus(file, { holdAssets = true, now = new Date() } = {}) {
  switch (fileState(file, { holdAssets })) {
    case 'removed': return `Removed from this server on ${dayText(file.removed_at, now)}`;
    case 'pending': return `Waits for the write · Studio removes it on ${dayText(file.expires_at, now)}`;
    case 'server_only': return 'On this server only';
    case 'on_host': return 'On its file host';
    default: return null;
  }
}

// "Download to This Device" works on a server in the two modes. A file needs
// its bytes on the server or a copy on a file host.
export function canDownloadToDevice(file, { server }) {
  return Boolean(server) && file.removed_at == null && (file.local || file.copies.length > 0);
}

// A copy or a move reads the bytes of the file, so a removed file has neither
// (ADR 0105 item 10). Studio gives the user only "Add the File Again".
export function canCopyOrMove(file) {
  return file.removed_at == null;
}

// The number of removed files that the feed uses. The write refuses the
// draft until the user adds each one again (ADR 0105 item 10).
export function missingCount(files) {
  return files.filter((file) => file.removed_at != null && file.in_use).length;
}

export function missingNotice(count) {
  return count === 1
    ? { strong: '1 file of this draft is missing.', text: 'Studio cannot write until you add it again.' }
    : { strong: `${count} files of this draft are missing.`, text: 'Studio cannot write until you add them again.' };
}

// The notice at the top of the Files page (ADR 0105 item 17).
export const SERVER_NOTICE = {
  strong: 'This server keeps no audio and no images.',
  text: 'After each write, Studio removes its copies. Your file hosts and your own computer keep your files.',
};

// The notice of the sheet "Add tracks" (ADR 0105 item 9).
export const ADD_TRACKS_NOTICE = 'This server keeps the files only until you write, for a maximum of 7 days. Then Studio removes them. Keep your own copies.';

// The notice of the review sheet (ADR 0105 item 19). `count` is the number
// of files that the write sends.
export function reviewNotice(count) {
  return count === 1
    ? 'After the write, Studio removes its copy of this file from this server. Keep your own copy.'
    : `After the write, Studio removes its copies of these ${count} files from this server. Keep your own copies.`;
}

// The texts of the open entry of a file, by state. `date` is the text of
// the date, `host` is the name of the file host.
export const ENTRY_TEXT = {
  pending: (date, host) => [
    `Studio keeps this file on this server until you write. If you do not write by ${date}, Studio removes the file, and you must add it again.`,
    `Studio sends this file to ${host} when you write. Then Studio removes its copy from this server.`,
  ],
  on_host: (host) => `This server keeps no copy. A move to a different file host reads the file from ${host}.`,
  server_only: (host) => ({
    strong: 'No file host has this file.',
    text: `This server keeps it until a file host has a copy. Write the draft to send it to ${host}, or download it to your computer.`,
  }),
  removed: (days = 7) => ({
    strong: 'Studio removed this file.',
    text: `The draft waited more than ${days} days. Add the file again to write the draft.`,
  }),
};

// The size of a number of bytes for the Settings section, such as "7.8 MB".
export function sizeText(bytes) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit += 1; }
  return unit === 0 || (value >= 10 && Number.isInteger(value))
    ? `${value} ${units[unit]}` : `${value.toFixed(1)} ${units[unit]}`;
}

// The rows of the Settings section "Files on this server" (ADR 0105 item
// 17). `storage` is `{ used, limit }` from the status route, or `null`.
// `size` formats a number of bytes.
export function serverFilesRows(storage, size) {
  const rows = [
    { label: 'Audio and images', summary: 'Kept until the write, for a maximum of 7 days' },
    { label: 'Feed XML, drafts and versions', summary: 'Kept' },
  ];
  if (storage) rows.push({ label: 'Space in use', summary: `${size(storage.used)} of ${size(storage.limit)}` });
  return rows;
}

export const SERVER_FILES_NOTE = 'The operator of this server set it to keep no audio and no images. Your file hosts and your own computer keep your files.';
