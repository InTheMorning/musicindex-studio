# Studio v1 Task 116: The Files Page Has A Folder Picker And One Step For Each Copy

Decision: [ADR 0180](../adr/0180-the-files-page-has-a-folder-picker-and-one-step-for-each-copy.md).

Status: In progress. Packets 116a to 116d are complete on 2026-11-12, with their screenshot visual checks. The screenshots of 116c show a written copy and a pending copy with **Use This Copy** and **Delete**. They do not show a copy that the feed uses. Packet 116d is complete: the core has no host of one file, and `LibraryFile` has no `connector_id`. In 116b, the browser and the server show a list of folder rows. The desktop app shows **Choose Folder…**. The human test of the folder dialog of the system is open. The row "Media files", the three values of "What moves", the heading "Old files to delete" and the release folder row with only the path and **Change…** show.

## Goal

The **Files** page uses clear names. The release folder row has one control, and its sheet selects a folder with a picker. Each copy of a file has one step to put it in the feed. A file row has no move of one file and no host select menu.

## Constraints

- No feed rule in the UI (ADR 0065). The core refuses a folder that is not in the library folder (ADR 0180 item 10).
- The window exposes no Tauri command (ADR 0124 item 2). Only the local router of the desktop app has `POST /api/v1/choose-folder`.
- A server that holds no assets has no section "On this computer" (ADR 0105, ADR 0177 item 8).
- No test sends a request outside the computer (ADR 0045). No test opens a dialog of the system.
- Task 120 also changes the Files page, in `editor.js`. Do task 116 first.

## Files To Read First

- ADR 0180, ADR 0177, ADR 0167 items 1, 2 and 6, and ADR 0036 item 12.
- `crates/api/ui/releasemove.js` and `crates/api/ui-tests/releasemove.test.mjs`: the row "Release files" and the select menu "What moves".
- `crates/api/ui/feedfacts.js`: the model of the release folder row.
- `crates/api/ui/editor.js`: `CopyRow`, the file rows with "File host of this file", the sheet of **Change…** with "Folders in the library", and the section "Moves that wait".
- `host_of` in `crates/core/src/inventory.rs`, and `crates/core/src/migrate.rs`.
- `library_folders` in `crates/api/src/routes.rs`.
- `local_router_with_drops` in `crates/api/src/lib.rs`, and `crates/desktop-app/src/main.rs`. The app uses `tauri-plugin-dialog` now.

## Packets

### 116a: The Names And The Folder Row

ADR 0180 items 1 to 4.

- Change "Release files" to "Media files".
- Give the select menu "What moves" the values "Feed and media files", "Feed file only" and "Media files only". It starts at "Feed and media files".
- Change the heading "Moves that wait" to "Old files to delete".
- The release folder row gives only the path and **Change…**. Remove the line "Not in the standard folder" and "Move to Standard Folder".
- Add the Node tests of the ADR for these items.

### 116b: The Sheet Of Change…

ADR 0180 items 5 to 12.

- The sheet shows the current path, **Choose Folder…** and **Default**. **Default** puts the standard path of ADR 0167 item 1 in the sheet. The core gives that path.
- The sheet shows the selected path. **Move the Folder** moves the release folder there. **Cancel** closes the sheet.
- Remove the text field of the path and the select menu "Folders in the library".
- In the desktop app, add `POST /api/v1/choose-folder` through the same merge as the drop routes. Use the access rule of ADR 0017. The route opens the folder dialog at the library folder. It gives the selected path, or no path after a cancel.
- In `studio-desktop` in a browser and on a server that holds assets, the sheet shows a folder browser. It shows the folders of the library folder, one level at a time. It has **New Folder**. Give `library_folders` a parent path, and add a route that makes a folder in the library. The core refuses a path that is not in the library folder.
- The UI calls `choose-folder` only when the shell is the desktop app. Use the current shell flag of the UI for the app.
- Add the API test of the ADR: the routers of `studio-desktop` and `studio-server` give 404 for the route.

### 116c: The Copies Of One File

ADR 0180 items 13 to 17.

- The copy that the feed uses shows "Used by the feed", with no button.
- Each other copy, and the row of a pending copy, has **Use This Copy** and **Delete**. **Use This Copy** is the operation "Reference" of ADR 0036 item 12.
- Remove **Move to {host}** and the select menu "File host of this file".
- Add a migration that clears the host of each library file. Do not change the address of a file that the feed uses on a different host.
- Find each other entry point of the operation "Move" of one file. When there is none, delete the operation, its route and its tests.
- Add the Node tests and the Rust test of the ADR.

### 116d: Remove The Host Of One File

ADR 0180 item 17. After 116c, no route sets the host of one library file.

- Delete `Collection::set_library_file_connector`. `host_of` in `crates/core/src/inventory.rs` gives the files host of the release, then the feed host.
- Remove `own_connector_id` from the files route and from `FileEntry` in `editor.js`.
- Delete or change the tests that set the host of one file: in `inventory.rs`, in `collection.rs` and in `crates/core/tests/pipeline.rs`.
- Keep the column `library_files.connector_id`. A migration that removes it is not necessary. Remove the field from `LibraryFile`.

## Acceptance Criteria

Mechanical: the criteria of ADR 0180. The Rust test of ADR 0167 that refuses a folder outside the library stays.

Visual, by the planning model:

- The Files page and the sheet of **Change…** at 1280 px and at 390 px, in the light and dark themes. Show the folder browser and a file with two copies.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0180, in the AppImage.

## Stop And Ask When

- A different entry point uses the operation "Move" of one file.
- The folder dialog cannot open from the local router without a Tauri command.
- The migration must change a draft or an address in a feed.
