// The model of the lines in a review (ADR 0114 items 8 to 12, ADR 0042). It
// has no Preact and no browser state. The core gives the lines and their
// parts. This module only chooses which rows to show. It also puts the field
// changes of the core into words (ADR 0114 items 1 to 5). The words, the
// profile names and the warning are display aids (ADR 0065).

import { shortKey } from './nostrkey.js';
import { profileName, keyWarning } from './people.js';

export const VIEWS = ['simplified', 'detailed'];
export const VIEW_LABELS = { simplified: 'Simplified', detailed: 'Detailed' };
export const VIEW_HINTS = {
  simplified: 'What changes in words, then the changed lines.',
  detailed: 'The changed lines of the XML, as the file host gets them.',
};
export const CONTEXT_LINES = 3;

// A stored value that is not a view gives the default view.
export function normalizeView(value) {
  return VIEWS.includes(value) ? value : 'simplified';
}

const hasChange = (line) => line.parts.some((part) => part.changed);

// The rows of a view. `kind` is "same", "add" or "remove". `sign` is the
// sign of the row. In "simplified", a pair with one side that has no changed
// part is one row with the sign "±".
export function shownLines(lines, view) {
  const simple = normalizeView(view) === 'simplified';
  const skipped = new Set();
  const rows = [];
  lines.forEach((line, index) => {
    if (skipped.has(index)) return;
    if (line.kind === 'same') {
      rows.push({ kind: 'same', sign: ' ', parts: line.parts });
    } else if (line.kind === 'added') {
      rows.push({ kind: 'add', sign: '+', parts: line.parts });
    } else {
      const other = simple && line.pair != null ? lines[line.pair] : null;
      if (other && !hasChange(line)) {
        skipped.add(line.pair);
        rows.push({ kind: 'add', sign: '±', parts: other.parts });
      } else if (other && !hasChange(other)) {
        skipped.add(line.pair);
        rows.push({ kind: 'remove', sign: '±', parts: line.parts });
      } else {
        rows.push({ kind: 'remove', sign: '−', parts: line.parts });
      }
    }
  });
  return rows;
}

// The rows with the context around each changed row. With `whole`, each row
// shows. Each run of other rows is one segment `{ hidden: true, id, count,
// rows }`, where `id` is the index of its first row. A shown row is a
// segment `{ hidden: false, row }`.
export function runs(rows, whole) {
  const visible = rows.map(() => Boolean(whole));
  if (!whole) {
    rows.forEach((row, index) => {
      if (row.kind === 'same') return;
      const from = Math.max(0, index - CONTEXT_LINES);
      const to = Math.min(rows.length - 1, index + CONTEXT_LINES);
      for (let i = from; i <= to; i += 1) visible[i] = true;
    });
  }
  const segments = [];
  let start = -1;
  const close = (end) => {
    if (start < 0) return;
    segments.push({ hidden: true, id: start, count: end - start, rows: rows.slice(start, end) });
    start = -1;
  };
  rows.forEach((row, index) => {
    if (visible[index]) {
      close(index);
      segments.push({ hidden: false, row });
    } else if (start < 0) {
      start = index;
    }
  });
  close(rows.length);
  return segments;
}

export const rowText = (row) => row.parts.map((part) => part.text).join('');

// The number of leading spaces that each row has. A tab counts as one space.
// An empty row does not count.
export function commonIndent(rows) {
  let least = null;
  rows.forEach((row) => {
    const text = rowText(row);
    if (text.trim() === '') return;
    const indent = text.length - text.replace(/^[ \t]+/, '').length;
    least = least === null ? indent : Math.min(least, indent);
  });
  return least ?? 0;
}

// The parts of a row without its first `count` characters. The text of the
// parts is not changed in any other way.
export function stripIndent(parts, count) {
  let left = count;
  const out = [];
  parts.forEach((part) => {
    if (left <= 0) {
      out.push(part);
      return;
    }
    const cut = Math.min(left, part.text.length);
    left -= cut;
    if (cut < part.text.length) out.push({ ...part, text: part.text.slice(cut) });
  });
  return out;
}

export function moreLabel(count) {
  return count === 1 ? 'Show 1 more line' : `Show ${count} more lines`;
}

