# Studio v1 Task 258: A Curator Link Holds A Playlist

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decision: [ADR 0266](../adr/0266-a-playlist-shows-below-each-page-that-its-curator-link-names.md) items 1 to 7. Read [ADR 0264](../adr/0264-overview-groups-the-releases-below-label-artist-and-curator-headings.md) item 5 and [ADR 0263](../adr/0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 10 and 11 first.

Status: built. The screenshot check of Overview and the left panel passed in Light and in Dark at 1280 px.

Operator choices (the main session):

- `catalogs_with_kept` uses the existing `link_roles` function of `crates/core/src/publisher.rs`. That function already joins the `rel` of the two sides of a link.
- The release part of Overview keeps the role set of the page (ADR 0264 items 1 to 3). Only the playlist part uses `publisher_link_roles`.

## Goal

A playlist shows below each page that a link with `curator` in its `rel` names. This applies on Overview, in the left panel and in "No curator yet". The role set of the page has no effect.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Catalog of a feed | extend | crates/core/src/publisher.rs | `Catalog` gets `link_roles: Vec<Vec<PublisherRole>>` in the sequence of `publishers`. `catalogs_with_kept` fills it from `roles_from_rel` of the `podcast:publisher` item of the release and of the channel `remoteItem` of the page that names the release, together, with no duplicate and in the sequence of `PublisherRole::ALL`. A kept link (ADR 0232 item 10) with no `rel` on each side gives an empty list. |
| Feed list | extend | crates/api/src/routes.rs | Adjacent to `publisher_ids` (near line 1559), add `publisher_link_roles` from `catalog.link_roles`, with the `rel` words of each role. |
| Pages of a role | extend | crates/api/ui/shell.js | Add one function that gives the pages of a playlist whose link holds `curator`, from `publisher_ids` and `publisher_link_roles`. A missing `publisher_link_roles` gives no page. Use it in `unlinkedPlaylists` and in the playlist part of `overviewModel` in place of `pagesWithRole(feed, byId, 'curator')`. Keep `pagesWithRole` for the release part. |
| Panel playlist rows | extend | crates/api/ui/shell.js | Near line 798: the playlist rows of a page row are the playlists of `release_ids` that have a curator link to that page. Use the function of the row above. Remove the condition on the `curator` role of the page. Keep the count rule of the row (task 256 choice for packet 256c). |
| Core tests | extend | crates/core/src/publisher.rs | The unit tests of `catalogs`, or the tests that already call `catalogs`. |
| API tests | extend | crates/api/src/tests.rs | Near the tests that read `listed(&app, ...)["publisher_ids"]`. |
| Node tests | extend | crates/api/ui-tests/shell.test.mjs | Change the fixtures near line 1189 and line 1242: each playlist link gets `publisher_link_roles`. Add the tests of the acceptance criteria. |

## Constraints

- No `rel` parse in the UI (ADR 0065). The UI reads only `publisher_link_roles`.
- A read and a write with no edit give the same bytes. This task changes no feed file.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A test message names the ADR item and the fix, for example: `ADR 0266 item 1: a playlist with a curator link to an artist page must show below that page. Use the curator pages function of crates/api/ui/shell.js.`
- Do not commit. Leave the change in the working tree and report it.

## Acceptance criteria

Mechanical:

1. A core test: the catalog of a release gives the roles of the `rel` of the release side only, of the page side only, and of the two sides together (ADR 0266 items 2 and 7).
2. An API test: the feed list gives `publisher_link_roles` in the sequence of `publisher_ids`.
3. A Node test: on Overview, a playlist with a curator link to an artist page shows below that page (items 1 and 4).
4. A Node test: a playlist on a page with the role `curator` and a link with no `curator` is in "No curator yet" (items 3 and 6).
5. A Node test: a playlist with two curator links shows below each page (item 4).
6. A Node test: the panel row of an artist page with a curator link shows the playlist row (item 5).
7. `tools/gate.sh` passes.

Visual, by the main session:

- Overview and the left panel at 1280 px, with a playlist below an artist page.
