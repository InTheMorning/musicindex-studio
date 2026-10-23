# Studio v1 Task 021a: The Records And The Library Folder

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 1 to 5 and 16.
Sequence: the first packet of task 021.

## Goal

Add the records of the library, the library folder module, and the deletion of a host file.

## Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `SCHEMA_VERSION`, `DraftFile`, the feed row), `crates/core/src/connector.rs`, `crates/core/src/connector/local.rs`, `crates/core/src/digest.rs`, `crates/core/src/imports.rs` (as a model of a file store), `crates/core/src/lib.rs`, the collection and connector tests.

Change: `collection.rs`, `connector.rs`, `connector/local.rs`, a new `crates/core/src/library.rs`, `lib.rs`, and the tests.

Do not change: `pipeline.rs`, `crates/api`, the UI. Do not remove `draft_files` or `imports.rs` in this packet.

## Architecture

These choices are fixed. Do not change them.

- A new migration. `SCHEMA_VERSION` goes up by one.

  ```sql
  CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
  ALTER TABLE feeds ADD COLUMN folder TEXT;
  ALTER TABLE feeds ADD COLUMN files_connector_id INTEGER REFERENCES connectors(id);
  CREATE UNIQUE INDEX feeds_folder ON feeds(folder) WHERE folder IS NOT NULL;
  CREATE TABLE library_files (
      id INTEGER PRIMARY KEY,
      feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
      path TEXT NOT NULL,
      size INTEGER NOT NULL,
      sha256 TEXT NOT NULL,
      media_type TEXT NOT NULL,
      local INTEGER NOT NULL,
      source_url TEXT,
      connector_id INTEGER REFERENCES connectors(id),
      added_at INTEGER NOT NULL,
      UNIQUE (feed_id, path)
  );
  CREATE TABLE host_files (
      connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
      path TEXT NOT NULL,
      size INTEGER NOT NULL,
      version TEXT NOT NULL,
      sha256 TEXT,
      seen_at INTEGER NOT NULL,
      PRIMARY KEY (connector_id, path)
  );
  CREATE TABLE draft_operations (
      id INTEGER PRIMARY KEY,
      feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
      kind TEXT NOT NULL CHECK (kind IN ('copy', 'delete')),
      library_file_id INTEGER REFERENCES library_files(id) ON DELETE CASCADE,
      connector_id INTEGER NOT NULL REFERENCES connectors(id),
      path TEXT NOT NULL,
      added_at INTEGER NOT NULL
  );
  ```

  `library_files.connector_id` is the host that the user selected for the file. `NULL` means the default host of the release.
- Collection methods, each with a type:
  - Settings: `setting(key) -> Option<serde_json::Value>` and `set_setting(key, &Value)`.
  - Feeds: `set_feed_folder(id, &str)` and `set_files_connector(id, Option<ConnectorId>)`. The feed row gets `folder` and `files_connector_id`.
  - Library files: `add_library_file`, `library_files(feed)`, `library_file(id)`, `set_library_file_connector` and `remove_library_file`.
  - Host files: `put_host_file`, `host_files(connector)`, `remove_host_file` and `set_host_file_sha`.
  - Draft operations: `add_draft_operation`, `draft_operations(feed)` and `remove_draft_operations(feed)`.
- `library::Library { root: PathBuf }`:
  - `Library::open(root)` makes the folder when it is absent, writes and removes the file `.studio-write-test`, and gives an error that names the path when it cannot write.
  - `release_folder_name(feed_path: &str, taken: &dyn Fn(&str) -> bool) -> String` gives the feed file name with no extension, then ` 2`, ` 3` and so on.
  - `put_file(folder, name, source: &Path) -> io::Result<(u64, String)>` moves or copies `source` into `<root>/<folder>/<name>`, and gives the size and the SHA-256. It refuses a name with `/`, `\`, `.` or `..`.
  - `write_feed_xml(folder, file_name, bytes)` writes the feed XML through a temporary file and a rename.
  - `path(folder, name) -> PathBuf`.
- `Connector::delete(&self, path: &str, expected: &Version) -> Result<(), ConnectorError>`. It gives `Conflict` when the file on the host has a different version, and `NotFound` when the file is absent. `LocalFolder` implements it.
- The default library path is `~/musicindex/studio`. `library::default_root()` gives it from the `HOME` variable.

## Steps

1. Add the migration and the collection methods.
2. Add `library.rs`.
3. Add `Connector::delete` and its local implementation.
4. Add the tests.

## Constraints

- A database at the previous schema version opens and gets the new tables.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A database at the previous schema version opens, and each new table and column exists.
- `release_folder_name("album/my-album.xml", taken)` gives `my-album`, and `my-album 2` when `my-album` is taken.
- `Library::open` of a path in a folder with no write permission gives an error that names the path.
- `put_file` gives the size and the SHA-256, and refuses `../x`.
- `LocalFolder::delete` removes a file with the correct version, gives `Conflict` for a different version, and `NotFound` for an absent file.
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

Stop and report when a migration needs a change of an existing table that this packet does not name.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021a-records-and-library-folder.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- Add the records of the library, the library folder module, and the deletion of a host file.

Constraints:
- Follow the Architecture section of the task file fully: the SQL, the methods and the types.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- pipeline.rs, crates/api, the UI. Do not remove draft_files or imports.rs.
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
