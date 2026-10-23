# Studio v1 Task 021g: The Library UI

Parent: [task 021](studio-v1-task-021-release-library-7a.md). Decisions: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 1, 11 to 14 and 17, [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) and [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: after [task 021f](studio-v1-task-021f-library-api.md). It is the last packet of task 021.

## Goal

The user sets the library folder, sees the files of a release on each host, starts the operations on files, and sees each host operation in the diff.

## Files

Inspect: `crates/api/ui/app.js` (the start screens, the settings screen, `FeedTree`), `crates/api/ui/editor.js` (`AlbumEditor`, the pages, `ReviewSheet` or the diff sheet, `ImportSheet`, `FileRows`), `crates/api/ui/styles.css`, and the routes of task 021f.

Change: `app.js`, `editor.js`, `styles.css`, and a new `crates/api/ui/files.js` when the Files page is large.

Do not change: `crates/core`, `crates/api/src`, `values.js`, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

- When `GET /library` gives `root: null`, a sheet opens after the vault and before the editor. It has a path field with `default_root`, the text "Studio keeps one folder for each release here: its feed file, its images and its audio files.", and the button "Use this folder". After the move, the sheet shows the `MoveReport` and the button "Continue".
- The settings screen shows the library path, and the two download limits as fields in MB.
- The tree gets the page "Files" for each release, after "Recipients".
- The Files page:
  - A group "Hosts" with the default host of the release files, as a list of the connected hosts.
  - A group for each library file: one summary line with the format badge of `FileRows`, the file name, `In use` or `Not in use`, and the number of copies. An open entry shows each copy as a row with its host, its path and its URL, and the buttons "Reference", "Move to…" and "Delete". It also has "Copy to…" and "Host of this file".
  - A group "Other files on the hosts" with each host file that is not a copy, and its "Delete" button.
  - The button "Scan the hosts" runs the scan of each host of the release.
- "Copy to…" and "Move to…" open a list of the hosts. Each operation calls `POST /operations`, and the toolbar status becomes "Saved, not written".
- The diff sheet shows the host operations above the XML diff. Each one is a row with its kind (Upload, Copy or Delete), its host, its path and its size. A blocked deletion is a row in the `--error` color with the titles of the feeds that refer to it. Each draft operation row has "Remove".

## Steps

1. Add the library sheet and the settings.
2. Add the Files page.
3. Add the host operations to the diff sheet.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- The styles agree with `each_background_uses_a_token`. Each `details` has a `name` or the class `disclosure`.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` with a new data folder, a release on host A, and a second local host B. It sets the library folder in the sheet, opens the Files page, copies one file to host B, moves a second file to host B, and opens the diff. The diff shows two copies and one deletion. `$small()` gives an empty list, and the console shows no error.
- A delete of a file in use shows the blocked row.

Visual:

- The visual check of milestone 7a covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the UI needs a route that task 021f does not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-021g-library-ui.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- The user sets the library folder, sees the files of a release on each host, starts the operations on files, and sees each host operation in the diff.

Constraints:
- Follow the Architecture section of the task file fully.
- Each control is 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, crates/api/src, values.js, draft.js.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
