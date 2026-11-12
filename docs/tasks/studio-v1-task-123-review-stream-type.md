# Studio v1 Task 123: Review Checks The Stream Type Of A Live Show

Decision: [ADR 0187](../adr/0187-review-checks-the-stream-type-of-a-live-show.md).

Status: Complete on 2026-11-12. Packet 123a: the diff and the write check each changed live stream in the core, also for a granted write. Packet 123b: Review shows the refusal with **Use {server label}** and no write, and the warning with the write. The screenshot visual check passed, with a real check of a loopback stream. The human test is open.

## Goal

Review checks the stream of each live item that the draft adds or changes. A live stream with a different type stops the write, and **Use {server label}** corrects it. A stream that is not live gives a warning, and the write stays available.

## Constraints

- The decision is in the core (ADR 0065). The core refuses the write, also when the write does not come from Review.
- A granted write from a connected app gets the same check and the same refusal (ADR 0162 item 7).
- Use the agreement test of ADR 0162 item 5. Do not make a second table of types.
- No test sends a request outside the computer (ADR 0045). Use a `StreamCheck` test value.

## Files To Read First

- ADR 0187, ADR 0162 and ADR 0161.
- `crates/core/src/stream.rs`: `StreamCheck` and `StreamStatus`.
- `crates/core/src/live.rs`: `stream_type_agrees` and the problems of a live item.
- `crates/core/src/review.rs`, and the diff and the write in `crates/core/src/pipeline.rs`.
- `crates/core/tests/pipeline.rs`: `StreamLive`.
- `crates/api/ui/review.js`, `crates/api/ui/live.js` and their tests.

## Packets

### 123a: The Check In The Core

ADR 0187 items 1, 2, 4 to 8.

- The diff checks the stream of each live item that the draft adds or changes. It waits at most 8 seconds for each item.
- A live stream with a server type that does not agree gives an error at that item, with the text of item 2. The diff is then not writable.
- A stream that is not live, or a check with no answer, gives the warning of item 4.
- The write runs the same check and refuses with the error of item 2. This includes a granted write.
- Add the three Rust tests of the ADR.

### 123b: Review

ADR 0187 items 2 to 5.

- The Review model gives the error with **Use {server label}** and no write. It gives the warning with the write.
- **Use {server label}** sets the stream type of the draft. Review then shows the new diff.
- Add the Node test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0187.

Visual, by the planning model:

- Review with the error and with the warning, at 1280 px and at 390 px, in the light and dark themes.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0187, with a real Icecast mount that is live.

## Stop And Ask When

- The check of a granted write makes a connected app wait more than its request time.
