# Studio v1 Task 196: Undo And Redo On The Person Page And Settings

Decision: [ADR 0227](../adr/0227-undo-and-redo.md) items 1 to 12.

Status: Open. It needs task 195.

## Problem

Finding 14 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). The person page and Settings have Save and Discard, but no undo of one edit.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Undo history | reuse | crates/api/ui/undohistory.js | `makeHistory` from task 195 (579fb40). |
| Person values | extend | crates/api/ui/peoplepages.js | `PersonPage` keeps the pending values in `values` near line 929. Save near line 943, Discard and the bar near line 1174. |
| Settings draft | extend | crates/api/ui/settingsdraft.js | `changeSetting`, `discardSettings` and `saveSettings`. `SettingsPage` in app.js near line 2886 holds the draft, and its bar is near line 2960. |
| Bar buttons | extend | crates/api/ui/editor.js | The two buttons of `DraftToolbar` (class `button icon-button undo` and `redo`). Move them into one exported component, for example `UndoButtons({ canUndo, canRedo, undo, redo })`, and use it in the three bars. |
| Key handler | extend | crates/api/ui/editor.js | The `useEffect` with `undoKey` in `DraftToolbar`. Move it into one exported hook, for example `useUndoKeys(undo, redo)`, and use it in the three bars. Keep the test for a field in an open sheet. |

## Changes

1. Person page: record the `values` before each change. Undo and Redo set `values`. Save and Discard clear the history (ADR 0227 item 8). The nickname saves at once (ADR 0194) and is not part of this history.
2. Settings: record the draft before each `changeSetting`. Undo and Redo set the draft, and the scale preview follows the values. Save and Discard clear the history.
3. Each bar gets the buttons of task 195 before the draft status, and the key handler of task 195 while the page shows.
4. The history of the person page has one entry for each person ID. Settings has one history.

## Files

- `crates/api/ui/peoplepages.js`
- `crates/api/ui/settingsdraft.js`
- `crates/api/ui/app.js`
- `crates/api/ui/editor.js` (the shared button component and key hook)
- `crates/api/ui-tests/people.test.mjs` and `crates/api/ui-tests/settings*.test.mjs`

## Checks

Mechanical:

- A UI test for each bar: the buttons "Undo" and "Redo" before `draft-status`.
- A UI test of the settings draft: a change, an undo and a redo give the values of each step, and Save clears the history.
- `tools/node --test crates/api/ui-tests/` and `bash tools/gate.sh` pass.

Visual (the main session):

- Walk at 1280 px and 390 px with `setup-194.py`: change the name of a person, Undo, Redo. Change the scale in Settings, Undo. The pending marks of the tabs follow each step.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
