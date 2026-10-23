# Studio v1 Task 021e: The Move Of The Current Collection Into The Library

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) item 22, and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 1 to 5.
Sequence: after [task 021d](studio-v1-task-021d-write-with-host-operations.md).

## Goal

When the user sets the library folder for the first time, each feed of the collection gets its release folder, and each pending file moves into it.

## Files

Inspect: `crates/core/src/collection.rs`, `crates/core/src/library.rs`, `crates/core/src/inventory.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/imports.rs`, `crates/core/src/connector.rs`.

Change: a new `crates/core/src/migrate.rs`, `lib.rs`, the tests. Remove `imports.rs`, the `draft_files` methods and their tests when nothing uses them after this packet. A new migration drops the table `draft_files`.

Do not change: `crates/api` except the removal of the store from `workspace.rs`, the UI.

## Architecture

These choices are fixed. Do not change them.

- `migrate::into_library(collection, hosts, library, store_root: &Path) -> Result<MoveReport, MigrateError>`:
  1. For each feed with a host and a path, and no folder: give it a release folder with `release_folder_name`, and make the folder.
  2. Read the feed file through its connector, and write it into the release folder with `write_feed_xml`. A feed with no hosted file (a new release) writes nothing.
  3. Set `files_connector_id` to the host of the feed XML.
  4. For each `draft_files` row: move its copy from `<store_root>/<feed>/<sha256>` into the release folder with its file name, and add a `library_files` row with `local = 1`. Remove the row.
  5. Run `inventory::scan` for each host.
  6. A feed with no host (a read-only feed) gets a release folder with its XML from its address, as the last read gave it.
- `MoveReport` gives the number of release folders, of feed files and of moved files, and each error with its feed.
- A second call does nothing for a feed that has a folder.
- The function does not download an audio file or an image. ADR 0035 item 6 makes downloads optional, and milestone 7b offers them.

## Steps

1. Add `migrate.rs`.
2. Remove the store and `draft_files` when nothing uses them, with a migration that drops the table.
3. Add the tests.

## Constraints

- A move does not change a hosted file.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A collection with two feeds on one local host and one pending file gives two release folders. Each folder has a feed XML file with the bytes of the hosted file. The pending file is in its release folder, with a `library_files` row.
- A second call changes nothing.
- No hosted file changes. The test compares the bytes of the host folder before and after.
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

Stop and report when a part of the API still uses the store or `draft_files`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021e-move-into-the-library.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- When the user sets the library folder for the first time, each feed gets its release folder, and each pending file moves into it.

Constraints:
- Follow the Architecture section of the task file fully.
- A move does not change a hosted file, and downloads no audio file or image.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api except the store in workspace.rs, the UI.
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
