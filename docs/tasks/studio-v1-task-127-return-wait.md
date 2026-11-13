# Studio v1 Task 127: Review Gives The Return Wait Of Music Index

Decision: [ADR 0190](../adr/0190-review-gives-the-return-wait-of-music-index.md).

Status: Not started. Stophammer `v0.20.0-rc.1` (commit `508e4db`) runs on `api.musicindex.org` with Stophammer ADR 0097. On 2026-11-12 at 19:24 EST, `GET /v1/feeds/7ea30e6c-c7ba-5b81-a7be-6465c1591b7a` gave the key `return_wait_until` with the value `null`.

## Goal

The Review warning of a return gives the end of the wait of Music Index. The podping of a return goes 30 seconds after the end of that wait.

## Facts From Stophammer ADR 0097

- `GET /v1/feeds/{guid}` gives `return_wait_until` in `data`. It is an integer of Unix seconds, or `null` when no wait is active.
- A server before ADR 0097 gives no key. A community node always gives `null`.
- The first return in one day has no wait. The second return waits 60 seconds. Each return after that doubles the wait, to a limit of 1 hour.
- The index does not read the feed when the wait ends.

## Files To Read First

- `crates/core/src/musicindex.rs`: `FeedData` (near line 450) and `MusicIndex::feed_presence` (near line 750).
- `crates/core/src/announce.rs`: `Timing` (line 71), `queue` (near line 229) and the 24-hour expiry (near line 340).
- `crates/core/src/outside.rs`: `last_written_move`.
- `crates/api/src/routes.rs`: `write_feed` (near line 3230) and the `feed_move` of the diff (near line 3190).
- `crates/api/src/feed_retire.rs`: a Music Index read outside the workspace lock.
- `crates/api/src/tests/feed_retire.rs`: a local test server for Music Index (near line 130).
- `crates/api/ui/movelink.js`: `RETURN_TITLE`, `RETURN_TEXT` and `returnWarning` (near line 77).
- `crates/api/ui/editor.js`: `ReturnWarning` (near line 1751).
- `crates/api/ui/outsidecheck.js`: the local time of the notice of ADR 0184.
- Tests: `crates/core/tests/move_link.rs` (`the_diff_of_a_return_has_the_new_value_and_a_normal_move_does_not`), `crates/core/tests/pipeline.rs` (`a_feed_move_queues_only_the_address_from_before`), `crates/api/ui-tests/movelink.test.mjs` (`only a return gets the warning, and the warning gives no time`).

## Packets

### Packet 127a: The Read And The Send Time (Core And API)

1. `musicindex.rs`: a type `ReturnWait` with three values: `Until(u64)`, `None` (the value `null`) and `Unknown` (no key). The function `MusicIndex::return_wait(guid)` reads `GET /v1/feeds/{guid}`. A failed read gives `Unknown`. Keep `feed_presence` as it is.
2. The core gives the state of the warning from `ReturnWait` and the current time. The states are `waits` (with the time and a flag `too_far`), `follows` and `unknown`. A time that is not in the future gives `follows`. `too_far` is true when the time plus 30 seconds is more than 23 hours after the current time (ADR 0190 items 2, 3, 4 and 7).
3. A route `GET /feeds/{id}/return-wait` gives `{ "state": "waits" | "follows" | "unknown", "until": u64 | null, "too_far": bool }`. The route reads Music Index outside the workspace lock, as `feed_retire.rs` does. A feed with no GUID gives `unknown`.
4. `announce.rs`: a function moves the `next_at` of the waiting announcement of one address to a given time. It does not change `created_at`. It does nothing when the time is more than 23 hours after the current time (ADR 0190 item 7).
5. `write_feed`: after a write with a return (a feed move with `replaces_pointer`), the route reads `return_wait` outside the lock. With `Until(t)` and `t` in the future, it moves the send time of the announcement of ADR 0189 item 1 to `t + 30` (ADR 0190 item 6). A failed read keeps the usual send time.
6. The diff ID does not change. Do not add the wait to the inputs of the diff ID (ADR 0190 item 8).

Mechanical acceptance criteria:

- A core or API test: a time in the future, `null`, a time in the past, no key and a failed read give the states `waits`, `follows`, `follows`, `unknown` and `unknown`.
- A core or API test: a time plus 30 seconds that is more than 23 hours after the current time gives `too_far`.
- An API test with a local Music Index server: a return write with a time in the future queues the podping. Its send time is that time plus 30 seconds. With a time more than 23 hours after the write, the send time stays at the write time plus `first_wait`.
- An API test: two diffs of the same draft, with two different times from the local server, give the same diff ID.
- Each new assertion has a failure message that names ADR 0190 and the fix.

### Packet 127b: The Warning (UI)

1. `movelink.js`: `returnWarning(move, wait)` gives the text of ADR 0190 item 2, 3 or 4 from the state of the route. With `too_far`, the text of item 2 adds "Send the podping again after that time." (item 7). The time is local, with the formatter of the notice of ADR 0184. Remove the text "you do not need to do more". Correct the comment above the constants.
2. `editor.js`: `ReturnWarning` loads `GET /feeds/{id}/return-wait` only when `move.replaces_pointer` is true. While the read runs, and when it fails, the warning shows the text of item 4.
3. Node tests: each state gives its text, and no text contains "you do not need to do more". Replace the test `only a return gets the warning, and the warning gives no time`.

Mechanical acceptance criteria: the Node tests above.

Visual acceptance criteria, by the planning model: Review with the warning of items 2, 3 and 4, at 1280 px in the light theme. `$edges()` gives no result.

## Acceptance Criteria

Mechanical: the criteria of each packet, and the four checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the planning model: the criterion of packet 127b.

Human, by the operator, in a release build of local `main`. This gate stays open until it runs:

- Two returns of the test album, one after the other. At the second return, Review gives the time of the index. After that time, the notice line changes to "Sent" with no click, and Music Index follows the return. The planning model records each value of `return_wait_until` and its time for the orchestrator.

## Stop And Ask When

- The write result does not tell the route that the write had a return.
- The notice of ADR 0184 does not show the moved send time.
- A different caller of `queue` needs a different send time.
