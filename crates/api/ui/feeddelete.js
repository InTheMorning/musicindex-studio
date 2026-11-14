// The view data for deleting a feed from its file host (ADR 0131).
// The API gives each state and each count. This module adds no feed rule.

const DELETE_CHOICES = [
  {
    title: 'Move It to a Different File Host',
    text: 'You keep control. The index follows the new address. An app can stay at the earlier address.',
    destructive: false,
  },
  {
    title: 'Retire It',
    text: 'The feed asks the indexes to remove it. After Music Index removes the entry, you can delete the file.',
    destructive: false,
    recommended: true,
  },
  {
    title: 'Delete the File Only',
    text: 'You lose control of the entry in the index.',
    destructive: true,
  },
];

export function deleteChoices(state) {
  if (state?.removal_done === true) return DELETE_CHOICES;
  return DELETE_CHOICES.slice(0, 2).map(({ recommended, ...choice }) => choice);
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
