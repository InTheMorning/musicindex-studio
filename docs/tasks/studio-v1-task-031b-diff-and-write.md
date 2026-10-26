# Studio v1 Task 031b: The Announcements In The Diff And The Write

Parent: [task 031](studio-v1-task-031-announce.md). Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) items 7 to 13, [ADR 0021](../adr/0021-public-address-of-a-feed.md) and [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 21.
Sequence: the second packet of task 031. It depends on 031a.

## Goal

The diff gives the planned announcements, or the reason for no announcement. The planned announcements are a part of the diff ID. A write that completes with no error puts them into the queue.

## Files

Inspect: `crates/core/src/announce.rs`, `crates/core/src/pipeline.rs` (`Diff`, `diff_at`, `write_draft_signed`, `feed_move`, `feed_move_bytes`, `written_feed_url`, `diff_id`, the pipeline tests), `crates/api/src/routes.rs` (`diff_json`), `crates/api/src/tests.rs` (the test that checks `diff["announce"]`).

Change: `crates/core/src/announce.rs`, `crates/core/src/pipeline.rs`, `crates/api/src/routes.rs` (`diff_json` only), `crates/api/src/tests.rs` (the `announce` assertion only).

Do not change: the collection schema, the API routes other than `diff_json`, the desktop crate, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Plan

- `announce.rs` gets:

  ```rust
  pub enum AnnounceSkip { Off, NoAddress }   // serde::Serialize, snake_case
  pub fn plan(settings: &AnnounceSettings, addresses: &[String]) -> (Vec<PlannedAnnouncement>, Option<AnnounceSkip>);
  ```

- With `settings.pubnotify` false, `plan` gives no announcement and `Some(Off)`.
- With no address, it gives no announcement and `Some(NoAddress)`.
- If not, it gives one `PlannedAnnouncement` for each address, in sequence, with `podping = settings.podping`. A second copy of an address goes out.
- `plan` is a pure function. Test it with no collection.

### The Addresses In The Pipeline

- A new private function in `pipeline.rs` gives the addresses of a write, before the first change on a host:
  - With no feed move: the address that `written_feed_url` gives for the current host and path. That is the public address of the feed, or the public address of the path on its host.
  - With a feed move: `feed_move.url`, then `feed_move.from_url`. Each one that is `None` goes out.
- `diff_at` and `write_draft_signed` call the same function with the same values, so that the two give the same plan.

### The Diff

- `Diff.announce: bool` goes out. `Diff` gets `announcements: Vec<PlannedAnnouncement>` and `announce_skip: Option<AnnounceSkip>`.
- The diff ID gets the bytes of the plan, after the bytes of the feed move: a zero byte, `announce`, and for each announcement a zero byte, the address, a zero byte and `1` or `0` for the podping. With `announce_skip`, add a zero byte and `off` or `no_address`. `diff_at` and `write_draft_signed` add the same bytes.
- `diff_json` gives `announcements` (a list of `{"url", "podping"}`) and `announce_skip` (`"off"`, `"no_address"` or `null`). It gives no `announce` field.

### The Write

- `write_draft_signed` reads `AnnounceSettings`, makes the plan before step 1, and includes it in the diff ID check.
- After step 5, when the write has no error, it calls `announce::queue` with the plan, `Timing::DEFAULT` and the `now` of the write. This is the last step before `Ok(version)`.
- A write that gives an error at any step puts nothing into the queue (ADR 0052 item 13).
- `write_draft` gives no new argument. The API worker of packet 031c uses its own `Timing` for the retries only, not for the queue. Thus `queue` in the write always uses `Timing::DEFAULT.first_wait`.

## Steps

1. Add `AnnounceSkip` and `plan`, with unit tests.
2. Add the address function and the plan to `diff_at`, with the diff ID bytes.
3. Add the plan to the ID check and the queue call to `write_draft_signed`.
4. Change `diff_json` and the API assertion that checked `announce`.
5. Add the pipeline tests of the acceptance criteria.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- The write order of ADR 0035 item 15 does not change. The queue call adds no step before step 5.
- No test sends a request to a host outside the computer. This packet sends no request.

## Acceptance Criteria

Mechanical:

- A pipeline test: a write of a feed with a public address puts one `waiting` row with that address and `podping` true.
- A pipeline test: with `announce_pubnotify` false, the diff gives `announce_skip` `Off` and the write puts no row.
- A pipeline test: with `announce_podping` false, the row has `podping` false.
- A pipeline test: a feed with no public address and a host with no public address gives `NoAddress`, and the write succeeds.
- A pipeline test: a feed move gives two rows, the new address first in the plan.
- A pipeline test: a change of a setting after the diff makes the write fail with `DiffNotApproved`.
- A pipeline test: a write that stops with `Conflict` puts no row.
- The API test of the feed move checks `diff["announcements"]` in place of `diff["announce"]`.
- The three commit checks of Rust pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when `diff_at` and `write_draft_signed` cannot get the same addresses. For example, the public address of a new feed can change during the write.
- Stop and report when a test of a different area fails because of the new diff ID bytes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-031b-diff-and-write.md
- docs/adr/0052-announcements-through-the-podcast-index-hub.md
- The files that the Files section names.

Goal:
- The diff gives the planned announcements or the reason for none, and they are a part of the diff ID. A write with no error puts them into the announcement queue.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.
- Do not change the write order of ADR 0035 item 15.

Do not touch:
- The collection schema, the API routes other than `diff_json`, the desktop crate, the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
