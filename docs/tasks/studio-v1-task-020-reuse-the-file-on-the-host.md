# Studio v1 Task 020: An Import Reuses The Same File On The Host

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decision: [ADR 0034](../adr/0034-an-import-reuses-the-same-file-on-the-host.md).
Depends on: [task 019](studio-v1-task-019-lossless-import.md).

## Goal

An import of a file that is on the host with the same bytes uses the file on the host. Studio continues to refuse a different file at the path.

## Files

Inspect: `crates/core/src/pipeline.rs` (`release_path`, `referenced_files`), `crates/core/src/digest.rs`, `crates/core/src/connector.rs` (`read`, `FileRead`), `crates/core/tests/pipeline.rs`, `crates/api/src/routes.rs` (`import_upload`, `check_release_path`, `add_lossless_file`), `crates/api/src/tests.rs`, `crates/api/ui/editor.js` (`ImportSheet`, `ImportRow`, `fileFacts`, `removeFiles`).

Change: `pipeline.rs`, the pipeline tests, `routes.rs`, `tests.rs`, `editor.js`.

Do not change: the connector interface, `collection.rs`, `media.rs`, `values.js`, `draft.js`, `app.js`.

## Architecture

These choices are fixed. Do not change them.

- `pipeline::host_file_matches(connector: &dyn Connector, path: &str, local: &Path) -> Result<HostFile, PipelineError>` with `pub enum HostFile { Absent, Same, Different }`. It reads the file on the host with `connector.read`, and compares its SHA-256 with the SHA-256 of `local`.
- `check_release_path` in the route uses it. `Absent` keeps the existing steps. `Same` gives no error, and the route adds no pending file for that path. `Different` gives 409 `file_exists`.
- The route checks each path before it adds a file, as before. A left-over pending file at a path is replaced, as before.
- The result gets `on_host: true` or `false` for the main file, and for the `lossless` object.
- The import sheet shows "On the host already" after the type and the size of a file line with `on_host: true`. `removeFiles` does not call `DELETE /feeds/{id}/files` for such a file.
- The add of the track does not change. The addresses come from the result, as before.

## Steps

1. Add `HostFile` and `host_file_matches`, with tests.
2. Change the route and its tests.
3. Change the import sheet.

## Constraints

- Each route needs the access token (ADR 0017).
- After each request, `<store root>/incoming/` is empty.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A pipeline test: `host_file_matches` gives `Absent`, `Same` and `Different` for the three cases.
- An API test: after a write of a WAV track, a second upload of the same WAV file gives 201 with `on_host: true` for the MP3 file and the FLAC file, and adds no pending file.
- An API test: an MP3 upload with different bytes at its release path gives 409 `file_exists`.
- An API test: the FLAC file of a WAV upload is on the host with the same bytes, and its MP3 path is free. The upload gives `on_host: true` for the FLAC file and `on_host: false` for the MP3 file. Only the MP3 file is a pending file.
- The three commit checks pass.
- The agent runs `drive-ui.mjs`. It writes a release with the WAV test file, removes the track, and imports the WAV test file again. The row shows "On the host already" twice. After the add, the diff lists no file, and the console shows no error.

Visual:

- The operator imports the written FLAC file again, adds the track, and examines the diff.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the comparison needs a change of the connector interface. Stop and report when two imports of the same lossless file give MP3 files with different bytes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-020-reuse-the-file-on-the-host.md
- docs/adr/0034-an-import-reuses-the-same-file-on-the-host.md
- The files that the Files section names.

Goal:
- An import of a file that is on the host with the same bytes uses the file on the host. A different file at the path is still refused.

Constraints:
- Follow the Architecture section of the task file fully.
- Studio does not replace a file on the host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connector interface, collection.rs, media.rs, values.js, draft.js, app.js.
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
