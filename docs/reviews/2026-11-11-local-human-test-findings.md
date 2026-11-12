# Human Test Findings Of Local Main On 2026-11-11

Date: 2026-11-11.

Status: advisory. This review records the results of the human tests of local `main` at `19371ce`. It states no rule. The ADRs that it names own the decisions.

The operator built `studio-desktop-app` in the release profile from local `main` and ran it on Linux. The data was the real collection of the operator.

## Results

| Check | Owner | Result |
|---|---|---|
| Build and start | — | Passed. The window opened on Releases. |
| Move with "Only the release files" | ADR 0177 | Passed. The media files went from `bastone` to `heycitizen`, at `https://music.cancelbandcamp.com/TestAlbum/test/`. Each new address answered `200`. The feed file stayed on `bastone` at its address. |

## Findings

### Finding 1: The Files Page Of A Release

Owner: [ADR 0177](../adr/0177-the-files-page-has-one-way-to-move.md) and [ADR 0167](../adr/0167-each-release-has-an-editable-library-folder.md).

The operator found three defects on the **Files** page:

1. The row name "Release files" is too general.
2. "Move to Standard Folder" is a second control on the row. The operator wants a **Default** button in the sheet of **Change…**.
3. The field "New folder path" is a text field. The operator wants a folder picker. A folder picker makes the select menu "Folders in the library" not necessary.

The operator selected these answers in the questionnaire:

- The name is "Media files". The values of "What moves" are "Feed and media files", "Feed file only" and "Media files only".
- On the desktop, **Change…** opens a sheet with **Choose Folder…** and **Default**. **Choose Folder…** opens the folder dialog of the operating system. A server with assets uses a folder browser in the page.
- The row "Release folder" shows only the path and **Change…**. It does not show the line "Not in the standard folder".

The planning model keeps ADR 0167 item 6: the core refuses a folder that is not in the library, with its cause. A new ADR must record these answers. It changes ADR 0124 item 2, because the page then asks the desktop shell for a dialog.

### Finding 2: The Line Of The Feed Address

Owner: [ADR 0128](../adr/0128-a-moved-feed-names-its-own-address.md).

After a move of only the media files, the **Files** page showed "The feed file names the new feed address `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`." That address is the current address, and it did not change. The word "new" tells the user that the address changed.

### Finding 3: The Name "Moves That Wait"

Owner: [ADR 0148](../adr/0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) item 8, and [ADR 0177](../adr/0177-the-files-page-has-one-way-to-move.md) item 10.

The move is complete when the section shows. Only the deletion of the old copies waits. The operator did not know what the name "Moves that wait" means.

The operator selected the name "Old files to delete" in the questionnaire.

### Finding 4: A Move Refuses An Old Copy Of The Same Feed

Owner: [ADR 0126](../adr/0126-studio-refuses-a-bad-move-link.md) items 19 and 21.

The operator moved only the feed file of "HeyCitizen's Test Album" from `bastone` to `heycitizen`. The move sheet showed "Something went wrong." and "a file exists at TestAlbum/test.xml on the new file host already".

The agent read the file at `https://music.cancelbandcamp.com/TestAlbum/test.xml`. It has the `podcast:guid` `7ea30e6c-c7ba-5b81-a7be-6465c1591b7a` of the feed. Its `itunes:new-feed-url` gives the current address `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`. Its `lastBuildDate` is 2026-11-06. Thus the file is an old copy of the same feed from an earlier move.

The core accepts such a file only when the collection has a record of it as a placed file or a pointer file. This collection has no record. The cause of the missing record is not known. The collection came from an import of an account file (ADR 0176), and the import can be the cause.

The sheet also kept **Add to the Draft** available after the refusal.
