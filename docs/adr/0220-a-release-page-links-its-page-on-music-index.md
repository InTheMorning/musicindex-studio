# ADR 0220: A Release Page Links Its Page On Music Index

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. The operator selected "Header, when held".
- Technical choices: the main session selected items 2, 4 and 5.
- Bar: crosses repositories. The address form belongs to musicindex ADR 0085 R-LINK-1, and HitIt uses the same form. An agent can also make the opposite choice by mistake: a link that shows before Music Index holds the feed.
- Uses: [ADR 0259](0259-studio-checks-each-feed-and-the-left-panel-marks-its-state.md) for the outside check of the release page. [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) item 5 for a link to another origin. [ADR 0175](0175-the-check-shows-what-music-index-holds.md) item 1 for the reads of Music Index.
- Source: [analysis of two journeys for a new independent artist](../research/2026-11-18-user-journeys/analysis.md), theme T1 and observation O8.

## Context

- **O8.** No control opens the release on musicindex.org. HitIt has such a link (`HitIt/src/view_models/musicindex_link.rs:9`).
- The address of a feed page is `https://musicindex.org/#feed={podcast:guid}`. Studio already makes it with `musicindexFeedUrl` (`people.js:2496`), and the People page shows it as "Open on musicindex.org" (`peoplepages.js:822`).
- Right after the first write, Music Index does not hold a new feed. A link at that time opens an empty page.
- The release page asks Music Index when it opens and after each write. The route `GET /feeds/{id}/outside?index=true` (`routes.rs:1856`) asks the index for the feed. Its answer does not tell if the index holds the feed.

## Decision

1. The header of a release page shows the link "Open on musicindex.org" under the feed address when Music Index holds the feed.
2. The fact comes from the outside check of the release page. Its answer gets the field `held`. It is `true` when the index holds the feed, `false` when the index does not hold it, and `null` when the route did not ask the index or the read failed. The link shows only with `true`.
3. The link address and the link text are the same as on the People page (Context). A feed with no `podcast:guid` shows no link.
4. The link opens in the system browser (ADR 0124 item 5).
5. Studio sends no new request for the link. When the outside check is off in Settings, the link does not show.

## Alternatives Considered

- A button in the result panel after a write. Not selected. For a new release, the page can be empty for some minutes.
- The header link and the result panel button. Not selected. Two controls for one action.
- A link that always shows. Not selected. It can open an empty page.

## Consequences

- An artist opens the release page on Music Index from Studio when the page exists.
- After the first write, the link shows when the outside check of the release page finds the feed in the index.

## How This Decision Is Checked

Mechanical:

- An API test examines `held` for a held feed, a feed that is not held, and a request with no `index=true`.
- A UI test examines the link for a GUID, and no link with `held` other than `true` or with no GUID.

Visual, by screenshots of the main session before the commit:

- The release page header with the link at 1280 px and 390 px.
