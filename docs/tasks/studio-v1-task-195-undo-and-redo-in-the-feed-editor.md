# Studio v1 Task 195: Undo And Redo In The Feed Editor

Decision: [ADR 0227](../adr/0227-undo-and-redo.md) items 1 to 12.

Status: Open.

## Problem

Finding 14 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). The feed editor has no undo. The draft saves the full model after each edit and keeps no history.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Undo history | new | crates/api/ui/undohistory.js | Searched crates/api/ui/*.js, crates/core and ~/build/musicindex-lib: no history of edits. The merge undo of crates/core/src/people.rs keeps one row snapshot, not a list of steps. |
| Draft model | extend | crates/api/ui/draft.js | `update` near line 268 sets `current.current` and starts the save timer. The history records the model before each edit. |
| Draft toolbar | extend | crates/api/ui/editor.js | `DraftToolbar` near line 2342 renders the draft status. The two buttons go at its left. |
| Icon | extend | crates/api/ui/app.js | `ICONS` near line 1898. Add `undo-2` from ~/build/musicindex-assets/icons/lucide/undo-2.svg. |
| Linux key script | reuse | crates/desktop-app/src/linux_page.js | The handler near line 55 returns when `event.defaultPrevented` is true. No change is necessary if the page takes the key first. |

## Changes

1. New module `crates/api/ui/undohistory.js`, with no Preact import, so that a test can load it:
   - `makeHistory({ limit = 100, pause = 1000 })` gives `{ record(before, field, now), undo(current), redo(current), clear(), canUndo, canRedo }`. `record` takes the state before an edit, a field key or null, and the time.
   - `record` starts a new step in three cases: `field` is null, `field` is different from the field of the last step, or `now` is more than `pause` ms after the last record.
   - In other cases, `record` keeps the last step. The state before the first edit of the field stays.
   - `record` deletes the redo steps (ADR 0227 item 6). The list keeps at most `limit` steps, and the oldest step goes first (item 11).
   - `undo(current)` gives the state of the last step and puts `current` on the redo list. `redo(current)` does the opposite. Each gives null when it has no step.
   - `undoKey(event)` gives `'undo'`, `'redo'` or null. Ctrl or Cmd with Z gives undo, with Shift gives redo. Ctrl+Y gives redo. Alt gives null (item 3).
2. In `draft.js`:
   - Keep one history for each feed ID in a module map, so that it stays when the user opens a different page (item 7).
   - `update` records the model before the change. The field key is the `id` or the `name` of `document.activeElement` when it is an `input` or a `textarea`, else null.
   - Add `undo()` and `redo()` to the hook. Each sets the model of the step and saves as after an edit (item 5). Add `canUndo` and `canRedo`.
   - Clear the history at a write, at Discard and at each load of the draft that a route other than the draft save causes (items 8 and 10). Find each call of `draft.load()` after such a route in editor.js and app.js.
3. In `DraftToolbar`, put two icon buttons before the draft status: class `button icon undo` and `button icon redo`, `aria-label` "Undo" and "Redo", `title` "Undo (Ctrl+Z)" and "Redo (Ctrl+Shift+Z)". Disable each button with no step. The Redo icon is `undo-2` mirrored with `transform="scale(-1 1)"` and `transform-origin="center"`.
4. A keydown handler on `document` in the capture phase, active while a feed editor shows. It calls `undoKey`. For a result, it calls `preventDefault` and the undo or the redo of the draft (item 9).
5. Write a short comment above each new function. Name ADR 0227 and its items.

## Files

- `crates/api/ui/undohistory.js` (new)
- `crates/api/ui/draft.js`
- `crates/api/ui/editor.js`
- `crates/api/ui/app.js`
- `crates/api/ui/styles.css` (only if the buttons need a rule)
- `crates/api/ui-tests/undohistory.test.mjs` (new)

## Checks

Mechanical:

- `crates/api/ui-tests/undohistory.test.mjs`: one step for a field, a new step for a different field and after 1001 ms, undo then redo, no redo after a new record, at most 100 steps, clear, and each key case of `undoKey`.
- A UI test that `DraftToolbar` has the buttons "Undo" and "Redo" before `draft-status`.
- `tools/node --test crates/api/ui-tests/` and `bash tools/gate.sh` pass.

Visual (the main session):

- Walk at 1280 px and 390 px: type in the title of a release, then in the description. Undo gives the old description, Undo again gives the old title, Redo gives the new title. The buttons are disabled before the first edit.

Human:

- In the Linux desktop app, Ctrl+Z in a text field of the feed editor gives the undo of the page.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
