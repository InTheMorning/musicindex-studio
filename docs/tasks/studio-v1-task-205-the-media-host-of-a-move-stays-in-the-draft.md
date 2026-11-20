# Studio v1 Task 205: The Media Host Of A Move Stays In The Draft

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 20 to 24.

Status: Open.

## Problem

Findings 6, 8 and 9 of the [check after rc.29](../reviews/2026-11-20-rc29-human-check.md).

- `move_release` sets `files_connector_id` at **Add to the Draft** (`crates/api/src/routes.rs` near line 5661). A discard does not put back the earlier value (`delete_draft` near routes.rs:3453, `pipeline::discard_draft` near `crates/core/src/pipeline.rs:6434`).
- The plan counts only the old copies of `plan.files` (routes.rs near 5474), not the copies of a file that is already on the target. The sheet then shows "Old files: None" and no check box, but the route still keeps holds (routes.rs near 5568).
- The first sheet of **Delete from File Host…** shows the error of the last try after a discard (`useAction` in `crates/api/ui/app.js:636`, the sheet near `crates/api/ui/editor.js:2514` and 2624).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Move route | extend | crates/api/src/routes.rs | `move_release` near line 5645 and `release_move_plan` near line 5447. |
| Collection feeds table | extend | crates/core/src/collection.rs | `set_files_connector` near line 2996, `delete_draft` near line 3089. Add one nullable column for the kept value, with a schema migration as the other migrations of `collection.rs`. |
| Draft discard | extend | crates/core/src/pipeline.rs | `discard_draft` near line 6434 puts back the kept value. The write near line 4936 clears it. |
| Move plan JSON | extend | crates/api/src/routes.rs | `from_connector_ids` near line 5474 also counts `target_copies`. |
| Action hook | reuse | crates/api/ui/app.js | `useAction` near line 636. Use a separate action for the first delete sheet, or clear its error when the sheet opens. |
| API tests | extend | crates/api/src/tests.rs | The move tests near lines 15797, 15918, 15937 and 15955. |

## Changes

1. Before `move_release` changes `files_connector_id`, keep the earlier value in a new nullable column of the feed (item 20). Keep it only when no kept value exists, so that two moves in one draft keep the first value.
2. A discard of the draft puts back the kept value and clears it. A write clears it (item 21).
3. The move plan gives the old copies of each media file that the move uses on the target, also of a file with "Already on {host}" (item 22). The sheet then shows "Old files" and the check box.
4. When the plan has no old copy, the route keeps no hold (item 23).
5. The first sheet of **Delete from File Host…** clears the error of the last try when it opens (item 24).

## Checks

Mechanical:

- API test: a move with **Add to the Draft**, then a discard, gives the earlier `files_connector_id`. A move, then a write, keeps the new value and clears the kept value.
- API test: two moves in one draft, then a discard, give the value from before the first move.
- API test: a plan to a host that holds each media file gives the old copies on the other host in `from_connector_ids`.
- API test: a plan with no old copy and `delete_later` true keeps no hold.
- Node test or drive-ui step: after a refused delete and a discard, the first delete sheet opens with no error.
- `bash tools/gate.sh` passes.

Visual:

- Screenshot of the move sheet of the test album to a host that holds each media file. It shows "Old files" and the check box.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
