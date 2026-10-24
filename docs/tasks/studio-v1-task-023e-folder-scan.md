# Studio v1 Task 023e: The Folder Scan

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 13 to 15, [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) items 16 and 23.
Sequence: after [task 023c](studio-v1-task-023c-download-queue.md).

## Goal

Studio compares each release folder with its records and gives each difference. The user decides what Studio does with each one.

## Files

Inspect: `crates/core/src/library.rs`, `crates/core/src/collection.rs` (`LibraryFile`, migration 6), `crates/core/src/connector/local.rs` (the stamp), `crates/core/src/inventory.rs` (`copies`, `planned_path`), `crates/core/src/download.rs`, `crates/api/src/routes.rs` (`reference_operation`, `reference_feed`, `file_addresses`, the draft routes), `crates/api/src/downloads.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Change: `library.rs`, `collection.rs`, `lib.rs` of the core, a new module `crates/core/src/folder_scan.rs`, `routes.rs`, `lib.rs` of the API, `tests.rs`.

Do not change: the UI, the schema, the connector interface.

## Architecture

These choices are fixed. Do not change them.

### The Scan

- `folder_scan::scan(collection, library) -> Result<Vec<Difference>, ...>`. It examines the files of each release folder, at the top level only. It does not examine a subfolder, a hidden file, a partial file (`.*.part`) or the feed XML of the release.
- `Difference { feed_id, kind }`, with `kind`:
  - `New { path, size }`: a file with no row of the feed.
  - `Missing { file }`: a row with `local` true and no file.
  - `Changed { file, size, sha256 }`: a row with `local` true whose stamp is different, and whose SHA-256 is different.
- The stamp of a library file is `{size}-{mtime_ns}`, as for the local folder connector. Put one function for it in `library`, and use it in `import_into_library` and in `download::finish` too.
- The scan reads a file only when its stamp is different from the row, or the row has no stamp. When the SHA-256 is the same, the scan records the new stamp and gives no difference. A test enforces that the scan reads no file whose stamp did not change (ADR 0038 item 13).
- The scan changes nothing else (ADR 0038 item 15).

### The Routes

- `POST /api/v1/library/scan`: run the scan, and give the differences, each with its feed ID and the feed title.
- `POST /api/v1/feeds/{id}/library-changes` with one action. Each action runs the scan again first, and gives 409 `no_change` when the difference is no longer there:
  - `{ "action": "add", "path": "..." }` for `New`: add a row with `local` true, its size, SHA-256, stamp, and the media type of its extension.
  - `{ "action": "forget", "file": id }` for `Missing`: when the file has a copy or a `source_url`, set `local` false. Otherwise remove the row.
  - `{ "action": "download", "file": id }` for `Missing`: set `local` false and queue the download of task 023c.
  - `{ "action": "use_new", "file": id }` for `Changed`. The route takes the copies of the file before the change, with `inventory::copies`. It records the new size, SHA-256 and stamp. It then changes each address of those copies in the draft to the new planned address of the file, with the new enclosure length. The draft starts from the hosted file when the feed has no draft. Each copy from before that the draft referred to gets a deletion in the draft operations (ADR 0035 item 16).
  - `{ "action": "restore", "file": id }` for `Changed`: when a copy with the recorded SHA-256 is on a connected host, set `local` false and queue the download. The download puts the recorded bytes back. Otherwise give 409 `no_copy`.
- No route writes to a host. The write of the draft does the uploads and the deletions, as in milestone 7a.

## Steps

1. Add the stamp function and the scan, with tests.
2. Add the routes, with an API test for each action.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.

## Acceptance Criteria

Mechanical:

- A new file, a missing file and a changed file in a release folder give one difference each. The feed XML, a hidden file and a partial file give none.
- A second scan with no change reads no file. A test enforces it.
- A file whose modification time changed with the same bytes gives no difference, and gets the new stamp.
- "add" gives a library file that the feed does not use.
- "forget" sets `local` false for a file with a copy, and removes the row of a file with no copy and no address.
- "use_new" changes the enclosure address in the draft to the hash name on the host, with the new length. The diff then has an upload of the hash name and a deletion of the file from before. After the write, the host has only the new file.
- "restore" puts the recorded bytes back from a copy on a local test host.
- The three commit checks pass.

Visual:

- The visual check of task 023 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when "use_new" needs a change of the draft that the existing draft functions cannot do.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023e-folder-scan.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- The files that the Files section names.

Goal:
- Studio compares each release folder with its records and gives each difference. The user decides what Studio does with each one.

Constraints:
- Follow the Architecture section of the task file fully.
- No route writes to a host. No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the schema, the connector interface.
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
