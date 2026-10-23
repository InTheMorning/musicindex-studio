# Studio v1 Task 021f: The API Of The Library

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 1, 6, 8, 11 to 13 and 17.
Sequence: after [task 021e](studio-v1-task-021e-move-into-the-library.md).

## Goal

Give the UI the library setting, the files of a release with their copies, the hosts of a release, and the user operations on files.

## Files

Inspect: `crates/api/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/workspace.rs`, `crates/api/src/error.rs`, `crates/api/src/tests.rs`, `crates/core/src/library.rs`, `crates/core/src/inventory.rs`, `crates/core/src/migrate.rs`, `crates/core/src/pipeline.rs`.

Change: `lib.rs`, `routes.rs`, `workspace.rs`, `error.rs`, `tests.rs`. Change `crates/core` only to add a query that a route needs.

Do not change: the UI.

## Architecture

These choices are fixed. Do not change them.

- `GET /api/v1/library` gives `{ root: null | string, default_root, download_max_file_bytes, download_min_free_bytes }`.
- `PUT /api/v1/library` with `{ root }`: `Library::open`, then `migrate::into_library` when the setting was absent, then the setting. It gives the `MoveReport`. A path that Studio cannot write gives 422 `library_not_writable`, with the path. A change of an existing root gives 409 `library_set`, because a move of the library is not in milestone 7a.
- `PUT /api/v1/library/limits` with the two download settings, in bytes. Each value must be a positive integer.
- `GET /api/v1/feeds/{id}/files` gives the release folder, its default host, and for each library file: its ID, path, size, media type, `local`, its host, its planned URL, `in_use` by the draft, and its copies with host, path and URL. It also gives each host file in the release folder on a host that is not a copy of a library file.
- `PUT /api/v1/feeds/{id}/hosts` with `{ files_connector_id }` sets the default host of the release files.
- `PUT /api/v1/feeds/{id}/files/{file}/host` with `{ connector_id: null | id }` sets the host of one file.
- `POST /api/v1/feeds/{id}/operations` with one of these bodies. Each one changes the draft or adds a draft operation, and gives the new diff summary:
  - `{ "op": "reference", "file": id, "connector_id": id, "from_url": url }`: in the draft, replace each address `from_url` with the URL of the copy of the file on that host, or with its planned URL there.
  - `{ "op": "dereference", "url": url }`: remove the enclosure, the `podcast:source` or the image with that address from the draft. An enclosure with a `default="true"` alternative enclosure takes the address of that alternative enclosure.
  - `{ "op": "copy", "file": id, "connector_id": id }`.
  - `{ "op": "move", "file": id, "from": connector_id, "to": connector_id }`: a copy, a reference, and a delete of the copy on `from`.
  - `{ "op": "delete", "connector_id": id, "path": path }`.
- `DELETE /api/v1/feeds/{id}/operations/{op}` removes one draft operation.
- `POST /api/v1/hosts/{id}/scan` runs `inventory::scan` and gives the `ScanResult`.
- The diff route gives `operations` and `blocked` from task 021c.

## Steps

1. Add the library routes.
2. Add the files route, the host routes and the operation routes.
3. Add the tests.

## Constraints

- Each route needs the access token (ADR 0017).
- No route writes to a host. The write does it.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- `PUT /library` with a test folder moves a collection with one feed, and `GET /feeds/{id}/files` then gives its files.
- `PUT /library` with a path that Studio cannot write gives 422 `library_not_writable`.
- A `copy` operation to a second host gives an upload in the diff. A `move` gives a copy, an address change in the draft and a delete. A `delete` of a file in use gives a `blocked` entry.
- `dereference` of an enclosure with a default alternative enclosure gives the enclosure the address of the alternative enclosure.
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

Stop and report when an operation needs a change of the write sequence of task 021d.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021f-library-api.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- Give the UI the library setting, the files of a release with their copies, the hosts of a release, and the user operations on files.

Constraints:
- Follow the Architecture section of the task file fully.
- Each route needs the token. No route writes to a host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI.
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
