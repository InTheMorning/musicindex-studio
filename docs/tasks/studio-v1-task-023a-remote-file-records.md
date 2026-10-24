# Studio v1 Task 023a: The Records Of Remote Files

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 1 to 5, 9, 13, 16 and 20, [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md).
Sequence: the first packet of task 023.

## Goal

The collection has the schema of milestone 7b. Each address of a feed with a release folder is a library file, also when its bytes are not in the library.

## Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `LibraryFile`, `add_library_file`, `library_files`), `crates/core/src/inventory.rs` (`copies`, `planned_path`, `file_matches`), `crates/core/src/pipeline.rs` (`feed_addresses`, `host_operations`, `operation_bytes`, `import_into_library`, `library_file_in_use`, `put_uploads_and_copies`), `crates/core/src/migrate.rs`, `crates/core/src/lib.rs`, `crates/api/src/routes.rs` (each use of `LibraryFile::sha256`).

Change: `collection.rs`, `inventory.rs`, `pipeline.rs`, `migrate.rs`, `lib.rs`, a new module `crates/core/src/remote.rs`, and `routes.rs` only where `sha256` becomes optional.

Do not change: the UI, the connector interface, `crates/core/src/connector/`.

## Architecture

These choices are fixed. Do not change them.

### Migration 6

One new entry of `MIGRATIONS`. SQLite cannot change a column, so the migration makes a new `library_files` table, copies the rows, drops the old table and renames the new one. Keep each column and each constraint, with these changes:

- `sha256 TEXT` with no `NOT NULL`.
- A new column `stamp TEXT`: the size and the modification time of the file in the library, as `{size}-{mtime_ns}`, as the stamp of the local folder connector. `NULL` for a remote file, and for a row from before this migration.

The migration also adds:

- `downloads`: `library_file_id INTEGER PRIMARY KEY REFERENCES library_files(id) ON DELETE CASCADE`, `state TEXT NOT NULL CHECK (state IN ('queued', 'failed'))`, `message TEXT`, `added_at INTEGER NOT NULL`. A row is a download that is not complete (ADR 0038 item 9).
- `draft_feed_moves`: `feed_id INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE`, `connector_id INTEGER NOT NULL REFERENCES connectors(id)`, `path TEXT NOT NULL`, `added_at INTEGER NOT NULL` (ADR 0038 item 16).
- `feed_pointers`: `connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE`, `path TEXT NOT NULL`, `feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE`, `added_at INTEGER NOT NULL`, `PRIMARY KEY (connector_id, path)` (ADR 0038 item 20).

Add no `Collection` functions for `downloads`, `draft_feed_moves` and `feed_pointers` in this packet. Packets 023c and 023g add them.

### The Optional SHA-256

- `LibraryFile::sha256` becomes `Option<String>`, and `LibraryFile` gets `stamp: Option<String>`. `add_library_file` takes `sha256: Option<&str>` and `stamp: Option<&str>`.
- `import_into_library` records the stamp of the file that it put into the release folder.
- A file with no SHA-256 is never an upload and never a copy in `host_operations`. `planned_path` of a file with no SHA-256 gives the clean path with `on_host` false. It does not read a host file for it.
- `inventory::copies` of a file with no SHA-256 gives each host file whose public address is its `source_url` (ADR 0038 item 5).
- `operation_bytes` uses an empty text for a file with no SHA-256.

### The Remote Records

New module `remote`, with:

```rust
pub fn record_remote_files(collection: &Collection, feed_id: FeedId, feed: &Feed) -> Result<RemoteReport, CollectionError>;
```

- Examine each address of `pipeline::feed_addresses(feed)`. Skip an address that is the `source_url` of a row of the feed, or the address of a copy of a local row. For each other address, add a row with `local` false, `source_url` the address, and `sha256` and `stamp` `NULL` (ADR 0038 item 1). To find the copies of the local rows, the function takes the `Hosts` as a second argument. Change the signature to `(collection, hosts, feed_id, feed)`.
- The path is the last segment of the address path, with the percent encoding decoded, and with the check of `library::is_plain_name`. Make that function `pub(crate)`. A segment that fails the check, or an empty one, becomes `file`, with the extension of the media type. A path in use gets ` 2`, ` 3` and so on before the extension (ADR 0038 item 2).
- The size is the `length` of the enclosure with this `url`, when the feed gives one. Otherwise 0. The media type is the enclosure `type`, or the `type` of the alternative enclosure of the source, or the type of the extension, or `application/octet-stream`.
- Each row with `local` false whose `source_url` is not in `feed_addresses(feed)` goes out, with its `downloads` row (ADR 0038 item 3).
- The function changes no file on disk.
- `RemoteReport` gives the number of added rows and of removed rows.

Do not call `record_remote_files` from a route or from the pipeline in this packet. Packet 023b calls it.

## Steps

1. Add migration 6, with a test. The test opens a version 5 database with one library file, and finds the row with its SHA-256 and a `NULL` stamp.
2. Make `sha256` optional, add `stamp`, and correct each use.
3. Add `remote::record_remote_files`, with the tests below.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- A schema change is a new migration ([ADR 0014](../adr/0014-collection-in-sqlite.md)).

## Acceptance Criteria

Mechanical:

- A version 5 database opens at version 6, and keeps each library file with its SHA-256.
- A feed with two enclosures, one alternative source, one image and no library file gets four remote rows. Each row has the correct path, size and media type.
- An address that is the planned address of a local library file, or the address of one of its copies, gets no remote row.
- Two addresses with the same last segment get the paths `a.mp3` and `a 2.mp3`.
- A second call with the same feed adds nothing. A call with a feed that no longer has an address removes its remote row.
- A remote file is not an upload in the diff, and `inventory::copies` gives the host file at its `source_url`.
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

Stop and report when a use of `sha256` needs a rule that this packet does not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023a-remote-file-records.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- The files that the Files section names.

Goal:
- The collection has the schema of milestone 7b. Each address of a feed with a release folder is a library file, also when its bytes are not in the library.

Constraints:
- Follow the Architecture section of the task file fully.
- A schema change is a new migration. No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the connector interface, crates/core/src/connector/.
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
