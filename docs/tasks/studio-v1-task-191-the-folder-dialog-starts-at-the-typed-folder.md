# Studio v1 Task 191: The Folder Dialog Starts At The Typed Folder

Decision: [ADR 0226](../adr/0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) items 1 to 4.

Status: Open.

## Problem

Finding 1 of the [rc.27 human check](../reviews/2026-11-19-rc27-human-check.md). "Choose Folder…" opens the dialog at the home folder, also when the field has a typed folder.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Folder dialog route | extend | crates/api/src/routes.rs | `choose_folder` near line 4709 selects the start folder: the library folder, else the home folder. |
| Folder picker | reuse | crates/api/src/lib.rs | `FolderPicker` near line 71 takes the start folder. The desktop app gives it. |
| First-run library sheet | extend | crates/api/ui/app.js | `LibrarySheet` near line 1786 calls the route with no body. |
| Files folder sheet | extend | crates/api/ui/editor.js | The folder sheet near line 4574 calls the route with no body. |
| Route test | extend | crates/api/src/tests.rs | `the_folder_dialog_starts_at_the_home_folder_when_no_library_is_set` near line 207 records the start folder. |

## Changes

1. `choose_folder` takes an optional JSON body `{ "start": string }`. A request with no body stays valid.
2. When `start` is an absolute path, the route walks up from it to the first folder that exists, and gives that folder to the picker.
3. In each other case, the route keeps its current start: the library folder, else the home folder.
4. `LibrarySheet` and the Files folder sheet send the text of their field as `start`.

## Checks

Mechanical:

- API tests with a recording picker: a typed folder that exists, a typed folder that does not exist under an existing parent, a relative path, an empty text, and no body.
- `cargo test -p studio-api` and `bash tools/gate.sh` pass.

Human: the operator types a folder in the first-run sheet of the desktop app and selects "Choose Folder…". The dialog of the system shows that folder.
