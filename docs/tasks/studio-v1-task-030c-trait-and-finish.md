# Studio v1 Task 030c: The Trait Functions, The Host Environment And `finish` In The Pipeline

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 4 to 9 and 18, [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 6 and 7.
Sequence: the third packet of task 030. It depends on 030b.

## Goal

The connector trait gets `finish`, `plan_events`, `planned_url` and `kept_blobs`, each with a default. `from_row_in` gives a connector its host environment. The pipeline calls `finish` after each write step and keeps the blob records of a host. The local folder, S3 and SFTP do not change their behavior.

## Files

Inspect: `crates/core/src/connector.rs`, `crates/core/src/connector/suite.rs`, `crates/core/src/connector/local.rs` (its suite tests), `crates/core/src/nostr_events.rs` (`Signing`), `crates/core/src/pipeline.rs` (`write_draft`, `put_uploads_and_copies`, `delete_each_operation`, `keep_draft_after_failed_confirmation`), `crates/core/src/inventory.rs` (`planned_path`, `Hosts`), `crates/core/src/collection.rs` (`MIGRATIONS`, the connector functions), `crates/core/tests/pipeline.rs`, `crates/api/src/workspace.rs` (`hosts`).

Change: `connector.rs`, `suite.rs`, `pipeline.rs`, `inventory.rs`, `collection.rs`, `crates/core/tests/pipeline.rs`, `crates/api/src/workspace.rs`. In `local.rs`, `s3.rs` and `sftp.rs`, add only one test that runs the new suite check, as each file runs the other checks.

Do not change: the behavior of the local folder, S3 and SFTP connectors, the API routes, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Types In `connector.rs`

- `pub struct PlannedWrite { pub path: String, pub sha256: String, pub size: u64 }`.
- `pub struct PlannedDelete { pub path: String, pub sha256: Option<String> }`.
- `pub struct HostChanges { pub writes: Vec<PlannedWrite>, pub deletes: Vec<PlannedDelete> }`: the changes of one write on one host.
- `pub struct PlannedEvent { pub event: nostr::UnsignedEvent, pub user_signs: bool }`. `user_signs` is true for a host with "My signer".
- `pub struct KeptBlob { pub path: String, pub sha256: String, pub size: u64 }`.
- `pub struct HostEnv<'a> { pub vault: Option<&'a OpenVault>, pub blobs: Vec<KeptBlob>, pub signing: nostr_events::Signing }`.

### The Trait Functions

| Function | Default | Purpose |
| --- | --- | --- |
| `fn finish(&self) -> Result<(), ConnectorError>` | `Ok(())` | Publish the changes of this write step. |
| `fn plan_events(&self, changes: &HostChanges, plan_time: u64) -> Result<Vec<PlannedEvent>, ConnectorError>` | an empty list | The events that a write with these changes signs. |
| `fn planned_url(&self, path: &str, sha256: Option<&str>) -> Option<String>` | `self.public_url(path)` | The public address of a file that is not on the host yet. |
| `fn kept_blobs(&self) -> Option<Vec<KeptBlob>>` | `None` | The blob records that the collection keeps for the host. |

### `from_row_in`

- `pub fn from_row_in(row: &ConnectorRow, env: &HostEnv) -> Result<Box<dyn Connector>, ConnectorError>` makes each kind, as `from_row` does now.
- `from_row(row, vault)` calls `from_row_in` with no blobs and `Signing::default()`.
- `Workspace::hosts` uses `from_row_in` with `collection.kept_blobs(row.id)` for each row.
- `Workspace::hosts_with(signing: Signing)` does the same with the given signing. `hosts()` calls it with `Signing::default()`. Packet 030f uses `hosts_with`.

### The Collection

- A new entry in `MIGRATIONS` adds the table `kept_blobs`: `connector_id` (a reference to `connectors` with `ON DELETE CASCADE`), `path`, `sha256` and `size`, with the primary key `(connector_id, path)`.
- `Collection::kept_blobs(id) -> Vec<KeptBlob>` in the sequence of the paths, and `Collection::replace_kept_blobs(id, &[KeptBlob])` in one transaction.

### The Pipeline

- In `write_draft`, after step 2 (the feed XML, or the moved feed XML and the pointer), the pipeline calls `finish` on each host in `touched`, in the sequence of the host IDs. This occurs before `set_feed_location`, `record_state` and the write of the release folder.
- After each `finish` step, the pipeline puts the `kept_blobs` of each host of the step into the collection, when the host gives `Some`.
- A failed `finish` returns the error. The feed state, the feed location and the draft do not change.
- After `delete_each_operation`, the pipeline calls `finish` on each host that a deletion touched, in the sequence of the host IDs. Then it keeps the blobs of those hosts. This occurs before the draft goes out.
- `inventory::planned_path` uses `planned_url(path, file.sha256)` in place of `public_url(path)`, in both of its branches.

### The Suite

- A new check, `finish_keeps_each_change`: write `a.txt`, call `finish`, and read the same bytes. Then delete `a.txt` with its stamp, call `finish`, and read `None`. `run_all` runs it.

## Steps

1. Add the types, the trait functions and `from_row_in`. Change `Workspace::hosts`.
2. Add the `kept_blobs` table and its functions, with a test of the migration and of `replace_kept_blobs`.
3. Change `write_draft` and `planned_path`.
4. Add the suite check and one test for it in each connector test module.
5. Add the pipeline tests below.

## Pipeline Tests

Add them to `crates/core/tests/pipeline.rs`. A test connector wraps `LocalFolder` and records each call as text, for example `write a/01.mp3`, `finish` and `delete old.mp3`. It can fail its first `finish`, and it can give `kept_blobs`.

- A write with one upload, the feed XML and one deletion gives this sequence: the upload, the feed XML, `finish`, the deletion, `finish`.
- A write whose first `finish` fails gives an error. The draft stays, and `collection.feed(id)` has the version from before the write.
- A connector that gives `kept_blobs` puts its records into the collection after the write.
- A host with no deletion gets one `finish` only.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- The existing tests of the local folder, S3, SFTP, the pipeline and the API pass with no change of their assertions.

## Acceptance Criteria

Mechanical:

- The pipeline tests of this packet pass.
- The local folder, S3 and SFTP pass the new suite check.
- A collection of the previous schema version opens and gets the `kept_blobs` table.
- The four commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report in these cases:

- An existing test needs a change of its assertion.
- `write_draft` needs `finish` at a different place than this packet gives.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030c-trait-and-finish.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- The connector trait gets finish, plan_events, planned_url and kept_blobs with defaults. from_row_in gives a connector its HostEnv. write_draft calls finish after the feed XML and after the deletions, and keeps the blob records of each host.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. The local folder, S3 and SFTP keep their behavior. Existing test assertions do not change.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The API routes, the UI, the behavior of the existing connectors.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
