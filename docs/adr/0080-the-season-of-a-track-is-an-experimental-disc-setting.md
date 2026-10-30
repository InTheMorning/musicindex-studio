# ADR 0080: The Season Of A Track Is An Experimental Disc Setting

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.
- Changes: [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 8, the "Season" fields of the "Identity" row.
- Uses: musicindex ADR 0034 R-EXPERIMENT-1 (the Experimental badge) and musicindex ADR 0025 R-STATUS-1 (the Warning color).

## Context

- On 2026-10-29 the operator examined the "Identity" row of the track page. It had the fields "Season number" and "Season name", the track number and the track GUID. The operator said that this is confusing to a music publisher.
- The fields write `podcast:season` and its `name` attribute. The Podcasting 2.0 namespace specifies the element for the seasons of a podcast.
- ADR 0064 found the value `1` in 8 of 27 music feeds, and no other value. MSP 2.0 writes `<podcast:season>1</podcast:season>` into each item. No source that the operator or the agent examined says that a music app reads the element as a disc.
- The track number is `podcast:episode` and `itunes:episode`. Studio sets it from the sequence ([ADR 0024](0024-track-numbers-follow-the-track-sequence.md), [ADR 0079](0079-the-core-writes-both-track-numbers.md)). A label "Track number" on the season field would thus give two track numbers.

## Decision

1. The "Identity" row of a track shows the track number and the track GUID first. Its summary line is "Track N", and "Track N · Disc D" when the track has a season. The number N is the `podcast:episode` of the track, or its `itunes:episode` when it has no `podcast:episode`.
2. Below them, a collapsed entry "Disc" has the Experimental badge of musicindex ADR 0034. It holds the fields "Disc number" and "Disc name".
3. The entry gives this hint: "Studio writes this as `podcast:season`. The Podcasting 2.0 namespace defines it for the seasons of a podcast, so apps can show it as a season."
4. The fields write `podcast:season` and its `name`, as before. The core does not change. A read and a write with no edit give the same bytes.
5. The UI has the token `--warning` with the Warning text color of musicindex ADR 0025 R-STATUS-1: `#ff9f0a` in the dark theme and `#a05a00` in the light theme.

## Alternatives Considered

- Remove the fields from the editor. Not chosen by the operator.
- Rename "Season number" to "Track number". Rejected: the track number is a different element, and Studio sets it.
- Keep the season words, with a hint. Not chosen by the operator.

## Consequences

- A user who sets a disc writes a season. An app can show it as a season.
- The type label of the setting and the element have different words. The hint gives the element.

## Invariants

- No track gets `podcast:season` unless the user sets it (ADR 0064 item 3).
