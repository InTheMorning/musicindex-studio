# Studio v1 Task 070: A Server That Holds No Assets

Decision: [ADR 0105](../adr/0105-a-public-server-holds-no-assets.md). It uses ADR 0007, ADR 0033 and ADR 0036 items 6, 7 and 15. It also uses ADR 0040, ADR 0059 item 4, ADR 0062 item 29 and ADR 0071.

Status: In progress. Packets 070a and 070b are complete.

## Goal

With `--hold-assets off`, the server keeps an audio file or an image only until the write that sends it, for a maximum of 7 days. A user can save each file to a device.

## Context

- The mockups are in `.studio-dev/mockup-105/site/` (`assets.js`). Git ignores that folder. They have seven screens. Four show the Files page: each state, a written file, a file on this server only and a file after the expiry. The other three are the review sheet, the sheet "Add tracks" and Settings. Use their texts.
- `ServerRules` in `crates/api/src/workspace.rs` gives the storage limit, the user folder and the library folder of a server workspace. The desktop has no `ServerRules`.
- The audio route (`crates/api/src/media.rs`) reads a byte range from the release folder or from the host, and keeps no full file. The route of item 15 of ADR 0105 uses the same method.
- On 2026-11-01, the production volume held 3.0 MiB, with no audio. The removal at the first start (ADR 0105 item 13) has almost no files to remove.

## Packets

### 070a: The option and the core rules

1. `studio-server serve` gets `--hold-assets off|on`, with the default `off`. `ServerRules` gets `hold_assets: bool`. The desktop has no option, and it holds assets.
2. `deploy/compose.yml` passes `--hold-assets ${STUDIO_HOLD_ASSETS:-off}`. `deploy/env.example` explains the two values. `docs/runbooks/studio-server.md` section 4 lists the option.
3. `GET /api/v1/status` gives `hold_assets` (true on the desktop), so that the UI knows the mode.
4. The core gets these functions. Each one does nothing when the workspace holds assets:
   - After the read of each new address (ADR 0036 item 15), remove the local bytes of each asset that the write sent. The row gets `local` false and keeps its SHA-256, its size and the address of its copy as `source_url` (ADR 0105 item 5).
   - At the start and each hour, remove the bytes of each pending file that is older than 7 days from its import. Record the time of the removal on the row (ADR 0105 items 8, 10 and 11).
   - At the start, remove the local bytes of each asset that a host has with the same SHA-256, from the inventory (ADR 0105 item 13).
5. A migration adds `imported_at` and `removed_at` to `library_files`. An import, an upload `finish` and a replacement set `imported_at`. A row from before the migration gets the time of the migration.
6. `pipeline::write_draft` refuses a draft that refers to a removed pending file, with 409 `file_missing`. The error names the path of each file.
7. With `off`, a copy, a move and a lossless conversion use a temporary file in `incoming/`. Studio removes it after the operation, also after a failure (ADR 0105 items 6 and 7).

### 070b: The routes

1. With `off`, the download routes of ADR 0038 and the folder scan give 403 `not_on_server`.
2. A new route gives the bytes of one library file for a media ticket, with `Content-Disposition: attachment` and the file name. It reads the local bytes, or the copy on the host through its connector, and writes no file. A ticket for a file holds the file ID, not an address. Use the ticket store of ADR 0062 item 29.
3. "Add the File Again" uses the upload routes of ADR 0059 with `replace` for the path of the removed file. After `finish`, the row gets its bytes, a new `imported_at` and no `removed_at`.

### 070c: The UI

1. A model module gives the status line of a file from its row: "On its file host", "Waits for the write · Studio removes it on DATE", "On this server only" and "Removed from this server on DATE". A Node test examines each state. The date uses the local time of the browser (ADR 0093).
2. The Files page, with `hold_assets` false:
   - starts with the notice of the mockups
   - has no "Download All", no "Download" and no "Scan the Library Folder"
   - shows the error notice of the mockups for a file on this server only, and for a removed file with "Add the File Again"
   - shows the notice "1 file of this draft is missing." at the top when a file is removed.
3. Each file entry has "Download to This Device" on the server, in the two modes. The UI gets a ticket, and opens the route of 070b item 2 in a hidden link.
4. At 760 px or less, the status line of a file row is on a second line, below the name (ADR 0105 item 18).
5. The review sheet adds the notice "After the write, Studio removes its copies of these N files from this server. Keep your own copies." when the write sends an asset and `hold_assets` is false.
6. The sheet "Add tracks" adds the notice of the mockups when `hold_assets` is false.
7. Settings gets the section "Files on this server" on the server, in the tier `limits`. Add it to `pages.js`. It shows the rules of the mode and the space in use.

### 070d: The user guide

1. `docs/user-guide/release-files.md` explains the two modes, the expiry and "Download to This Device".
2. `docs/user-guide/start-on-desktop.md` explains how to keep a full local library with Studio desktop and "Download All".

## Files

Change: `crates/server/src/config.rs`, `crates/server/src/users.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/media.rs`, `crates/api/src/lib.rs`, `crates/core/src/collection.rs`, `crates/core/src/library.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/download.rs`, `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/pages.js`, a new UI model module and its Node test, `deploy/compose.yml`, `deploy/env.example`, `docs/runbooks/studio-server.md`, `docs/user-guide/release-files.md`, `docs/user-guide/start-on-desktop.md`.

## Constraints

- No feed rule in the UI (ADR 0065). The status line is a display value of the row.
- With `on`, and on the desktop, no behavior changes. The tests of ADR 0035, ADR 0036 and ADR 0038 pass with no change.
- No upload to a host before the approved write (ADR 0007).
- The removal of a local file needs a host copy with the same SHA-256, or the expiry of a pending file. No other condition removes the bytes of an asset.
- Each new UI text is in the case rules: title case on a button, sentence case elsewhere.

## Acceptance Criteria

Mechanical:

- The core and API tests of the acceptance criteria of ADR 0105.
- A core test: the migration gives each row from before it an `imported_at`.
- A core test: a pending file at 6 days keeps its bytes, and at 8 days it has none and has `removed_at`.
- An API test: "Add the File Again" gives the row its bytes and clears `removed_at`, and the write then succeeds.
- An API test: the route of 070b item 2 refuses a ticket of a different user and an expired ticket.
- A Node test: each status line of 070c item 1.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` against a test server with `--hold-assets off`, at 1100 px and at 390 px with touch, in the two themes:

- The seven screens of the mockups.
- `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human, on `studio.musicindex.org`: the human test of ADR 0105.
