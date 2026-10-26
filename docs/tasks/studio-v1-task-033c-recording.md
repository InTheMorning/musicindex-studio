# Studio v1 Task 033c: The Recording Of An Ended Show

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) item 6, [ADR 0026](../adr/0026-import-music-files-through-the-draft.md), [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) and [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md).
Sequence: the third packet of task 033. It depends on 033b.

## Goal

The page of an ended show has "Add the recording". The user selects an audio file. Studio imports it as it imports the new audio of a track, and the enclosure of the live item points to the file.

## Files

Inspect: `crates/api/ui/editor.js` (the "Replace the audio" flow of the track page of ADR 0040, and the live item page of 033b), `crates/api/src/routes.rs` (`upload_file`, `UploadQuery`, `import_upload`), `crates/api/src/tests.rs` (the tests of the replacement of the audio of a track), `crates/api/ui/live.js`.

Change: `crates/api/ui/editor.js`, `crates/api/ui/live.js`, `crates/api/ui-tests/live.test.mjs`, `crates/api/src/tests.rs`. `crates/api/src/routes.rs` only when the upload route cannot give the facts of the file for a live item.

Do not change: the core, the connected apps and the grants.

## Architecture

These choices are fixed. Do not change them.

- The button "Add the recording", with the action role, shows only when the status is `ended`. When the live item has a recording, the button is "Replace the recording".
- The flow uses the upload route with `replace=true`, as "Replace the audio" does. The live item in the draft then gets the enclosure (address, length and type) of the new file. A lossless file also gives an alternative enclosure, as for a track ([ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md)).
- The stream enclosure of the show goes out of the draft and is replaced by the enclosure of the recording.
- `live.js` gets `canRecord(liveItem)`, true for the status `ended`.

## Steps

1. Add `canRecord` and its Node test.
2. Add the button and the flow on the live item page.
3. Add an API test that writes an ended live item with the enclosure of an uploaded file.
4. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- The import follows the rules of the import of a track. Do not make a second import path.

## Acceptance Criteria

Mechanical:

- An API test uploads a test MP3 for an ended live item, puts its enclosure into the draft, and writes. The hosted feed has the enclosure on the live item, and the host has the file.
- The four commit checks pass.

Visual:

- The planning model checks the button on an ended show, and the page after the recording, in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a live item needs a second import path.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033c-recording.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- An ended show gets "Add the recording". It reuses the "Replace the audio" flow of a track, and the enclosure of the live item then points to the new file.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.
- Do not make a second import path.

Do not touch:
- The core, the connected apps and the grants.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

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
