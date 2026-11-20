# ADR 0231: A New Release Writes Its Artist Page, And The Files Of A Restored Release

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, after the desktop check of tasks 205 to 209 (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of Tasks 205 To 209", findings 2, 3, 4, 7 and 8). The operator selected items 1 to 10 from drawings in the questionnaire.
- Technical choices: the main session selected items 11 to 17 from the causes that a research agent found. The main session examined each code citation. The operator did not receive them as a question.
- Bar: crosses modules. It changes the core links, the remote records, the download source, the move plan, the routes and the UI. Without this record, an agent can use ADR 0230 item 9 against item 6, or ADR 0205 item 8 against item 7.
- Changes: [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 9, 11, 27, 31 and 32 (items 1 to 6 and items 14 to 17).
- Changes: [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) item 8, the write of the two feeds of a link (items 7 to 10).
- Changes: [ADR 0179](0179-a-release-keeps-one-library-record-for-each-file.md) items 3 and 4, the join of a remote row and a local row (items 11 to 13).
- Changes: [ADR 0155](0155-the-sort-menu-names-both-directions.md) item 1 and [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) item 7, the name of the sort "Feed Order" (item 4). This change waits for item 5.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md), the write of a draft. [ADR 0036](0036-library-records-host-inventory-and-host-operations.md), the planned address of a library file. [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 2 and 3, the files of a move plan. [ADR 0198](0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md), the Retire block. [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) item 17, the link to an artist page.

## Context

The operator did the desktop check of tasks 205 to 209 on 2026-11-20, with the release binary of `da76cb0` and the real collection. These findings need a decision:

- Finding 2: the Retire block of a deleted release is in the section **Feed** at the bottom of the tab **Release**. The operator had to scroll to find it.
- Finding 3: the row "HeyCitizen" of the left panel has the pencil mark, and each release says "No changes".

  The cause: `link_new_release_pages` in `crates/api/src/routes.rs` calls `publisher::link`. That adds a `podcast:remoteItem` to the draft of the artist page. The write of the new release writes only the release (ADR 0205 item 8). Thus the artist page keeps a change that is not written.
- Finding 4: the rows with " 2" in the tab **Files** stay after **Put the File Back** and its write. ADR 0230 item 32 is false as written.

  The cause: `record_remote_files` in `crates/core/src/remote.rs` removes a remote row only when its address leaves the feed. ADR 0179 item 3 joins two records only by SHA-256, and a remote row has no SHA-256. A local row knows only its planned address on the media host of the feed. The rows thus stay when the media host of the draft is not the host of the address.
- Finding 7: the move plan of "new Feed" offers to download and send 2 media files that are on no host. The plan uses the remote rows of finding 4. `download::source_of` gives the address of a remote row when no connected host has a copy, and `check_download` accepts it.
- Finding 8: the sheet **Find on Music Index** has the group title "FEEDS" and the row text "Feed · {artist}".
- The sort "Feed Order" on the page of an artist or a label has the word "feed". [ADR 0054 of musicindex](../../../musicindex/docs/adr/0054-the-sort-menu-names-both-directions.md) gives this name to Studio, HitIt and the website.
- ADR 0230 item 9 lists the label pages as a parent of a new release. ADR 0213 item 17 and ADR 0230 item 33 need a link to an artist page.

## Decision

### A Deleted Release Has A Notice Under Its Head (Finding 2)

1. A feed with a done delete shows a notice under the head of its page, on each tab. The notice has the text "The feed file of this release is deleted from its file host.", and the buttons **Put the File Back** and **Check Again**. The buttons do the same actions as the Retire block.
2. The Retire block stays in the section **Feed** with its steps. This changes the place of ADR 0230 item 27, not its condition.

### The Words Of The Search Sheet (Finding 8)

3. The sheet **Find on Music Index** of a new release has the group title "Releases". Each result row has the text "Release · {artist}", or "Release" when Music Index gives no artist. This adds to ADR 0230 item 11.

### Page Order

4. The sort "Feed Order" has the name "Page Order", and its reversed direction has the name "Page Order, Reversed". The accessible name of the menu uses the same words.
5. Item 4 starts when musicindex records the same name for the three apps. The main session sends the proposal to the orchestrator session. Until then, Studio keeps "Feed Order". Open: the decision of musicindex.

### The Parent Of A New Release

6. **Change…** and **Select…** on the parent card open the list of the artist pages of the collection. The list has no label page. This corrects ADR 0230 item 9.

### A New Release Writes Its Artist Page Too (Finding 3)

7. Review of a new release shows the check box "Also write {artist page}" when the draft of its artist page has changes. The text under it is "Adds the link to {release} on the artist page." Review lists each change of the artist page draft under the check box. The check box is on at the start.
8. With the check box on, one press of **Write to the File Host** writes the release, then the artist page. Each feed keeps its own diff and its own write (ADR 0007). The check box "Send a podping" applies to the two writes.
9. When the write of the release fails, Studio does not write the artist page. When the write of the artist page fails, the release stays written, and Review shows the error of the artist page.
10. With the check box off, Studio writes only the release. The artist page keeps its change and its pencil mark.
11. Before Studio adds the link of a new release, it starts a draft of the artist page when the artist page has none. Thus the create of a release does not stop with the error `no_draft` after it made the feed row. Inference, not examined in a run: the code shows this order. A test proves it.

### A Remote Row Of A Library File Goes (Finding 4)

12. A remote row and a local row of one release can be one file. This is true when the address of the remote row is on a connected file host. That address must also be the planned address of the local row on that host (ADR 0036). The address of a copy of the local row also joins. Studio puts the address on the local row and removes the remote row.
13. Studio does this at the record times of ADR 0179 item 4 and at the start. It does not do this while a draft operation names the remote row.
14. Thus the rows with " 2" go at the first start or write after this change. This replaces ADR 0230 item 32.

### A Move Plan Does Not Download A File That Is Gone (Finding 7)

15. A remote file with an address on a connected file host, and with no copy on that host, has no download source. `download::source_of` gives none for it, and the move plan does not download it. The plan says "Not on {host}. Studio cannot download it.", and the file does not move (ADR 0148 item 3).
16. A file that item 12 joins to a local row moves from the library. The plan says "Not on a file host. Studio sends it from the library." The source of a move is "The library", not "the feed".
17. The repair of ADR 0230 item 31 stays: **Move This Release…** with "Media files only" to "bastone". For the test album it copies no file. For "new Feed" it sends the two media files from the library. This corrects ADR 0230 item 31.

## Consequences

- A new release and the link on its artist page go to the file host in one step.
- The tab **Files** of a restored release shows each file once.
- A move plan offers no download that cannot work.
- A deleted release shows what the user can do at the top of each tab.
- Studio keeps the sort name of the other two apps until musicindex changes it.

## Alternatives Considered

- A notice on the page Releases for an artist page with changes that are not written. Not chosen by the operator: the user must remember a second write.
- Only a tooltip on the pencil mark. Not chosen by the operator: it is easy to miss.
- Move the Retire block to the top of the tab **Release**. Not chosen by the operator: the notice shows on each tab.
- Join a remote row to a local row only by the media host of the draft. Not chosen: finding 4 has rows on a different host.
- Change "Feed Order" in Studio now. Not chosen: the three apps use one name (musicindex ADR 0054).

## Checks

Mechanical:

- UI test: a feed with a done delete gives the notice and its two actions on each tab. A feed with no delete gives no notice.
- UI test: the sheet **Find on Music Index** has the group title "Releases" and the row text "Release · {artist}".
- UI test: the parent list of the form of a release has artist pages only.
- API test: the create of a release with an artist page that has no draft makes the link and gives no `no_draft`.
- API test: Review of a new release gives the changes of the artist page. A write with the check box on writes the two feeds. A failed write of the release writes no artist page.
- Core test: a remote row at the planned address of a local row on a second connected host goes at the next record time.
  Extend `a_deleted_feed_makes_no_remote_rows_and_the_restore_gives_its_path_again` in `crates/core/tests/feed_retire.rs`.
- Core test: `download::source_of` gives none for an address on a connected host with no copy.
- UI test: the plan texts of items 15 and 16.

Visual:

- Walk screenshots at 1280 px and 390 px of these screens: the notice of a deleted release, Review of a new release with the check box, and the move plan of items 15 and 16.
- Human check: make a release on the real collection and write it with the check box on. The pencil mark of the artist page goes.
- Human check: the tab **Files** of "new Feed" after the first start of the new binary shows each file once.
