# Studio v1 Task 114: A Move Replaces A File At The Feed Path

Decision: [ADR 0178](../adr/0178-a-move-replaces-a-file-at-the-feed-path-in-studio.md).

Status: Open.

## Goal

A move of the feed file continues when the new host has an old copy of the same feed at the feed path. For a different file, the move sheet offers **Replace the File**, and the replace goes through Review.

## Constraints

- Each rule is in the core or the API. The UI model gives the rows and the values, and the UI shows them (ADR 0065, ADR 0042).
- Use only the current tokens and the current classes. A new marker or a new color needs a shared musicindex ADR first.
- No test sends a request outside the computer (ADR 0045). Use the test connectors.
- Keep ADR 0126 items 17 to 22 for a file that has a record.

## Files To Read First

- ADR 0178, and ADR 0126 items 17 to 22.
- `set_feed_host` in `crates/core/src/pipeline.rs`, near the text "a file exists at". The same text is at three places in that file.
- `is_placed_feed_file` and `is_pointer_of_feed` in `crates/core/src/collection.rs`.
- `MoveReleaseSheet` in `crates/api/ui/editor.js`, and `crates/api/ui/releasemove.js`.

## Packets

### 114a: The Core And The API

ADR 0178 items 1, 2, 4, 6 and 7.

- Read the file at the feed path on the new host when the collection has no record of it.
- Give a refusal with the path, the host, the title and the GUID.
- Add the mark of item 6 to the draft move, and examine it in the write.
- Add the Rust tests of the ADR.

### 114b: The Sheet And Review

ADR 0178 items 3, 5, 6 and 8.

- Show the refusal with **Replace the File**. Make **Add to the Draft** not available during a refusal.
- Show the Review rows of items 3 and 6.
- Add the Node test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0178.

Visual, by the planning model: the criteria of ADR 0178.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0178.

## Stop And Ask When

- The read of the file needs a connector function that Studio does not have.
- A rule of ADR 0126 conflicts with ADR 0178.