// ---- What changes (ADR 0114 items 1 to 5, task 077c) ----

// One word for each key of the serialized model, in the words of the editor.
export const KEY_WORDS = {
  alternate_enclosures: 'Alternate files', atom_links: 'Atom links', author: 'Author', bitrate: 'Bitrate',
  blocks: 'Blocks', content_links: 'Content links', default: 'Default', description: 'Description',
  duration: 'Duration', duration_seconds: 'Duration', email: 'Email', enclosure: 'Audio file', end: 'End',
  episode: 'Episode', explicit: 'Explicit', field: 'Field', fundings: 'Support links', guid: 'GUID',
  image: 'Image', images: 'Images', item: 'Live item', items: 'Tracks', itunes_block: 'Block in Apple Podcasts',
  itunes_episode: 'Apple Podcasts episode', keywords: 'Keywords', language: 'Language', length: 'Size',
  license: 'License', link: 'Link', live_items: 'Live items', live_value: 'Live payments', locked: 'Locked',
  managing_editor: 'Managing editor', media_type: 'File type', medium: 'Medium', method: 'Method',
  name: 'Name', new_feed_url: 'New feed address', op3: 'OP3', owner: 'Owner', persons: 'Credits',
  podcast_guid: 'Podcast GUID', podroll: 'Podroll', pub_date: 'Publication date',
  pub_date_time: 'Publication date', recipients: 'Recipients', rel: 'Relation',
  remote_items: 'Remote items', remote_percentage: 'Remote percentage', remote_start_time: 'Remote start time',
  rss_image: 'RSS image', season: 'Disc', sources: 'Sources', start: 'Start', start_time: 'Start time',
  status: 'Status', suggested: 'Suggested', time_splits: 'Time splits', title: 'Title',
  transcripts: 'Transcripts', txts: 'Text channels', url: 'Address', value: 'Value', value_type: 'Value type',
  values: 'Payments', web_master: 'Webmaster', publisher: 'Artist or label',
  // Keys of the entries that the list above does not name.
  npub: 'Nostr key', role: 'Role', group: 'Group', href: 'Information address', img: 'Picture',
  address: 'Address', split: 'Split', fee: 'Fee', type: 'Type', purpose: 'Purpose', text: 'Text',
};

export const keyWord = (key) => KEY_WORDS[key] || key;

// The words of a path. The path of a live item starts with `item.`, and the
// word drops that prefix. A number in a path is a position and has no word.
export function pathWord(path) {
  const parts = String(path || '').split('.').filter((p) => p !== '' && !/^\d+$/.test(p));
  if (parts[0] === 'item' && parts.length > 1) parts.shift();
  return parts.map(keyWord).join(' · ');
}

// A word in the middle of a sentence starts with a small letter, except a
// word that starts with a name or an abbreviation, such as "Nostr key".
const KEEP_CAPITAL = /^(Nostr|GUID|OP3|RSS|Apple|Atom|Podcast GUID)\b/;
export const inSentence = (word) => (KEEP_CAPITAL.test(word) ? word : word.charAt(0).toLowerCase() + word.slice(1));

const lastKey = (path) => String(path || '').split('.').pop();
const plural = (n, one, many) => (n === 1 ? one : many);

// The keys of the changes of a field view, for `useProfiles`.
export function changeKeys(groups) {
  const keys = [];
  (groups || []).forEach((g) => g.changes.forEach((c) => {
    if (lastKey(c.path) !== 'npub') return;
    [c.old, c.new].forEach((v) => { if (typeof v === 'string' && v) keys.push(v); });
  }));
  return keys;
}

// A short word for an object that a change adds or removes.
function brief(value) {
  if (typeof value === 'string') return value;
  if (!value || typeof value !== 'object') return '';
  for (const key of ['name', 'title', 'url', 'address', 'role', 'guid']) {
    if (typeof value[key] === 'string' && value[key]) return value[key];
  }
  return '';
}

// The text of a value. A key shows its profile name and its short key.
export function valueText(value, path, profiles, short = false) {
  if (value === null || value === undefined || value === '') return 'none';
  if (lastKey(path) === 'npub' && typeof value === 'string') {
    const name = profileName(value, profiles);
    if (short) return name || shortKey(value);
    return name ? `${name} (${shortKey(value)})` : shortKey(value);
  }
  if (typeof value === 'object') return brief(value) || JSON.stringify(value);
  return String(value);
}

