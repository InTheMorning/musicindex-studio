# Studio v1 Task 033e: Show Grants And Granted Writes

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 11 to 18, [ADR 0007](../adr/0007-draft-diff-write-and-announce.md) and [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md).
Sequence: the fifth packet of task 033. It depends on 033d.

## Goal

The user asks for a show grant in the draft. The approved write records it. A connected app then changes the status and the live value of that live item through `/app/v1/`. Studio writes each covered change with no new diff.

## Files

Inspect: `crates/core/src/pipeline.rs` (`diff_at`, `write_draft_signed`, `announce_bytes`, `current_draft`, `DraftRow`, `draft_model`), `crates/core/src/live.rs`, `crates/core/src/apps.rs`, `crates/core/src/collection.rs` (`MIGRATIONS`, the draft functions), `crates/api/src/routes.rs` (`write_feed`, `diff_json`), `crates/api/src/lib.rs` (the `/app/v1` router), `crates/api/src/tests.rs`.

Change: `crates/core/src/collection.rs`, the new file `crates/core/src/grant.rs`, `crates/core/src/lib.rs`, `crates/core/src/pipeline.rs`, `crates/core/tests/pipeline.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Do not change: the UI (033f), the connector code, the announcement queue.

## Architecture

These choices are fixed. Do not change them.

### The Tables

- A new migration adds `draft_grants (feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE, item_guid TEXT NOT NULL, PRIMARY KEY (feed_id, item_guid))`: the grants that the draft asks for.
- The same migration adds `show_grants (id INTEGER PRIMARY KEY, feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE, item_guid TEXT NOT NULL, created_at INTEGER NOT NULL, ended_at INTEGER, UNIQUE (feed_id, item_guid))`.
- The same migration adds `grant_writes (id INTEGER PRIMARY KEY, grant_id INTEGER NOT NULL REFERENCES show_grants(id) ON DELETE CASCADE, app_name TEXT NOT NULL, change TEXT NOT NULL, written_at INTEGER NOT NULL)`.

### The Grant In The Draft

- `PUT /api/v1/feeds/{id}/grants/{guid}` and `DELETE` of the same path add or remove a draft grant. A `guid` that is not a `pending` live item of the draft gives 422 `invalid_grant`.
- `Diff` gets `grants: Vec<String>`: each item GUID of `draft_grants` that is a `pending` live item of the draft. The diff ID gets the bytes of the list after the bytes of the announcements, in the same form. `diff_json` gives `grants`.
- After step 5 of a write with no error, each GUID of `Diff.grants` becomes a `show_grants` row (or the row of that GUID starts again with no `ended_at`). The `draft_grants` rows of the feed go out with the draft.

### The Rules In `grant.rs`

```rust
pub enum GrantChange { GoLive, End, SetLiveValue { uri: String, protocol: String }, RemoveLiveValue }
pub fn active(collection, feed_id, item_guid, now) -> Result<Option<ShowGrant>, …>;
pub fn covered(before: &Feed, after: &Feed, item_guid: &str) -> bool;
```

- A grant is active when it has no `ended_at` and the live item is in the hosted feed. Also, `now` must be less than 24 hours after its `end`, or after its `start` when there is no `end`. An unreadable time ends the grant.
- `covered` is true only when the two feeds are equal after this step: in a copy of each, the status and the `live_value` of that live item are made equal. All other parts must be equal.

### The Granted Write

`pipeline::write_granted(collection, hosts, library, addresses, feed_id, item_guid, change, app_name, now) -> Result<Version, PipelineError>`:

1. Refuse with `GrantNotActive` when `grant::active` gives none.
2. Refuse with `DraftOpen` when the feed has a draft that is different from its hosted file. A draft with no change goes out first.
3. Read the hosted file, make a draft from it, and apply the change with `live::set_status` or the live value.
4. Refuse with `NotCovered` when `covered` is false.
5. Make the diff with `diff_at`, and call `write_draft_signed` with its ID. The snapshot, the conflict check and the announcements are those of each write.
6. Add a `grant_writes` row. After `End`, set `ended_at` of the grant.

On an error at steps 3 to 5, the draft of step 3 goes out. Thus the feed has no draft that the user did not make.

### The App Routes

- `GET /app/v1/shows` gives each active grant: `{"grant_id", "feed_id", "feed_title", "item_guid", "title", "status", "start", "end", "live_value"}`.
- `POST /app/v1/shows/{grant_id}/status` with `{"status": "live"}` or `{"status": "ended"}`.
- `PUT /app/v1/shows/{grant_id}/live-value` with `{"uri", "protocol"}`, and `DELETE` of the same path.
- Each change route gives `{"version", "status", "live_value"}` after the write. The errors are: 404 `no_grant`, 409 `grant_ended`, 409 `draft_open`, 409 `bad_status`, 423 `vault_locked`, 409 `conflict`, and 422 `not_covered`.
- A change route wakes the announcement worker after a write, as `write_feed` does.
- `GET /api/v1/feeds/{id}/grants` gives the grants of the feed with their writes, for the UI.

## Steps

1. Add the migration and the collection functions.
2. Add `grant.rs` with unit tests of `active` and `covered`.
3. Add the draft grants, the `grants` of the diff, and the record at the write.
4. Add `write_granted`, with pipeline tests.
5. Add the app routes and the grants route, with API tests that use an app token as a test client.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- A write occurs only through `write_draft_signed`. `write_granted` does not write a host file itself.
- A granted write never writes a draft that the user made.

## Acceptance Criteria

Mechanical:

- A pipeline test: a draft with a pending live item and its draft grant is written, and the grant is active.
- An API test with an app token: `live`, then a live value, then `ended`. The hosted file has each change. `GET /app/v1/shows` is empty after `ended`.
- A pipeline test: a change of the title with a granted write gives `NotCovered`, and the host file does not change.
- A pipeline test: a granted write while the feed has a draft with a change gives `DraftOpen`, and the draft of the user does not change.
- A pipeline test: a granted write 24 hours and 1 second after the `end` gives `GrantNotActive`.
- A pipeline test: a change of the draft grant after the diff gives `DiffNotApproved`.
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

- Stop and report when `diff_at` needs a change of the draft functions for the draft of `write_granted`.
- Stop and report when `covered` needs a field by field compare of the whole model. Propose a compare of the written XML in place of it.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033e-show-grants.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- Add the draft grant in the diff and its ID, and the record of the grant at the write.
- Add `grant.rs` with `active` and `covered`, and `pipeline::write_granted`.
- Add the `/app/v1/shows` routes for connected apps.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- A write occurs only through `write_draft_signed`. A granted write never writes a draft that the user made.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the connector code, the announcement queue.
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
