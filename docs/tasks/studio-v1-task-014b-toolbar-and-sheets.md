# Studio v1 Task 014b: The Draft Toolbar And The Sheets

Parent: [task 014](studio-v1-task-014-editor-layout.md). Decision: [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 1 to 4.
Sequence: after [task 014a](studio-v1-task-014a-background-levels.md).

## Goal

Move the draft into the shell. Show the draft controls in the toolbar. Show the diff and the snapshots in sheets.

## Files

Inspect: `crates/api/ui/app.js` (`Shell`, `FeedDetail`, `useAction`), `crates/api/ui/editor.js`, `crates/api/ui/values.js` (its imports), `crates/api/ui/styles.css`, `crates/api/src/ui.rs` (`FILES`), and the mockup `docs/notes/2026-10-22-editor-layout-mockup.html` (the `.bar` and `dialog.sheet` styles, and the `review` and `versions` dialogs).

Change: `app.js`, `editor.js`, `styles.css`, `ui.rs`, and the new file `crates/api/ui/draft.js`.

Do not change: `values.js`, `crates/core`, the API routes, and the tests.

## Architecture

These choices are fixed. Do not change them.

- The new module `draft.js` holds `useDraft`, `withCids`, `strip`, `positions`, `rekey`, `copyServerFields`, `SAVE_DELAY` and `STATUS`. `editor.js` imports them. `editor.js` continues to export `withCids` with `export { withCids } from './draft.js';`, so that `values.js` does not change.
- `useDraft(feedId)` accepts `null`. With `null`, it loads nothing and gives `model: null` and `status: 'idle'`. When `feedId` changes, it sets `model` to `null` before it loads the new draft.
- `Shell` calls `useDraft` one time. The ID is the selected feed when the feed has a connector and its type is Album. For all other feeds, the ID is `null`.
- `Shell` gives the draft to `FeedDetail`, and `FeedDetail` gives it to `AlbumEditor`. `AlbumEditor` does not call `useDraft`.
- The toolbar is the component `DraftToolbar` in `editor.js`. `FeedDetail` shows it in its `.bar` when `draft.model` is not `null`.
- Each sheet is a `dialog` element. A component opens it with `showModal()` in an effect, and closes it with `close()`. The `close` event of the dialog sets the open state to false, so that the Escape key works.

## Steps

1. Make `draft.js` and move the parts of the architecture list into it. Add `draft.js` to `FILES` in `ui.rs`.
2. Make `useDraft` accept `null`, and call it in `Shell`.
3. Change the `.bar` of `FeedDetail` into a row: the feed title, then `DraftToolbar`.
4. `DraftToolbar` shows a short status, a "Versions" button and a "Review" button (`.button.primary`). The short status texts are:
   - `idle` or `loading`: "Loading…"
   - `waiting` or `saving`: "Saving…"
   - `saved` and `draft.changed`: "Saved, not written"
   - `saved` and not `draft.changed`: "No changes"
   - `failed`: "Not saved", in the `--error` color.
5. "Review" opens `ReviewSheet`. It is the current `Changes` component in a sheet. When the sheet opens, it runs the current `examine`. Keep each notice and each button of `Changes`. Put "Discard the draft" and "Write to the host" in the sheet footer. When the write is correct, the sheet closes.
6. "Versions" opens `VersionsSheet`. It is the current `Snapshots` component in a sheet, with no `details` element. When "Put in the draft" is correct, the sheet closes.
7. Remove the `Changes` panel and the `Snapshots` panel from `AlbumEditor`.
8. Add the styles for `.bar` as a row, the short status, and `dialog.sheet` with its header, body and footer. Copy them from the mockup. Use the level tokens of task 014a. The sheet uses `--level-2`, and a group in a sheet uses `--level-3`.

## Constraints

- No write occurs without a diff that the user saw ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)). The write button shows only with a current diff that has changes, and only when no address is incorrect.
- After a user edit, the diff of an open sheet is not current. Keep the current rule that `draft.edits` clears the diff.
- Each control is 44 px high or more. Each control works from the keyboard.
- Keep the text of each notice and each confirmation.

## Acceptance Criteria

Mechanical:

- The tests of ADR 0016 (`each_ui_address_in_the_ui_files_is_served`) and ADR 0028 pass.
- The three commit checks pass.
- The agent runs `tools/drive-ui.mjs` at 1280 px. It edits the album title, waits 1500 ms, clicks "Review", and reads the sheet text. The text contains the new title in a `+` line. `$small()` gives an empty list. The console shows no error.

Visual:

- None in this packet. Task 014 has the visual check.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p studio-desktop -- --data-dir .studio-dev/task-014/data
node tools/drive-ui.mjs "<address from the run command>" .studio-dev/task-014/shots '<steps-json>'
```

Before the first run, copy `.studio-dev/check-011/data` to `.studio-dev/task-014/data`. It has two album feeds and needs no passphrase.

The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot. A test result alone does not show a visual defect.

## Escalation

Stop and report when:

- a notice of `Changes` has no clear position in the sheet.
- `values.js` must change.
- the draft must load again when the user selects a different page of the same feed.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-014b-toolbar-and-sheets.md
- docs/adr/0029-editor-pages-toolbar-and-three-levels.md
- crates/api/ui/app.js
- crates/api/ui/editor.js
- crates/api/ui/styles.css
- crates/api/src/ui.rs
- docs/notes/2026-10-22-editor-layout-mockup.html
- tools/drive-ui.mjs (its header)

Goal:
- The draft lives in the shell. The toolbar shows the draft status, "Versions" and "Review". The diff and the snapshots open in sheets.

Constraints:
- Follow the Architecture section of the task file fully.
- No write without a current diff that the user saw.
- Keep the text of each notice. Keep each control 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- values.js, crates/core, the API routes, the tests.
- Do not commit.

Acceptance criteria:
- The criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- The drive-ui.mjs check of the task file.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
