// The model of the lines in a review (ADR 0114 items 8 to 12, ADR 0042). It
// has no Preact and no browser state. The core gives the lines and their
// parts. This module only chooses which rows to show.

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
