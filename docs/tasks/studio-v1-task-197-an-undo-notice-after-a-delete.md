# Studio v1 Task 197: An Undo Notice After A Delete

Decision: [ADR 0227](../adr/0227-undo-and-redo.md) items 13 to 19.

Status: Done.

## Problem

Finding 14 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). Delete Person and Remove from Collection delete the rows of the collection, with no way back. The menu Kind and the nickname save at once, with no undo.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Undo record of the core | extend | crates/core/src/people.rs | The merge undo stores a JSON snapshot in `person_merge_undo` near line 3437 and puts it back in one transaction near line 3493. Use the same snapshot and restore functions for a second record. Do not change the merge record (ADR 0217 items 14 to 17). |
| Delete Person | extend | crates/core/src/people.rs | The delete near line 532. Its route is in crates/api/src/people.rs near line 300. |
| Remove from Collection | extend | crates/core/src/collection.rs | The delete near line 3035. Its route is crates/api/src/routes.rs near line 2162. |
| Notice | extend | crates/api/ui/peoplepages.js | The merge notice with **Undo** near line 1179. Move it into one notice component that the shell shows. |
| Confirmation texts | extend | crates/api/ui/confirms.js | The sheet of Remove from Collection near line 16. |
| Kind menu | reuse | crates/api/ui/editor.js | `PUT /feeds/{id}/kind` near line 2790. |
| Nickname | reuse | crates/api/ui/peoplepages.js | `PATCH /people/{id}` near line 1200. |

## Changes

1. Core: a table for one undo record of the last operation (a schema change: add a migration and raise the schema number). Delete Person and Remove from Collection write the record in the transaction of the delete, with the rows that the delete removes. A new record replaces the old one.
2. Core: a restore function that puts back the rows of the record with their IDs in one transaction, then deletes the record. A function that deletes the record when the notice ends.
3. Remove from Collection with pending files or pending host operations writes no record. The confirmation sheet then adds "This cannot be undone." (ADR 0227 item 17).
4. API: `POST /api/v1/undo` puts back the last record, and `DELETE /api/v1/undo` deletes it. The answer of each delete route says if a record exists.
5. UI: one notice component in the shell with the text of ADR 0227 item 13 and **Undo**, for 10 seconds. A new operation replaces it (item 15). At the end of the 10 seconds, the UI sends `DELETE /api/v1/undo`.
6. UI: for the menu Kind and the nickname, **Undo** sends the value before the change through the same route (item 18).
7. When an undo fails, the notice shows the error text and no **Undo** (item 19).

## Checks

Mechanical:

- Core tests: Delete Person, then the restore, gives the same rows with the same IDs, the links and the change set references. The same for Remove from Collection with drafts and snapshots. A second delete replaces the first record. Remove from Collection with a pending file writes no record.
- API tests of `POST /api/v1/undo` and `DELETE /api/v1/undo`, and of the refusal with no record.
- A UI test of the notice texts of item 13.
- `bash tools/gate.sh` passes.

Visual (the main session):

- Walk at 1280 px and 390 px: Delete Person, the notice, **Undo**, and the person is back in People. Remove from Collection, the notice, **Undo**, and the release is back in Releases.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

- The first delegate run stopped with no change. The second run, at effort `high`, made the core record, the routes and the notice.
- Routes: `POST /api/v1/undo`, `DELETE /api/v1/undo` and `GET /api/v1/feeds/{id}/remove-info`. The last route tells the confirmation sheet if the feed has pending files or host operations (item 17). The schema is 45.
- The main session corrected these defects of the delegate run:
  - A feed delete also removes the `person_members` rows of the feed, but the feed list of the record did not keep them. After Undo, the release came back with no people. The test `delete_undo_tables_cover_each_foreign_key` now compares the two lists with each foreign key of the schema.
  - The run moved the merge notice of the person page into the new notice, with no time limit. ADR 0227 item 16 keeps the merge undo as it is, so the merge notice is again on the person page.
  - The notice was at the top of the window and covered the title of the bar. Now it is at the bottom center, with the `--level-2` background and a shadow.
- Walks: `.studio-dev/walk/u197-1280.json` and `u197-390.json` with `setup-194.py`. Delete Person shows "Ann Bell is deleted.", and Undo puts Ann Bell back in People. Remove from Collection shows the notice, and Undo puts feed 18 back with its 20 tracks. After Undo, no notice shows.
- Open: the walks do not include the notices of the menu Kind and the nickname. A UI test checks their texts.
- Open: when the app stops during the 10 seconds, the record stays in the collection until the next operation of item 13. No notice can then use it.
