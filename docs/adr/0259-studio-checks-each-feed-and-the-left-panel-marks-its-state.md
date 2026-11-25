# ADR 0259: Studio Checks Each Feed And The Left Panel Marks Its State

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25. The global decision on ADR overuse (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each subsequent change.
- Supersedes: [ADR 0150](../archive/adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md), Studio Shows A Change From Outside When A Page Opens.
- Supersedes: [ADR 0165](../archive/adr/0165-the-left-panel-marks-the-state-of-each-feed.md), The Left Panel Marks The State Of Each Feed.
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 1, 2, 5, 11, 18, 19 and 23, on 2026-11-25. The checks of Releases run when Overview opens, and the row of Overview takes the place of the tile (items 2 and 7).
- Uses: [ADR 0150](../archive/adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md) for the source of items 1 to 13. [ADR 0088](../archive/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) for the earlier panel rows. ADR 0254 owns the Catalog rows. musicindex ADR 0032 R-DOT-1 keeps the dot for the type only.

## Context

- ADR 0150 gave the check for a change from outside. ADR 0165 extended the check to each feed and added state marks.
- Each ADR has three changes. This ADR keeps the current check, the marks and the open point about "Check Everything".
- ADR 0211 gives the moved result and the Warning color. ADR 0184 gives the notice after a move by Studio.
- ADR 0261 gives the check on a click. ADR 0254 gives the feeds that a Catalog row covers.

## Decision

### The Check For A Change From Outside

1. After the feed list loads, Studio checks each feed with a public address (item 18, from ADR 0165 item 5). Each open of Releases checks again. A feed page checks its feed when it opens. No worker runs in the background.

2. The check compares the hosted file with the last file that Studio read. The result "Changed outside Studio" shows on the tile and in a notice on the feed page, with "Read Again". The tile mark and the notice have the Warning color and the triangle icon (ADR 0211 items 20 and 22).

3. A pointer file at the folder address gives the result "moved" (ADR 0211 items 1 to 5). The folder address is the address that the file host gives for the path of the feed. A pointer file names a different address in `itunes:new-feed-url`. The last read can name no new address, or a different one, in `itunes:new-feed-url`. A new value in the hosted file at the public address also gives "moved". 

    The result "moved" comes before "changed". Studio shows no second notice about other sources for the same address. The move notice has the Warning color and the triangle icon (ADR 0211 item 20).

4. The warning says "A different Studio moved this feed." It gives the new address and the connected file host that gives it, when one does. It says "The old address has a pointer file." It has **Follow the Move**, **Move It Back…**, **Show the Difference** and **Keep This File Host** (ADR 0211 items 6 to 8). **Show the Difference** opens the comparison of ADR 0129. **Move It Back…** shows only in the conditions of ADR 0211 item 18. **Keep This File Host** hides the notice until the hosted file changes again.

5. The check asks Music Index for the feed address of each feed page. A different address gives a notice on the feed page, with the two addresses, and no mark on the tile. With no answer, or with no entry for this feed, the page shows nothing. The source address of the last written move by this Studio gives the move notice of ADR 0184 items 3 to 9. That notice gives the podping state. ADR 0189 items 3 to 7, ADR 0191 item 4 and ADR 0229 item 6 give its current line and send buttons.

6. The check reads only public addresses, so it needs no vault. A feed with no public address has no check.
7. A read that fails shows nothing, and it does not change an earlier result. A page does not wait for a check.
8. The check writes nothing and changes no draft. A change goes through "Read Again" or the merge of ADR 0129.
9. The core makes the comparison and gives the result. The UI shows it (ADR 0065).
10. Settings has the check box "Check for outside changes", which starts on. With the box off, a page sends no request for this check.
11. Each read has a time limit, and the Releases page reads a maximum of 4 feeds at one time. On a server, the network policy of ADR 0057 applies.
12. A feed with no record takes the bytes of its first check as its start, so that check gives no result. Each subsequent read or write of Studio records its bytes.
13. After a write, the bytes from before that write are not a change, because a public address can give them for a short time.

### The Marks And Their Store

14. A row of the left panel shows at most one state mark, between its name and its count. A feed that is up to date has no mark.

15. The marks, from the most urgent:
    - **Changed outside Studio**: a cloud with "!", in the Warning color (ADR 0211 items 20 and 21). The check of items 1 to 13 found the state "changed" or "moved". The tooltip gives "Changed outside Studio" or "Moved outside Studio".
    - **Changes not written**: a pencil, in the `--accent` color. The draft state is "new" or "changed".
    - **Not checked**: a dashed circle, in the `--muted` color. Since the start, Studio did not read the hosted file correctly. The check runs, the read did not succeed, the feed has no public address, or the check is off in Settings. A click on "Not checked" also checks the feeds that are not checked (ADR 0261 items 17 to 19). During that check, the mark is a spinner with the text "Checking".

16. A tree row covers its feed and each release that it shows, also when the row is open. It does not cover the rows below it, because these rows have their own marks. "No artist yet" and "Other feeds" cover the feeds that they hold (ADR 0254 item 6a).

17. Each mark has a tooltip and an accessible name. The accessible name of the row adds the state. The shape of each mark is different, so the color is not the only sign.

18. After the feed list loads, Studio checks each feed with a public address: releases, artist pages, label pages, publisher pages and other feeds. Each open of Releases checks again (ADR 0165 item 5). Items 6, 7, 10 and 11 continue to apply. At most 4 reads run at one time. No worker runs, and no request goes with the check off.

19. One store in the shell keeps the results. The left panel and the tiles of Releases read the same results.

20. The response of `GET /feeds/{id}/outside` gives `read`. It is true when the read of the hosted file succeeded in this request.

21. Each draft answer of the core gives `draft_state`, with the same value as the feed list. The UI puts this value in the row of that feed in the list. Thus the pencil shows or goes after each save, with no new read of the list. The core stays the source of the state (ADR 0065).

22. The feed list loads again when the window gets the focus again. Thus a change from a different window shows. At most one load runs in 30 seconds.

23. A subsequent ADR decides the button "Check Everything" of the Releases page, the check of the media files and the report of the results. That ADR needs a mockup of the report first.

## Consequences

- Each open of Releases sends one read for each feed, and each host can see these reads in its log.
- Each open of a feed page sends one request to musicindex with the feed GUID.
- A file host that changes the bytes of a file can give a false "Changed outside Studio". The comparison uses the file bytes, so a change of a header does not count.

- Each start of Studio sends one read for each feed with a public address. Each file host can see these reads in its log.
- A function of the UI gives the mark of a set of feeds from the draft states and the results of the check.

## How This Decision Is Checked

Mechanical:

- The mark function gives these results. A more urgent state wins.
    - "outside" for a set with a changed or a moved feed
    - "pending" for a set with a draft that is not written
    - "unchecked" for a set with a feed that Studio did not read correctly
    - no mark for a set of feeds that are up to date
- The set of a publisher row holds the feed and each ID of `release_ids`.
- The route gives `read: true` after a correct read. It gives `read: false` when the read does not succeed or the feed has no public address.
- Each draft answer gives `draft_state`: "unchanged" after the open of a written feed, and "changed" after a save that changes it.
- The Node tests and the Rust tests pass.

Visual, from agent screenshots at 1280 px in the dark and the light theme, and at 390 px:

- The marks show between the name and the count, as in the mockup.
- An up-to-date row has no mark.
- A tooltip shows on each mark.
- After an edit of a feed with no draft, the pencil shows on its row with no reload of the list.

ADR 0150 has no check section. Its archived text keeps that record.
This consolidation adds no test or visual pass. The visual checks above remain required.

## Item Map

| Superseded item | New item | Source of the change |
| --- | --- | --- |
| ADR 0150 item 1 | 1 and 18 | ADR 0165 item 5: Studio checks each feed after the list loads and at each open of Releases. |
| ADR 0150 item 2 | 2 | ADR 0211 items 20 and 22: Warning color and triangle. |
| ADR 0150 item 3 | 3 | ADR 0211 items 1 to 5 and 20: pointer file, precedence and Warning color. |
| ADR 0150 item 4 | 4 | ADR 0211 items 6 to 8: one move warning and its actions. |
| ADR 0150 item 5 | 5 | ADR 0184 items 3 to 9, with ADR 0189, ADR 0191 and ADR 0229. |
| ADR 0150 items 6 to 13 | 6 to 13 | None. |
| ADR 0165 item 1 | 14 | None. |
| ADR 0165 item 2 | 15 | ADR 0211 items 20 and 21, and ADR 0230 items 17 to 19. |
| ADR 0165 item 3 | 16 | ADR 0254 item 6a: the tree row and its shown releases. |
| ADR 0165 item 4 | 17 | None. |
| ADR 0165 item 5 | 18 | None. |
| ADR 0165 items 6 and 7 | 19 and 20 | None. |
| ADR 0165 item 8 | 23 | The report and "Check Everything" remain open. |
| ADR 0165 items 9 and 10 | 21 and 22 | None. |
| ADR 0165 "Acceptance Criteria" | "How This Decision Is Checked" | No new pass is recorded. |
