# ADR 0261: One Add Menu, The Parent Of A New Release, And The Check After rc.29

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25. The global decision on ADR overuse (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each subsequent change.
- Supersedes: [ADR 0230](../archive/adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md), One Add Menu, The Parent Of A New Release, And The Check After rc.29.
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) item 1, on 2026-11-25. The button **＋ Add** is in the head of Overview (item 3).
- Uses: [ADR 0164](0164-the-releases-of-a-publisher-feed-have-an-add-row.md), the add row of a publisher page. [ADR 0211](0211-studio-follows-a-move-that-a-different-studio-made.md) item 16, the notice **Use This File Host**. [ADR 0036](0036-library-records-host-inventory-and-host-operations.md), the remote records.

## Context

- ADR 0230 records the operator findings after rc.29. It has three changes from ADR 0231, ADR 0233 and ADR 0240.
- ADR 0231 corrects the artist selection, the delete notice and the repair of the remote file rows.
- ADR 0233 replaces the earlier add page and expands a selection from Music Index. ADR 0254 keeps the current Publisher menu words.
- Item 21 remains open for operator review. ADR 0233 replaced the restricted copy scope of superseded item 16.
- The archived ADR 0230 keeps the dated findings and their causes. This ADR keeps its live items and the required checks.

## Decision

### One Add Menu (Finding 3)

1. The head of the page Releases has one button **＋ Add**. Its menu has these values, in this sequence: Release, Artist, Label, Publisher, Show, Playlist (ADR 0254 item 44). It has no separator and no **Add One You Have…** (ADR 0233 item 1).

2. The head of an artist page has the same button and menu. There, the value Release has that artist as its parent. The menu of a label page or a publisher page has no Release. Artist and Label on a publisher page make pages below it. Artist on a label page makes a page below it. A page-head menu has no role with the same rank or a higher rank (ADR 0254 item 17).

3. Release opens the form of items 8 to 12. Artist, Label and Publisher open the publisher-page form with that role as the start value (ADR 0254 item 44). Show and Playlist open their forms of ADR 0212 item 7.


5. The add row **＋ Add a Release** of ADR 0164 stays in the list of an artist page.

### No Word "Feed" In A User Action (Finding 2)

6. The label of a button, a menu value, a sheet title or a page title that starts a user action names the thing: release, artist, label, show or playlist. It does not use the word "feed". A text that explains a file can say "feed file".
7. The head of a publisher page has the button **{Type} Page**, for example **Artist Page** or **Label Page**. It replaces **Open the Feed**. The type is the type label of the head.

### The Parent Of A New Release (Findings 3 And 4)

8. The title of the sheet is "New release for {parent}". With no parent, it is "New release".
9. The first row of the form is a parent card. It shows the artwork, the name and the type of the parent page, and the button **Change…**. **Change…** opens only the artist pages of the collection, with no label page (ADR 0231 item 6).

10. With no start parent, the card says "No artist page" and has the button **Select…**. The form cannot make the release until the user selects a parent. This keeps ADR 0256 item 17.
11. The row "Find on Music Index" with the summary "Fill the form from a release." follows the parent card. Each add form has this search (ADR 0233 item 2). For a release, it shows releases and artists. An artist selection shows its releases, with **‹ Results** (ADR 0233 item 13). 

    The group title is "Releases". A result says "Release · {artist}", or "Release" with no artist (ADR 0231 item 3). ADR 0233 items 3 to 12 give the folder and backup rows, address input, import and copy.

12. The fields follow the row: "Title", "Kind", "Artist name", "File host" and "Feed file". "Artist name" is the name that podcast apps show. Its start value is the name of the parent.
13. The form has no "Start from". The copy of a release of the collection goes, with its route field `from` and its core code.

### The Pick From Music Index

14. When Studio cannot read the selected feed file, Music Index supplies the start values (ADR 0233 item 14). A pick fills "Title" and "Artist name", and "Kind" when Music Index gives a track count. The form says "Studio could not read the feed file of {title}. No tracks and no payment recipients are copied."

15. In that fallback, a pick also keeps the description, the artwork address and the website of the found release. The core puts them in the first draft. Review shows them before the first write (ADR 0233 item 14). A feed read that succeeds follows the broader copy rules of ADR 0233 items 8 to 12.


### A Click On "Not Checked" (Finding 5)

17. A click on a row of the left panel with the mark "Not checked" opens the row, as before. It also starts the check of each feed of the row that is not checked.
18. While the check runs, the mark of the row is a spinner with the text "Checking". After the check, the row shows the mark of ADR 0259 item 15, or no mark when the feeds are up to date.
19. A row with a different mark, or with no mark, does not start a check on a click.

### The Media Host And The Old Files Of A Move (Findings 6, 8 And 9)

20. **Add to the Draft** of **Move This Release…** keeps the earlier value of `files_connector_id` with the draft. Then it changes the value.
21. A discard of the draft puts back the earlier value. A write clears the kept value. Open for a review by the operator: a value that only the write sets is cleaner, but `host_operations` must then read the pending value for "Media files only". This ADR selects the kept value because it is the smaller change.
22. The move plan counts the old copies of each media file that the move uses on the target host. This includes a file that is there ("Already on {host}"). The sheet then shows "Old files" and the check box of ADR 0148 item 5 for these copies.
23. When the plan has no old copy, the route keeps no hold, also when the request has `delete_later`.
24. The first sheet of **Delete from File Host…** clears the error of the last try when it opens.

