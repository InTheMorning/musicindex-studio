# Studio v1 Task 041c: The Lyrics Kind Of The Picker

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 9, [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) items 1 to 3, 11, 13 and 20, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the third packet of task 041. It depends on 041b. Packet 041d uses the new kind.

Status: Not started.

## Goal

The file picker has a third kind of file: a lyrics file. "This release", "Hosts", "This computer" and a drop accept it in a field of this kind. Packet 041d adds the field.

## Files

Inspect: `crates/core/src/media.rs` (`FileKind`, `kind_of_path`, `import`, `ImportedKind`, `sniff_image`), `crates/core/src/remote.rs` (`media_type_of_name`), `crates/core/src/folder_scan.rs`, `crates/core/src/connector/blossom.rs` and `crates/core/src/connector/s3.rs` (the users of `media_type_of_name`), `crates/api/src/routes.rs` (the picker routes, the upload facts with `"kind"`), `crates/api/src/uploads.rs`, `crates/api/ui/picker.js`, `crates/api/ui/editor.js` (`FIELD_NOUN`, `useFilePick`, `FilePicker`, `ImportSheet`, `guessKind`), `crates/api/ui-tests/picker.test.mjs`.

Change: `crates/core/src/media.rs`, `crates/core/src/remote.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/api/ui/picker.js`, `crates/api/ui/editor.js`, `crates/api/ui-tests/picker.test.mjs`.

Do not change: the connectors, the write pipeline, the track page. Packet 041d adds the lyrics field.

## Architecture

These choices are fixed. Do not change them.

### The Kind In The Core

- `FileKind` gets `Lyrics`. `FileKind::parse` reads `lyrics`.
- `kind_of_path` gives `Lyrics` for `.srt`, `.vtt`, `.txt`, `.json` and `.html`. The comparison ignores case. The test that gives no kind for `a.txt` changes in the same change, because ADR 0064 item 9 changes the rule.
- `media_type_of_name` gives these media types: `.srt` gives `application/srt`, `.vtt` gives `text/vtt`, `.txt` gives `text/plain`, `.json` gives `application/json` and `.html` gives `text/html`. These are the types of the `podcast:transcript` specification.

### The Upload

- `media::import` accepts a file with a lyrics extension when its content is valid UTF-8. The result has `ImportedKind::Lyrics`, the media type of `media_type_of_name` and the size. Studio reads no tags from it.
- A lyrics file with content that is not valid UTF-8 gives `MediaErrorKind::Unsupported`.
- The image check and the audio check come first, as now. A lyrics extension does not change the result for an image or an audio file.
- The upload facts give `"kind": "lyrics"`, with `title`, `track_number` and `duration` set to `null`.
- The upload goes to the release folder, as an image does (ADR 0064 item 9). The write puts it on the host after the diff.

### The Picker Routes

- The two picker routes of task 039a accept `kind=lyrics`. They use `kind_of_path`, so they need no other change. Confirm this with the API tests.

### The UI

- `picker.js`: the field `lyrics` has the kind `lyrics`. `EXTENSIONS.lyrics` holds the five extensions. `ACCEPT.lyrics` is `.srt,.vtt,.txt,.json,.html`. `WRONG_KIND_TEXT.lyrics` is "Select a lyrics file".
- `editor.js`: `FIELD_NOUN` gets the noun of the field `lyrics`: "lyrics file". The sheet title for this kind is "Choose a lyrics file".
- The import sheet of the album accepts audio files and image files only, as now. A lyrics file there shows the error of an unsupported file, and no track changes.

## Steps

1. Add the kind and the media types to the core, with core tests.
2. Add the lyrics import, with core tests.
3. Add the API tests of the picker routes and of the upload.
4. Add the kind to `picker.js`, with Node tests.
5. Add the noun and the sheet title to `editor.js`, and confirm the import sheet.
6. Run the four checks.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No logic in two places: one list of lyrics extensions in the core, and one in `picker.js`, as for the other kinds.

## Acceptance Criteria

Mechanical:

- A core test gives `Lyrics` for each of the five extensions, in lower case and in upper case.
- A core test gives the media type of each of the five extensions.
- A core test imports a `.srt` file with UTF-8 text. The result has `ImportedKind::Lyrics` and `application/srt`.
- A core test imports a `.txt` file with bytes that are not valid UTF-8. The result is `MediaErrorKind::Unsupported`.
- An API test lists the release files for `lyrics`. The list holds only the lyrics files.
- An API test lists a local test host for `lyrics`. The list holds only the lyrics files.
- An API test uploads a `.vtt` file. The facts give `"kind": "lyrics"` and `text/vtt`.
- A Node test: `fieldKind('lyrics')` gives `lyrics`.
- A Node test: a drop of a `.srt` file on a lyrics field is accepted, and a drop of a `.png` file gives `wrong_kind`.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a connector test or a folder scan test changes because of the new media types.
- Stop and report when the upload route needs a change to `uploads.rs` for a lyrics file.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-041c-lyrics-kind.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- docs/adr/0062-a-file-picker-for-each-file-field.md
- The files that the Files section names.

Goal:
- Add the lyrics kind to the core, the upload and the picker routes.
- Add the lyrics kind to `picker.js`.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. One list of lyrics extensions in the core, and one in `picker.js`.
- The import sheet of the album does not accept a lyrics file.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The connectors, the write pipeline and the track page.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
