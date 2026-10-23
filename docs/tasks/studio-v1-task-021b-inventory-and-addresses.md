# Studio v1 Task 021b: The Host Inventory And The Addresses

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 6 to 9, and [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) items 12, 13 and 16.
Sequence: after [task 021a](studio-v1-task-021a-records-and-library-folder.md).

## Goal

Record the files of each host, and give the host and the address of each library file.

## Files

Inspect: `crates/core/src/collection.rs` (the methods of task 021a), `crates/core/src/connector.rs` (`list`, `FileList`, `from_row`), `crates/core/src/connector/local.rs`, `crates/core/src/library.rs`, `crates/core/src/web.rs` (`url_path`), `crates/core/src/digest.rs`.

Change: a new `crates/core/src/inventory.rs`, `lib.rs`, `collection.rs` only if a query is missing, and the tests.

Do not change: `pipeline.rs`, `crates/api`, the UI.

## Architecture

These choices are fixed. Do not change them.

- `pub struct Hosts` holds one `Box<dyn Connector>` for each `ConnectorId`. `Hosts::get(id) -> Result<&dyn Connector, InventoryError>`. The API makes it with `connector::from_row` and the vault. Tests make it from local folders.
- `inventory::scan(collection, hosts, connector_id) -> Result<ScanResult, InventoryError>`:
  - It calls `list` and puts each file into `host_files` with its size, its version and `seen_at`.
  - It removes each `host_files` row of that host that the list does not have.
  - A row keeps its `sha256` when its version did not change. Otherwise the `sha256` becomes `NULL`.
  - `ScanResult` gives the number of files that are new, changed and gone.
- `inventory::host_sha(collection, hosts, connector_id, path) -> Result<Option<String>, InventoryError>` gives the recorded SHA-256, or reads the file, records its SHA-256 and gives it. It gives `None` when the file is gone.
- `inventory::copies(collection, hosts, file: &LibraryFile) -> Result<Vec<Copy>, InventoryError>` gives each host file with the same size and the same SHA-256 as the library file, with its host, its path and its public address. It reads a SHA-256 only for a host file with the same size.
- `inventory::host_of(feed: &FeedRow, file: &LibraryFile) -> Option<ConnectorId>` gives `file.connector_id`, else `feed.files_connector_id`, else `feed.connector_id`.
- `inventory::planned_path(collection, hosts, feed: &FeedRow, file: &LibraryFile) -> Result<PlannedPath, InventoryError>`:
  - The clean path is `<folder on the host>/<file path>`. The folder on the host is the feed path with no extension, as in `pipeline::release_path`.
  - When the inventory has no file at the clean path, or a file with the same SHA-256, the result is the clean path.
  - When the inventory has a different file there, the result is the hash name: the first 8 hexadecimal characters of the SHA-256 before the extension, for example `my-album/01 Song.3f9a2c1d.mp3`.
  - `PlannedPath { connector_id, path, url, on_host: bool }`. `url` is `public_url(path)`. `on_host` is true when a file with the same SHA-256 is at the path.

## Steps

1. Add `Hosts` and `scan`.
2. Add `host_sha`, `copies`, `host_of` and `planned_path`.
3. Add the tests with two local folders.

## Constraints

- A scan reads no file bytes.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A scan of a folder with three files records three rows. After one file changes and one is removed, a second scan gives one changed and one gone, and the changed row has no SHA-256.
- `copies` of a library file finds its copy on the second host, and does not find a file with the same size and different bytes.
- `planned_path` gives the clean path for an empty host, the clean path with `on_host: true` for the same file, and the hash name for a different file at the clean path.
- `host_of` gives the file host, then the default host of the release, then the host of the feed XML.
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

Stop and report when the inventory needs a change of the connector interface.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021b-inventory-and-addresses.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- Record the files of each host, and give the host and the address of each library file.

Constraints:
- Follow the Architecture section of the task file fully.
- A scan reads no file bytes.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- pipeline.rs, crates/api, the UI.
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
