# Human Test Findings Of Local Main On 2026-11-11

Date: 2026-11-11.

Status: advisory. This review records the results of the human tests of local `main` at `19371ce`. It states no rule. The ADRs that it names own the decisions.

The operator built `studio-desktop-app` in the release profile from local `main` and ran it on Linux. The data was the real collection of the operator.

## Results

| Check | Owner | Result |
|---|---|---|
| Build and start | — | Passed. The window opened on Releases. |
| Settings: Discard with the real lock and close settings | ADR 0166, task 099 | Passed. After a tab change, the two changes stayed. Discard gave back the saved values, and the close followed the saved value. |
| Settings: Save with the real lock and close settings | ADR 0166, task 099 | Passed. Save showed "Saved". The close followed the new value, the values stayed after a restart, and the vault locked after the lock time. |
| Move with "Only the release files" | ADR 0177 | Passed. The media files went from `bastone` to `heycitizen`, at `https://music.cancelbandcamp.com/TestAlbum/test/`. Each new address answered `200`. The feed file stayed on `bastone` at its address. |
| Move with "Only the feed file", on `7e66e56` | ADR 0177, ADR 0178 | Passed. The old copy at `TestAlbum/test.xml` on `heycitizen` did not stop the move. After the write at 11:37 UTC on 2026-11-12, the new address gave the feed. The old address gave the same feed, with `itunes:new-feed-url` set to the new address. Music Index still gave the old address after the write. |
| Move with "The feed file and the release files", back to `bastone` | ADR 0177, ADR 0179 | Stopped before the write. Review showed duplicate uploads (finding 9). |
| The same move again, on `7c581a8` | ADR 0177, ADR 0179 | Passed. The **Files** page showed no " 2" names. Review showed the return warning and no upload, because each media file had a copy on `bastone`. After the write at 13:47 UTC on 2026-11-12, both addresses gave the feed with `itunes:new-feed-url` set to the `bastone` address. Each media address in the feed was on `bastone` and answered. |
| Keys of the left panel and of the Credits page, with a real keyboard | ADR 0172, task 108 | Passed in the release build of local `main`, not in the AppImage. Up, Down, Home, End, Tab, Shift+Tab, Enter and Left acted as the task gives. Right, Left, Enter and Tab acted on a credit row. The focus ring had defects (finding 11). |
| A repair from the **Check** tab | ADR 0174, task 109 | Passed in the release build of local `main`, not in the AppImage. "HeyCitizen's Test Album" showed the notice "No self link". **Add the Link** put the Atom namespace, a new `lastBuildDate` and `<atom:link rel="self">` with the `bastone` address in Review. After the write at 14:03 UTC on 2026-11-12, the hosted feed had the link, and **Check Again** showed no problem. |
| Move of a real release folder | ADR 0167, task 105 | Passed. **Move to Standard Folder** moved `library/test` to `library/HeyCitizen/HeyCitizen's Test Album`. The new folder held the seven files with their sizes, and the old folder was not there. After **Scan the Library Folder**, the **Files** page showed the same six media files, with no new name. |
| Wallets of the linked page of Matt Finlay | ADR 0159, task 101 | Passed. The one matching wallet was `mattfinlay@phafe.com`, with the name "Matt Finlay". `podcastindex@getalby.com` and `chadf@getalby.com` were in the fold "2 other recipients on Matt Finlay", each with its recipient name. |
| Update from linked pages | ADR 0170, task 102 | Passed. The sheet listed the person "Matt" with "Picture", "Website" and the wallet `mattfinlay@phafe.com`, and not the other recipients of the page. After **Update**, the page of the person had the picture, the wallet as the default and the information address `https://phafe.com/mattfinlay`, with no **Save** bar and no suggestion. The rows of the sheet had a defect (finding 15). |

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

### Finding 5: The Controls Of One Copy Of A File

Owner: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) item 13, [ADR 0037](../adr/0037-button-roles.md) and [ADR 0177](../adr/0177-the-files-page-has-one-way-to-move.md).

On the **Files** page, a copy that the feed does not use has the button **Reference**. The operator did not know what it does. It puts the address of that copy in the feed, in place of the current address.

The operator agreed to the name **Use This Copy**. The other copy shows "Used by the feed", so the two rows use the same word.

The row of one file also has three controls that change the copy that the feed uses: **Reference**, **Move to {host}** and the select menu "File host of this file". ADR 0177 gave the release one way to move. The operator agreed that the ADR of finding 1 also examines these three controls.

