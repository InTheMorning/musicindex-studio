# Studio v1 Task 027c: The Confirmation Before The Deletions

Parent: [task 027](studio-v1-task-027-connector-interface.md). Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) items 7 to 9, [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 14 to 16, [ADR 0021](../adr/0021-public-address-of-a-feed.md) and [ADR 0007](../adr/0007-draft-diff-write-and-announce.md).
Sequence: the third packet of task 027. It depends on 027a.

## Goal

Before the deletions, the write confirms each new file with `stat`, and then with its public address. It also requests the public address of the feed XML. When a file does not pass, Studio deletes nothing. The draft then starts from the written feed XML, with each deletion and a confirm operation for each new file. The next write, with a new approval, confirms the files again and then does the deletions.

## Files

Inspect: `crates/core/src/pipeline.rs` (`write_draft`, `confirm_new_addresses`, `host_operations`, `operation_bytes`, `OperationKind`, `HostOperation`), `crates/core/src/collection.rs` (`MIGRATIONS`, `DraftOperationKind`, `draft_operations`, `add_draft_operation`, `save_draft`), `crates/core/src/web.rs` (`media_facts`), `crates/core/tests/pipeline.rs` (`UnreachableAfterWrite` and its test), `crates/api/src/routes.rs` (the write route), `crates/api/ui/editor.js` (`operationKindLabel`, `OPERATION_NOTE`).

Change: `pipeline.rs`, `collection.rs`, `web.rs`, `routes.rs`, `editor.js`, `crates/core/tests/pipeline.rs`, `crates/api/src/tests.rs`.

Do not change: the connector trait, `LocalFolder`, the inventory scan, the vault.

## Architecture

These choices are fixed. Do not change them.

### The Address Check

- `pipeline.rs` gets this trait:

  ```rust
  /// Request a public address, and give an error when the host does
  /// not give a success status (ADR 0043 item 8).
  pub trait AddressCheck {
      fn check(&self, url: &str) -> Result<(), String>;
  }
  ```

- `web.rs` implements `AddressCheck` for `Web` with `Web::media_facts`. `Ok` from `media_facts` is a pass. An error gives its message.
- `write_draft` gets one more argument after `library`: `addresses: &dyn AddressCheck`. The write route gives `&w.web`. Each test gives a test double.

### The Confirmation

- `confirm_new_addresses` replaces its `read` with these two steps, for each operation of kind `Upload`, `Copy` or `Confirm`:
  1. `stat` through the connector of the operation. The step passes when `stat` gives a file with the size of the operation.
  2. When the operation has a public address, `addresses.check(url)`. An operation with no public address skips this step.
- Then the function requests the public address of the feed XML, when it has one. The address is `FeedRow::public_url` after the write. When that is null, it is `public_url` of the connector for the written path. When both are null, Studio skips the step.
- The function examines each file before it gives an error. Its error has the kind `NotReachable`, and its message names each file that did not pass.

### The State After A Failed Confirmation

- When `confirm_new_addresses` gives an error, `write_draft` does these steps, and then gives the error:
  1. It keeps the draft. The draft gets the written version as `base_version` and the written bytes as `base_xml`. The draft XML does not change.
  2. It removes each draft operation of kind `copy`, and the draft feed move. It keeps each operation of kind `delete`.
  3. It adds one draft operation of kind `confirm` for each `Upload`, `Copy` and `Confirm` operation of the write. The new operation keeps the host and the path. The feed XML needs no confirm operation, because the next write requests its address again.
  4. It scans each host that the write touched.
- After a write with no error, each confirm operation goes out with the other draft operations, as now.

### The Confirm Operation

- `OperationKind` gets `Confirm`, with the text `confirm`. `DraftOperationKind` gets `Confirm`, with the text `confirm`.
- A new migration at the end of `MIGRATIONS` makes the `kind` check of `draft_operations` accept `'confirm'`. SQLite cannot change a check. The migration thus makes a new table and copies the rows. Then it removes the old table and gives the new table its name. Each row and each ID stays.
- `host_operations` gives a `HostOperation` of kind `Confirm` for each confirm draft operation. Its size is the size of the host file in the inventory, or 0 when the inventory has no row. Its URL is `public_url` of its connector. A confirm operation does no write and no deletion.
- `operation_bytes` includes each confirm operation, so a confirm operation changes the diff ID.
- The UI shows a confirm operation with the label "Confirm" and the note "Studio makes sure that the host gives this file." It has the Remove button of the other draft operations.

## Steps

1. Add the migration and the `confirm` kinds. Add a test: a collection with the previous schema keeps its draft operations after the migration.
2. Add `AddressCheck` and its `Web` implementation, and the `write_draft` argument. Change each caller.
3. Change `confirm_new_addresses` to `stat` and the address check.
4. Add the state after a failed confirmation.
5. Add the UI label and the note.
6. Change the test of `UnreachableAfterWrite` and add the tests of the acceptance criteria.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- The UI change is in `editor.js` only, and the UI loads nothing from other servers ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- `confirm_new_addresses` calls no `read` of the connector.

## Acceptance Criteria

Mechanical:

- `stat` gives an upload with a different size. The write gives `NotReachable`, and the deletion of the same write does not occur.
- The public address of an upload fails the address check. The write gives `NotReachable`, and the deletion of the same write does not occur.
- A write whose feed XML address fails the address check gives `NotReachable`, and the deletion does not occur.
- After a `NotReachable` write, the base of the draft is the written version. The draft keeps the delete operation, and it has one confirm operation for the upload.
- After a `NotReachable` write, a second write uses the ID of the new diff and an address check that passes. It does the deletion, and no draft remains.
- A second write with the diff ID from before the failed write gives `DiffNotApproved`.
- A write in which each address passes behaves as before: each current pipeline test passes.
- `confirm_new_addresses` calls no `read`: a test connector that panics in `read` for the uploaded path passes the confirmation.
- The migration keeps each draft operation and its ID.
- The four commit checks pass.

Visual:

- The Review sheet shows a confirm operation with the label "Confirm" and its note, in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report in these cases:

- `save_draft` or a different collection function cannot change the base of a draft and keep its draft XML.
- The feed move needs a different state after a failed confirmation.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-027c-confirmation.md
- docs/adr/0043-phase-9-connectors-and-the-interface-change.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- Before the deletions, the write confirms each new file with stat and then with its public address. It also requests the public address of the feed XML. When a file does not pass, Studio deletes nothing. The draft keeps each deletion and gets a confirm operation for each new file.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connector trait, LocalFolder, the inventory scan, the vault.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
