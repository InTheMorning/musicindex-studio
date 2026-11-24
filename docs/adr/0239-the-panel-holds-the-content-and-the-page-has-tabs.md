# ADR 0239: The Panel Holds The Content And The Page Has Tabs

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of the ADR that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0135](../archive/adr/0135-the-panel-holds-the-content-and-the-page-has-tabs.md), The Panel Holds The Content And The Page Has Tabs.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0069](0069-the-outline-head-opens-the-feed-page.md) (the outline head), [ADR 0075](0075-the-publisher-page-has-payments-and-credits.md) (the tabs Value Routes and Credits of a publisher page), [ADR 0157](0157-the-page-tabs-are-a-segmented-control.md) (the look of the tabs), [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) (the tab Check), [ADR 0197](0197-the-tabs-and-the-field-rows-follow-the-editor-column.md) (the select menu of the tabs), [ADR 0206](0206-the-shell-and-the-editor-pages.md) (the outline of an album), [ADR 0225](0225-the-first-tab-of-a-release-is-release.md) (the first tab of a release), [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) (the add rows of the outline).
- Uses: musicindex ADR 0080 R-ACT-1 item 3 and musicindex ADR 0085 R-CASE-1.
- Source: [findings of local use, finding 4](../reviews/2026-11-06-local-use-findings.md), and the mockups in `.studio-dev/mockup-outline/`.
- Changed by: [ADR 0240](../archive/adr/0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) item 7, on 2026-11-21. Item 4: the outline of a publisher feed shows the groups of its role.
- Changed by: [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) item 7, on 2026-11-24. Item 4: the panel shows the tree for a publisher feed, so a publisher feed has no outline.

## Context

- ADR 0135 moved the pages of a feed from the outline of the left panel to tabs on the page of the feed. The outline kept the content: the tracks of an album and the releases of a publisher feed.
- ADR 0157, ADR 0197 and ADR 0225 changed ADR 0135. ADR 0135 had three "Changed by" lines. Global `CLAUDE.md` §Consolidate A Changed Decision permits three, and a subsequent change of the outline of a publisher feed needs this consolidation first.
- ADR 0174 added the tab Check, and ADR 0232 added the add rows of the outline. They did not name ADR 0135 as changed. This ADR restates their result at each item, and names the owner.
- The archived ADR 0135 keeps the problem of 2026-11-06: in an album with 19 tracks, the user scrolled through each track to find "Files".

## Decision

1. The outline of an album holds its head, its tracks and "Add tracks".
2. The page of a feed has tabs below its head. A feed that the editor does not open has no tabs. The tabs are, in this sequence:
   - A release (medium `music`): Release, Live Shows, Value Routes, Credits, Files, Check and Settings. The first tab is "Release" for each kind ([ADR 0225](0225-the-first-tab-of-a-release-is-release.md) item 1).
   - A publisher feed (an artist, a label or a publisher, [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md)): its type word, Value Routes, Credits, Files, Check and Settings.
   - A playlist: Playlist, Tracks, Value Routes, Files, Check and Settings.

   The tab Check comes before Settings, and its label has the count of the findings ([ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) item 17).
3. The outline head opens the first tab. It stays selected on each tab, and it is not selected on a track page.
4. The outline of a publisher feed holds its head and its releases. Its group Releases has the add row **＋ Add release** ([ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) items 1 and 3). A playlist has no track pages, so its outline holds only its head.
5. The tab Live Shows has one row for each show and the button that adds a show. The page of one show keeps this tab. A track page has no tabs.
6. The tabs are a segmented control with the selected look of musicindex ADR 0051 R-SELECT-1 ([ADR 0157](0157-the-page-tabs-are-a-segmented-control.md) items 1 and 2). When a segment cannot hold its longest label on one line, the tabs are one select menu. Then no tab starts outside the column ([ADR 0197](0197-the-tabs-and-the-field-rows-follow-the-editor-column.md) items 1 to 4).
7. The first page of a feed has no row for Value Routes, Credits or Settings. It keeps the Feed facts.
8. The tabs are in `pageTabs` of `crates/api/ui/shell.js`.

## Item Map

| ADR 0135 item | Item here | Change from a subsequent ADR |
| --- | --- | --- |
| 1 | 1 | None. |
| 2 | 2 | ADR 0225 item 1: the first tab of a release is "Release". ADR 0174 item 17: the tab Check. |
| 3 | 3 | None. |
| 4 | 4 | ADR 0232 items 1 and 3: the group Releases and its add row. |
| 5 | 5 | None. |
| 6 | 6 | ADR 0157 items 1 and 2: the segmented control. ADR 0197 items 1 to 4: the select menu when the tabs do not fit their column, not at 760 px. |
| 7 | 7 | None. |
| 8 | 8 | None. |

## Alternatives Considered

- Headings and indentation in the outline. Not chosen: the feed rows stay below a long list of tracks.
- A collapsible track list. Not chosen: the user opens and closes the list to reach a page.
- Live shows in the panel. Not chosen: an empty section takes two rows in each album. A show needs one more selection.
- Tabs that scroll to the side. Not chosen: Credits, Files and Settings start outside the column.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/shell.test.mjs` examines the outline and `pageTabs`. `crates/api/ui-tests/pages.test.mjs` examines the sections.

Visual: a person examines the tabs in both themes and at 390 px, with real feeds.
