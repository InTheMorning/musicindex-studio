# Studio v1 Task 035e: Uploads In Parts

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md) items 1 to 7, [ADR 0026](../adr/0026-import-music-files-through-the-draft.md), [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) and [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) item 21.
Sequence: the fifth packet of task 035. It depends on task 034.

## Goal

The UI sends each music file in parts of 8 MiB, in both shells. After a failed part or a closed page, the upload continues from the first missing part. The full upload route goes away.

## Files

Inspect: `crates/api/src/routes.rs` (`upload_file`, `UploadQuery`, `TempFile`, `temp_file_name`, the import after the body), `crates/api/src/lib.rs` (`MAX_UPLOAD_BYTES`, the files route), `crates/api/src/workspace.rs` (`incoming_dir`, the server rules of 034d), `crates/api/ui/app.js` and `crates/api/ui/editor.js` (each call of `/files` with a `File` body), `crates/api/src/tests.rs` (the upload tests).

Change: `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/workspace.rs`, the new file `crates/api/src/uploads.rs`, `crates/api/src/tests.rs`, the new model module `crates/api/ui/upload.js`, the new test file `crates/api/ui-tests/upload.test.mjs`, `crates/api/ui/app.js`, `crates/api/ui/editor.js`.

Do not change: the import itself, the pending files, the list route and the remove route of `/feeds/{id}/files`.

## Architecture

These choices are fixed. Do not change them.

### The API

- `uploads.rs` keeps the uploads in the collection with a new migration: `uploads (id TEXT PRIMARY KEY, feed_id INTEGER NOT NULL, name TEXT NOT NULL, size INTEGER NOT NULL, replace TEXT, received TEXT NOT NULL, updated_at INTEGER NOT NULL)`. `received` is a JSON list of part numbers. The ID is 16 random bytes as hexadecimal.
- The routes are those of ADR 0059 item 1. The part size is 8 MiB. The body limit of the part route is the part size.
- `POST /feeds/{id}/uploads` checks the name with `upload_file_name`. It refuses a size over 2 GiB with 413 `too_large`. It checks the storage limit of the server rules for the full size. It makes the file `incoming/<upload>.part` with the full size.
- `PUT /uploads/{upload}/parts/{n}` refuses a part number outside the file, and a body that is not the size of that part, with 422 `bad_part`. It writes the bytes at `n × part size` and records the part. It does not hold the lock of the workspace during the write.
- `GET /uploads/{upload}` gives `{"upload", "feed", "name", "size", "part_size", "received"}`. `GET /feeds/{id}/uploads` lists the open uploads of a feed.
- `POST /uploads/{upload}/finish` refuses a missing part with 409 `upload_incomplete`. Then it runs the import of the full upload route on the file, and gives the same response. It removes the upload row.
- `DELETE /uploads/{upload}` removes the row and the file.
- At the start and each hour, Studio removes each upload with no new part for 24 hours.
- The route `POST /feeds/{id}/files` and `MAX_UPLOAD_BYTES` go away.

### The UI

- `upload.js` has no Preact and no browser state. It has `partCount(size, partSize)`, `missingParts(size, partSize, received)`, `partRange(n, size, partSize)` and `findUpload(uploads, name, size)`.
- The import sends each file with these steps: find an open upload of the feed with the same name and size, or start one. Send each missing part with `File.slice`. Try a failed part again 3 times, after 1, 2 and 4 seconds. Then finish.
- The progress of a file is the sum of the received parts.

## Steps

1. Add the migration, `uploads.rs` and the routes, with API tests.
2. Move the import code of `upload_file` into a function that `finish` calls, and remove the full upload route.
3. Add `upload.js` with its Node tests.
4. Change each upload of the UI to parts.
5. Do the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No logic is in two places: the import of a finished upload is the import of the earlier route.

## Acceptance Criteria

Mechanical:

- An API test sends a file of 20 MiB in parts in the sequence 2, 0, 1, sends part 1 two times, and finishes. The imported file has the SHA-256 of the source.
- An API test refuses `finish` with a missing part with 409. It refuses a part of the wrong size and a part number outside the file with 422.
- An API test with a storage limit refuses a start over the limit with 507.
- An API test confirms that an upload older than 24 hours has no row and no file after the clean step.
- The existing import tests pass through the new routes.
- The Node tests examine the four functions of `upload.js`.
- The four commit checks pass.

Visual:

- The planning model checks the import sheet during an upload in parts, and after a part that failed one time. Each one in the dark theme, in the light theme and at 390 px.

Human test, in task 037:

- A large upload from a phone on a mobile network.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the import code of `upload_file` cannot move into one function without a change of its behavior.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035e-uploads-in-parts.md
- docs/adr/0059-uploads-in-parts-and-the-home-screen-app.md
- The files that the Files section names.

Goal:
- Add the upload routes in parts, with the import of the earlier route at `finish`.
- Remove the full upload route.
- Add the model module `upload.js`, and send each file of the UI in parts.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No logic in two places.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The import itself and the pending file routes.
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
