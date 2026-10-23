# Studio v1 Task 021c: The Import Into The Library, And The Diff

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 5, 10, 12 and 14.
Sequence: after [task 021b](studio-v1-task-021b-inventory-and-addresses.md).

## Goal

An imported file goes into its release folder. The diff gives each upload, copy and deletion with its host.

## Files

Inspect: `crates/core/src/pipeline.rs` (`referenced_files`, `diff`, `diff_id`, `Diff`, `DiffFile`, `discard_draft`, `remove_pending_file`), `crates/core/src/library.rs`, `crates/core/src/inventory.rs`, `crates/core/src/collection.rs`, `crates/core/tests/pipeline.rs`.

Change: `pipeline.rs`, the pipeline tests.

Do not change: `crates/api`, the UI, `media.rs`. Keep `write_draft` working for the API until task 021d: it can keep its current signature.

## Architecture

These choices are fixed. Do not change them.

- `pipeline::import_into_library(collection, library, hosts, id, source: &Path, file_name, media_type) -> Result<ImportedFile, PipelineError>`:
  - The feed must have a folder. It gives `NoFolder` otherwise.
  - `library.put_file` puts the file into the release folder. A file with the same name and the same SHA-256 is the same file. A file with the same name and different bytes gives `FileExists`.
  - It adds the `library_files` row with `local = 1`.
  - It gives the `PlannedPath` of task 021b with the library file ID.
- A library file is in use by a draft when its planned URL, or the URL of one of its copies, is an address of the draft: an enclosure, a `podcast:source`, or an `itunes:image` of the channel or of an item.
- `pub struct HostOperation { kind: OperationKind, connector_id, path, size, library_file_id: Option<i64>, url: Option<String> }` with `OperationKind::{Upload, Copy, Delete}`.
  - Upload: each library file in use whose planned path has no copy with the same SHA-256 on its host.
  - Copy: each `draft_operations` row of the kind `copy`, when the host does not have the file.
  - Delete: each `draft_operations` row of the kind `delete`.
- `Diff` gets `operations: Vec<HostOperation>` and `blocked: Vec<BlockedDelete>`. A deletion of a host file whose public address is in the draft of this feed, or in the hosted XML of a different feed of the library, is not an operation. It goes into `blocked`, with the titles of the feeds that refer to it.
- `diff_id` adds the kind, the host, the path and the SHA-256 of each operation, in sequence.
- `Diff.files` stays, with the uploads, until task 021g changes the UI.
- `discard_draft` removes the `draft_operations` of the feed. It does not remove library files.

## Steps

1. Add `import_into_library` and the use of library files.
2. Add the operations, the blocked deletions and the diff ID.
3. Add the tests.

## Constraints

- No file goes to a host in this packet.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- An import puts the file into the release folder and gives its planned URL. An import of the same file again gives the same row.
- A draft that refers to an imported file gives one upload. After the file is on the host with the same bytes, the diff gives no upload.
- A file whose host is a second folder gives an upload to that host, with its URL.
- A copy operation gives a copy. A deletion of a file that the draft refers to goes into `blocked`, with the feed title.
- The diff ID changes when an operation changes.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the API no longer compiles because of a change of this packet.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021c-library-import-and-diff.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- An imported file goes into its release folder. The diff gives each upload, copy and deletion with its host.

Constraints:
- Follow the Architecture section of the task file fully.
- No file goes to a host in this packet.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api, the UI, media.rs.
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
