# Studio v1 Task 019c: The Upload Route Gives Two Files

Parent: [task 019](studio-v1-task-019-lossless-import.md). Decision: [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) items 1 to 3 and 7.
Sequence: after [task 019a](studio-v1-task-019a-alternate-enclosure-model.md) and [task 019b](studio-v1-task-019b-mp3-encoder.md).

## Goal

An upload of a WAV file or a FLAC file adds two pending files, and gives the facts of the two files to the UI.

## Files

Inspect: `crates/api/src/routes.rs` (`upload_file`, `import_upload`, `delete_pending_file`), `crates/api/src/tests.rs`, `crates/core/src/media.rs` (`Imported`, `ImportedFile`), `crates/core/src/pipeline.rs` (`release_path`, `referenced_files`, `remove_pending_file`).

Change: `routes.rs`, `error.rs` only if necessary, `tests.rs`.

Do not change: `crates/core`, the UI.

## Architecture

These choices are fixed. Do not change them.

- The route calls `media::import` in `blocking`, before it locks the workspace. An MP3 encode takes some seconds, and other requests must not wait for it. The steps that use the collection and the store then lock the workspace.
- For an `Imported` with `lossless`, the route checks the two release paths before it adds a file. A path that the draft uses gives 409 `file_pending`. A file on the host gives 409 `file_exists`. A left-over pending file is replaced, as before. The route adds no file when one path fails.
- The route adds the MP3 file, then the FLAC file, with `store.put_file` and `add_draft_file`. When the second add fails, the route removes the first.
- The result keeps its fields, which give the MP3 file. It gets `lossless`: `null`, or `{ path, url, size, media_type, file_name }` of the FLAC file.
- `DELETE /feeds/{id}/files?path=` does not change. The UI calls it for each file.
- Each temporary file is removed after each result, also after an error.

## Steps

1. Move `media::import` before the lock.
2. Add the second file and the `lossless` result.
3. Add the tests.

## Constraints

- Each route needs the access token (ADR 0017).
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- An upload of the WAV test file gives 201, a `.mp3` path with `audio/mpeg`, `converted_from_wav: true`, and `lossless` with a `.flac` path and `audio/flac`. The two files are pending files.
- An upload of the FLAC test file gives an MP3 path and `lossless` with the FLAC path.
- An upload of the MP3 test file gives `lossless: null`.
- When the FLAC path of a WAV upload is a file on the host, the upload gives 409 `file_exists`, and no pending file is added.
- After each request, `<store root>/incoming/` is empty.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the route needs a change of `crates/core`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-019c-upload-two-files.md
- docs/adr/0033-a-lossless-import-gives-an-mp3-enclosure.md
- The files that the Files section names.

Goal:
- An upload of a WAV file or a FLAC file adds two pending files, and gives the facts of the two files.

Constraints:
- Follow the Architecture section of the task file fully.
- The encode occurs with no lock of the workspace. No temporary file stays.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, the UI.
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
