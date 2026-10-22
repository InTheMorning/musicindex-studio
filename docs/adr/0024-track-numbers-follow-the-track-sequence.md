# ADR 0024: Track Numbers Follow The Track Sequence

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

On 2026-10-22 the operator moved a track in the album editor ([task 011](../tasks/studio-v1-task-011-album-editor.md)). The move changed only the sequence of the `item` elements. Each track kept its own `podcast:episode` and its own `pubDate`, because the writer keeps each element with its track ([ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md)).

MSP 2.0 writes the track number in `podcast:episode`. An app that sorts by `podcast:episode` thus showed the sequence from before the move.

The operator records the release date of each track in its `pubDate`. Some apps sort tracks by date. The operator accepts that these apps do not show the sequence of an album, and builds an app that reads music data.

## Decision

1. The core models the text of `podcast:episode` of an item. It keeps the attributes of the element, such as `display`.
2. When a sent model changes the sequence of the tracks, the core gives each track its position as its `podcast:episode`, from 1. A move, an added track and a removed track each change the sequence.
3. The core does not change a `pubDate` when the sequence changes.
4. The editor shows the track number of each track and does not let the user change it. It tells the user that Studio sets the number from the sequence.
5. The editor tells the user that the publication date is the release date of the track, and that some apps sort tracks by this date.

## Alternatives Considered

- Give the tracks the same set of dates in the new sequence. Rejected by the operator: a `pubDate` is the release date of its track.
- Change nothing, and let the user change the numbers by hand. Rejected by the operator: the number then does not agree with the sequence after each move.

## Consequences

- An album with no `podcast:episode` gets a number on each track after its first change of sequence. The diff shows each new number.
- An app that sorts by `pubDate` can show a sequence that is different from the editor.

## Invariants

- After a change of sequence, the `podcast:episode` of each track is its position, from 1.
- A change of sequence does not change a `pubDate`.
