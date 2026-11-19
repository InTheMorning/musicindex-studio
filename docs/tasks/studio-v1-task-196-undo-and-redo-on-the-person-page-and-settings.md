# Studio v1 Task 196: Undo And Redo On The Person Page And Settings

Decision: [ADR 0227](../adr/0227-undo-and-redo.md) items 1 to 12.

Status: Done.

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
- `crates/api/ui/styles.css` (the narrow bar)

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

## Result

- The delegate run made `UndoButtons` and `useUndoKeys` in editor.js, and the histories of the person page and of Settings.
- The main session corrected these defects of the delegate run:
  - The person page recorded each key as a step. Now the person page and the feed editor use one function, `focusedField` in undohistory.js, for the field of the step (ADR 0227 item 4).
  - The person page and Settings kept the history in the component. A page that opened again lost it. Now each history is in a module map, as in draft.js (item 7).
  - A new read of the person cleared the history, also the first read when the page opened. Now only a read after the first one clears it (items 8 and 10). A confirmed leave with pending changes also clears it.
  - Settings used the draft of the last render for a change. Two changes in one event lost the first one. Now the person page and Settings keep the latest state in a ref.
  - In a narrow window, the bar rule gave each button 14 px padding on each side, so the icon of an icon button had no space. This also applied to the feed editor of task 195. Now the narrow icon buttons have no padding.
- Walks: `.studio-dev/walk/u196-1280.json` and `u196-390.json` with `setup-194.py`. Two edits of the name are one step. Undo, Redo and Ctrl+Z give each value, and the tab marks follow. After a visit to People, Redo still works. Discard clears the history. In Settings, Undo and Redo of the UI scale change the scale of the page.
- The 390 px walk has no touch, because with touch the system gives the size and the scale section does not show (ADR 0134).
