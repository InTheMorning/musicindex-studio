// The model of the field list of a search field (ADR 0154, musicindex ADR
// 0052 R-QUERY-7). This module has no Preact and no browser state. The core
// gives the rows and each range in UTF-16 code units, which are the indexes
// of a JavaScript string. This module only puts the text of a row into the
// range that the core gave. It holds no rule of the query text (ADR 0151).

// Only `artist:` gets name rows. `release:` gets none until an API route
// gives release titles (R-QUERY-7 item 2).
export const NAME_FIELD = 'artist:';

// The time after the last key before Studio asks for names, in milliseconds.
export const NAME_PAUSE = 300;

// The source of a name row, in words (R-QUERY-7 item 2).
export const NAME_SOURCE = 'Artist on Music Index';

// The prefix of the name rows to ask for, or null. `answer` is the answer of
// `GET /musicindex/fields`.
export function namePrefix(answer) {
  const names = answer && answer.names;
  if (!names || names.field !== NAME_FIELD || names.prefix === '') return null;
  return names.prefix;
}

// The rows that the list shows: the rows of the core, then the name rows of
// `GET /musicindex/artist-names`. Each row has a label, a line and the text
// that it puts into the field.
export function listRows(answer, names = []) {
  if (!answer) return [];
  const rows = (answer.rows || []).map((row) => (row.row === 'field'
    ? { label: row.token, line: [row.values_text, row.example].filter(Boolean).join(' · '), insert: row.insert }
    : { label: row.value, line: row.description || '', insert: row.insert }));
  const named = namePrefix(answer) === null ? [] : (names || [])
    .map((name) => ({ label: name.name, line: NAME_SOURCE, insert: name.insert }));
  return [...rows, ...named];
}

// The text and the cursor after a row goes into the range `edit` of `text`,
// or null when the core gave no range.
export function applyRow(text, edit, insert) {
  if (!edit) return null;
  const value = String(text ?? '');
  return {
    text: value.slice(0, edit.start) + insert + value.slice(edit.end),
    cursor: edit.start + insert.length,
  };
}

// The result of one key in the field (ADR 0154 items 7 and 8, v4vmm ADR 0087
// Decision 3). `open` is true when the list shows rows, and `selected` is the
// index of the selected row or null. `act` is one of:
// - 'move': the selection changes, and the key does nothing else.
// - 'insert': the selected row goes into the text.
// - 'close': the list closes, and the key does nothing else.
// - 'pass': the key does what it does with no list. Enter with no selected
//   row thus runs the search or sends the form.
export function listKey({ open, selected }, key, count) {
  const pass = { open, selected, act: 'pass' };
  if (!open || count === 0) return pass;
  switch (key) {
    case 'ArrowDown':
      return { open, selected: selected === null ? 0 : Math.min(selected + 1, count - 1), act: 'move' };
    case 'ArrowUp':
      if (selected === null) return { open, selected: count - 1, act: 'move' };
      return { open, selected: selected === 0 ? null : selected - 1, act: 'move' };
    case 'Enter':
    case 'Tab':
      return selected !== null && selected < count ? { open, selected: null, act: 'insert' } : pass;
    case 'Escape':
      return { open: false, selected: null, act: 'close' };
    default:
      return pass;
  }
}