### The Text About Music Index (Finding 7)

25. The third row of the sheet "What do you want to do?" has the text "Music Index does not hold this feed." It replaces "Music Index removed the entry.".
26. The state point of the sheet "Delete the Feed File" has the same text. The rule of ADR 0200 does not change: Studio deletes a feed file only when Music Index does not hold the feed.

### After A Delete (Findings 10 To 12)

27. Each feed with a done delete shows the Retire block, also when its draft does not hide the feed. A notice below the page head shows on each tab (ADR 0231 items 1 and 2). It says "The feed file of this release is deleted from its file host." It has **Put the File Back** and **Check Again**, with the same actions as the Retire block.

28. A feed with a done delete does not show the notice **Use This File Host**. The API gives `host_for_address` false for it.
29. The write of a delete and each start of Studio make no remote records for a feed with a done delete.
30. **Put the File Back** gives the feed its path again. Then the remote records of item 29 run as for each feed.

### Repair Of The Data

31. The task of items 20 to 24 does not change the collection of the operator. To repair findings 6 and 9, the operator uses **Move This Release…** with "Media files only" to "bastone". For the test album it copies no file. For "new Feed" it sends the two media files from the library (ADR 0231 item 17).


### What Does Not Change

33. A release needs a link to an artist page (ADR 0256 items 17 to 23). The links of the new release follow ADR 0256 items 7 and 8, with ADR 0257 item 16a.

## Consequences

- One button adds each kind of item, and a release can start from the page Releases.
- The user sees the parent of a new release before the first field.
- A discard of a move leaves the media host as it was.
- A deleted feed can come back from its own page.

## How This Decision Is Checked

Mechanical:

- UI test: the add menu of the page Releases has the values of item 1. The menu of an artist page gives that artist as the parent of Release.
- UI test: no label of an add control or of a head button has the word "Feed".
- UI test: the form of a release has no "Start from", and it cannot make the release with no parent.
- API test: **Add to the Draft** of a move, then a discard, gives the earlier `files_connector_id`. A write keeps the new value.
- API test: a plan to a host that holds each media file gives the old copies and the check box.
- API test: after a delete, `host_for_address` is false, the retire state has the restore action with no hidden draft, and the remote records add no row.
- UI test: the texts of items 25 and 26.

Visual:

- Take screenshots of the add menu and the release form, with and without a parent. Use widths of 1280 px and 390 px. Examine the left panel during a check at the same widths.
- Human check: use **Delete from File Host** for a test release that is not hidden. Then use **Put the File Back**.

This consolidation adds no visual or human result. Earlier results stay in their review files.

## Open Points

- Item 21 remains open for operator review, as in ADR 0230.
- Item 7 keeps the button **{Type} Page** of ADR 0230 unchanged. ADR 0254 item 11 says "Open the feed".
- This consolidation does not select between these button labels.

## Item Map

| Superseded item | New item or current owner | Source of the change |
| --- | --- | --- |
| ADR 0230 item 1 | 1 | ADR 0233 item 1, ADR 0240 item 1, ADR 0253 item 12 and ADR 0254 item 44. |
| ADR 0230 item 2 | 2 | ADR 0240 item 4, restated in ADR 0254 item 17. |
| ADR 0230 item 3 | 3 | The Release form is in items 8 to 12. Publisher follows ADR 0254 item 44. |
| ADR 0230 item 4 | ADR 0233 items 3 and 4 | The earlier add page is removed. Its flows are in the search sheet. |
| ADR 0230 items 5 to 8 | 5 to 8 | None. |
| ADR 0230 item 9 | 9 | ADR 0231 item 6: artist pages only. |
| ADR 0230 item 10 | 10 | None. |
| ADR 0230 item 11 | 11 | ADR 0231 item 3 and ADR 0233 items 2 to 13. |
| ADR 0230 items 12 and 13 | 12 and 13 | None. |
| ADR 0230 items 14 and 15 | 14 and 15 | ADR 0233 item 14 keeps these values only when the feed read fails. |
| ADR 0230 item 16 | ADR 0233 items 8 to 12 | The restricted copy scope is removed. Tracks and payment recipients are copied. |
| ADR 0230 items 17 to 26 | 17 to 26 | None. Item 21 remains open for operator review. |
| ADR 0230 item 27 | 27 | ADR 0231 items 1 and 2: notice below the head on each tab. |
| ADR 0230 items 28 to 30 | 28 to 30 | None. |
| ADR 0230 item 31 | 31 | ADR 0231 item 17 corrects the repair. |
| ADR 0230 item 32 | ADR 0231 items 12 to 14 | The remote-row join replaces the earlier removal claim. |
| ADR 0230 item 33 | 33 | ADR 0256 owns the artist-link rules that ADR 0213 gave. |
| ADR 0230 "Checks" | "How This Decision Is Checked" | No new pass is recorded. |