On 2026-11-12 the operator selected this answer in the questionnaire: the file row has no **Move to {host}**. The copy that the feed uses has no button. Each other copy has **Use This Copy** and **Delete**. A move of one file is these two steps. The move of the release on the **Files** page moves all files. The select menu "File host of this file" has no answer at this time.

### Finding 6: The Pencil Of A Group In The Left Panel

Owner: [ADR 0165](../adr/0165-the-left-panel-marks-the-state-of-each-feed.md) items 1 to 4.

The row "HeyCitizen" in the left panel showed the pencil and the number 4. The operator read it as "4 changes not written", and could not find the changes. The number is the count of releases. The pencil tells that at least one feed of the group has a draft that is not written.

The page of the group then showed "No changes" on each of the four release tiles. The pencil thus came from the artist feed "HeyCitizen". The page of the group does not show the state of that feed, so the user cannot find the change from the mark.

On 2026-11-12 the operator opened the group. The row "HeyCitizen · Artist" in the left panel had no pencil. The page of the artist feed showed "Saved, not written" and **Review**. Thus the open group did not show the mark of the feed with the draft, as ADR 0165 item 2 gives.

The operator selected this design in the questionnaire, as a drawing:

- The row of the artist feed in the open group shows the pencil.
- The page of the group shows the artist feed as the first tile, above the releases, with its state.

### Finding 7: The Merge Of The Artist Feed

Owner: [ADR 0129](../adr/0129-a-draft-takes-a-change-of-the-hosted-file.md).

Review of the artist feed "HeyCitizen" showed "The hosted file changed after the draft started." The operator uses **Read Again** before each write, and did not remember a change with a different tool.

The agent read the hosted file. The host gave `Last-Modified: Fri, 06 Nov 2026 18:46:46 GMT`, and the feed gave the `lastBuildDate` 18:46:24 of the same day. Thus the draft started before a write on 2026-11-06. The probable cause is a write from a different Studio collection, or from the rc.21 AppImage, on the same hosts. This is not confirmed. If it is correct, the text "A different tool or a different person changed the file" is wrong.

In **Examine the Changes**, the field "Credits" was in "Both changed. Select one". Each option of its select menu held the full value, so the open menu was wider than the screen. The fix shows the two values as text above a select menu with the short options "Your edit" and "Hosted file".

### Finding 8: The Notice After A Move

Owner: [ADR 0150](../adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md) item 5.

Right after the move of the feed file, the release page showed "Music Index reads this feed from a different address." The notice ended with "A different collection or a different tool can hold this feed." Studio itself had moved the feed a few seconds before. Music Index needs time to read the new address. The notice does not tell this, and it points to a cause that is not correct.

### Finding 9: A Move Back Gives Duplicate Uploads With New Names

