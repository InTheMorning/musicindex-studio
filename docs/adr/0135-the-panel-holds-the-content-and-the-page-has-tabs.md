# ADR 0135: The Panel Holds The Content And The Page Has Tabs

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) item 6, [ADR 0041](0041-publisher-feeds-and-two-way-links.md) item 3, [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 3, [ADR 0069](0069-the-outline-head-opens-the-feed-page.md) item 3 and the rows of the details sections in [ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) and [ADR 0075](0075-the-publisher-page-has-payments-and-credits.md)
- Uses: musicindex ADR 0020 R-ACT-1 item 3 and musicindex ADR 0023 R-CASE-1
- Source: [findings of local use, finding 4](../reviews/2026-11-06-local-use-findings.md), and the mockups in `.studio-dev/mockup-outline/`

## Context

The outline of an album mixed its tracks with the pages of the feed. In an album with 19 tracks, the user had to scroll through each track to find "Files". Only "Live shows" had a heading, so the rows below it looked like parts of that group.

Published documents of other products show a list of content in a column, and tabs for each level of settings on the edit page. The operator selected this direction from mockups on 2026-11-06.

## Decision

1. The outline of an album holds its head, its tracks and "Add tracks".
2. The page of a feed has tabs below its head. An album has Album, Live Shows, Value Routes, Credits, Files and Settings. An artist or label has its type word, Value Routes, Credits, Files and Settings. A playlist has Playlist, Tracks, Value Routes, Files and Settings. A feed that the editor does not open has no tabs.
3. The outline head opens the first tab. It stays selected on each tab, and it is not selected on a track page.
4. The outline of a publisher feed holds its head and its releases. A playlist has no track pages, so its outline holds only its head.
5. The tab Live Shows has one row for each show and the button that adds a show. The page of one show keeps this tab. A track page has no tabs.
6. A selected tab has a tint and an outline, and it is not filled. At 760 px or less, the tabs are one select menu, so that no tab starts outside the window.
7. The first page of a feed has no row for Value Routes, Credits or Settings. It keeps the Feed facts.
8. The tabs are in `pageTabs` of `crates/api/ui/shell.js`.

## Alternatives Considered

- Headings and indentation in the outline. Not chosen: the feed rows stay below a long list of tracks.
- A collapsible track list. Not chosen: the user opens and closes the list to reach a page.
- Live shows in the panel. Not chosen: an empty section takes two rows in each album. A show now needs one more selection.
- Tabs that scroll at a narrow width. Not chosen: Credits, Files and Settings start outside the window.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/shell.test.mjs` examines the outline and `pageTabs`. `crates/api/ui-tests/pages.test.mjs` examines the sections.

Visual: the planning model examines the tabs in both themes and at 390 px. The operator judges the result with real feeds.
