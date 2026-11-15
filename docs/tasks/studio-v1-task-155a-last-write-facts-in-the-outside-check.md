# Studio v1 Task 155a: The Outside Check Gives The Facts Of The Last Write

Decision: [ADR 0175](../adr/0175-the-check-shows-what-music-index-holds.md) items 1 and 10. The main session changes item 1 in the commit of task 155b, with a `- Revised:` line. Item 1 then permits the requests of the notice after a write.

Status: Not started. The packet is ready for a delegate. Task 155b needs this task first.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 11.

## Goal

The response of `GET /api/v1/feeds/{id}/outside?index=true` gives the UI three facts about the last write. The facts are its time, the state of its podping, and if Music Index has read it.

## Operator Choices

- Operator choice (2026-11-15), from drawings: "Notice on the feed page". After a write with a podping, the feed page shows a notice with the state of the podping, and Studio asks Music Index at intervals.
- A write with a move keeps the notice of ADR 0184. This task gives no second notice for that write.

## Code Locations

- `crates/api/src/routes.rs`, `outside_check`, lines 1847 to 1926, and `outside_check_reads`, lines 1936 to 1955.
- `crates/core/src/outside.rs`, `last_written_move`, lines 65 to 120: the last move with the state of each podping. `index_difference`, lines 467 to 473: the one request to Music Index.
- `crates/core/src/feed_check/index.rs`, `index_line`, lines 54 to 70: the comparison of the `last_build_date` of the index with the `lastBuildDate` of the last write.
- `crates/core/src/pipeline/check_repair.rs`, `check_inputs`, lines 41 to 57: the build date of the last write, from `draft.base_xml`.
- `crates/core/src/musicindex.rs`, `feed_presence`, lines 808 to 873: `GET /v1/feeds/{guid}`.
- `crates/core/src/announce.rs`, `Announcement`, lines 187 to 209: the queue entry of a podping.
- Tests: `crates/api/src/tests.rs`, lines 16367 to 16485 (the outside check), and `crates/core/src/outside.rs`, lines 485 to 733.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Podping state of a write | extend | crates/core/src/outside.rs | `last_written_move` reads the queue for a move. A new function reads it for the last write with no move, in the same form. |
| Build date comparison | extend | crates/core/src/feed_check/index.rs | The comparison of `index_line` moves to one function that the check and this task use. |
| Music Index request | reuse | crates/core/src/musicindex.rs | `feed_presence` gives the address and the build date in one request. |
| Outside check route | extend | crates/api/src/routes.rs | The response gets the key `last_write`. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- The keys that the response has today, and the notice of ADR 0184
- The texts and the rows of the tab **Check**
- `crates/api/ui/`

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` and the skill `rust-dev` before you write code, a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- The route sends at most one request to Music Index for each call, as today. A call with no `index=true` sends none.
- Do not copy the date comparison. Make one function and use it in the two places.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. In `crates/core/src/outside.rs`, add a function that gives the last write of a feed with no move: the time of the write and the podpings of that write, in the form of the `announcements` of `last_written_move`. It gives nothing when the last write is a move, or when the write planned no podping.
2. In `crates/core/src/feed_check/index.rs`, make one public function for the comparison of the two build dates. It gives "read", "not read" or "not known". `index_line` uses it.
3. In `outside_check_reads`, get the address and the `last_build_date` of the index from one `feed_presence` request. `index_url` keeps its value of today.
4. In `outside_check`, add the key `last_write` to each response: `null`, or an object with `written_at`, `announcements` and `index_read`. `index_read` is `true`, `false` or `null`.
5. `index_read` is `null` when the request did not ask the index, when the index does not hold the feed, when a date is missing, or when the request failed.

## Acceptance Criteria

Mechanical:

1. A core test: the new function gives the time and the podping state for a write with a podping, and nothing for a write with the check box off and for a move.
2. A core test of the comparison function: an earlier index date gives "not read", an equal or later date gives "read", and a missing date gives "not known". The tests of `index_line` pass with no change of their assertions.
3. An API test: after a write with a podping, the response has `last_write` with `index_read: false` for an index with an earlier date, and `true` for an index with the date of the write.
4. An API test: with no `index=true`, the route sends no request to the index and `index_read` is `null`. With `index=true`, the test index gets one request.
5. An API test: the response for a move has `last_move` as today and `last_write: null`.
6. `tools/gate.sh` passes.

Visual: none. Task 155b has the visual gate.

## Test Commands

- `cargo test -p studio-core outside`
- `cargo test -p studio-core feed_check`
- `cargo test -p studio-api outside`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-155a-last-write-facts-in-the-outside-check.md` (this file), each section
- `docs/adr/0175-the-check-shows-what-music-index-holds.md`, items 1 and 10
- `docs/adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md`
- `crates/api/src/routes.rs`, `outside_check` and `outside_check_reads`
- `crates/core/src/outside.rs`
- `crates/core/src/feed_check/index.rs`
- `crates/core/src/musicindex.rs`, `feed_presence`

Goal:
- The response of `GET /api/v1/feeds/{id}/outside?index=true` gets the key `last_write`, with the time of the last write, the state of its podping, and `index_read`.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- The keys that the response has today, and the notice of ADR 0184
- The texts and the rows of the tab **Check**
- `crates/api/ui/`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `cargo test -p studio-core outside`
- `cargo test -p studio-core feed_check`
- `cargo test -p studio-api outside`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
