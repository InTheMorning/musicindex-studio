# Studio v1 Task 039a: The File Sources In The API

Parent: [task 039](studio-v1-task-039-file-picker.md). Decisions: [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) items 3, 6 to 10 and 14 to 18, [ADR 0019](../adr/0019-select-a-feed-from-the-host.md) items 2 and 3, [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) item 1, [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) item 7, [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 1 and [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md) item 6.
Sequence: the first packet of task 039. It depends on task 038.

Status: Implemented - 2026-10-27. The four commit checks are Green. The visual check is open. The [review checklist](../reviews/2026-10-27-tasks-038-039-review-checklist.md) records the review.

## Goal

The API gives the picker two lists for one kind of file: the files of this release, and the file tree of each host. A replaced file that another feed of the collection uses gets no deletion.

## Files

Inspect: `crates/api/src/routes.rs` (`list_release_files`, `connector_feeds`, `delete_unused_copies`, `feed_address_list`, `is_in_use`, `remote_file_in_use`), `crates/api/src/lib.rs` (the routes), `crates/core/src/connector.rs` (`Connector::list`, `Connector::public_url`, `FileList`, `ListedFile`), `crates/core/src/discover.rs` (the limits of ADR 0019), `crates/core/src/inventory.rs` (`copies`), the code that gives a draft address with no library file its row (ADR 0038 item 1), `crates/api/src/tests.rs`.

Change: `crates/api/src/routes.rs` or a new file `crates/api/src/picker.rs`, `crates/api/src/lib.rs`, `crates/core/src/media.rs` or the module that owns the formats of ADR 0032 item 1, `crates/api/src/tests.rs`.

Do not change: the connectors, the upload routes, the write pipeline.

## Architecture

These choices are fixed. Do not change them.

### The Kinds

- The core has one function that gives the kind of a file from its path: `image`, `audio` or none. It uses the extensions of the formats of ADR 0032 item 1: `.jpg`, `.jpeg`, `.png`, `.webp`, `.gif` for images, and `.mp3`, `.m4a`, `.flac`, `.ogg`, `.opus` for audio. The comparison ignores case.
- When a function for these formats exists, extend it. Do not add a second list.

### This Release

- `GET /feeds/{id}/picker/release?kind=image|audio` gives `{"files": [...]}`. Each file has `name`, `address` (the public address, or `null`), `local` (true when the library holds the bytes), `size`, `media_type`, `duration` (always `null`, because the library records no duration) and `in_use` (true when the draft uses the address).
- The list holds the library files of this feed only, of the asked kind. It uses the same records as `list_release_files`. Do not read a host for this list.
- A file with no public address and no planned address is in the list with `address` `null`. The UI shows it, and the user cannot select it.

### Hosts

- `GET /connectors/{id}/picker?kind=image|audio` gives `{"files": [...], "truncated": bool}`. It calls `Connector::list` once, keeps the files of the asked kind, and gives for each file `path`, `size`, `media_type` and `address` from `Connector::public_url`. A file with no public address has `address` `null` and `reason` `no_public_address`.
- The route follows the vault rule of `connector_feeds`: a connector with a secret and a locked vault gives the error of `ApiError::vault_locked`.
- The route does not change a file on the host (ADR 0019 item 7).

### A Selection From A Host

- A selection is a draft change of the UI: the field gets the address. No route is new for it.
- The address gets its library row with `local` false after the next write. The code that records the addresses of a written feed today does this (ADR 0038 items 1 and 3). Do not add a row at a draft save. The diff makes no upload for an address with no SHA-256.

### The Deletion Rule

- `delete_unused_copies` adds no deletion for an address that another feed of the collection uses. Another feed uses an address when the address is in `feed_address_list` of its draft or of its live file.
- The rule applies to the addresses of images and of audio files. The UI sends the addresses from before in `delete_unused`, as the replacement of the audio does now.

### The Facts Of An Audio File

- No route is new. Both lists give `size` and `media_type`. The `media_type` comes from `remote::media_type_of_name`. Do not add a second map of media types. Task 039c reads the duration from the player metadata.

## Steps

1. Add the kind function with core tests.
2. Add the release list route with API tests.
3. Add the host list route with API tests on a local test host.
4. Confirm that the diff uploads no file for a host address, and that the write gives the address its library row, with an API test.
5. Change `delete_unused_copies`, with API tests.
6. Run the four checks.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No logic in two places: one kind function, one list of formats, and one deletion rule.

## Acceptance Criteria

Mechanical:

- A core test gives the kind of each extension of ADR 0032 item 1, in lower case and in upper case.
- A core test gives no kind for `.xml` and `.txt`.
- An API test lists the release files for `image` and for `audio`. Each list holds only the files of this release, and only the asked kind.
- An API test lists a local test host with images, audio files, an XML file and a hidden folder. The list holds only the asked kind, and no file of the hidden folder.
- An API test lists a host with more files than the limit of ADR 0019 item 3. `truncated` is true.
- An API test lists a host whose connector gives no public address. Each file has `address` `null` and `reason` `no_public_address`.
- An API test saves a draft with a host address in a track image. The diff lists no upload. After the write, the library has a row for that address with `local` false.
- An API test replaces a track image and sends `delete_unused` with the address from before. The draft gets one deletion.
- An API test gives the address from before to a second feed of the collection, then does the same replacement. The draft gets no deletion.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `Connector::list` of a connector cannot give a public address for each file.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-039a-file-sources.md
- docs/adr/0062-a-file-picker-for-each-file-field.md
- The files that the Files section names.

Goal:
- Add the release list and the host list of the picker, each for one kind of file.
- Keep a file that another feed uses from a deletion.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No logic in two places.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The connectors, the upload routes and the write pipeline.
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
