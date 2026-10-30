# ADR 0081: A Position Menu Sets The Sequence Of A List

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.

## Context

- On 2026-10-29 the operator said that a change of the track sequence is confusing and is not easy to do.
- On that date, an album had only the buttons "Move up" and "Move down" at the bottom of each track page. One click moved a track one position. To move track 9 to the first position, the user had to click 8 times. The user did not see the sequence during the move.
- The Tracks page of a playlist and the Releases section of a publisher page had "Move up" and "Move down" on each row.
- v4vmm changes the sequence of a playlist with a drag. A line in the accent color shows the position of the drop (`../v4vmm/src/ui/shells/playlist.rs`). No shared musicindex rule gives a control for a sequence.
- Each function of Studio works from the keyboard ([ADR 0010](0010-studio-uses-the-afterglow-direction.md)). A drag alone does not.
- [ADR 0041](0041-publisher-feeds-and-two-way-links.md) items 9 and 10, and [ADR 0053](0053-playlists-are-musicl-feeds-with-remote-items.md) item 6, say that the user changes a sequence. They do not give the control.

## Decision

1. Each entry of these lists has a position menu:
   - the Tracks section of an album page,
   - the Tracks page of a playlist,
   - the Releases section of a publisher page.
2. The position menu is a select menu with the positions 1 to N. Its name for a screen reader is "Position of TITLE". A selection moves the entry to that position. The entries between the start position and the new position move one position.
3. In the Tracks section of an album, the position menu replaces the track number, at the start of the row. The remaining part of the row opens the track.
4. On a track page, the field "Position" has the same menu. It replaces "Move up" and "Move down".
5. In the Releases section of a publisher page, a move changes only the sequence of the release entries. Each other entry of the list keeps its position.
6. A model module with no Preact gives the moves, and Node tests examine it ([ADR 0042](0042-ui-models-with-node-tests.md)).
7. The core sets the track numbers after each change of the sequence ([ADR 0024](0024-track-numbers-follow-the-track-sequence.md), [ADR 0079](0079-the-core-writes-both-track-numbers.md)). A position menu sets no track number.

## Alternatives Considered

- A drag handle on each row, with the insertion line of v4vmm. Deferred by the operator: the insertion line becomes a shared marker, so it needs a musicindex ADR first. A drag also needs a keyboard method.
- A drag in the outline. Rejected: the targets are small, a drag conflicts with a click that opens a track, and the outline is a different page at narrow width.
- A number field on each row. Rejected: a typed number can be out of the list, and the user must find the other numbers.

## Consequences

- Each move takes one selection.
- The list "Artists and labels" of an album page, and the entry lists of [ADR 0028](0028-summary-lines-and-one-open-entry.md), keep "Move up" and "Move down". The operator did not include them.
- A subsequent decision can add a drag to the same lists.

## Invariants

- A position menu changes only the sequence of its list.
