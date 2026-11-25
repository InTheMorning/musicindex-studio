> Archived: superseded by [ADR 0261](../../adr/0261-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) on 2026-11-25. The new ADR restates each live rule and gives its Item Map.

# ADR 0230: One Add Menu, The Parent Of A New Release, And The Check After rc.29

- Status: Superseded by ADR 0261
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, in the desktop check after rc.29 (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of The Fixes After rc.29", findings 2 to 12). The operator selected items 1 to 13 and items 17 to 19 from drawings in the questionnaire. In the chat, the operator said that labels of user actions must not use the word "feed". The operator also said that a new item must show its parent clearly.
- Technical choices: the main session selected items 14 to 16 and items 20 to 33 from the causes that two research agents found. The operator did not receive them as a question. Items 16 and 21 are open for a review by the operator.
- Bar: crosses modules: the core drafts, the delete records, the remote records, the routes and the UI. An agent can also add the word "feed" to a new button by mistake. The earlier ADRs 0212 and 0213 give the opposite of items 1 to 13.
- Changes: [ADR 0212](../../adr/0212-the-add-pages.md) items 1, 5, 6 and 8, the add controls of the page Releases and the form (items 1 to 13).
- Changes: [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) items 1, 2, 5 and 6, where a release starts and the fields of its form (items 1 to 13).
- Changes: [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) item 2, the click on a row with the mark "Not checked" (items 17 to 19).
- Changes: [ADR 0148](../../adr/0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 4 and 5, the media host and the old files of a move in the draft (items 20 to 24).
- Changes: [ADR 0131](../../adr/0131-delete-a-feed-from-its-file-host.md) items 4 and 10, the text about Music Index (items 25 and 26).
- Changes: [ADR 0198](../../adr/0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md) items 6 and 9, **Put the File Back** after each done delete (items 27 to 29).
- Changes: [ADR 0077](0077-a-new-album-can-start-from-an-album.md), the copy of a release in "Start from". Item 13 removes the copy, and ADR 0077 has no other live rule. ADR 0077 goes to the archive in the commit that removes the copy.
- Uses: [ADR 0164](../../adr/0164-the-releases-of-a-publisher-feed-have-an-add-row.md), the add row of a publisher page. [ADR 0211](../../adr/0211-studio-follows-a-move-that-a-different-studio-made.md) item 16, the notice **Use This File Host**. [ADR 0036](../../adr/0036-library-records-host-inventory-and-host-operations.md), the remote records.
- Changed by: [ADR 0231](../../adr/0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 1 to 6 and 14 to 17, on 2026-11-20. Item 9: the parent list has artist pages only. Item 11: the sheet says "Release". Item 27: a notice under the head shows the actions. Items 31 and 32: the corrected repair and the join of the rows with " 2".
- Changed by: [ADR 0233](../../adr/0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) items 1 to 14, on 2026-11-20. Items 1, 4 and 11: **Add One You Have…** leaves the menu, and each form gets the search. Items 14 to 16: a selection copies the tracks and the payment recipients.
- Changed by: [ADR 0240](0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) items 1 and 4, on 2026-11-21. Item 1: the menu gets Hosting after Label. Item 2: the menu of a label page or a hosting page has no Release.

## Context

The operator did the desktop check of the build after rc.29 on 2026-11-20. These findings need a decision:

- Finding 2: the head of an artist page has the button **Open the Feed**.
- Finding 3: the page Releases has **＋ Add a Feed** at its head. It makes an artist or label page, a show or a playlist. A release starts only on the page of its artist (ADR 0213 item 1).
- Finding 4: "Start from" in the form of a release copies a release of the collection (ADR 0077). The operator wants the tool "Find on Music Index" there, to fill the fields.
- Finding 5: a click on a row of the left panel with the mark "Not checked" opens the row and does not check its feeds.
- Findings 6 and 9: the tab **Files** of two releases shows the media files on the host "CloudFlare". They are on "bastone".

  The cause: the route `move_release` sets `files_connector_id` at **Add to the Draft**, outside the draft. A discard of the draft does not put back the earlier value. The same sheet showed "Old files: None" and no check box, because the plan counts only the old copies of the files that move. Studio still kept holds on the copies on "bastone".
- Finding 7: for a new feed, the delete sheets said "Music Index removed the entry". Music Index gives the same 404 for a feed that it never read and for a removed feed. Studio reads both as "removed". The delete is correct (ADR 0200). Only the text is false.
- Finding 8: after a discard, **Delete from File Host** showed the refusal of the earlier try again. The sheet shows the error of the last action, and its open does not clear it.
- Findings 10 and 12: after a delete, **Put the File Back** did not show. The Retire block shows only when the draft hides the feed (ADR 0198 items 5 and 6), and the feed of the check was not hidden. The head showed **Use This File Host**, because the feed has an address under a connected host and no connector.
- Finding 11: after the delete, the file list shows the media files again with names that end in " 2". The write and each start of Studio make the remote records of the feed. The feed has no path after the delete, so Studio does not find its own copies, and it adds new rows.

## Decision

### One Add Menu (Finding 3)

1. The head of the page Releases has one button **＋ Add**. It opens a menu with these values, in this sequence: Release, Artist, Label, Show, Playlist. A separator follows, then **Add One You Have…**.
2. The head of an artist page has the same button and menu. There, the value Release has that artist as its parent. The head of a label page and of a hosting page has the same button and menu, and Release has no start parent.
3. Release opens the form of item 6. Artist and Label open the form that makes a publisher page, with the role Artist or Label as its start value. Show and Playlist open their forms of ADR 0212 item 7.
4. **Add One You Have…** opens the page with the rows "By address or name", "From a file host folder" and "From MSP 2.0 backups" (ADR 0212 item 1). The title of the page is "Add One You Have".
5. The add row **＋ Add a Release** of ADR 0164 stays in the list of an artist page.

### No Word "Feed" In A User Action (Finding 2)

6. The label of a button, a menu value, a sheet title or a page title that starts a user action names the thing: release, artist, label, show or playlist. It does not use the word "feed". A text that explains a file can still say "feed file".
7. The head of a publisher page has the button **{Type} Page**, for example **Artist Page** or **Label Page**. It replaces **Open the Feed**. The type is the type label of the head.

### The Parent Of A New Release (Findings 3 And 4)

8. The title of the sheet is "New release for {parent}". With no parent, it is "New release".
9. The first row of the form is a parent card. It shows the artwork, the name and the type of the parent page, and the button **Change…**. **Change…** opens the list of the artist pages and the label pages of the collection.
10. With no start parent, the card says "No artist page" and has the button **Select…**. The form cannot make the release until the user selects a parent. This keeps ADR 0213 item 17.
11. The row "Find on Music Index" with the summary "Fill the form from a release." follows the parent card. It opens the sheet "Find on Music Index" (`FindOneSheet`) with the releases of Music Index.
12. The fields follow the row: "Title", "Kind", "Artist name", "File host" and "Feed file". "Artist name" is the name that podcast apps show. Its start value is the name of the parent.
13. The form has no "Start from". The copy of a release of the collection goes, with its route field `from` and its core code.

### The Pick From Music Index

14. A pick fills "Title" and "Artist name", and "Kind" when Music Index gives a track count.
15. A pick also keeps the description, the artwork address and the website of the found release. The core puts them in the first draft of the new release. The user sees them in Review before the first write.
16. Open for a review by the operator: items 14 and 15 are the recommended scope. The tracks of the found release are not copied.

### A Click On "Not Checked" (Finding 5)

17. A click on a row of the left panel with the mark "Not checked" opens the row, as before. It also starts the check of each feed of the row that is not checked.
18. While the check runs, the mark of the row is a spinner with the text "Checking". After the check, the row shows the mark of ADR 0165 item 2, or no mark when the feeds are up to date.
19. A row with a different mark, or with no mark, does not start a check on a click.

### The Media Host And The Old Files Of A Move (Findings 6, 8 And 9)

20. **Add to the Draft** of **Move This Release…** keeps the earlier value of `files_connector_id` with the draft. Then it changes the value.
21. A discard of the draft puts back the earlier value. A write clears the kept value. Open for a review by the operator: a value that only the write sets is cleaner, but `host_operations` must then read the pending value for "Media files only". This ADR selects the kept value because it is the smaller change.
22. The move plan counts the old copies of each media file that the move uses on the target host. This includes a file that is already there ("Already on {host}"). The sheet then shows "Old files" and the check box of ADR 0148 item 5 for these copies.
23. When the plan has no old copy, the route keeps no hold, also when the request has `delete_later`.
24. The first sheet of **Delete from File Host…** clears the error of the last try when it opens.

### The Text About Music Index (Finding 7)

25. The third row of the sheet "What do you want to do?" has the text "Music Index does not hold this feed." It replaces "Music Index removed the entry.".
26. The state point of the sheet "Delete the Feed File" has the same text. The rule of ADR 0200 does not change: Studio deletes a feed file only when Music Index does not hold the feed.

### After A Delete (Findings 10 To 12)

27. Each feed with a done delete shows the Retire block with **Put the File Back** and **Check Again**, also when its draft does not hide the feed.
28. A feed with a done delete does not show the notice **Use This File Host**. The API gives `host_for_address` false for it.
29. The write of a delete and each start of Studio make no remote records for a feed with a done delete.
30. **Put the File Back** gives the feed its path again. Then the remote records of item 29 run as for each feed.

### Repair Of The Data

31. The task of items 20 to 24 does not change the collection of the operator. To repair the two releases of findings 6 and 9, the operator uses **Move This Release…** with "Media files only" to "bastone". This sets the value and copies no file.
32. The rows with " 2" of finding 11 go when the feed gets its path again (item 30), or with **Remove from Collection**.

### What Does Not Change

33. A release still needs a link to an artist page (ADR 0213 items 17 to 23). The links of the new release still follow ADR 0213 items 7 and 8.

## Consequences

- One button adds each kind of item, and a release can start from the page Releases.
- The user sees the parent of a new release before the first field.
- A discard of a move leaves the media host as it was.
- A deleted feed can come back from its own page.

## Alternatives Considered

- Two buttons, **＋ Add a Release** and **＋ Add an Artist**. Not chosen by the operator.
- A select menu "Artist page" as the first field. Not chosen by the operator: it shows no artwork, and it looks like each other field.
- Keep the copy of a release in a closed fold under "Find on Music Index". Not chosen by the operator.
- A small button on the mark that only checks. Not chosen by the operator.
- Set `files_connector_id` only at the write. Not chosen now (item 21).

## Checks

Mechanical:

- UI test: the add menu of the page Releases has the values of item 1. The menu of an artist page gives that artist as the parent of Release.
- UI test: no label of an add control or of a head button has the word "Feed".
- UI test: the form of a release has no "Start from", and it cannot make the release with no parent.
- API test: **Add to the Draft** of a move, then a discard, gives the earlier `files_connector_id`. A write keeps the new value.
- API test: a plan to a host that holds each media file gives the old copies and the check box.
- API test: after a delete, `host_for_address` is false, the retire state has the restore action with no hidden draft, and the remote records add no row.
- UI test: the texts of items 25 and 26.

Visual:

- Walk screenshots of the add menu, the form of a release with a parent and with no parent, and the left panel during a check, at 1280 px and 390 px.
- Human check: delete a test release that is not hidden, then **Put the File Back**.
