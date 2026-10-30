# ADR 0079: The Core Writes Both Track Numbers

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.
- Changes: [ADR 0024](0024-track-numbers-follow-the-track-sequence.md) items 1 and 2, which name only `podcast:episode`.

## Context

- The human test of task 028 on 2026-10-29 used a feed from Podtunes. Each track had `itunes:episode` 1, 2 and 3, and no `podcast:episode` ([human test record](../reviews/2026-10-29-task-028-human-test.md), section "Two Track Numbers For Each Track After A Removal").
- The operator removed the first track. The core gave the other two tracks `podcast:episode` 1 and 2, as ADR 0024 item 2 requires. Their `itunes:episode` stayed 2 and 3.
- Thus the feed gave two different numbers for one track. An app that reads `itunes:episode` showed a different number from an app that reads `podcast:episode`.
- The core models `podcast:episode` (`crates/core/src/feed.rs`). It does not model `itunes:episode`. The lossless tree keeps that element with no change ([ADR 0012](0012-lossless-tree-with-a-typed-model.md)).
- MSP 2.0 writes only `podcast:episode` (`src/utils/xmlGenerator.ts:493` in `../MSP-2.0`).

## Decision

1. The core models the text of `itunes:episode` of an item. It keeps the attributes of the element.
2. When the core gives each track its position (ADR 0024 item 2), it sets the position as the `podcast:episode` and as the `itunes:episode` of the track. It adds each element that the track does not have.
3. Without a change of sequence, the core does not add or change an `itunes:episode`. A read and a write with no edit give the same bytes ([ADR 0008](0008-feed-types-and-feed-identity.md), ADR 0012).
4. The editor shows one track number. It tells the user that Studio sets the two elements from the sequence.

## Alternatives Considered

- Change `itunes:episode` only when the track has one. Not chosen by the operator.
- Remove `itunes:episode`, as MSP 2.0 writes no such element. Not chosen by the operator: an app that reads only `itunes:episode` then shows no number.

## Consequences

- After its first change of sequence, each track of an album has the two elements. The diff shows each new element.
- A feed that Studio does not change in sequence keeps its numbers, also when they do not agree.
- The UI keeps its copy of each item between saves. After a save, it takes the two numbers from the saved draft. Before this decision, it took only `podcast:episode` from the saved draft.

## Invariants

- After a change of sequence, the `podcast:episode` and the `itunes:episode` of each track are its position, from 1.
