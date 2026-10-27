# Studio v1 Task 039b: The Picker Sheet And The Drop

Parent: [task 039](studio-v1-task-039-file-picker.md). Decisions: [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) items 1 to 5, 7, 9 to 13, 17, 20 to 22, [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md), [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md), [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md).
Sequence: the second packet of task 039. It depends on 039a.

Status: Implemented - 2026-10-27. The four commit checks are Green. The visual check is open. The [review checklist](../reviews/2026-10-27-tasks-038-039-review-checklist.md) records the review.

## Goal

Each image field and each audio field has "Choose…". The sheet has three sources. A field accepts one dropped file. The buttons "Replace the Audio File" and "Import Image" go away.

## Files

Inspect: `crates/api/ui/editor.js` (`ChannelFields`, the track page "Track image", the live item image, the publisher page `importImage`, the replacement of the audio near line 320, `ImageThumb`, `sendInParts`, the import sheet), `crates/api/ui/upload.js`, `crates/api/ui/cover.js` (task 038), `crates/api/ui/styles.css` (the sheet and the field styles), the routes of task 039a.

Change: the new model module `crates/api/ui/picker.js`, the new test file `crates/api/ui-tests/picker.test.mjs`, `crates/api/ui/editor.js`, `crates/api/ui/styles.css`.

Do not change: the API, `upload.js`, the import sheet.

## Architecture

These choices are fixed. Do not change them.

### The Model Module

- `picker.js` has no Preact and no browser state. It has these functions:
  - `fieldKind(field)`: `image` or `audio` for each field of ADR 0062 item 1.
  - `sources(field, connectors)`: the three sources in the sequence of item 2, with one host entry for each connector.
  - `hostTree(files)`: a tree of folders from the flat `path` list of the host route. A host with no folders gives one flat level.
  - `canSelect(file)`: false when `address` is `null`. Its reason text is "This file has no public address".
  - `acceptsFile(field, facts)`: true when the kind of the uploaded file is the kind of the field.
  - `dropResult(field, files)`: `upload` for one file of the right kind, else an error code `too_many` or `wrong_kind`.
  - `replacedAddresses(field, model, newAddress)`: the addresses of the field before the change, for `delete_unused`.

### The Sheet

- One component `FilePicker` opens as a sheet ([ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md)). It has the three sources of ADR 0062 item 2 as segments.
- "This release" calls `GET /feeds/{id}/picker/release`. "Hosts" calls `GET /connectors/{id}/picker` for the host that the user opens. "This computer" uses `sendInParts` and a file input with the accept list of the kind.
- Each image shows its small copy through `ImageThumb`. A file that `canSelect` refuses shows its reason, and its row is disabled.
- A selection or an upload gives the address to the field and closes the sheet. A cover field uses `withCover` of `cover.js`. Then the UI sends `delete_unused` with `replacedAddresses`.
- An audio field uses the change of the audio of a track that the replacement of ADR 0040 uses today (`withNewAudio` or its successor). The size and the media type come from the release list, from the host list, or from the upload. The duration of a selection comes from task 039c.
- An upload that `acceptsFile` refuses is removed with `DELETE /feeds/{id}/files?file=` and shows "Select an image file" or "Select an audio file".

### The Field

- Each field of ADR 0062 item 1 has the action button "Choose…" adjacent to the address field.
- Each field accepts a drop. While a file is over the field, the field shows its drop state. `dropResult` decides. An error shows below the field and changes nothing.
- The button "Replace the Audio File", its hint text, and the button "Import Image" of the publisher page go away. No code that only they used stays.

## Steps

1. Add `picker.js` with its Node tests.
2. Add `FilePicker` and the "Choose…" button on each field.
3. Add the drop on each field.
4. Remove the two old buttons and their code.
5. Run the four checks. Give the operator the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- Colors, spaces and sizes come from the tokens of `styles.css`. No raw value.
- No logic in two places: the replacement of the audio and the upload use their existing functions.

## Acceptance Criteria

Mechanical:

- The Node tests examine each function of `picker.js`:
  - `fieldKind` for each field, and `sources` with two connectors
  - `hostTree` with nested paths and with flat paths
  - `canSelect` with and without an address, and `acceptsFile` for each kind
  - `dropResult` for one file, for two files and for a file of the wrong kind
  - `replacedAddresses` for a cover field and for an audio field
- A Node test reads `editor.js` and finds no text "Replace the Audio File" and no text "Import Image".
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The sheet of the album cover with the three sources.
- A track with no image gets an image from "This computer", by the button and by a drop. The review lists the upload.
- A track gets an image from "Hosts". The review lists no upload.
- A track image is replaced. The review lists the deletion of the image from before, and "Undo" keeps it.
- The audio file of a track is replaced from "This release".

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the replacement of the audio cannot take facts from the two picker lists. Do not change its behavior for an upload.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-039b-picker-sheet.md
- docs/adr/0062-a-file-picker-for-each-file-field.md
- The files that the Files section names.

Goal:
- Add the model module `picker.js` and the sheet `FilePicker`.
- Add the "Choose…" button and the drop on each image field and each audio field.
- Remove "Replace the Audio File" and "Import Image".

Constraints:
- Follow the Architecture section of the task file fully.
- No logic in two places. Use the tokens of `styles.css`.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The API, `upload.js` and the import sheet.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The operator does the visual check.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