Owner: [ADR 0148](../adr/0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 2 and 3, and [ADR 0177](../adr/0177-the-files-page-has-one-way-to-move.md) item 3.

After step 2b, the feed and the four media files of "HeyCitizen's Test Album" were on `heycitizen`. The operator then moved "The feed file and the release files" to `bastone`. The feed part of Review was correct, with the return warning. The media part was not correct:

- Four uploads to `bastone` with new names: `TestAlbum/test/CheckYourPockets 2.mp3`, `Lofi-Experience 2.png`, `TestTickles 2.png` and `dancin-in-the-moonlight 2.mp3`. The feed used these names.
- Three copies to `bastone` at the usual names, `CheckYourPockets.mp3`, `Lofi-Experience.png` and `TestTickles.png`, each with the text "A spare copy. The feed does not use it."
- A "Keep" row for each old copy on `heycitizen`.

The agent read `bastone` at `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test/`. `dancin-in-the-moonlight.mp3` was there, with 3961306 bytes, the size of the file in the feed, and `Last-Modified` 2026-10-27. The other three usual names gave 404. Thus the move sends three files two times, and it does not use the copy of `dancin-in-the-moonlight.mp3` that is on `bastone`. The operator did not write the draft.

The agent read a copy of the collection database of the operator. The release has two library records for each of the four files, with the same SHA-256:

| Record | Path | Added | Source address |
|---|---|---|---|
| 11, 12, 13, 63 | `CheckYourPockets.mp3` and the other usual names | 2026-11-02 | the old addresses on `bastone` |
| 77 to 80 | the names with " 2" | 2026-11-12 11:24:46 UTC | the new addresses on `heycitizen` |

The new records came from `record_remote_files` in `crates/core/src/remote.rs`. That function adds a record for each feed address that it does not know. It knows an address when the address is the planned address or the address of a copy of a library file.

The function ignores an error in each of these two reads. Thus, when the read of the `heycitizen` inventory fails, the function does not know the `heycitizen` addresses. It then adds a second record with a free name for each address. The probable cause of the error at 11:24:46 is a locked vault. On 2026-11-12, a build with the first part of task 115 gave this log line for the same release: "host 1 needs the vault, and the vault is locked".

At 11:40:01, **Add to the Draft** downloaded records 77 to 80 into the library folder. The draft then had a copy operation for each of the eight records.

The plan in `release_move_plan` also makes a copy operation for record 13. But `bastone` has a file with the same SHA-256 at the target path. The write gives `FileExists` for that operation.

### Finding 10: Review Stays Open After A Discard

Owner: [ADR 0007](../adr/0007-draft-diff-write-and-announce.md).

After **Discard the Draft**, the sheet "Review changes" stayed open. It showed "The draft is saved. The hosted file does not change until you write it." and **Examine the Changes**. No draft was there. The operator closed the sheet with ✕. The fix closes the sheet after a discard.

### Finding 11: The Focus Ring Of The First Row Of The Left Panel

Owner: [ADR 0172](../adr/0172-a-list-is-one-tab-stop.md).

The focus ring on the first row of the left panel, "‹ Releases", had no top edge. The scroll area of the left panel had space only at its left and right sides, so it clipped the ring. The fix gives the scroll area 4 px of space at each side. The rows stay at the same positions.

On the **Credits** page, the ring on the first credit row had four edges, but its two top corners were not complete. The group has a corner radius of 14 px and clips its content. The ring was 2 px inside the row, with a radius of 8 px, so the group clipped its corners. The fix gives that ring a radius of 12 px.

### Finding 12: The Button "Use" Of A Page Suggestion

Owner: [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md) items 8 and 13 to 15, and [ADR 0170](../adr/0170-update-people-from-their-linked-pages.md) items 1 and 2.

On the page of Matt Finlay, the operator clicked **Use** on the wallet `mattfinlay@phafe.com`. The wallet then showed in the payment section with "Not used". The button adds the wallet to the person. It does not put the wallet in a feed. Thus the words "Use" and "Not used" tell the user two different things.

The operator selected these answers in the questionnaire:

- The button of a page suggestion is **Add**.
- The button of ADR 0170 item 1 is **Add All Suggestions**.

The same row of `peoplepages.js` shows each page suggestion: a picture, a website and a wallet. The ADR that records these answers must give the button for each kind of value.

### Finding 13: Add A Person Has No Search Of Music Index

Owner: [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) and [ADR 0163](../adr/0163-one-sheet-finds-and-links-a-page-on-music-index.md).

The sheet "Add a Person" has only the field "Name" and **Add the Person**. To link a page, the user adds the person, opens the page of the person and selects **Find on Music Index**. The sheet does not show a person that is in People already, so the user can add a second person with the same name.

The operator selected this design in the questionnaire, as a drawing:

- The field "Name" also searches People and Music Index while the user types.
- The sheet shows the group "Already in People" with **Open**, and the group "On Music Index".
- A result of Music Index adds the person and opens the sheet "From Music Index: {title}" of ADR 0163 items 4 to 11.
- The button at the bottom is **Add "{name}" Without a Page**.

A mockup of the real sheet comes before the ADR.

### Finding 14: The Linked Pages Are Low On The Page Of A Person

Owner: [ADR 0099](../adr/0099-the-people-destination-and-the-page-of-a-person.md) and [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md).

The section "Linked artist and label pages" is below the sections "Payment", "Credits" and "Support links". The linked page gives the suggestions in these sections, but the user sees the link last. **Use All Suggestions** is also below the suggestions that it adds.

The operator selected this design in the questionnaire, as a drawing: the section moves to the top of the page, below the head with **Find on Music Index** and above "Payment". The section keeps its rows and its buttons.

After the update of task 102, the operator also asked for these changes:

- The section "Credits" comes after the linked pages, then "Payment", then "Support links".
- A pencil next to the name in the head changes the name of the person in People. That name is the field "Name" in "Credits". **Rename** in "Change your feeds" changes the name in the feeds. The mockup must keep the two actions clearly separate.

### Finding 15: The Update Sheet Does Not Show Each Value

Owner: [ADR 0170](../adr/0170-update-people-from-their-linked-pages.md) item 5.

The sheet "Update from Linked Pages" showed the rows "Picture" and "Website" with no value. The row of the wallet showed its address. Thus the user cannot see the new website `https://phafe.com/mattfinlay` or the picture before **Update**. ADR 0170 item 5 gives "the values that change", so each row must show its value: the address of the website, and a small picture or its address.
