# ADR 0180: The Files Page Has A Folder Picker And One Step For Each Copy

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaires of the human checks. The operator selected the names, the row with only the path, and the sheet of **Change…** with **Choose Folder…** and **Default**. The operator also selected **Use This Copy**, no **Move to {host}**, and no select menu "File host of this file".
- Technical choices: items 6, 8, 9, 15 and 17 are choices of the planning model.
- Changes: [ADR 0177](0177-the-files-page-has-one-way-to-move.md) items 1, 3, 6 and 10. [ADR 0167](0167-each-release-has-an-editable-library-folder.md) items 6, 7 and 10. [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 7 and 8. [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) items 8 and 12. [ADR 0037](0037-button-roles.md), the examples of the action role. [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) item 3.
- Uses: [ADR 0167](0167-each-release-has-an-editable-library-folder.md) items 1 and 2 for the standard path. [ADR 0105](0105-a-public-server-holds-no-assets.md) for a server with no assets.
- Source: findings 1, 3 and 5 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, the operator found these defects on the **Files** page of a release:

- The row name "Release files" is too general.
- The row "Release folder" had a second control, "Move to Standard Folder", and the line "Not in the standard folder".
- The sheet of **Change…** had a text field for the new path and a select menu of the library folders. A text field is not a good control for a folder.
- The section "Moves that wait" shows after the move is complete. Only the deletion of the copies from before waits.
- The operator did not know what the button **Reference** does. It puts the address of that copy in the feed.
- One file had three controls that change the copy that the feed uses: **Reference**, **Move to {host}** and the select menu "File host of this file". ADR 0177 gave the release one move control.

The select menu "File host of this file" sets the host of one file (`host_of` in `crates/core/src/inventory.rs`). Studio uses it only when it plans the address of a file with no copy on that host. A change does not move a file that the feed uses, so the user sees no result.

## Decision

### Names

1. The row "Release files" of the section "File hosts" has the name "Media files".
2. The select menu "What moves" has the values "Feed and media files", "Feed file only" and "Media files only". It starts at "Feed and media files".
3. The section "Moves that wait" has the name "Old files to delete". Its rows do not change.

### The Release Folder

4. The section "On this computer" has one row, "Release folder". It shows the path of the release folder and **Change…**. It has no line "Not in the standard folder" and no "Move to Standard Folder".
5. **Change…** opens a sheet with the current path, **Choose Folder…** and **Default**. **Default** selects the standard path of ADR 0167 item 1.
6. The sheet shows the selected path, and **Move the Folder** moves the release folder to it. **Cancel** closes the sheet with no change.
7. In the desktop app, **Choose Folder…** opens the folder dialog of the operating system, at the library folder.
8. The local router of the desktop app has the route `POST /api/v1/choose-folder`, with the access rule of ADR 0017. It opens the folder dialog and gives the selected path, or no path when the user cancels. The page has no other control that opens a dialog. The routers of `studio-desktop` and `studio-server` do not have this route. ADR 0124 item 2 stays correct: the window exposes no Tauri command.
9. `studio-desktop` in a browser and a server that holds assets show a folder browser in the sheet. It shows the folders of the library folder, one level at a time, and it has **New Folder**.
10. The core refuses a folder that is not in the library folder, with the cause (ADR 0167 item 6). The folder dialog of the system can select a folder that is not in the library folder, so the check stays in the core.
11. The sheet has no text field for the path and no select menu "Folders in the library".
12. A server that holds no assets has no section "On this computer" (ADR 0177 item 8).

### The Copies Of One File

13. The copy that the feed uses shows "Used by the feed". It has no button.
14. Each other copy has **Use This Copy** and **Delete**. **Use This Copy** is the operation "Reference" of ADR 0036 item 12. It replaces the current address in the feed with the address of that copy.
15. A move of one file is two steps: **Copy to {host}**, then **Use This Copy** on the new copy. The row of a pending copy also has **Use This Copy**, so the two steps can go in one draft and one write.
16. A file row has no **Move to {host}**. After **Use This Copy**, the copy from before stays on its host with **Delete**. It is not in "Old files to delete".
17. A file row has no select menu "File host of this file". The host of each library file is the default host of its release. A migration clears the host of each file that had its own host. A file that the feed uses on a different host keeps its copy and its address.

## Alternatives Considered

- Keep the text field for the path. Not chosen by the operator: a folder picker is the usual control, and it makes the select menu "Folders in the library" not necessary.
- Keep "Move to Standard Folder" on the row. Not chosen by the operator: the row then has two controls.
- Show the select menu "File host of this file" only on a new file, with the name "Send to". Not chosen by the operator: most users do not use it, and **Copy to {host}** with **Use This Copy** does the same work.
- Keep **Move to {host}** on the copy that the feed uses. Not chosen by the operator: a file then has a second move control.

## Consequences

- A user who wants one file on a different host makes two steps. Review shows each step.
- **Default** puts the standard path in the sheet. The user then sees the path before the move.
- The Files page has no entry point for the operation "Move" of ADR 0036 item 12 on one file. When no other entry point uses it, the task deletes it.
- A move of the media files of a release (ADR 0177 item 4) also moves a file that had its own host.

## Acceptance Criteria

Mechanical:

- A Node test: the UI model of the Files page gives the row name "Media files" and the three values of item 2.
- A Node test: the UI model of the release folder row gives only the path and **Change…**, also when the folder is not at the standard path.
- A Node test: the UI model gives no button for the copy that the feed uses. It gives **Use This Copy** and **Delete** for each other copy, and no **Move to {host}**.
- A Node test: the UI model of a file row gives no host select menu.
- An API test: the router of `studio-desktop` and the router of `studio-server` give 404 for `POST /api/v1/choose-folder`.
- A Rust test: the migration clears the host of each library file, and a file that the feed uses on a different host keeps its address.
- The Rust test of ADR 0167 that refuses a folder that is not in the library stays.

Visual, by the planning model:

- The Files page and the sheet of **Change…** at 1280 px and at 390 px, in the light and dark themes. Show the folder browser of item 9, and a file with two copies.

Human, by the operator. This gate stays open until it runs:

- In the AppImage, **Choose Folder…** opens the folder dialog of the system. A folder in the library moves the release folder. A folder that is not in the library gives the cause.
