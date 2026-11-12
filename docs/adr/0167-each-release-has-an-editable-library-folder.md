# ADR 0167: Each Release Has An Editable Library Folder

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire of the rc.21 human test. The operator selected the local library folder, the layout by artist and label, a move only on request, and folders that stay after a rename.
- Technical choices: items 2, 3, 5, 6, 9 and 12 are choices of the planning model. The screenshots of the task show the Files row to the operator before the commit.
- Changed by: [ADR 0177](0177-the-files-page-has-one-way-to-move.md). Items 5, 7, 10 and 12: one row in the section "On this computer", with a line for the standard folder.
- Changes: [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 2 for a new release folder.
- Uses: [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md), [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) for the levels, [ADR 0149](0149-the-desktop-app-has-many-collections.md) for the library of a collection.

## Context

A release folder had the name of its feed file, directly in the library folder (ADR 0036 item 2). Many feeds have names such as `feed.xml` or `test.xml`, so the folders were `feed`, `feed 2` and `feed 3`. A label with many releases got many folders side by side. The user could not see or change the folder of a release.

## Decision

### The Default Layout

1. A new release folder gets a path from its levels (ADR 0088):
    - A release of an artist: `<library>/<artist>/<release title>/`.
    - A release of an artist on a label: `<library>/<label>/<artist>/<release title>/`.
    - A release with no artist: `<library>/Other/<release title>/`.
2. The core makes each part of the path from the title. It removes path separators and each character that the file system refuses, and it keeps the other characters. An empty part becomes "Untitled". When a folder of that name is in use, it adds ` 2`, ` 3` and so on, as ADR 0036 item 2 does.
3. A release with more than one artist or label uses the first artist and the first label of the left panel (ADR 0088).
4. The feed XML keeps its file name in the release folder.

### The Files Page

5. The **Files** page of a release has the row **Library folder**. It shows the full path of the release folder.
6. The row has **Change…**. It opens a sheet with the folders of the library folder and a field for a new folder name. Studio has no system folder picker for this, so the server and the desktop use one sheet. A path outside the library folder is refused, with the cause.
7. When the path is not the default path of item 1, the row also shows the default path and **Move to Default**.
8. A change of the folder moves the files of the release. The library records keep their files. A move that fails changes nothing.

### Existing Folders And Renames

9. An existing release folder stays where it is. No start of Studio moves it.
10. A rename of an artist or a label moves no folder. The row then shows **Move to Default** (item 7).
11. A move of the folder changes no feed and no host. It changes only the library on this computer.

### The Server

12. A server has no library folder for the user when it holds no assets (ADR 0105). There, the Files page has no **Library folder** row.

## How This Decision Is Checked

Mechanical:

- A Rust test gives the default path for an artist release, a label release and a release with no artist, and a name in use.
- A Rust test moves a release folder, and the library records and the files stay correct. A refused folder outside the library changes nothing.
- A Rust test renames an artist, and no folder moves.

Visual, by the agent: the **Library folder** row in the light and the dark theme, at the desktop width and at 390 px.

Human, by the operator: a move of a real release folder, and the files open in the release afterwards.
