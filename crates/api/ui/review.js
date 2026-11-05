// The model of the lines in a review (ADR 0114 items 8 to 12, ADR 0042). It
// has no Preact and no browser state. The core gives the lines and their
// parts. This module only chooses which rows to show. It also puts the field
// changes of the core into words (ADR 0114 items 1 to 5). The words, the
// profile names and the warning are display aids (ADR 0065).

import { shortKey } from './nostrkey.js';
import { profileName, keyWarning } from './people.js';

const rowText = (row) => row.parts.map((part) => part.text).join('');

export const VIEWS = ['simplified', 'detailed'];
export const VIEW_LABELS = { simplified: 'Simplified', detailed: 'Detailed' };
export const VIEW_HINTS = {
  simplified: 'What changes in words, with each time in your local time.',
  detailed: 'The changed lines of the XML, as the file host gets them.',
};
export const CONTEXT_LINES = 3;

// A stored value that is not a view gives the default view.
export function normalizeView(value) {
  return VIEWS.includes(value) ? value : 'simplified';
}

const hasChange = (line) => line.parts.some((part) => part.changed);

// The rows of the lines. `kind` is "same", "add" or "remove". `sign` is the
// sign of the row.
export function shownLines(lines) {
  return lines.map((line) => {
    if (line.kind === 'same') return { kind: 'same', sign: ' ', parts: line.parts };
    return line.kind === 'added'
      ? { kind: 'add', sign: '+', parts: line.parts }
      : { kind: 'remove', sign: '−', parts: line.parts };
  });
}

// ---- Local time (ADR 0114 items 14 and 15). A display aid: the file keeps
// its text (ADR 0065). ----

const TIME_KEYS = new Set(['pub_date', 'pub_date_time', 'last_build_date', 'start', 'end']);
const TIME_LINE = /<(?:pubDate|lastBuildDate)>([^<]+)<|\b(?:start|end)=\x22([^\x22]+)\x22/g;
const LIVE_LINE = /<podcast:liveItem\b/;

// A time of a feed in the local time of the browser, or null when the text is
// not a time. `zone` is for the tests.
export function localTime(text, zone) {
  const time = Date.parse(text);
  if (Number.isNaN(time)) return null;
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'medium', timeZone: zone }).format(time);
}

// The text below a changed line with a time, or null. A `start` or an `end`
// is a time only in a live item.
export function localTimeNote(row, zone) {
  if (row.kind === 'same') return null;
  const text = rowText(row);
  const times = [...text.matchAll(TIME_LINE)]
    .filter((match) => match[1] || LIVE_LINE.test(text))
    .map((match) => localTime(match[1] || match[2], zone))
    .filter(Boolean);
  return times.length > 0 ? `Local time: ${times.join(' to ')}` : null;
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
  values: 'Payments', web_master: 'Webmaster', last_build_date: 'Build date', publisher: 'Artist or label',
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
  if (TIME_KEYS.has(lastKey(path))) return localTime(String(value)) ?? String(value);
  return String(value);
}

// The row of one change: its label, its text and its warning or null.
export function changeRow(group, change, profiles) {
  const field = group.kind === 'field';
  const word = pathWord(change.path);
  const label = field ? keyWord(group.name) : (change.entry || word || keyWord(group.name) || 'Entry');
  const lead = field ? word : (change.entry ? word : '');
  // A scalar field of the channel has an empty path, and the group names it.
  const valuePath = field && !change.path ? group.name : change.path;
  let text;
  if (change.op === 'changed') {
    text = `${valueText(change.old, valuePath, profiles)} → ${valueText(change.new, valuePath, profiles)}`;
  } else {
    const verb = change.op === 'added' ? 'Added' : 'Removed';
    const value = change.op === 'added' ? change.new : change.old;
    const shown = brief(value);
    text = shown && !change.entry ? `${verb}: ${shown}` : verb;
  }
  if (lead) text = `${lead}: ${text}`;
  let warning = null;
  if (lastKey(change.path) === 'npub' && typeof change.new === 'string') {
    const who = change.name || (group.kind === 'credits' ? group.name : null);
    if (who) warning = keyWarning(change.new, profiles, who);
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

// ---- A merge of the hosted file into the draft (ADR 0129). The core gives
// each field and its side, and applies the choices. This model only gives
// the words. ----

export const MERGE_TITLE = 'The hosted file changed outside Studio.';
export const MERGE_INTRO = 'Studio can apply your edits to the new file. Nothing changes on a file host until you write.';
export const MERGE_HEADINGS = {
  kept: 'Your edits stay',
  incoming: 'New from the hosted file',
  conflicts: 'Both changed. Select one',
};

const mergeLabel = (field) => {
  const word = field.key ? keyWord(field.key) : 'Whole track';
  return field.track ? `${field.track} · ${word}` : word;
};

// A whole track shows its title, or that it is gone.
const mergeValue = (field, value) => {
  if (!field.key) return value ? (value.title || 'Track') : 'No track';
  return valueText(value, field.key, {});
};

// The rows of a plan of the core, with the words of each value.
export function mergeRows(plan) {
  const row = (field) => ({
    id: field.id,
    label: mergeLabel(field),
    mine: mergeValue(field, field.mine),
    hosted: mergeValue(field, field.hosted),
  });
  return {
    kept: (plan.kept || []).map(row),
    incoming: (plan.incoming || []).map(row),
    conflicts: (plan.conflicts || []).map(row),
  };
}

// True when each field that both sides changed has a choice.
export function mergeReady(plan, choices) {
  return (plan.conflicts || []).every((field) => Boolean(choices[field.id]));
}

