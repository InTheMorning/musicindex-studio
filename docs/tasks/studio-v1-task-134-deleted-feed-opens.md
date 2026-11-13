# Studio v1 Task 134: A Deleted Feed Opens From Its Last Copy

Decision: [ADR 0131](../adr/0131-delete-a-feed-from-its-file-host.md) item 14. This task corrects a defect. It makes no new rule.

Status: Complete. The mechanical criteria pass. The human check passed on 2026-11-13 with a local build that has commit `51dc619`. "HeyCitizen's Test Feed" opened as read-only with its tracks and no error, and the Retire block showed the three steps as done.

## Goal

After a delete on a host with ETag versions, such as S3 or R2, the feed opens as read-only with its last copy. A restore also works. Finding 6 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md) gives the fault.

The probable cause:

- The delete write calls `Collection::record_feed_host_delete`. It sets `connector_id` and `path` to NULL. It does not change `last_version`, and it writes no copy to the release folder (`crates/core/src/pipeline.rs`, the `deleting_feed` branches of the write).
- Before the delete, the write keeps a snapshot of the live file with its host version (`collection.add_snapshot(id, &file.version, &file.bytes)`).
- `pipeline::start_public_draft` compares `digest::content_version` of the release folder copy, a `sha256:` value, with `last_version`. `feed_delete::start_restore_draft` makes the same comparison.
- On an S3 or R2 host, `last_version` is the ETag of the host. So the two values cannot agree.
- The local folder host gives `sha256:` versions. Thus the test `after_the_delete_the_feed_is_read_only_and_a_write_to_the_same_address_brings_the_file_back` in `crates/core/tests/feed_delete.rs` passes.

## The Correction

For a feed with a record in `feed_host_deletes`, the last copy is the snapshot that the delete write kept. It is the newest snapshot of the feed with the version `last_version`.

1. `start_public_draft` uses the bytes of that snapshot for a feed with a delete record. It does not read the release folder for that feed.
2. `start_restore_draft` uses the same bytes.
3. A feed with a delete record and no such snapshot gives `PipelineErrorKind::NotFound` with a message that names the missing last copy.
4. A feed with no delete record keeps the current behavior.

This correction also repairs a feed that an earlier build deleted, because that write kept the snapshot.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Snapshot store | reuse | crates/core/src/collection.rs | `snapshots` and `snapshot_xml` give the bytes of a kept snapshot. |
| Delete record | reuse | crates/core/src/collection.rs | `feed_host_delete` tells if a write deleted the feed file. |
| Public draft start | extend | crates/core/src/pipeline.rs | `start_public_draft` gets the snapshot source for a deleted feed. |
| Restore draft start | extend | crates/core/src/feed_delete.rs | `start_restore_draft` gets the same source. |
| Test host wrapper | extend | crates/core/tests/placeholder_guid.rs | `ReadCounter` shows the form of a wrapper around `LocalFolder`. The new wrapper gives ETag versions. |

## Rules For The Delegate

- Read ADR 0131 items 12 to 14 first. Then read the functions in the table above.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Write the test first. Run it, and record that it fails before the correction.

## Acceptance Criteria

Mechanical, each proved by a test in `crates/core/tests/feed_delete.rs`:

1. A test host wraps `LocalFolder` and gives versions in the ETag form, for example `"<hex>"`, not `sha256:<hex>`. It maps each `Expected` and `Stamp` back to the inner host. After a delete write on that host, `pipeline::start_public_draft` gives the bytes of the feed file before the delete. This test fails before the correction.
2. On the same host, `feed_delete::start_restore_draft` gives the same bytes, and the write of that draft puts the file back at its path.
3. A feed with a delete record and no snapshot with the version `last_version` gives `NotFound` from `start_public_draft`.
4. The existing tests of `crates/core/tests/feed_delete.rs` and `crates/core/tests/pipeline.rs` pass with no change to their assertions.

The commit checks: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test` for the UI tests.

Visual, by the operator, with a build that has this task:

- Open "HeyCitizen's Test Feed". The head shows **Read-only**, the page shows the album with its tracks, and no error shows. The Retire block shows step 3 as done.

## Stop And Ask When

- The delete write kept no snapshot in a path that the code allows.
- A reader other than the two functions above compares a release folder copy with `last_version` for a deleted feed.
- The correction needs a schema change.

## Report

- The changed functions, with file and line.
- The test names, and the output of the new test before the correction.
- The output of the commit checks.
