// The words of a move of a release (ADR 0148, ADR 0042). This module has no
// Preact and no browser state. The core gives the plan and the state of a
// move, and these functions give only their text (ADR 0065).
import { sizeText } from './assets.js';

const counted = (count, one, many) => `${count} ${count === 1 ? one : many}`;

function hostName(connectors, id) {
  const found = connectors.find((c) => c.id === id);
  return found ? found.name : 'a removed file host';
}

// The values of "What moves" in the sheet "Move This Release…" (ADR 0177
// item 3). `value` is the word that the move routes take. The first value is
// the start value.
export const MOVE_SCOPES = [
  { value: 'both', label: 'The feed file and the release files' },
  { value: 'feed', label: 'Only the feed file' },
  { value: 'files', label: 'Only the release files' },
];

// The two text rows of the section "File hosts" (ADR 0177 item 1). `feed`
// is the feed row, `files` is the answer of the Files route. A row has no
// control: "Move This Release…" is the only move.
export function fileHostRows(feed, files, connectors) {
  const feedId = files.feed_move ? files.feed_move.connector_id : feed.connector_id;
  const releaseId = files.files_connector_id ?? feed.connector_id;
  return [
    {
      label: 'Feed file',
      host: feedId ? hostName(connectors, feedId) : 'Not on a file host',
      address: feed.public_url || null,
    },
    {
      label: 'Release files',
      host: releaseId ? hostName(connectors, releaseId) : 'Not on a file host',
      address: null,
    },
  ];
}

// The file host or the file hosts that give the old files.
function fromText(plan, connectors) {
  const ids = plan.from_connector_ids;
  return ids.length === 1 ? hostName(connectors, ids[0]) : counted(ids.length, 'file host', 'file hosts');
}

// The rows of the plan: a label and a value, one row for each part that has
// a change (ADR 0148 items 2 and 3).
export function planRows(plan, connectors) {
  const to = hostName(connectors, plan.connector_id);
  const rows = [];
  if (plan.feed.moves) {
    const from = plan.feed.from_connector_id ? hostName(connectors, plan.feed.from_connector_id) : 'its address';
    rows.push({ label: 'Feed file', value: `${from} → ${to}` });
  } else if (plan.feed.stays) {
    rows.push({ label: 'Feed file', value: `Stays. ${to} cannot keep a feed file.` });
  }
  const part = (entry, one, many) => {
    if (entry.count > 0) {
      rows.push({ label: counted(entry.count, one, many), value: `${fromText(plan, connectors)} → ${to} · ${sizeText(entry.bytes)}` });
    }
  };
  part(plan.audio, 'audio file', 'audio files');
  part(plan.images, 'image', 'images');
  part(plan.other, 'other file', 'other files');
  if (plan.download.count > 0) {
    rows.push({ label: `${counted(plan.download.count, 'file', 'files')} not in the library`, value: 'Studio downloads them first' });
  }
  if (plan.on_target > 0) {
    rows.push({ label: counted(plan.on_target, 'file', 'files'), value: `Already on ${to}` });
  }
  return rows;
}

const STAY_TEXT = {
  not_in_library: 'Not in the library, so it does not move.',
  not_written: 'Not on a file host yet. Write the draft first.',
};

// The rows of the files that do not move (ADR 0148 item 3).
export const stayRows = (plan) => plan.stays.map((stay) => ({ name: stay.name, text: STAY_TEXT[stay.reason] || '' }));

// The label of the check box of the deletion (ADR 0148 item 5).
export function deleteLabel(plan, connectors) {
  const ids = plan.from_connector_ids;
  const place = ids.length === 1 ? hostName(connectors, ids[0]) : 'the file hosts from before';
  return `Delete the files on ${place} when the move is complete`;
}

// The label of the check box of "Move All Releases…" (ADR 0148 item 16).
export const deleteAllLabel = (host) => `Delete the files on ${host} when each move is complete`;

export const DELETE_NOTE_ON = 'Studio asks you first. The feed pointer file always stays.';
export const DELETE_NOTE_OFF = 'The old files stay, and Studio does not ask again.';
export const waitLabel = (days) => `${days} days`;

const INDEX_TEXT = {
  follows: '✓ Follows the new feed address',
  different: 'Gives a different feed address',
  unknown: 'No answer. The days decide.',
};

// The rows of a move that waits (ADR 0148 item 8).
export function waitingView(move, connectors) {
  const from = hostName(connectors, move.connector_id);
  const to = hostName(connectors, move.to_connector_id);
  const files = counted(move.records.length, 'file', 'files');
  const allFiles = move.records.length === 1 ? 'The file is' : `All ${move.records.length} are`;
  return {
    title: `Move to ${to}`,
    summary: `Waiting. ${files} ${move.records.length === 1 ? 'stays' : 'stay'} on ${from} until the checks pass.`,
    rows: [
      { label: 'New files', value: `✓ ${allFiles} on ${to}` },
      { label: 'Music Index', value: INDEX_TEXT[move.index] },
      { label: 'Waiting time', value: `${move.days_passed} of ${move.wait_days} days` },
    ],
  };
}

// The notice of a ready move (ADR 0148 item 10).
export function readyNotice(move, connectors) {
  const count = move.records.length;
  return {
    strong: `The move to ${hostName(connectors, move.to_connector_id)} is complete.`,
    text: `${counted(count, 'old file is', 'old files are')} ready to delete from ${hostName(connectors, move.connector_id)}.`,
  };
}

// The notice of a feed move with files that stay (ADR 0148 item 15).
export function partMoveNotice(count, toId, fromId, connectors) {
  const from = fromId ? hostName(connectors, fromId) : 'the file host from before';
  return {
    strong: `The feed moves to ${hostName(connectors, toId)}. ${counted(count, 'file stays', 'files stay')} on ${from}.`,
    text: `The feed still works while ${from} keeps ${count === 1 ? 'this file' : 'these files'}.`,
  };
}
