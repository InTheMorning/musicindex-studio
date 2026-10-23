# Studio v1 Task 013c: The Picker Sheet

Parent: [task 013](studio-v1-task-013-recipient-picker.md). Decisions: [ADR 0027](../adr/0027-the-recipient-picker.md) items 1, 5 to 7, [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) and [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: after [task 013b](studio-v1-task-013b-recipient-routes.md). It is the last packet.

## Goal

Let the user add recipients from a list, save a recipient, and import recipients from a musicindex feed or track.

## Files

Inspect: `crates/api/ui/values.js` (`Recipients`, `Payments`, `AlbumPayments`, `TrackPayments`, `fresh`), `crates/api/ui/editor.js` (`AddTrackSheet`, `useSheet`, `openOnce`, `startOpen`), `crates/api/ui/app.js` (`api`, `FindFeeds`), `crates/api/ui/styles.css` (`dialog.sheet`, `.group`, `.link-row`), and the routes of task 013b in `crates/api/src/lib.rs`.

Change: `values.js`, `editor.js` (only to export `useSheet`), and `styles.css`.

Do not change: `app.js`, `draft.js`, `crates/core`, `crates/api/src`, and the tests.

## Architecture

These choices are fixed. Do not change them.

- `Recipients` gets one more row after "＋ Add a recipient": "＋ Pick from your recipients". It opens `PickerSheet`, a `dialog.sheet` that uses `useSheet`.
- `PickerSheet` has two parts, one after the other in the sheet body:
  1. **Your recipients**: a filter field, then a group with one row for each entry of `GET /recipients`. Each row has a checkbox, the name, the address and "In N feeds" or "Saved". A saved entry has a "Remove" button that calls `DELETE /recipients/saved/{id}` and loads the list again.
  2. **From musicindex**: a search field that calls `GET /search?q=`. A hit opens its feed: `GET /musicindex/feeds/{guid}/recipients`. The sheet shows the routes of the feed as rows with checkboxes, then the tracks of the feed as link rows.
     - A track row shows the routes of that track from `GET /musicindex/feeds/{guid}/tracks/{track}/recipients`.
     - A "Back" row goes back from a track to its feed, and from a feed to the hits.
- The identity of a recipient is the address, the custom key and the custom value. When the identity of a row is in the editor list, the row shows "Already in this list". Its checkbox is disabled.
- The footer has "Cancel" and "Add N recipients". N is the number of selected rows of the two parts.
- The add appends each selected recipient after the current recipients, through `fresh` and `withCids` (ADR 0027 item 7). A recipient from **Your recipients** gets the split `''`. A recipient from musicindex keeps its split and its fee. After the add, the sheet closes. No new entry opens.
- An open recipient of the editor gets a "Save to your recipients" button adjacent to its Move and Remove buttons. It calls `POST /recipients/saved`, then shows "Saved" for 2 seconds.

## Steps

1. Export `useSheet` from `editor.js`.
2. Make `PickerSheet` in `values.js`, with the two parts and the footer.
3. Add the "Pick from your recipients" row to `Recipients`, and the "Save to your recipients" button to an open recipient.
4. Add the styles. A row in the sheet is a `.link-row` in a `.group` on level 3. Use the level tokens and the fill tokens only.

## Constraints

- The picker does not remove or replace a recipient of the editor (ADR 0027 invariant).
- A list in the sheet has no `details` element, or each `details` has a `name` (ADR 0028).
- Each control is 44 px high or more, and each control works from the keyboard. A checkbox is in a label of 44 px or more.
- Each network error shows in an `ErrorNotice` in the sheet. The sheet stays open.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` at 1280 px and at 390 px. It opens the track page of a track with its own recipients, and opens the picker. It selects two rows of **Your recipients** that are not in the list, and adds them. The list of the track then has its first recipients, and the two new recipients at the end. `$small()` gives an empty list. The console shows no error.

Visual:

- None in this packet. Task 013 has the visual check.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p studio-desktop -- --data-dir .studio-dev/task-013/data
node tools/drive-ui.mjs "<address from the run command>" .studio-dev/task-013/shots '<steps-json>'
```

## Escalation

Stop and report when:

- the picker needs a change to `app.js` or to an API route.
- the sheet needs a fourth background level.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-013c-picker-sheet.md
- docs/adr/0027-the-recipient-picker.md
- docs/adr/0029-editor-pages-toolbar-and-three-levels.md
- crates/api/ui/values.js
- crates/api/ui/editor.js
- crates/api/ui/app.js
- crates/api/ui/styles.css
- crates/api/src/lib.rs (the recipient routes)

Goal:
- A picker sheet adds recipients from the user's list and from musicindex. An open recipient can be saved.

Constraints:
- Follow the Architecture section of the task file fully.
- The picker only appends. Keep each control 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- app.js, draft.js, crates/core, crates/api/src, the tests.
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
