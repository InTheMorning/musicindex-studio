# Studio v1 Task 256: Overview And The Sequence Of A Page

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0263](../adr/0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 1 to 18. [ADR 0264](../adr/0264-overview-groups-the-releases-below-label-artist-and-curator-headings.md) items 1 to 8 replace ADR 0263 items 5 and 7 on Overview.
Mockup: `.studio-dev/design-brief/prototype/overview.html`. Git ignores it.

Status: Packets 256a, 256b and 256c are built. Packet 256d is not started.

Operator choices (the main session, in the approved ADR 0263):

- The name of the outline button of item 18 is "‹ Sections". This value is open in ADR 0263. The packets use it.
- Packet 256a: a new playlist gets its channel `pubDate` when Studio makes its first draft, not at its first file write. The draft is the first save, and a later write keeps the value.
- Packet 256a: the channel `pubDate` is a field of the feed model, so Review shows a change of it. A new link on the `podcast:publisher` side stays last, because `sort_publisher_links` gives that sequence.
- Packet 256b: the feed list gives no role for each link of a release. Overview puts an album below its first linked page with the role `artist` and its first other linked page with the role `label`, in the sequence of the tree.
- Packet 256b: the group page of a Catalog row (`GroupPage`) keeps its sort until packets 256c and 256d replace it, so the old sort code stays until then.
- Packet 256b: Overview tiles have a minimum width of 140 px, so a phone shows two tiles on each line.
- Packet 256c: the count of a row is the number of its playlists only when the first level role of the feed is curator (ADR 0253 item 15). A page with the roles `artist curator` keeps its release count.
- Packet 256c: the playlist rows below a curator row come before its child page rows. A playlist row is not the current row at any time, because a playlist opens its release page.

## Goal

Overview is the start page and shows the full collection as tiles below label, artist and curator headings. The Catalog shows each playlist below its curator page, and the list pages "No artist yet", "No curator yet" and "Other feeds". The group Releases of a page has one sequence, the sequence of its feed, with a drag handle and the moves **Move up**, **Move down** and **Move to the top**.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed list of the core | extend | crates/core/src/collection.rs | `FeedRow` and `Collection::feeds` get `release_date` (item 6). |
| Feed reader | extend | crates/core/src/feed.rs | `Feed::read` reads the channel `pubDate` (item 6). Keep `pub_date_time` of an item. |
| Feed list route | extend | crates/api/src/routes.rs | `list_feeds` gives `release_date`. |
| Link write of a page | extend | crates/core/src/publisher.rs | `upsert` puts a new release link before each other release entry (item 15). |
| Panel model | extend | crates/api/ui/shell.js | `leftPanel` and `groupDestinations` get Overview, the playlist rows and the three list rows (items 9 to 11). |
| Release list model | extend | crates/api/ui/shell.js | `releasesList` becomes the Overview model with the headings of ADR 0264, and the list page model of ADR 0263 item 9a. The sorts, the reversed sequence and the letters go. |
| Navigation | extend | crates/api/ui/shell.js | `startNav`, `narrowStack`, `rootNav`, `listQuery` and `queryText` (items 1, 8 and 17). |
| Releases page | extend | crates/api/ui/app.js | `ReleasesPage` becomes `OverviewPage`. Reuse `SearchField` and the tile markup of the old view "Tiles" (class `releases-tile`). `SortMenu`, `LetterBar` and the view buttons go. |
| Welcome check | reuse | crates/api/ui/welcome.js | `showWelcome` (item 1). |
| Segmented control | reuse | crates/api/ui/app.js | The keys of a segmented control (line 2389, musicindex ADR 0051 R-SEG-1) and the page tabs. Do not write a new one. |
| Sequence helpers | reuse | crates/api/ui/order.js | `moveTo` and `moveAmong` (items 13 and 13b). |
| Draft save | reuse | crates/api/ui/draft.js | A move is one draft edit (item 14). |
| Undo history | reuse | crates/api/ui/undohistory.js | The draft save records the operation now. Do not add a second record. |
| Drag handle and insertion line | extend | crates/api/ui/app.js | The handle and the marks of task 254 packet 254d (commit e4300e1 and e047874), for a row in a list. |
| Page actions | extend | crates/api/ui/movepages.js | A release row menu with the three moves (item 13). |
| Node tests | extend | crates/api/ui-tests/shell.test.mjs | Panel, Overview and list page models. Also `narrow-root.test.mjs`, `release-search.test.mjs` and `welcome.test.mjs`. |
| Core tests | extend | crates/core/src/feed.rs | `release_date` and the channel `pubDate`. |

## Constraints

- Each rule of the data is in the core (ADR 0065). The UI does not calculate the release date.
- A move does not go into the change set of the Catalog (item 14).
- A row of a list has one labeled line for each type of value.
- Each button has an accessible name. The drag has a menu alternative for the keyboard and the phone (item 13).
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A UI module passes `node --check` on a `.mjs` copy.
- A test message names the ADR item and the fix.
- The agent does not commit and does not stage. The main session reads the diff, takes the screenshots, runs the gate and commits.

## Files To Read First

- ADR 0263. ADR 0254 items 2, 4a, 5 and 34. ADR 0081 items 1 and 5. ADR 0227 items 1 to 5. ADR 0259 item 14. ADR 0223 item 5.
- The mockup `.studio-dev/design-brief/prototype/overview.html`.
- `crates/api/ui/shell.js`: `leftPanel` (line 318), `groupDestinations` (696), `releasesList` (912), `compareNewest` (830), `listQuery` (1013), the navigation functions (1037 to 1180).
- `crates/api/ui/app.js`: `LeftPanel` (3639), `DestinationRow` (3699), `destinationRows` (3761), `SortMenu` (3977), `ReleasesPage` (4005), the navigation record (4265 to 4280), the button "‹ Overview" (2563).
- `crates/core/src/feed.rs` (`Feed::read`, line 159; the item dates, line 663), `crates/core/src/collection.rs` (`FeedRow`, line 1153; `feeds`, line 2190), `crates/core/src/publisher.rs` (`upsert`, near line 1280).

## Packets

### 256a: The Release Date And The First Place Of A New Link (core and API)

1. `Feed::read` reads the channel `pubDate`. `Feed::write` writes it when it has a value. A new playlist of Studio gets a channel `pubDate` at its first write (item 6).
2. The feed list gives `release_date` for each album and each playlist. Read it from the draft when the feed has one, else from the last file that Studio read. An album: the earliest item `pub_date_time`. A playlist: the channel `pubDate` as an RFC 3339 time. Else `null`. Use the date parse of ADR 0093 that `pub_date_time` uses.
3. `list_feeds` gives `release_date` in its JSON.
4. `upsert` in `publisher.rs`: a new release link goes before each other release entry of the channel. An entry that is not a release link keeps its position (items 13b and 15). A changed link keeps its position.
5. Tests: `release_date` of an album with two dates, of a playlist with a channel `pubDate`, of a feed with no readable date, and from a draft. A new link goes first. An API test for `release_date` in `crates/api/src/tests.rs`.

### 256b: Overview (UI)

1. The panel gives Overview first, with the icon of Releases, then the heading "Catalog" (items 1 and 9). The start page is Overview, or "Welcome to Studio" with no feed.
2. `OverviewPage` replaces `ReleasesPage`: the head with **＋ Add**, the counts line, the notice rows, the search field "Search the collection" and the filters All, Releases and Playlists (ADR 0263 items 3 and 4).
3. Below the filters, the release part and the playlist part of ADR 0264 items 1 to 6. The model gives the headings in the sequence of the tree, from the label page and the artist page of each release. Below a heading, it sorts by `release_date`: newest instant first, then no date, then A to Z for equal values. Reuse the tree sequence of `groupDestinations`. Do not write a second sort of the tree.
3a. The tile of ADR 0264 item 7, with no menu **⋯**. The filters and the search of ADR 0264 item 8.
4. The filter and the search go into the navigation record. The address of Overview has no query. An old address with `sort`, `reversed` or `view` opens Overview with no error (item 8).
5. Each other place that opens `{ name: 'releases' }` opens Overview.
6. Delete `SortMenu`, `LetterBar`, the view buttons, `compareNewest`, `letterGroups` and each sort constant that no entry point reaches.
7. The button "‹ Overview" of a narrow feed page becomes "‹ Sections" (item 18).
8. Node tests: the panel sequence, the counts line with a zero count, each notice row and its target, the tests of ADR 0264 "How This Decision Is Checked", the address with old keys, and "‹ Sections". Delete the tests of the old sorts and views.

### 256c: The Catalog Rows (UI)

1. A curator row with a linked playlist gets the open button. Below it, one row for each linked playlist, in the sequence of the curator feed. A playlist with two curator pages shows below each one. The count of a curator row is the number of its playlists (item 10).
2. "No curator yet" holds each playlist with no link to a curator page. "No artist yet" holds only albums (item 11).
3. "No artist yet", "No curator yet" and "Other feeds" open a list page with the rows of item 7, the sequence of item 5 and the search of item 4, with no filters (item 9a).
4. Node tests for items 9a to 11.

### 256d: The Release List Of A Page (UI)

1. The group Releases of a page has one sequence, the sequence of item 4a of ADR 0254. The select menu "Sort" goes (items 12 and 17).
2. The row of item 16. A curator page shows its playlists in the same way.
3. The menu **⋯** of a row has **Move up**, **Move down** and **Move to the top**. A move that does not move the row is not available (item 13).
4. Wider than 760 px, a row has the drag handle at its left side. During the drag, an insertion line in the accent color shows the drop place (item 12). Reuse the drag state of packet 254d: it starts in the next task, because WebKitGTK stops a drag on a DOM change in `dragstart`.
5. The conditions of item 13a: no handle and no menu when Studio cannot write the page feed, when the group has one row, or for a release that only names the page. Those releases go after the channel releases.
6. A move changes only the release entries of the channel (item 13b), with `moveAmong`. It is one draft edit through the draft save, so the Undo of the bar removes it (item 14).
7. Node tests: the three moves at the first, a middle and the last row; the conditions of item 13a; item 13b with an entry that is not a release; the row model of item 16. A test that a move and its Undo give the old `remote_items`.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes. `cargo test -p studio-core`, `cargo test -p studio-api`, `node --test crates/api/ui-tests/` and `tools/gate.sh` pass.
- `rg "'releases'|SortMenu|LetterBar|‹ Overview" crates/api/ui` gives no line that an entry point reaches.

Visual (the main session with screenshots and `$edges()`, then the operator):

- Overview at 1280 px and at 390 px, in Light and in Dark, with label, artist and curator headings and tiles.
- A drag of a release row with the insertion line, in the release binary.
- A curator row with its playlists, and "No curator yet".
- The menu **⋯** of a release row at 390 px.

## Stop And Ask When

- The release date needs a value that the draft or the last read file does not hold.
- The drag needs a change in `crates/desktop-app/`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-256-overview-and-the-sequence-of-a-page.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (256a, 256b, 256c or 256d). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- docs/, AGENTS.md, crates/desktop/, crates/desktop-app/. Change crates/core/ and crates/api/src/ only in packet 256a.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- cargo test -p studio-core (packet 256a only)
- cargo test -p studio-api (packet 256a only)
- node --test crates/api/ui-tests/
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
