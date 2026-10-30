// The sequence of the entries of a list (ADR 0081). It has no Preact and no
// browser state, so that the Node tests can examine it (ADR 0042).

// A new list with the entry at `from` moved to `to`. The entries between
// the two positions move one position. A position out of the list gives the
// list with no change.
export function moveTo(list, from, to) {
  const inside = (i) => Number.isInteger(i) && i >= 0 && i < list.length;
  if (!inside(from) || !inside(to) || from === to) return list;
  const next = list.slice();
  const [entry] = next.splice(from, 1);
  next.splice(to, 0, entry);
  return next;
}

// A new list in which only the entries at `slots` change their sequence.
// `slots` holds the indices of those entries, from low to high. The entry at
// slot `from` moves to slot `to`, and each other entry keeps its index
// (ADR 0081 item 5).
export function moveAmong(list, slots, from, to) {
  const picked = moveTo(slots.map((i) => list[i]), from, to);
  if (picked.length !== slots.length || slots.every((i, n) => list[i] === picked[n])) return list;
  const next = list.slice();
  slots.forEach((i, n) => { next[i] = picked[n]; });
  return next;
}
