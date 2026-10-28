// The OP3 switch (ADR 0076, ADR 0065). This module gives display text only.
// The core decides the start value of the switch, and adds or removes the
// prefix of each track enclosure at the draft save. The UI sends the value of
// the switch.

// The hint for a mixed set of tracks. `counts` is `op3_tracks` of the draft
// answer: `{ prefixed, tracks }`. It gives `null` when no track has the prefix
// or when each track has it.
export function op3Hint(counts) {
  if (!counts) return null;
  const { prefixed, tracks } = counts;
  if (!prefixed || prefixed === tracks) return null;
  return `${prefixed} of ${tracks} tracks use OP3`;
}

// The OP3 statistics page of a feed, by its `podcast:guid`.
export function op3Page(guid) {
  return `https://op3.dev/show/${encodeURIComponent(guid)}`;
}
