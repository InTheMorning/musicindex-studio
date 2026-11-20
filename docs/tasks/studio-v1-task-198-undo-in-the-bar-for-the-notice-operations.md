# Studio v1 Task 198: Undo In The Bar For The Notice Operations

Decision: [ADR 0228](../adr/0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 1 to 7.

Status: Open.

## Problem

Finding 2 of the [rc.28 human check](../reviews/2026-11-20-rc28-human-check.md). Delete Person, Remove from Collection, the menu **Kind** and the nickname show a notice with **Undo** at the bottom of the window for 10 seconds. The operator expected **Undo** in the bar, as for each other edit.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Undo history | extend | crates/api/ui/undohistory.js | `makeHistory` keeps state snapshots. Add a step that holds two functions, `undo` and `redo`, for an operation that saves through its own route. Do not make a second history. |
| Undo buttons and keys | reuse | crates/api/ui/editor.js | `UndoButtons` near line 2389 and `useUndoKeys` near line 2395. |
| Undo notice | extend | crates/api/ui/undonotice.js | Remove the notice markup and the 10-second timer. Keep the texts of `undoNoticeText` for the bar status of items 3 and 5, and keep one provider that holds the delete step of the app. |
| Bar | reuse | crates/api/ui/app.js | `Bar` near line 2003. The People page bar is in crates/api/ui/peoplepages.js near line 259. The Releases page bar is in app.js near line 3551. |
| Core undo record | reuse | crates/core/src/people.rs | `undo_delete` near line 3851 and `clear_delete_undo` near line 3879. Routes `POST /api/v1/undo` and `DELETE /api/v1/undo`. |
| Delete Person and nickname | extend | crates/api/ui/peoplepages.js | `deletePerson` near line 1080 and the `NicknameSheet` change near line 1246. |
| Remove from Collection and Kind | extend | crates/api/ui/editor.js | Remove from Collection near line 2435. `ReleaseKindField` near line 2823. |
| Remove from Collection, read-only feed | extend | crates/api/ui/app.js | `ReadOnlyFeed` near line 2046. |

## Changes

1. The nickname change adds a step to the history of the person page. **Undo** sends the nickname before through `PATCH /people/{id}`. **Redo** sends the new nickname again (ADR 0228 item 2).
2. The menu **Kind** adds a step to the history of the feed editor. **Undo** sends the kind before through `PUT /feeds/{id}/kind`, and **Redo** sends the new kind.
3. After Delete Person or Remove from Collection with a core record, the page that opens shows **Undo** and **Redo** in its bar. The position of the draft status shows the text of `undoNoticeText`. **Undo** sends `POST /api/v1/undo`. **Redo** sends the same delete again with no confirmation sheet (items 3 and 4).
4. The delete step stays while the app is open. A new delete replaces it. Do not send `DELETE /api/v1/undo` on a timer.
5. When an undo or a redo fails, the bar shows the error text of the core in the status position, and the step goes (ADR 0227 item 19).
6. The keys of ADR 0227 item 3 work on the page of item 3.
7. Remove the bottom notice and its CSS (`.undo-notice`). Delete each test that asserts the notice.

## Checks

Mechanical:

- UI test: a nickname change and a kind change add a step, and undo calls the route with the value before.
- UI test: after a delete, the page bar model has the status text and an enabled **Undo**. No module renders `.undo-notice`.
- `bash tools/gate.sh` passes.

Visual (the main session):

- Walks `.studio-dev/walk/u197-1280.json` and `u197-390.json`, changed for the bar. Screenshots of the People bar after Delete Person and after **Undo**, and of the page after Remove from Collection.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
