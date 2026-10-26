# Studio v1 Task 033a: The Live Item Rules In The Core

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 2 to 5, and [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md).
Sequence: the first packet of task 033.

## Goal

The core models `podcast:liveValue` on a live item. It makes a new live item, changes its status, and refuses a write of a live item that is not correct.

## Files

Inspect: `crates/core/src/feed.rs` (`LiveItem`, `Item`, `Modeled`), `crates/core/src/feed/elements.rs` (the form of a modeled element with attributes), `crates/core/src/feed/sync.rs`, `crates/core/src/pipeline.rs` (`write_draft_signed`, `bad_addresses`, `PipelineErrorKind`), `crates/core/src/guid.rs`, `crates/core/tests/corpus.rs`, `crates/core/tests/pipeline.rs`, the two corpus feeds with `podcast:liveValue`.

Change: `crates/core/src/feed.rs`, `crates/core/src/feed/elements.rs`, the new file `crates/core/src/live.rs`, `crates/core/src/lib.rs`, `crates/core/src/pipeline.rs`, `crates/core/tests/pipeline.rs`, `crates/api/src/error.rs` (the status of the new error kind only).

Do not change: the API routes, the UI, the corpus files.

## Architecture

These choices are fixed. Do not change them.

### The Model

- `LiveItem` gets `live_value: Option<LiveValue>`. `LiveValue` is `podcast:liveValue` with `uri` and `protocol`, both `Option<String>`, and the lossless rules of the other modeled elements ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- A read and a write with no edit give the same bytes for each corpus feed.

### The Rules

`live.rs` has:

```rust
pub enum LiveStatus { Pending, Live, Ended }
impl LiveStatus { pub fn parse(text: &str) -> Option<Self>; pub fn as_str(self) -> &'static str; }

pub fn new_live_item(title: &str, start: &str) -> LiveItem;
pub fn set_status(item: &mut LiveItem, next: LiveStatus) -> Result<(), LiveError>;
pub fn problems(feed: &Feed) -> Vec<LiveProblem>;
```

- `new_live_item` gives a live item with a new UUID v4 `guid` (not a permalink), the status `pending`, the `start` and the title.
- `set_status` permits only `pending` to `live` and `live` to `ended`. Each other change gives `LiveError` with a text that names the two states.
- `problems` gives one `LiveProblem { guid, field, text }` for each live item with no `start`, no `contentLink`, or a `status` that `LiveStatus::parse` refuses. The texts are "This show has no start time.", "This show has no content link." and "This show has a status that is not pending, live or ended."

### The Write

- `write_draft_signed` gives `PipelineErrorKind::BadLiveItem` when `problems` of the draft is not empty. The check is after the check of the bad addresses, before any change on a host. The message lists each problem.
- The API maps `BadLiveItem` to 422 `bad_live_item`.

## Steps

1. Add `LiveValue` to the model, with a round trip test of the two corpus feeds.
2. Add `live.rs` with unit tests.
3. Add the write check, with a pipeline test.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. The `uuid` crate is already a dependency of the core. If it is not, stop and report.
- The writer changes only the elements that the user changed ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).

## Acceptance Criteria

Mechanical:

- The corpus tests pass with no change of bytes.
- A model test reads the `uri` and the `protocol` of the liveValue of `kolomona-podcast-v4vmusic-live.xml`, changes the `uri`, and the written XML changes only that attribute.
- Unit tests of `set_status` give the two permitted changes and refuse `ended` to `live` and `pending` to `ended`.
- A pipeline test refuses a write of a live item with no `contentLink` with `BadLiveItem`, and the host file does not change.
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

- Stop and report when a corpus feed with a live item does not pass the new check. Name the feed and the problem. Do not change the corpus.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033a-live-item-rules.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- The core models `podcast:liveValue` on a live item. `live.rs` makes a new live item, changes its status, and gives the problems of the live items of a feed. A write with a live item problem gives `BadLiveItem` before any change on a host.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.
- The corpus tests must give the same bytes.

Do not touch:
- The API routes, the UI, the corpus files.
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
