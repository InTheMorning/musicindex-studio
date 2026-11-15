// The view data for deleting a feed from its file host (ADR 0131).
// The API gives each state and each count. This module adds no feed rule.

// The goals of the choice sheet (ADR 0208 items 5 to 7). `action` is the
// stable key of a row, so the sheet does not compare titles.
const DELETE_CHOICES = [
  {
    action: 'move',
    title: 'Put the feed on a different file host',
    text: 'Apps and Music Index follow. Nothing is lost.',
    destructive: false,
  },
  {
    action: 'retire',
    title: 'Take the feed out of the apps',
    text: 'The file stays. You can stop this later.',
    destructive: false,
  },
  {
    action: 'delete',
    title: 'Remove the file',
    text: 'Music Index removed the entry.',
    destructive: true,
  },
];

// The row "Remove the file" shows only after Music Index removed the entry
// (ADR 0200 item 4).
export function deleteChoices(state) {
  if (state?.removal_done === true) return DELETE_CHOICES;
  return DELETE_CHOICES.slice(0, 2);
}

// The domain line (ADR 0208 item 1): the title, then the text. The core
// gives the host as `address_host`, so this function does not read an
// address.
export function domainLine(host) {
  if (typeof host !== 'string' || host.length === 0) return null;
  return {
    title: `Keep the domain ${host}.`,
    text: 'If it expires, its new owner can take over this feed and its payments.',
  };
}

// The state points of the last sheet (ADR 0208 item 10). The third point
// gives the button name apart, so that the sheet shows it in bold.
export const DELETE_POINTS = [
  { mark: '✓', text: 'Music Index removed the entry' },
  { mark: '✓', text: 'Studio keeps a copy of the feed' },
  { mark: '↺', text: 'You can put the file back later with', button: 'Put the File Back', after: 'on this feed' },
];

// The buttons of the Retire block, in their sequence (ADR 0208 item 13).
export function retireActions(state) {
  return state?.delete === 'done' ? ['restore', 'check'] : ['check', 'stop'];
}

const step = (number, title, text, done, button = null) => ({ number, title, text, done, button });

export function retireSteps(state) {
  const askDone = state.ask === 'draft_only' || state.ask === 'written';
  const askText = state.ask === 'draft_only'
    ? 'Hide from apps is on in the draft.'
    : state.ask === 'written_cleared'
      ? 'Hide from apps is off in the draft. The last write still hides the feed.'
      : askDone ? 'Hide from apps is on in the last write.' : 'Hide from apps is not on in the draft or last write.';
  const removal = state.removal;
  let removalText = 'Music Index still holds this feed.';
  if (removal.answer === 'removed') removalText = 'Music Index does not hold this feed.';
  if (removal.answer === 'held_with_other_guid') removalText = 'Music Index holds this feed with a different GUID.';
  if (removal.answer === 'not_known_failed') removalText = `Studio could not read ${removal.address}.`;
  if (removal.answer === 'not_known_no_guid') removalText = 'This feed has no GUID. Studio did not send a request.';
  if (removal.answer === 'not_read') removalText = 'Studio has not checked Music Index.';
  const removed = state.removal_done;
  const deleted = state.delete === 'done';
  const deleteButton = state.removal_done ? 'Delete the File' : null;
  return [
    step(1, 'The feed asks to be removed', askText, askDone),
    step(2, 'Music Index removes the entry', removalText, removed),
    step(3, 'Delete the file', deleted ? 'The file is deleted.' : removed
      ? 'Music Index removed the entry. You can delete the file now.'
      : 'Wait until Music Index removes the entry.', deleted, deleted ? null : deleteButton),
  ];
}

export function deleteButtonEnabled(confirmText, typedText) {
  return typeof confirmText === 'string' && confirmText.length > 0 && confirmText === typedText;
}

export function deleteWrite(diff) {
  const operations = diff?.operations || [];
  if (!operations.some((op) => op.deletes_feed_file)) return null;
  const deletes = operations.filter((op) => op.kind === 'delete');
  return {
    label: deletes.length === 1
      ? 'Delete the Feed File'
      : `Delete ${deletes.length} Files`,
    destructive: true,
    cancel: 'Cancel the Delete',
  };
}

// The second line of an operation row: what the operation means for the
// feed (task 022b, review of milestone 7a).
const OPERATION_NOTE = {
  upload: 'The feed uses this file.',
  copy: 'A spare copy. The feed does not use it.',
  delete: 'Studio deletes it after the feed is written.',
  confirm: 'Studio makes sure that the file host gives this file.',
  hold: 'The old copy stays. Studio asks you before it deletes this file.',
};

// The note of an operation row. The core marks the row that deletes the feed
// file with `deletes_feed_file`, so the UI does not compare paths
// (ADR 0131 item 12).
export function operationNote(op) {
  if (op.kind === 'delete' && op.deletes_feed_file) return 'This write deletes the feed file.';
  return OPERATION_NOTE[op.kind] || '';
}
