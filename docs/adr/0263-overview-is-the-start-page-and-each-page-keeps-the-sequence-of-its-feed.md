# ADR 0263: Overview Is The Start Page, And Each Page Keeps The Sequence Of Its Feed

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25 by the operator, after the clickable mockup `.studio-dev/design-brief/prototype/overview.html` (Git ignores it). The operator accepted the mockup and selected these values in the questionnaire: A1 one list for Overview, "Move up / down / top" as the only actions of the menu **⋯** of a release row, and C1 the Catalog heading "No curator yet".
- Operator choice, 2026-11-25: a destination Overview replaces Releases. It is the start page, it shows the full collection, and it has a search. A playlist shows below its curator page in the Catalog.
- Operator choice, 2026-11-25: the release list of a page has one standard sequence, which the user can change. It has no select menu "Sort". A row shows the art and the release date.
- Technical choices: the main session selected items 6, 9a, 10, 13a, 13b, 14, 15 and 18, the notice rows of item 3 and the text of each row. An independent review of this ADR asked for these rules. The operator did not receive them as a question.
- Bar: crosses modules. It changes the core feed list, the left panel, the start page, the address and the page of a publisher feed. Without this record, an agent can keep Releases or the sort menu of a page.
- Source: findings 4 and 5 of the [check of task 254](../reviews/2026-11-25-task-254-check.md).
- Changes: [ADR 0258](0258-the-shell-and-the-editor-pages.md) items 1 to 4, 13 and 14. Overview replaces Releases in the panel and as the start page. Overview has no sort and no view, and item 7 gives the full row (items 1 to 8). The narrow stack starts on Overview, and the button "‹ Overview" of a feed page has the name "‹ Sections" (item 18).
- Changes: [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 2, 3, 5, 6, 13, 13a, 16 and 44, the non-goal "A drag of a release row" and the open point on **＋ Add**. The panel gets Overview, the list pages of item 9a and the playlist rows below a curator row (items 9 to 11). The group Releases has one sequence, a drag handle and three moves (items 12 to 17). The menu **＋ Add** is on Overview.
- Changes: [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) item 10. A new link goes first, not last (item 15). The user still changes the sequence on the page.
- Changes: [ADR 0259](0259-studio-checks-each-feed-and-the-left-panel-marks-its-state.md) items 1, 2, 5, 11, 18, 19 and 23. The checks of Releases run when Overview opens, and the row of Overview takes the place of the tile (items 2 and 7).
- Changes: [ADR 0260](0260-the-welcome-page-and-the-first-steps.md) item 1. The Welcome page shows in the place of an empty Overview (item 1).
- Changes: [ADR 0261](0261-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) item 1. The button **＋ Add** is in the head of Overview (item 3).
- Changes: [ADR 0262](0262-a-user-can-export-and-import-an-account.md) item 9. An import into an empty collection is on the Welcome page or on Overview (item 2).
- Changes: [ADR 0155](0155-the-sort-menu-names-both-directions.md) items 1, 3, 4 and 5. Overview and the group Releases of a page have no sort menu, no reversed sequence and no view icons (items 5 and 17). The sort "Feed Order" is not in a menu.
- Changes: [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) items 3 and 7. The page of an artist or a label has no sort. Its one sequence is the sequence of the feed (items 12 and 17).
- Changes: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) item 4. The group Releases has no sort menu, so the name "Page Order" does not show (item 12).
- Changes: [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 5, 6 and 8. The search is on Overview, with the text of item 4. The narrow stack starts on Overview.
- Changes: [ADR 0226](0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) item 7. Overview comes first on the root page of a narrow window (item 1).
- Changes: [ADR 0072](0072-add-a-feed-opens-from-releases.md) items 1, 2 and 5. Overview is the top row of the panel, and the Add feeds page is a page of Overview (items 1 and 2).
- Changes: [ADR 0256](0256-a-release-starts-on-the-page-of-its-artist.md) item 1. A release starts from **＋ Add** on Overview or on an artist page (item 2).
- Changes: [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) item 2. Overview, not Releases, lists each album and each playlist. "No curator yet" follows "No artist yet" (items 9 to 11).
- Changes: [ADR 0092](0092-file-hosts-and-the-destination-icons.md) item 5. Overview has the icon of Releases (item 1).
- Changes: [ADR 0172](0172-a-list-is-one-tab-stop.md) item 3. Overview has no tiles view (item 5).
- Changes: [ADR 0250](0250-studio-corrects-the-type-of-a-feed-with-a-bad-or-missing-medium-tag.md) item 8. A feed with a corrected type shows in Overview (item 2).
- Changes: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) item 9. The type label shows in the list of Overview (item 7).
- Changes: [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 5 and 11. Studio opens Overview after Remove from Collection (item 2). The button that opens the outline page has the name "‹ Sections" (item 18).
- Changes: [ADR 0157](0157-the-page-tabs-are-a-segmented-control.md) item 4. Releases has no view icons now. The filters of Overview are a segmented control (item 3).
- Uses: musicindex ADR 0080 R-LIST-2 and R-LIST-3 for the drag handle and the insertion line of item 12. [ADR 0227](0227-undo-and-redo.md) for Undo. [ADR 0207](0207-recipients-change-sets-and-wallets.md) for a change set.

## Context

- Before this ADR, the left panel started with the destination Releases above the heading "Catalog". Releases listed each album and each playlist, with the sorts "Newest" and "A to Z" and the views rows and tiles.
- The sort "Newest" used `added_at`, the time that Studio added the feed (`crates/api/ui/shell.js` `compareNewest`). It was not the release date. The feed list of the core gave no release date.
- The group Releases of an artist page had the select menu "Sort" with "Feed order", "Newest" and "A to Z". Its menu **⋯** gave only "Open", which a click on the row also does. A row showed no art and no release date.
- In the check of task 254, the operator found that Releases had low value and that the release list of a page was more useful.
- A playlist showed only in Releases. A curator page showed no playlist in the Catalog.

## Decision

### Overview

1. The destination Overview replaces the destination Releases. It is the first row of the left panel, before the heading "Catalog", and it is the start page. A user with no feed in the collection sees the page "Welcome to Studio" in its place. Overview has the icon of Releases.
2. Each current rule for the page Releases or the destination Releases applies to Overview. This ADR can give a different rule. Examples are the button **＋ Add**, the feed checks when the page opens, and the import of an account into an empty collection. ADR 0069 items 6 to 8 stay replaced by ADR 0254 item 7: the first row of an outline is "‹ Catalog".
3. The page shows, from the top:
    - The head: the title "Overview" and the button **＋ Add**.
    - One line of counts, for example "1 publisher · 1 label · 1 artist · 1 curator · 4 releases · 3 playlists". In this ADR, a release is an album, and a playlist is not a release. A count of zero does not show.
    - The notice rows. Each row shows only when its count is more than zero, and it opens the place that it names. "{n} releases have no artist yet." opens the list page "No artist yet" (item 9a). "{n} playlists have no curator yet." opens the list page "No curator yet". "{n} pages and releases are not written." counts each feed of the collection with the state "Saved, not written" (ADR 0259). It opens the first of these feeds: first the pages in the sequence of the tree, then the rows in the sequence of item 5.
    - The search field of item 4.
    - The filters All, Releases and Playlists, as a segmented control. All is the default.
    - The list of item 5.
4. The search field has the placeholder "Search the collection". A row shows when its title, or the name of one of its artist, label or curator pages, has the typed text. The search ignores capital letters. When no row matches, the list shows "Nothing has “{text}”.". This is the search of ADR 0223 item 5 on a new page.
5. The list holds each album and each playlist of the collection, in one sequence: the newest release date first (item 6). A row with no release date goes after the rows with a date. The compare uses the instant of the date. Rows with the same instant, or with no date, go A to Z. The list has no select menu "Sort", no reversed sequence, no view icons and no letter bar.
6. The core gives the release date of each album and each playlist in the feed list, as `release_date`. The core reads it from the draft when the feed has one, else from the last file that Studio read. The release date of an album is the earliest item `pubDate` that the core can read (`pub_date_time`, musicindex R-DATE-1 item 1). The release date of a playlist is its channel `pubDate`, which the core reads for this value. When no date can be read, the value is `null`. A new playlist of Studio gets a channel `pubDate` at its first write.
7. A row shows the art, the title and the artist or the curator. It also shows the type label, the number of tracks, the release date and the state of the draft ([ADR 0259](0259-studio-checks-each-feed-and-the-left-panel-marks-its-state.md)). A click on the row opens the release or the playlist. A row has no menu **⋯**.
8. Overview keeps the filter and the search in the navigation record of the session. The address of Overview has no query.

### The Catalog

9. The left panel shows, in this sequence: Overview, then the heading "Catalog" with the tree, then "No artist yet", "No curator yet" and "Other feeds". A publisher feed opens from its row in the tree.
9a. "No artist yet", "No curator yet" and "Other feeds" are rows of the panel. Each row opens a list page with the rows of item 7, the sequence of item 5 and the search of item 4. These pages have no filters. The list page replaces the sorts of ADR 0254 item 2 and the search of ADR 0223 item 5 on a feed group.
10. A curator row has the open button of ADR 0254 item 5 when a playlist links to it. Below the open row, the tree shows each playlist that links to that page, in the sequence of item 12. A playlist that links to two curator pages shows below each one. A playlist row has the art, the dot, the title, the type label "Playlist" and the marks of ADR 0259 item 14. It has no drag handle, no count and no button **⋯**. A selection opens the playlist. The count of a curator row is the number of its playlists.
11. "No curator yet" shows only when a playlist has no link to a curator page of the collection. It holds those playlists. A playlist that links only to an artist page or a label page is in "No curator yet". "No artist yet" holds only albums.

### The Release List Of A Page

12. The group Releases of a page has one sequence: the sequence of item 4a of ADR 0254, which is the sequence in the feed file of the page. It has no select menu "Sort". Wider than 760 px, each row has a drag handle at its left side (musicindex ADR 0080 R-LIST-2). A drag changes the sequence. During the drag, an insertion line in the accent color shows the place of the drop (R-LIST-3 item 4.4).
13. The menu **⋯** of a release row has only **Move up**, **Move down** and **Move to the top**. A value that does not move the row is not available, for example **Move up** on the first row. The phone and the keyboard use this menu, because they have no drag.
13a. A row has no drag handle and no menu **⋯** in these conditions: Studio cannot write the page feed, the group has only one row, or the release only names the page and is not in the channel of the page (the second part of ADR 0254 item 4a). Such a release goes after the releases of the channel.
13b. A move changes only the sequence of the release entries of the channel. Each other entry keeps its position (ADR 0081 item 5). An entry that the group does not show, for example a feed that is not in the collection, also keeps its position. **Move to the top** puts the release before each other release entry of the channel.
14. A move is one edit of the draft of the page feed, through the draft save, as the position menu of ADR 0081 does. The Undo of the bar removes it (ADR 0227 items 1 to 5). Review of the page feed shows it in the diff. A move does not go into the change set of the Catalog (ADR 0254 item 34).
15. A new link of a release to a page goes first in the channel of the page. Examples are a new release on its artist page, **Add the Release** in "Linked releases" and a change of the fields of a release (ADR 0257 item 8). The user changes the sequence after that.
16. A row of the group Releases shows the art, the title and the type label. It also shows the number of tracks, the release date of item 6 and the state of the draft. A curator page shows its playlists in the same way, with the same handle and the same menu.
17. The page keeps no sort in the navigation record and in the address.

### The Outline Button Of A Narrow Window

18. On a feed page in a narrow window, the button that opens the outline page has the name "‹ Sections", not "‹ Overview" (ADR 0228 item 11). The word Overview names only the destination of item 1.

## Item Map Of The Changed Rules

The map gives the rule of this ADR for each changed item.

| Changed item | Rule of this ADR |
| --- | --- |
| ADR 0258 items 1 to 4, 13 and 14 | Overview replaces Releases in the panel and as the start page. Overview has no sort and no view, and item 7 gives the full row (items 1 to 8). The narrow stack starts on Overview, and the button "‹ Overview" of a feed page has the name "‹ Sections" (item 18). |
| ADR 0254 items 2, 3, 5, 6, 13, 13a, 16 and 44, the non-goal "A drag of a release row" and the open point on **＋ Add** | The panel gets Overview, the list pages of item 9a and the playlist rows below a curator row (items 9 to 11). The group Releases has one sequence, a drag handle and three moves (items 12 to 17). The menu **＋ Add** is on Overview. |
| ADR 0255 item 10 | A new link goes first, not last (item 15). The user still changes the sequence on the page. |
| ADR 0259 items 1, 2, 5, 11, 18, 19 and 23 | The checks of Releases run when Overview opens, and the row of Overview takes the place of the tile (items 2 and 7). |
| ADR 0260 item 1 | The Welcome page shows in the place of an empty Overview (item 1). |
| ADR 0261 item 1 | The button **＋ Add** is in the head of Overview (item 3). |
| ADR 0262 item 9 | An import into an empty collection is on the Welcome page or on Overview (item 2). |
| ADR 0155 items 1, 3, 4 and 5 | Overview and the group Releases of a page have no sort menu, no reversed sequence and no view icons (items 5 and 17). The sort "Feed Order" is not in a menu. |
| ADR 0153 items 3 and 7 | The page of an artist or a label has no sort. Its one sequence is the sequence of the feed (items 12 and 17). |
| ADR 0231 item 4 | The group Releases has no sort menu, so the name "Page Order" does not show (item 12). |
| ADR 0223 items 5, 6 and 8 | The search is on Overview, with the text of item 4. The narrow stack starts on Overview. |
| ADR 0226 item 7 | Overview comes first on the root page of a narrow window (item 1). |
| ADR 0072 items 1, 2 and 5 | Overview is the top row of the panel, and the Add feeds page is a page of Overview (items 1 and 2). |
| ADR 0256 item 1 | A release starts from **＋ Add** on Overview or on an artist page (item 2). |
| ADR 0090 item 2 | Overview, not Releases, lists each album and each playlist. "No curator yet" follows "No artist yet" (items 9 to 11). |
| ADR 0092 item 5 | Overview has the icon of Releases (item 1). |
| ADR 0172 item 3 | Overview has no tiles view (item 5). |
| ADR 0250 item 8 | A feed with a corrected type shows in Overview (item 2). |
| ADR 0253 item 9 | The type label shows in the list of Overview (item 7). |
| ADR 0228 items 5 and 11 | Studio opens Overview after Remove from Collection (item 2). The button that opens the outline page has the name "‹ Sections" (item 18). |
| ADR 0157 item 4 | Releases has no view icons now. The filters of Overview are a segmented control (item 3). |

## Alternatives Considered

- A2, one shelf for each artist on Overview, with large covers. Rejected by the operator: it needs more space and more scrolling.
- C2, a playlist with no curator page only in Overview. Rejected by the operator: the playlist is then not in the panel.
- The actions "Change artist…", "Change label…", "Hide from apps" and "Remove from this page…" in the menu **⋯** of a release row. Not selected by the operator. The fields of the release page do these changes (ADR 0257).
- The date that Studio added the feed as the sequence of Overview. Rejected: it is not a fact of the music, and it changes after an import.

## Consequences

- The start page shows the full collection and the work that is open.
- The core feed list gets `release_date`.
- The address of the start page loses `sort`, `reversed` and `view`. An old address with these keys opens Overview with no error.

## Open Points

- Studio departs from these shared musicindex rules. The main session tells the orchestrator, and a musicindex record decides if they change:
    - ADR 0053 R-LIST-9, row "Releases": the sorts "Newest" and "A to Z", the reversed sequence and the view icons. Overview has none of them.
    - ADR 0053 R-LIST-9, row "The releases of an artist, a label or a name": the sorts "Newest", "A to Z" and "Page Order", with "Page Order" as the default in an editor, the reversed sequence and the view icons. The group Releases of a page has none of them. R-LIST-8 item 1 agrees with no sort for a list in the sequence of its source.
    - ADR 0053 R-LIST-9, rows "Playlists, public" and "Playlists that a person curates" ("Curator Order"), and R-LIST-9 item 5.
    - ADR 0054 R-LIST-14 item 1: a fixed sequence has a "Reverse" toggle. The group Releases has no toggle.
    - ADR 0085 R-SCOPE-1 items 1, 2 and 6: a playlist is in the scope "Releases", and each scope shows a count. The filter Releases of Overview holds only albums, and the filters have no count.
- The name "‹ Sections" of item 18. The operator can select a different name.
- The section "Linked releases" of a publisher page keeps its position menu (ADR 0081 item 1). The group Releases and the section then give two controls for one sequence. A subsequent decision can remove one.

## How This Decision Is Checked

Mechanical:

- A core test: the feed list gives `release_date` from the earliest readable item `pubDate` of an album. It gives the channel `pubDate` of a playlist, and `null` with no readable date.
- `crates/api/ui-tests/shell.test.mjs`: the view model of the panel gives Overview first, then the tree, "No artist yet", "No curator yet" and "Other feeds" (item 9). The model of a curator row gives its playlists (item 10).
- A Node test of the Overview model: the sequence of item 5, the filters, the search and the notice rows of item 3.
- A Node test of the menu of a release row: the three moves, with the first and the last row (item 13).
- A test that a move changes the draft of the page feed, and the Undo of the bar removes it (items 13b and 14).
- A test that a new link goes first in the channel (item 15).
- A Node test of a curator row and its playlist rows, of "No curator yet", and of the three list pages (items 9a to 11).
- A Node test of the conditions of item 13a, and of the row model of item 16.
- A Node test that the button of item 18 has the name "‹ Sections".
- A test that an address with `sort`, `reversed` or `view` opens Overview.

Visual (the main session with screenshots, then the operator):

- Overview at 1280 px and at 390 px, in Light and in Dark.
- A drag of a release row with the insertion line, in the release binary.
- A curator row with its playlists, and "No curator yet".
- The menu **⋯** of a release row at 390 px.
