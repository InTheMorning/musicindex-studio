# Studio v1 Task 023b: Published Feeds In The Library

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 3 to 5, [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) item 5.
Sequence: after [task 023a](studio-v1-task-023a-remote-file-records.md).

## Goal

A published feed gets its release folder, its feed XML and its remote records when the user adds it. Studio keeps the remote records current after each read and each write. The files route and the Files page show each remote file.

## Files

Inspect: `crates/api/src/routes.rs` (`add_read_feed`, `import_feeds`, `import_msp`, `match_connector`, `refresh_feed`, `write_feed`, `give_release_folder`, `list_release_files`), `crates/api/src/workspace.rs` (`library`), `crates/core/src/remote.rs`, `crates/core/src/migrate.rs` (`move_read_only_feed`, `file_name`), `crates/core/src/pipeline.rs` (`refresh`, `refresh_public`, `write_draft`), `crates/core/src/library.rs`, `crates/api/ui/editor.js` (`FileEntry`, `FilesPage`), `crates/api/src/tests.rs`.

Change: `routes.rs`, `workspace.rs`, `pipeline.rs`, `migrate.rs`, `library.rs`, `editor.js`, `styles.css`, `tests.rs`, and the tests of the core modules that change.

Do not change: the schema, the connector interface, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### The Release Folder Of A Published Feed

- New core function `library::give_published_folder(collection, library, feed_id, bytes) -> Result<String, ...>`. It selects a free release folder name from the feed address with `release_folder_name`, as `migrate::move_read_only_feed` does. It writes `bytes` as the feed XML, and records the folder. `migrate::move_read_only_feed` uses it too, so the two places do not have two copies of the same steps.
- `add_read_feed` calls it with the bytes that Studio read. The read gives the bytes: when `PublicFeed` has no bytes, add them to it. When the collection has no library folder yet, `add_read_feed` adds the feed with no folder, as before. The move into the library of milestone 7a gives it a folder later.
- `match_connector` links a read-only feed to a host. A feed that has a folder keeps it.

### When Studio Records The Remote Files

Call `remote::record_remote_files` with the hosted feed (not the draft) at each of these points (ADR 0038 item 3):

- After `add_read_feed` gives a folder.
- After `pipeline::refresh` and `pipeline::refresh_public`, in the route that calls them.
- After `pipeline::write_draft`, in the write route, with the feed that the write wrote.
- At the start: `Workspace::open` does not have the library. Add `Workspace::record_all_remote_files()`. Call it from the library route after `migrate::into_library`. When a library is set, also call it from `studio-desktop` after the workspace opens. It runs for each feed with a folder, with the feed from the release folder XML.

An error of `record_remote_files` for one feed goes into the log with `eprintln!`, and does not stop the route. The route gives its normal result.

### The Files Route

- `GET /feeds/{id}/files`: each file gets `local: bool` (it has it), `source_url`, and `length_in_feed: number | null`, the enclosure length in the draft for its address. A remote file has `copies` from `inventory::copies` (023a) and `in_use`.
- The summary of the files gets `remote_count` and `remote_bytes`: the number and the total size of the files with `local` false.

### The Files Page

- A remote file shows the badge, the name and "Not in the library" in the summary line, instead of the number of copies.
- Its open entry shows each copy as before, and the line "Not in the library. Studio uses the file on its host." It has no "Copy to" button in this packet. Packet 023d adds the download and the copy of a remote file.
- No other change of the page.

## Steps

1. Add `library::give_published_folder`, and use it in `migrate` and in `add_read_feed`.
2. Call `record_remote_files` at each point.
3. Add the fields of the files route, and the Files page change.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No route writes to a host.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Run `drive-ui.mjs` with `bun` when `node` is not on the PATH.

## Acceptance Criteria

Mechanical:

- An API test adds a read-only feed from a test HTTP server, with a library set. The feed gets a release folder with the feed XML bytes, and one remote row for each address.
- After a refresh that removes an enclosure from the hosted feed, its remote row goes out.
- After a write, the remote rows agree with the written feed.
- `GET /feeds/{id}/files` gives `local` false, `source_url`, and `remote_count` and `remote_bytes` for a feed with remote files.
- A `drive-ui.mjs` check of the Files page shows "Not in the library" for a remote file. `$small()` gives an empty list, and the console shows no error.
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

Stop and report when the read of a published feed does not keep its bytes, and a change of `discover` is necessary.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023b-published-feeds-in-the-library.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- The files that the Files section names.

Goal:
- A published feed gets its release folder, its feed XML and its remote records when the user adds it. Studio keeps the remote records current after each read and each write. The files route and the Files page show each remote file.

Constraints:
- Follow the Architecture section of the task file fully.
- No route writes to a host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The schema, the connector interface, draft.js.
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
