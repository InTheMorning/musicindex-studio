// The texts of the confirmation sheets (task 057, musicindex ADR 0023
// R-CONFIRM-1). Each entry gives the title, the text, the action and the
// role of the action button. `askConfirm` of app.js shows the sheet.
const destructive = (title, text, action) => ({ title, text, action, role: 'destructive' });

export const CONFIRMS = {
  removeFeed: (name) => destructive(
    `Remove “${name}” from your collection?`,
    'The feed file on its file host does not change. You can add the feed again later.',
    'Remove from Collection'),
  removeApp: (name) => destructive(
    `Remove the app “${name}”?`,
    'Its token stops working. The app can no longer change your live shows.',
    'Remove the App'),
  newAppToken: (name) => destructive(
    `Make a new token for “${name}”?`,
    'The old token stops working. Give the new token to the app.',
    'Make a New Token'),
  removeFileHost: (name) => destructive(
    `Remove the file host “${name}”?`,
    'Studio forgets this file host. The files on it do not change.',
    'Remove the File Host'),
  discardChangeSet: () => destructive(
    'Discard this change set?',
    'Each feed that you did not write gets back its draft from before the change. A feed that you wrote does not change.',
    'Discard the Change Set'),
  discardDraft: () => destructive(
    'Discard the draft?',
    'Studio deletes each change that you did not write. The feed file on its file host does not change.',
    'Discard the Draft'),
  startAgain: () => destructive(
    'Start again from the hosted file?',
    'Studio deletes each change in the draft. Then it starts a new draft from the feed file on its file host.',
    'Start Again'),
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
  removeRecipients: (noun) => destructive(
    `Remove the recipients of this ${noun}?`,
    `The ${noun} then uses the recipients of the album.`,
    'Remove the Recipients'),
};
