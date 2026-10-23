# Studio v1 Task 018c: The Upload Route

Parent: [task 018](studio-v1-task-018-import-music-files.md). Decisions: [ADR 0026](../adr/0026-import-music-files-through-the-draft.md) and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md).
Sequence: after [task 018a](studio-v1-task-018a-pending-files.md) and [task 018b](studio-v1-task-018b-tags-and-conversion.md).

## Goal

Let the UI send one file to the draft of a feed, and get its facts and its address.

## Files

Inspect: `crates/api/src/lib.rs`, `crates/api/src/routes.rs` (`blocking`, `media_facts`), `crates/api/src/error.rs`, `crates/api/src/workspace.rs` (`store`), `crates/api/src/tests.rs`, `crates/core/src/imports.rs`, `crates/core/src/media.rs`, `crates/core/src/pipeline.rs` (`release_path`), `crates/core/src/collection.rs` (`add_draft_file`).

Change: `lib.rs`, `routes.rs`, `error.rs`, `workspace.rs` only if necessary, `crates/api/Cargo.toml` and `tests.rs`.

Do not change: `crates/core`, the UI.

## Architecture

These choices are fixed. Do not change them.

- Route `POST /api/v1/feeds/{id}/files?name=<file name>`. The body is the bytes of the file. The route has `DefaultBodyLimit::max(2 GiB)`. The other routes keep the default limit.
- The route keeps only the last part of `name`, after the last `/` or `\\`. An empty name, `.` and `..` give 422 `invalid_name`. A file goes only into the folder of its release.
- The handler writes the body to a temporary file in `<store root>/incoming/`, chunk by chunk, with no lock of the workspace. It then does these steps in `blocking`:
  1. The feed must have a connector and a path. The connector must give a public address for the release path, or 422 `no_public_address`.
  2. `media::import(temp, name)`. `Unsupported` gives 415 `unsupported_media`.
  3. `path = release_path(feed path, imported.file_name)`. A path that the draft has gives 409 `file_pending`. A file on the host at that path gives 409 `file_exists`.
  4. `store.put_file`, then `add_draft_file`.
  5. Remove the temporary files.
- The result is 201 with `{ path, url, size, media_type, kind, title, track_number, duration, converted_from_wav, file_name }`. `kind` is `"audio"` or `"image"`. `url` is the public address of the file.
- The route does not change the model of the draft. The UI adds the tracks and the images.
- Route `DELETE /api/v1/feeds/{id}/files?path=<path>` removes a pending file and its copy. It gives 204, or 404.

## Steps

1. Add the routes and the error mapping.
2. Add the tests with the test files of task 018b.

## Constraints

- Each route needs the access token (ADR 0017).
- A temporary file is removed after each result, also after an error.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- An upload of the tagged MP3 test file gives 201 and an address in the release folder. It also gives the title, the track number, the duration, the size and `audio/mpeg`. The diff then lists no file, because the draft does not refer to it.
- An upload of the WAV test file gives a `.flac` path, `audio/flac` and `converted_from_wav: true`.
- An upload of a text file gives 415. A second upload with the same name gives 409 `file_pending`.
- An upload to a feed in a folder with no public address gives 422.
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

Stop and report when the route needs a change of `crates/core`. Stop and report when the body cannot go to a file with no lock of the workspace.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-018c-upload-route.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- The files that the Files section names.

Goal:
- The UI sends one file to the draft of a feed, and gets its facts and its address.

Constraints:
- Follow the Architecture section of the task file fully.
- Each route needs the token. No temporary file stays.
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
