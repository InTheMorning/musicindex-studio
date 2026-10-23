# Studio v1 Task 018a: The Pending Files And The Write

Parent: [task 018](studio-v1-task-018-import-music-files.md). Decisions: [ADR 0026](../adr/0026-import-music-files-through-the-draft.md) items 5 to 9, and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) items 4, 10 to 12.
Sequence: after [task 017](studio-v1-task-017-new-album-feed.md).

## Goal

Keep the imported files of a draft in the data folder. The diff lists them, and the approved write puts them on the host before the feed file.

## Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `DraftRow`, `delete_draft`, `delete_feed`), `crates/core/src/pipeline.rs` (`diff`, `diff_id`, `write_draft`, `draft_from_snapshot`, `start_draft`), `crates/core/src/connector.rs` (`write`, `Expected::Absent`, `public_url`), `crates/core/src/lib.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs` (`get_diff`, `write_feed`, `delete_draft`, `draft_from_snapshot`), `crates/api/src/error.rs`.

Change: `collection.rs`, `pipeline.rs`, a new `crates/core/src/imports.rs`, `lib.rs`, `workspace.rs`, `routes.rs`, `error.rs`, and the tests.

Do not change: the connector interface, the feed model, the UI.

## Architecture

These choices are fixed. Do not change them.

- Migration 3 adds a table. `SCHEMA_VERSION` becomes 4.

  ```sql
  CREATE TABLE draft_files (
      feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
      path TEXT NOT NULL,
      size INTEGER NOT NULL,
      media_type TEXT NOT NULL,
      sha256 TEXT NOT NULL,
      added_at INTEGER NOT NULL,
      PRIMARY KEY (feed_id, path)
  );
  ```

- `imports::Store { root: PathBuf }` keeps the bytes of each pending file at `root/<feed id>/<sha256>`. Methods: `put_file(feed, source: &Path) -> Result<(String, u64), io::Error>` (it moves or copies the file, and gives the SHA-256 and the size), `open(feed, sha256) -> io::Result<Vec<u8>>`, `remove(feed, sha256)`, `remove_feed(feed)`.
- `Workspace` keeps the data folder, and its `store` has the root `<data folder>/imports`.
- `pub struct DraftFile { pub path: String, pub size: u64, pub media_type: String, pub sha256: String }`. Collection methods: `add_draft_file(id, &DraftFile)`, `draft_files(id) -> Vec<DraftFile>`, `remove_draft_file(id, path)`, `remove_draft_files(id)`. A path that the draft has gives `CollectionErrorKind::InUse`.
- `pipeline::release_path(feed_path: &str, file_name: &str) -> String` gives the path in the release folder. `album/my-album.xml` and `01 A.mp3` give `album/my-album/01 A.mp3`.
- `pipeline::referenced_files(collection, connector, id, feed: &Feed) -> Vec<DraftFile>` gives each pending file that the draft uses. The draft uses a file when its `connector.public_url(path)` is the `url` of an enclosure. It also uses a file that is the `href` of `itunes:image` of the channel or of an item.
- `Diff` gets `files: Vec<DiffFile>`, with `DiffFile { path, size, media_type }`, from `referenced_files`. `diff_id` adds the path, the size and the SHA-256 of each referenced file, in sequence.
- `write_draft` gets `store: &Store`. After the checks, and before the snapshot:
  1. For each referenced file: read its bytes from the store, and call `connector.write(path, bytes, &Expected::Absent)`. A `Conflict` gives a new error kind `FileExists`, with the path in the message. The feed file then does not change.
  2. After each file is on the host, remove its row and its copy.
  3. After the feed file write, remove the rows and the copies that remain.
- A discard of the draft, a version put in the draft, and a deleted feed remove the rows and the copies of the feed. `pipeline::discard_draft(collection, store, id)` does it for the routes.
- `GET /api/v1/feeds/{id}/diff` gives `files` also. `FileExists` maps to 409 `file_exists`.

## Steps

1. Add the migration, `DraftFile` and the collection methods.
2. Add `imports.rs` and the `Workspace` fields.
3. Add `release_path`, `referenced_files`, the diff files and the write.
4. Change the routes that discard a draft, put a version in a draft, and delete a feed.
5. Add the tests of the acceptance criteria.

## Constraints

- No file goes to a host without a diff that the user approved. A changed file changes the diff ID.
- An import does not replace a file on the host.
- A database at schema version 3 opens and gets the table.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A write with two referenced files puts the two files, then the feed file. The test connector records the sequence of the writes.
- A file at the path of a pending file stops the write with `FileExists`, and the feed file does not change. After the user removes that track, a new write puts the remaining file and the feed file.
- A pending file that the draft does not refer to does not go to the host, and the write removes its copy.
- A discard of the draft removes the rows and the copies.
- The diff ID changes when a referenced file changes.
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

Stop and report when the write needs a change of the connector interface. Stop and report when a route that this packet does not name changes a draft.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-018a-pending-files.md
- docs/adr/0026-import-music-files-through-the-draft.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- The files that the Files section names.

Goal:
- The data folder keeps the pending files of a draft. The diff lists them, and the approved write puts them on the host first.

Constraints:
- Follow the Architecture section of the task file fully: the SQL, the types and the sequence of the write.
- No file goes to a host without an approved diff. No file on the host is replaced.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connector interface, the feed model, the UI.
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
