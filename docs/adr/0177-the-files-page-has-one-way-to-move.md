# ADR 0177: The Files Page Has One Way To Move

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Accepted: 2026-11-11 by the operator, from the drawings of two questionnaires.
- Technical choices: items 3, 8 and 10 are choices of the planning model.
- Changes: [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) item 1. [ADR 0167](0167-each-release-has-an-editable-library-folder.md) items 5, 7, 10 and 12.
- Uses: [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 8 for the default file host of a release.
- Source: the human checks of 2026-11-11.
- Changed by: [ADR 0180](0180-the-files-page-has-a-folder-picker-and-one-step-for-each-copy.md). Items 1, 3, 6 and 10: the names "Media files" and "Old files to delete", the values of "What moves", and a folder row with only the path.

## Context

On 2026-11-11, the Files page of a release had two sections. "Files" had the scans, "Move This Release…" and two library rows. The rows were "Library folder" and "Default folder". "File hosts" had two select menus. A change of the select "File host of the feed XML" also moved the feed. Thus the page had two ways to move, and the operator could not see which control did what. The two library rows also did not say why there were two.

## Decision

### File Hosts

1. The Files page starts with the section "File hosts". The row "Feed file" gives the name of the file host and the feed address. The row "Release files" gives the name of the default file host of the release. These rows show text, and they have no select menu.
2. The section has "Move This Release…" and "Scan the File Hosts". "Move This Release…" is the only control on the page that moves the feed file or the release files.
3. The move sheet has the select menu "What moves", before the plan, with three values: "The feed file and the release files", "Only the feed file" and "Only the release files". It starts at the first value. The plan and the deletion check box follow the value.
4. A move of the release files also changes the default file host of the release (ADR 0036 item 8).

### On This Computer

5. Below "File hosts", the section "On this computer" has one row with the path of the release folder and "Change…".
6. The path can differ from the standard path of ADR 0167 item 1. Then the same row has the line "Not in the standard folder:", with the standard path in the library, and "Move to Standard Folder". The page shows no second row.
7. The section also has "Scan the Library Folder".
8. A server that holds no assets has no section "On this computer" (ADR 0105).

### Layout

9. At a narrow width, the buttons of a row go below the text of the row. A button does not cover text.
10. The section "Moves that wait" stays below "On this computer".

## Alternatives Considered

- Keep the two select menus, and put "Move This Release…" in the same section. Not chosen by the operator: the page continues to have two ways to move.
- Two library rows with clearer names. Not chosen by the operator: the second row gives only one fact.

## Consequences

- A user who wants to change only the host of new files moves the release files. A release with no file on the previous host moves nothing, and only the default changes.

## Acceptance Criteria

Mechanical:

- A Node test: the UI model of the Files page gives no select menu for a file host, and gives the values of item 3.
- A Node test: the library row gives the standard path only when the folder is not at that path.
- An API test: a move with "Only the feed file" moves no release file, and a move with "Only the release files" does not move the feed file.

Visual, by the planning model:

- The Files page at 1280 px and at 390 px, in the light and dark themes. Show the folder at the standard path and away from it. At 390 px, no button covers text.

Human, by the operator. This gate stays open until it runs:

- Move a test release with each value of "What moves".
