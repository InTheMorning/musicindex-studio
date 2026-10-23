# Studio v1 Task 021d: The Write With Host Operations

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 3, 13, 15 and 16, and [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) items 14 to 17.
Sequence: after [task 021c](studio-v1-task-021c-library-import-and-diff.md).

## Goal

The approved write puts the uploads and the copies on their hosts, then the feed XML, then does the deletions. It updates the inventory and the library XML.

## Files

Inspect: `crates/core/src/pipeline.rs` (`write_draft` and the work of task 021c), `crates/core/src/inventory.rs`, `crates/core/src/library.rs`, `crates/core/src/connector.rs`, `crates/core/tests/pipeline.rs`, `crates/api/src/routes.rs` (`write_feed`, `get_diff`, `upload_file`, `delete_pending_file`), `crates/api/src/workspace.rs`.

Change: `pipeline.rs`, the pipeline tests, `routes.rs` and `workspace.rs` only as much as the new signature needs, and the API tests that the new signature needs.

Do not change: the UI, `media.rs`, the connector interface.

## Architecture

These choices are fixed. Do not change them.

- `write_draft(collection, hosts: &Hosts, library: &Library, id, approved: &DiffId) -> Result<Version, PipelineError>`. The checks of the current `write_draft` stay: the diff ID, the addresses and the version of the feed file.
- The sequence of the write:
  1. Each upload and copy: read the library file, and `write(path, bytes, &Expected::Absent)`. A `Conflict` gives `FileExists`, and the feed file does not change. After each file, put it into `host_files` with the size and the stamp from `stat`, and its SHA-256.
  2. The feed XML, as before. Then put the hosted bytes into the release folder with `library.write_feed_xml`.
  3. Read each new address of the feed through its connector. When one is not there, stop before the deletions with `NotReachable`, and name the address.
  4. Each deletion: check again that no feed refers to the address. Then `delete(path, stamp)` with the stamp from `host_files`, and remove the row. A `Conflict` stops the deletions and names the path. The feed XML stays written.
  5. Remove the `draft_operations` of the feed, and the draft.
- After the write, `inventory::scan` runs for each host that the write touched.
- `write_draft` no longer uses `draft_files` or the store. The import route of the API calls `import_into_library`. The API keeps its result fields, with `on_host` from `PlannedPath`.
- The API makes `Hosts` from each connector row, and keeps a `Library` when the setting `library_root` exists. When it does not exist, the import route and the write give 409 `no_library`.

## Steps

1. Change `write_draft`.
2. Change the import route, the diff route and the write route.
3. Add the tests.

## Constraints

- No file goes to a host without an approved diff.
- Studio does not write over a file on a host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A release with one file on host A and one file on host B writes the two files and a feed with the correct address for each. The library XML has the same bytes as the hosted feed file.
- A changed library file gets a hash name on the host. The previous file stays until the feed is written. With a delete operation for it, the write deletes it after the feed.
- A delete operation for an address that a different feed of the library refers to does not delete the file.
- A host file that changed after the scan has a different stamp. It stops the deletion with `Conflict`, and the feed stays written.
- An address that is not on its host after the uploads stops the write before the deletions.
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

Stop and report when the API needs a change of a route that this packet does not name.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021d-write-with-host-operations.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- The approved write puts the uploads and copies on their hosts, then the feed XML, then does the deletions.

Constraints:
- Follow the Architecture section of the task file fully: the sequence of the write.
- No file goes to a host without an approved diff. Studio does not write over a host file.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, media.rs, the connector interface.
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