// The row of one change: its label, its text and its warning or null.
export function changeRow(group, change, profiles) {
  const field = group.kind === 'field';
  const word = pathWord(change.path);
  const label = field ? keyWord(group.name) : (change.entry || word || keyWord(group.name) || 'Entry');
  const lead = field ? word : (change.entry ? word : '');
  let text;
  if (change.op === 'changed') {
    text = `${valueText(change.old, change.path, profiles)} → ${valueText(change.new, change.path, profiles)}`;
  } else {
    const verb = change.op === 'added' ? 'Added' : 'Removed';
    const value = change.op === 'added' ? change.new : change.old;
    const shown = brief(value);
    text = shown && !change.entry ? `${verb}: ${shown}` : verb;
  }
  if (lead) text = `${lead}: ${text}`;
  let warning = null;
  if (lastKey(change.path) === 'npub' && group.kind === 'credits' && typeof change.new === 'string') {
    warning = keyWarning(change.new, profiles, group.name);
  }
  return { label, text, warning };
}

export function groupLabel(group) {
  switch (group.kind) {
    case 'field': return null;
    case 'credits': return `Credits · ${group.name}`;
    case 'track': return `Track ${group.position}`;
    case 'live': return 'Live item';
    default: return keyWord(group.name);
  }
}

// The largest set of more than two changes of one path with one new value,
// or null (ADR 0114 item 2).
function cluster(group) {
  const sets = new Map();
  group.changes.forEach((c, i) => {
    if (c.op !== 'changed' || c.path === '') return;
    const id = `${c.path}\u0000${JSON.stringify(c.new)}`;
    if (!sets.has(id)) sets.set(id, []);
    sets.get(id).push(i);
  });
  let best = null;
  sets.forEach((indexes) => { if (indexes.length > 2 && (!best || indexes.length > best.length)) best = indexes; });
  return best;
}

// The rows of the field view: one object for each group, with `label` (or
// null), `summary` (or null), `rows` and `more` (or null). `more` is the
// collapsed line `{ summary, rows }` of the changes that the summary covers.
export function fieldRows(groups, profiles) {
  return (groups || []).map((group) => {
    const label = groupLabel(group);
    const set = cluster(group);
    if (!set) {
      return { label, summary: null, more: null, rows: group.changes.map((c) => changeRow(group, c, profiles)) };
    }
    const first = group.changes[set[0]];
    const covered = new Set(set);
    const count = set.length;
    const noun = group.kind === 'credits' ? 'credits' : 'entries';
    const word = inSentence(pathWord(first.path));
    const value = valueText(first.new, first.path, profiles, true);
    const summary = first.new === null || first.new === undefined || first.new === ''
      ? `${count} ${noun} lose the ${word}`
      : `${count} ${noun} get the ${word} ${value}`;
    const rest = set.slice(2).map((i) => group.changes[i]);
    const names = rest.map((c) => c.entry).filter(Boolean);
    const moreNoun = group.kind === 'credits' ? plural(rest.length, 'credit', 'credits') : plural(rest.length, 'entry', 'entries');
    const moreSummary = `${rest.length} more ${moreNoun}${names.length === rest.length ? `: ${names.join(', ')}` : ''}`;
    const rows = [
      ...set.slice(0, 2).map((i) => changeRow(group, group.changes[i], profiles)),
      ...group.changes.filter((_, i) => !covered.has(i)).map((c) => changeRow(group, c, profiles)),
    ];
    return {
      label, summary, rows,
      more: { summary: moreSummary, rows: rest.map((c) => changeRow(group, c, profiles)) },
    };
  });
}

// The guard row of ADR 0114 item 4, or null with no uncovered change.
export function guardRow(uncovered) {
  if (!uncovered || uncovered < 1) return null;
  return uncovered === 1
    ? { title: '1 more change in the XML', text: 'This list does not show it. Select the detailed view to see each changed line.' }
    : { title: `${uncovered} more changes in the XML`, text: 'This list does not show them. Select the detailed view to see each changed line.' };
}
