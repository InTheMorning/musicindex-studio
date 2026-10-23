# Studio v1 Task 019d: The Import Sheet And The Track Page

Parent: [task 019](studio-v1-task-019-lossless-import.md). Decision: [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) items 5, 6, 8 and 9.
Sequence: after [task 019c](studio-v1-task-019c-upload-two-files.md). It is the last packet of task 019.

## Goal

The import sheet tells the user about the MP3 file and the FLAC file. A lossless track gets its two alternative enclosures. The track page shows them.

## Files

Inspect: `crates/api/ui/editor.js` (`ImportSheet`, `WAV_NOTICE`, `AddTrackSheet`, `TrackPage`, `Enclosure`), `crates/api/ui/styles.css`, the route of task 019c.

Change: `editor.js` and `styles.css`.

Do not change: `values.js`, `draft.js`, `app.js`, `crates/core`, `crates/api/src`.

## Architecture

These choices are fixed. Do not change them.

- A WAV row and a FLAC row each show a `.notice` with the text of ADR 0033 item 9. The notice names the file, and does not use the `--error` color.
- When a file is ready, its facts show the name of each file that Studio keeps: "File" with the MP3 name, and "Lossless file" with the FLAC name when there is one.
- The add gives a lossless track `enclosure` from the MP3 file and `alternate_enclosures` with two entries, in this sequence:
  1. `{ media_type: 'audio/mpeg', length, title: 'MP3', default: 'true', sources: [{ uri: <MP3 url> }] }`
  2. `{ media_type: 'audio/flac', length, title: 'Lossless (FLAC)', sources: [{ uri: <FLAC url> }] }`

  Each other attribute is `null`. Each list entry gets a key, as the other lists of the model do.
- Each other new track, from the import or from `AddTrackSheet`, gets `alternate_enclosures: []`.
- "Cancel", and a row that the user removes, call `DELETE /feeds/{id}/files?path=` for the MP3 file and for the FLAC file.
- The track page shows each alternative enclosure after the audio file, as one summary line: its title or its type, and its size. The line has a "Remove" button. There is no form to edit an alternative enclosure.

## Steps

1. Change the notice and the facts of a row.
2. Change the add and the removal of files.
3. Add the summary lines to the track page.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- The styles agree with `each_background_uses_a_token`. Each `details` has a `name` or the class `disclosure`.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` on a new release in a folder with a public address. It sets the files of the sheet to the WAV test file and the MP3 test file, with a `DataTransfer` in the page. After the add:
  - The WAV track has an MP3 enclosure and two alternative enclosures.
  - The MP3 track has no alternative enclosure.
  - The diff gives the MP3 file and the FLAC file of the WAV track, and the MP3 test file.
  - No path in the diff ends with `.wav`.
  - `$small()` gives an empty list, and the console shows no error.
- After "Remove" of the FLAC line, the diff does not list the FLAC file.

Visual:

- None in this packet. The visual check of task 019 covers it.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the sheet needs a change of an API route, or when the draft API refuses the new field.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-019d-import-sheet-and-track-page.md
- docs/adr/0033-a-lossless-import-gives-an-mp3-enclosure.md
- The files that the Files section names.

Goal:
- The import sheet tells the user about the MP3 file and the FLAC file. A lossless track gets its two alternative enclosures, and the track page shows them.

Constraints:
- Follow the Architecture section of the task file fully.
- Each control is 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- values.js, draft.js, app.js, crates/core, crates/api/src.
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
