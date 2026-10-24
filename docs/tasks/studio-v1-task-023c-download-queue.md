# Studio v1 Task 023c: The Download Queue

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 6 to 12, [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) item 17.
Sequence: after [task 023b](studio-v1-task-023b-published-feeds-in-the-library.md).

## Goal

Studio downloads remote library files into the library, two at a time, in the background. A download continues after a stop or a restart. A copy or a move of a remote file waits for its download, and the write refuses the draft until then.

## Files

Inspect: `crates/core/src/web.rs`, `crates/core/src/library.rs`, `crates/core/src/inventory.rs` (`copies`), `crates/core/src/collection.rs` (migration 6, `library_files`), `crates/core/src/pipeline.rs` (`host_operations`, `write_draft`, `PipelineErrorKind`), `crates/api/src/workspace.rs`, `crates/api/src/routes.rs` (`add_operation`, `reference_operation`, `write_feed`, `download_limit`), `crates/api/src/lib.rs`, `crates/api/src/error.rs`, `crates/desktop/src/main.rs`.

Change in the core: `web.rs`, `collection.rs`, `pipeline.rs`, `lib.rs`, `Cargo.toml`, and a new module `crates/core/src/download.rs`.

Change in the API and the desktop shell: `workspace.rs`, `routes.rs`, `lib.rs`, `error.rs`, `tests.rs`, a new file `crates/api/src/downloads.rs`, and `crates/desktop/src/main.rs`.

Do not change: the UI, the schema, the connector interface.

## Architecture

These choices are fixed. Do not change them.

### The Core

- `Collection` gets the functions of the `downloads` table: `queue_download(file)`, `fail_download(file, message)`, `remove_download(file)`, `downloads() -> Vec<DownloadRow>`. It also gets `set_library_file_local(file, size, sha256, stamp)`.
- `Web::get_to_file(url, part: &Path, max_bytes, progress: &mut dyn FnMut(u64, Option<u64>), stop: &AtomicBool) -> Result<u64, WebError>`. When `part` has bytes, it sends `Range: bytes={len}-`. A 206 response adds to the file. A 200 response starts the file again from 0.
  The function reads the body in blocks of 64 KiB. After each block, it calls `progress(done, total)` and examines `stop`. More than `max_bytes` gives an error. The function gives the size of the complete file.
- Module `download`:
  - `enum Source { Host { connector_id, path }, Web { url } }`. `source_of(collection, hosts, file)` gives the first copy on a connected host, from `inventory::copies`. When there is none, it gives the `source_url`. A file with no copy and no address gives `None` (ADR 0038 item 8).
  - `part_path(library, folder, name)`: `.{name}.part` in the release folder.
  - `check_limits(collection, library, size)`: refuses a size more than `download_max_file_bytes`. It also refuses a download that leaves less than `download_min_free_bytes` of free space on the disk of the library. The error kinds are `TooLarge` and `NoSpace`, and each message names its setting (ADR 0038 item 10). Use the crate `fs4` for the free space (`fs4::available_space`). It is the only new crate.
  - `fetch(hosts, web, source, part, max_bytes, progress, stop)`: a host source reads the file through its connector and writes `part`. A web source calls `Web::get_to_file`.
  - `finish(collection, library, file, part)`: renames `part` to the file name in the release folder, reads its size, SHA-256 and stamp, calls `set_library_file_local`, and removes the `downloads` row (ADR 0038 item 11).
- `pipeline::HostOperation` gets `waiting: bool`. A copy of a library file with `local` false has `waiting` true, and is in the diff. `write_draft` refuses a draft with a `waiting` operation, with the new kind `PipelineErrorKind::WaitingForDownload`, before any file changes (ADR 0038 item 12).

### The API

- `crates/api/src/downloads.rs` keeps the queue: `DownloadQueue`, in an `Arc` in `Workspace`. A route locks the workspace, clones the `Arc`, and unlocks the workspace again.
- The queue has two worker threads (ADR 0038 item 7). The first queued download starts them. A worker locks the workspace only to read the source and the limits, and to call `finish`. It does not lock the workspace during a transfer.
- The queue keeps the progress of each file in memory: `feed_id`, `state` (`queued`, `running`, `failed`), `bytes`, `total`, `message`. Each file has a stop flag.
- Routes:
  - `GET /api/v1/feeds/{id}/downloads`: each download of the feed, with its progress.
  - `POST /api/v1/feeds/{id}/downloads` with `{ "files": [id, ...] }` or `{ "all": true }`: queue each remote file. A file with `local` true, or a file that is in the queue already, does nothing. A limit error gives 422 `download_limit` with the message. The collection records each queued file (`queue_download`) first.
  - `DELETE /api/v1/feeds/{id}/downloads/{file}`: stop the download, and remove its `downloads` row. The partial file stays, so a later download continues from it.
- A failed download gets `state` `failed` with the message, in memory and with `fail_download`.
- `POST /feeds/{id}/operations` with `copy` or `move` of a file with `local` false: the route adds the operation as before and queues the download of the file.
- The write route gives 409 `waiting_for_download` for `PipelineErrorKind::WaitingForDownload`.
- At the start, `studio-desktop` queues each `downloads` row with `state` `queued` again (ADR 0038 item 9).

## Steps

1. Add the collection functions and `Web::get_to_file`, with a test against a local HTTP test server. The test serves 206 for a range request, and continues a partial file.
2. Add the module `download`, with tests of `source_of`, `check_limits` and `finish`.
3. Add `waiting` and the write refusal, with a test.
4. Add the queue and the routes, with API tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No route writes to a host. A download writes only into the library.
- A worker does not keep the workspace lock during a transfer.
- The new crate is `fs4` only.

## Acceptance Criteria

Mechanical:

- A download of a 1 MB file from a test HTTP server that stops after 300 KB continues from 300 KB with a range request, and gives the correct SHA-256.
- A file on a connected local host downloads through the connector, with no HTTP request.
- A file larger than `download_max_file_bytes` gives 422 `download_limit`, and the message names the setting.
- After `finish`, the row has `local` true, its size, its SHA-256 and its stamp, and no `downloads` row.
- A copy of a remote file is in the diff with `waiting` true. The write gives 409 `waiting_for_download`, and no file changes on a host. After the download, the write puts the copy.
- A `downloads` row from before a restart is in the queue again after `studio-desktop` starts.
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

Stop and report when the worker threads need a change of the router state, or when `fs4` does not build.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023c-download-queue.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- The files that the Files section names.

Goal:
- Studio downloads remote library files into the library, two at a time, in the background. A download continues after a stop or a restart. A copy or a move of a remote file waits for its download, and the write refuses the draft until then.

Constraints:
- Follow the Architecture section of the task file fully.
- No route writes to a host. A worker does not keep the workspace lock during a transfer. The new crate is fs4 only.
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
