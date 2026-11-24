# ADR 0233: Each Add Form Finds On Music Index And Copies The Found Feed

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, after the desktop check of rc.30 (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of rc.30", findings 1, 2, 4 and 5). The operator selected items 1, 3, 4, 6 and 10 from drawings in the questionnaire. The operator asked for items 9 and 13 in the chat.
- Technical choices: the main session selected items 2, 5, 7, 8, 11, 12, 14 and 15. The operator did not receive them as a question.
- Bar: crosses modules. It changes the Music Index client, the feed reader, the make of a feed, the remove of a feed, the routes and the UI. Without this record, an agent can keep the scope of ADR 0230 item 16 and copy no tracks.
- Changes: [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1, 4, 11, 14, 15 and 16, the add menu and the scope of a selection (items 1 to 12).
- Changes: [ADR 0212](0212-the-add-pages.md) item 1, the page with the rows "By address or name", "From a file host folder" and "From MSP 2.0 backups" (items 3 and 4).
- Changes: [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) item 8, the page **Add One You Have** as a start of a new feed (item 3).
- Changed by: [ADR 0234](0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) item 11, on 2026-11-20. Item 15: the remove also clears a link that only a draft has for a written release.
- Changed by: [ADR 0238](0238-a-found-release-gives-the-page-of-each-role.md) items 1 to 9, on 2026-11-21. Items 6 to 9: in the Artist, Label and Hosting forms, a found release gives the page of the form role.
- Uses: [ADR 0227](0227-undo-and-redo.md), the undo of a remove. [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 7 and 11, the link of a new release to its artist page.

## Context

The operator did the desktop check of rc.30 on 2026-11-20. Four findings need a decision:

- Finding 1: in the sheet **Find on Music Index** of a new release, a click on an artist row gives "Select a release. This row is not a release." The search gives artist rows, but a selection gives no choices (`crates/api/ui/pickers.js` near line 312).
- Finding 2: a release made from a selection has no tracks and no payment recipients. ADR 0230 items 14 to 16 fill only five values: the title, the artist name, the description, the artwork and the website. Music Index gives the tracks and the payment recipients, but not the audio file addresses (`crates/core/src/musicindex.rs` near line 40). The feed file of the found feed has all of them. `discover::read_feed` (`crates/core/src/discover.rs` near line 280) reads a feed file with its items, enclosures and value blocks.
- Finding 4: the operator finds **Add One You Have…** in the menu **＋ Add** unnecessary, and wants **Find on Music Index** in each form of the menu.
- Finding 5: after **Remove from Collection** of an unwritten release, the draft of its artist page keeps the remote item that the make of the release added.

**Add One You Have…** opens the only page with three ways to add a feed. These are "By address or name", "From a file host folder" and "From MSP 2.0 backups". With "By address or name", Studio manages that same feed.

## Decision

### Find On Music Index In Each Form (Finding 4)

1. The menu **＋ Add** has these values, in this sequence: Release, Artist, Label, Show, Playlist. It has no separator and no **Add One You Have…**. The page **Add One You Have** goes.
2. Each of the five forms starts with the row "Find on Music Index", after the parent card of a release. The search shows only results of the kind of the form: releases and artists for Release, artists for Artist, labels for Label, shows for Show, and playlists for Playlist. A result that Music Index gives with no kind shows in each form.
3. Below the results, the sheet has the rows **Add from a File Host Folder…** and **Add from MSP 2.0 Backups…**. They open the flows of ADR 0212 item 1. The folder row shows only when a file host is connected.
4. A feed address in the search field reads that feed, as "By address or name" did.
5. A form that needs a file host and has none shows the step to connect one, as the page **Add One You Have** did.

### A Found Feed That Is Yours

6. A result whose feed address is on a connected file host shows the mark "Yours". A selection adds that same feed to Studio, with its identity and its file, as "By address or name" did (`POST /api/v1/feeds/import`). It makes no copy.
7. A result that is in the collection already shows "In Studio". A selection opens it.

### A Copy Of A Different Feed (Finding 2)

8. A selection of a different result reads the feed file of the found feed. The form then shows the line "Copies {n} tracks and {m} payment recipients from {title}."
9. The new feed gets each value of the found feed: the title, the author, the description, the artwork, the website, the value recipients of the feed, and each item with its title, enclosure, duration, artwork, description and value recipients. A publisher feed also gets the remote items of the found feed.
10. The parent of the form stays the parent of the new release (ADR 0230 item 9). The artist name of the found feed fills "Artist name".
11. The new feed gets its own identity: a new `podcast:guid` from its address, its own self link and a new guid for each item. The enclosure addresses stay the addresses of the found feed. Studio copies no audio file. **Move This Release…** can copy the files later.
12. When the author of the found feed is not the name of the parent, the form shows: "The payments go to the recipients of {title}. Examine them on **Value Routes** before the write." Review shows each copied value before the first write.
13. In the form of a release, a selection of an artist row shows the releases of that artist on Music Index, with the back row **‹ Results**. A selection of one of these releases follows items 6 to 12 (finding 1).
14. When Studio cannot read the feed file, the form fills the values that Music Index gives, as ADR 0230 items 14 and 15 did. It says: "Studio could not read the feed file of {title}. No tracks and no payment recipients are copied."

### The Remove Of An Unwritten Release (Finding 5)

15. **Remove from Collection** of a release that was never written also removes the changes that its make put in the drafts of other feeds. These are the remote item in the draft of its artist page (ADR 0231 item 11) and its link to the label page. When such a draft then has no change except the build date, the draft goes. **Undo** gives back the release and these changes (ADR 0227).

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0230 item 1 | Item 1 removes **Add One You Have…** from the menu. |
| ADR 0230 item 4 | Items 3 and 4 move the three ways to add into the sheet. |
| ADR 0230 item 11 | Item 2 puts the row into each form. Item 13 adds the releases of an artist row. |
| ADR 0230 items 14 to 16 | Items 8 to 12 copy the tracks and the payment recipients. Item 14 keeps the earlier fill when the feed file cannot be read. |
| ADR 0212 item 1 | Items 3 to 6 replace the page. |
| ADR 0232 item 8 | The page **Add One You Have** is no longer a start of a new feed. |

## Consequences

- One sheet adds each kind of feed: a feed that is yours, a copy of a different feed, a folder and a backup.
- A copy of a release of a different artist sends payments to that artist until the user changes the value recipients. Item 12 warns about this.
- A copied release plays its audio from the addresses of the found feed until a move.

## Alternatives Considered

- Two actions on each result, **Add to Studio** and **Use as a Start**. Not chosen by the operator.
- Keep the start values only (ADR 0230 items 14 to 16). Not chosen by the operator.
- Keep **Add One You Have…** in the menu. Not chosen by the operator.
- Move the three ways to add to the page **File hosts**. Not chosen by the operator.
- Copy the item guids of the found feed. Not chosen: two feeds with the same item guids can mix in Music Index.

## Checks

Mechanical:

- UI test (`crates/api/ui-tests/addfeeds.test.mjs`): the menu has the five values of item 1 and no **Add One You Have…**.
- UI test: the search model of each form gives only the kinds of item 2. The sheet model gives the rows of item 3, and the folder row only with a file host.
- UI test: a result on a connected file host has the mark "Yours" and the action import. A result in the collection has "In Studio". A different result has the action copy.
- UI test: a selection of an artist row in the form of a release gives the releases of that artist and the back row of item 13.
- API test: the make of a release from a found feed file gives the items, the enclosure addresses and the value recipients of that file, a new `podcast:guid`, and new item guids.
- API test: a feed file that cannot be read gives the values of item 14 and no item.
- API test: **Remove from Collection** of an unwritten release removes its remote item from the draft of its artist page. With no other change, the draft goes. **Undo** gives back both.

Visual, by the main session with screenshots at 1100 px and 390 px:

- The sheet of each form with results, the mark "Yours", the rows of item 3, and the warning of item 12.
- The releases of an artist row with **‹ Results**.

Human check: the next desktop check of the candidate copies a release of Music Index and examines its tracks and payment recipients in Review.
