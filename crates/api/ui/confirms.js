// The texts of the confirmation sheets (task 057, musicindex ADR 0023
// R-CONFIRM-1). Each entry gives the title, the text, the action and the
// role of the action button. `askConfirm` of app.js shows the sheet.
const destructive = (title, text, action) => ({ title, text, action, role: 'destructive' });

export const CONFIRMS = {
  deleteOldFilesNow: (count, host) => destructive(
    `Delete ${count === 1 ? 'the old file' : `the ${count} old files`} before the move is ready?`,
    `An app that still uses an old address gets an error. You review each deletion, and ${host} keeps the files until you write.`,
    'Add the Deletion'),
  removeFeed: (name) => destructive(
    `Remove “${name}” from your collection?`,
    'The feed file on its file host does not change. You can add the feed again later.',
    'Remove from Collection'),
  removeApp: (name) => destructive(
    `Remove the app “${name}”?`,
    'The grants of the app end at once. The app can no longer change your live shows.',
    'Remove the App'),
  removeFileHost: (name) => destructive(
    `Remove the file host “${name}”?`,
    'Studio forgets this file host. The files on it do not change.',
    'Remove the File Host'),
  discardChanges: () => destructive(
    'Discard these changes?',
    'Each feed that you did not write gets back its draft from before the change. A feed that you wrote does not change.',
    'Discard the Changes'),
  discardDraft: () => destructive(
    'Discard the draft?',
    'Studio deletes each change that you did not write. The feed file on its file host does not change.',
    'Discard the Draft'),
  editAudioValues: () => destructive(
    'Edit these values by hand?',
    'Studio reads these values from the audio file. A wrong value stays in your feed, and apps can show the wrong length or fail to play the file. To change them safely, select a new audio file.',
    'Edit Values'),
  replaceDraft: () => destructive(
    'Replace the draft with this version?',
    'Studio deletes each change in the draft. Then it starts a new draft from this earlier version.',
    'Replace the Draft'),
  removeTrackCredits: () => destructive(
    'Remove the credits of this track?',
    'The track then uses the credits of the album.',
    'Remove the Credits'),
  stopGrant: () => destructive(
    'Stop the grant?',
    'Connected apps can no longer change this show.',
    'Stop the Grant'),
  // `size` is the text of the file size, or null when the row has no size.
  downloadFirst: (size) => ({
    title: 'Download this file first?',
    text: `Studio downloads the file${size ? ` (${size})` : ''} first. The copy goes into the draft now, and the write waits for the download.`,
    action: 'Download the File',
    role: 'primary',
  }),
  deletePerson: (name) => destructive(
    `Delete ${name} from People?`,
    'Studio deletes this person and its links. Studio changes no feed.',
    'Delete the Person'),
  // The leave sheet of the page of a person (ADR 0116 item 8). `cancel` names
  // the button that keeps the page open.
  leavePerson: (text) => ({
    title: 'Leave without saving?',
    text,
    action: 'Discard Changes',
    role: 'destructive',
    cancel: 'Keep Editing',
  }),
  removeRecipients: (noun) => destructive(
    `Remove the recipients of this ${noun}?`,
    `The ${noun} then uses the recipients of the album.`,
    'Remove the Recipients'),
};

// The buttons of a confirmation sheet, left to right (ADR 0193 items 5 and
// 6). A destructive action is first. The cancel button gets the first focus.
export function confirmButtons(entry) {
  const cancel = { label: entry.cancel || 'Cancel', role: 'cancel', kind: 'cancel', focus: true };
  const action = { label: entry.action, role: entry.role, kind: 'action', focus: false };
  return entry.role === 'destructive' ? [action, cancel] : [cancel, action];
}
