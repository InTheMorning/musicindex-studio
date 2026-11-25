# ADR 0266: A Playlist Shows Below Each Page That Its Curator Link Names

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25 by the operator: "Overview shows playlists under their parent page. Parent page can be artist, label, publisher or other, provided stated rel includes curator."
- Technical choices: the main session selected items 2, 5 and 6. The operator did not receive them as a question.
- Bar: opposite choice by mistake. Without this record, an agent can keep the role set of the page as the test.
- Changes: [ADR 0264](0264-overview-groups-the-releases-below-label-artist-and-curator-headings.md) item 5. The playlist part has a heading for each page that a curator link names, not for each page with the role `curator` (items 1 to 3).
- Changes: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 10 and 11. The left panel and "No curator yet" use the same test (items 4 and 5).
- Uses: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) item 3 (how Studio reads `rel`) and item 20 (each link has its own roles). [ADR 0251](0251-review-writes-the-two-sides-of-a-changed-link.md) for the two sides of a link. musicindex ADR 0098 R-LIST-16 for the grouped catalog.

## Context

- ADR 0264 item 5 puts a playlist below each linked page with the role `curator` in its role set. The `rel` of the link has no effect.
- A person can curate a playlist from the artist page or the label page. The `rel` of that link then holds the word `curator`. The playlist shows in "No curator yet", although its feed names its curator.
- The feed list of the core gives `publisher_ids` for each release, but no roles for each link.

## Decision

1. A playlist shows below a page when a link joins the playlist and the page, and the `rel` of that link holds the word `curator` (ADR 0253 item 3). The page can have each role set: artist, label, publisher (`hosting`), curator or a different set.
2. The `rel` of one side of the link is sufficient: the `podcast:remoteItem` in the `podcast:publisher` of the playlist, or the channel `podcast:remoteItem` of the page that names the playlist.
3. The role set of the page has no effect. A page with the role `curator` holds a playlist only through a link of item 1.
4. On Overview, the playlist part has a heading for each page of item 1, in the sequence of the page rows of the Catalog tree. A playlist that two pages hold shows below each one. The other rules of ADR 0264 item 5 do not change.
5. In the left panel, the row of each page of item 1 shows the playlist rows below it (ADR 0263 item 10). The count of a row does not change.
6. "No curator yet" holds each playlist that no page of item 1 holds (ADR 0263 item 11). A playlist that links to an artist page with no `curator` in the `rel` is in "No curator yet".
7. The core feed list gives, for each release, the roles of each link adjacent to `publisher_ids`: `publisher_link_roles`, a list of role lists in the sequence of `publisher_ids`. The roles of a link are the roles of the `rel` of its two sides together. The UI reads this value and does not read `rel`.

## Alternatives Considered

- The role set of the page (ADR 0264 item 5). Replaced by the operator.
- The `rel` of the two sides together as a condition, so that each side must hold `curator`. Not selected: a link that is in one draft only also counts as a link (the `catalogs` function of `crates/core/src/publisher.rs`).

## Consequences

- An artist page or a label page can hold its own playlists on Overview and in the left panel.
- The UI gets the roles of each link. A subsequent decision can use them for the release part.

## How This Decision Is Checked

Mechanical:

- A core test: the feed list gives `publisher_link_roles` from the `rel` of one side of the link, and from the two sides together (items 2 and 7).
- A Node test of the Overview model: a playlist shows below an artist page with a curator link (items 1 and 4). A playlist on a curator page with no curator link is in "No curator yet" (items 3 and 6). A playlist with two curator links shows two times (item 4).
- A Node test of the panel model: the playlist rows below an artist row with a curator link (item 5).

Visual (the main session with screenshots):

- Overview and the left panel at 1280 px, with a playlist below an artist page.
