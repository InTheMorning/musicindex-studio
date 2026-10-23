# Studio v1 Task 018d: The Import Sheet

Parent: [task 018](studio-v1-task-018-import-music-files.md). Decisions: [ADR 0026](../adr/0026-import-music-files-through-the-draft.md), [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) and [ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md).
Sequence: after [task 018c](studio-v1-task-018c-upload-route.md). It is the last packet of task 018.

## Goal

Let the user select music files and images, see their facts, and add them to the album.

## Files

Inspect: `crates/api/ui/editor.js` (`AlbumEditor`, `AlbumPage`, `AddTrackSheet`, `useSheet`), `crates/api/ui/app.js` (`api`, `FeedTree`), `crates/api/ui/values.js` (`PickerSheet`, as a model of a sheet with a list), `crates/api/ui/styles.css`, and the route of task 018c.

Change: `editor.js`, `app.js` and `styles.css`.

Do not change: `values.js`, `draft.js`, `crates/core`, `crates/api/src`.

## Architecture

These choices are fixed. Do not change them.

- The tree and the album page get the row "＋ Import files", after "＋ Add a track". It opens `ImportSheet`, a `dialog.sheet`.
- The sheet has a file field with `multiple` and `accept="audio/*,image/*,.wav,.flac,.m4a,.ogg,.opus"`. It sends each selected file with `fetch` to `POST /feeds/{id}/files?name=`, one file at a time, with the token. The body is the `File`.
- Each file is one row: its name, its state (waiting, sending, ready, or the error), and its facts when it is ready.
- A WAV file shows a `.notice` before the send and after it. The notice does not use the `--error` color, because the conversion is not an error. Its text is: "Studio converts this WAV file to FLAC before the import. Some listener apps do not play FLAC." The notice names each WAV file.
- An image row has a list: "Album art", each audio file of the sheet, or "Not used". The first image is "Album art" when the album has no image.
- The footer has "Cancel" and "Add N tracks". The add puts the audio files in the sequence of their track numbers, then the files with no number in the sequence of selection (ADR 0026 item 3). Each becomes a track with `title`, `duration`, and an `enclosure` with `url`, `length` and `media_type`. The images become the `image` of the album or of their track.
- After the add, the sheet closes and the editor shows the album page.
- "Cancel", and a row that the user removes, call `DELETE /feeds/{id}/files?path=` for each file that the sheet sent.

## Steps

1. Add `ImportSheet` and its rows.
2. Add the two rows that open it.
3. Add the styles. A row in the sheet is a `.link-row` in a `.group` on level 3.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- The styles agree with `each_background_uses_a_token`. Each `details` has a `name` or the class `disclosure`.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` on a new release in a folder with a public address. It sets the files of the sheet from the test files of task 018b, with a `DataTransfer` in the page. After the add, the album has one track for each audio file, in the sequence of the track numbers. The diff lists each file with its path in the release folder. `$small()` gives an empty list. The console shows no error.

Visual:

- None in this packet. The visual check of Phase 6 covers it.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the sheet needs a change of an API route. Stop and report when a large file stops the page.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-018d-import-sheet.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- The files that the Files section names.

Goal:
- The user selects music files and images, sees their facts, and adds them to the album.

Constraints:
- Follow the Architecture section of the task file fully.
- Each control is 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- values.js, draft.js, crates/core, crates/api/src.
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
