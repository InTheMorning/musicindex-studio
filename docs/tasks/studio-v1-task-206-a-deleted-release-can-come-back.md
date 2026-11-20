# Studio v1 Task 206: A Deleted Release Can Come Back

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 25 to 30.

Status: Open.

## Problem

Findings 7, 10, 11 and 12 of the [check after rc.29](../reviews/2026-11-20-rc29-human-check.md).

- A plain 404 of Music Index gives `Removed` (`crates/core/src/musicindex.rs` near line 820, `crates/core/src/feed_retire.rs:176`). The UI then says "Music Index removed the entry" (`crates/api/ui/feeddelete.js:22` and 58). The delete itself is correct.
- **Put the File Back** shows only in the Retire block, and the block reads the retire state only when the draft hides the feed (`crates/api/ui/editor.js` near lines 2528, 2544 and 2602).
- `host_for_address` is true for a deleted feed (`crates/api/src/routes.rs` near line 1470), so the notice **Use This File Host** shows (`crates/api/ui/outsidecheck.js:60-66`).
- After the delete, `record_remote_files` runs with a feed that has no path (`crates/api/src/routes.rs` near 3755 and 3792, `crates/api/src/workspace.rs` near 2045 and 2087). It adds the media files again as remote rows with " 2" in the name (`crates/core/src/remote.rs` near 121 and 277).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Delete sheets | extend | crates/api/ui/feeddelete.js | The texts at lines 22 and 58. Line 80 already has "Music Index does not hold this feed." |
| Retire block | extend | crates/api/ui/editor.js | `retiring` near line 2528 and `activeRetire` near line 2602 also read `feed.host_deleted`. |
| Feed row JSON | extend | crates/api/src/routes.rs | `host_for_address` near line 1470 is false when `row.host_deleted` is true. |
| Remote records | extend | crates/api/src/workspace.rs | `record_remote_files_with` near line 2045 returns when `feed_host_delete(id)` is Some, as for `folder.is_none()`. |
| Retire state | reuse | crates/core/src/feed_retire.rs | No change. The UI shows the restore action of a done delete. |
| API retire tests | extend | crates/api/src/tests/feed_retire.rs | The existing delete and retire tests. |
| Core delete tests | extend | crates/core/tests/feed_delete.rs | Line 265 asserts `No` for a deleted feed that is not hidden. Keep the core rule. The UI change reads `host_deleted`. |
| Delete UI tests | extend | crates/api/ui-tests/feeddelete.test.mjs | The texts of items 25 and 26. |
| Notice UI tests | extend | crates/api/ui-tests/outsidecheck.test.mjs | A case for a deleted feed near line 305. |

## Changes

1. Items 25 and 26: the third row of "What do you want to do?" and the state point of "Delete the Feed File" say "Music Index does not hold this feed.".
2. Item 27: a feed with `host_deleted` reads the retire state and shows the Retire block with **Put the File Back** and **Check Again**, also when its draft does not hide the feed. Do not show **Stop Retiring** for it (ADR 0198 item 6).
3. Item 28: `host_for_address` is false for a feed with a done delete.
4. Item 29: the write of a delete and each start of Studio make no remote records for a feed with a done delete.
5. Item 30: after **Put the File Back** and its write, the feed has its path, and the remote records run as for each feed. Find out if this is already true. Report it.

## Checks

Mechanical:

- Node test: the texts of items 25 and 26.
- API test: a feed with no index entry and no hidden draft is deleted with its files. Then `host_for_address` is false, the retire state gives the restore action, and the file list has no row with " 2".
- API test: **Put the File Back** and its write give the feed its path, and the file list has the original rows.
- `bash tools/gate.sh` passes.

Visual:

- drive-ui screenshot at 1280 px and 390 px of a deleted feed that was not hidden: the Retire block with **Put the File Back** shows, and the notice **Use This File Host** does not show.
- Human check: delete a test release that is not hidden, then **Put the File Back**.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
