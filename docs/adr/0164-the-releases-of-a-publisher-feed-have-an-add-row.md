# ADR 0164: The Releases Of A Publisher Feed Have An Add Row

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire. The operator selected the drawing "Row in Releases".
- Changes: [ADR 0119](0119-a-new-user-starts-as-a-musician-or-a-label.md) item 9.
- Uses: [ADR 0041](0041-publisher-feeds-and-two-way-links.md) for the two-way link and its notices. [ADR 0119](0119-a-new-user-starts-as-a-musician-or-a-label.md) item 8 for the first-step rows. [ADR 0007](0007-draft-diff-write-and-announce.md) for the write.

## Context

ADR 0119 item 8 gives a publisher feed with no release one large first-step row. Since the correction of item 3 of the [findings of 2026-11-10](../reviews/2026-11-10-local-use-findings.md), that row does not show for a feed with a release.

Thus an artist with releases had no action on its page that makes a new album. The section **Releases** only linked an album of the collection. The operator asked where an artist makes a new album.

The first-step row also did not link the new album to the artist. The form sent no publisher feed, so the album was not a release of the artist.

## Decision

1. The section **Releases** of a publisher feed has an add row at its end, below the releases. The row shows when the feed has a file host and has no first-step row of ADR 0119 item 8.
2. The words of the row come from the first role of the feed:
    - artist: **＋ Add an Album**. The form starts with the name of the artist in **Artist**.
    - each other role: **＋ Add a Release**. The form has no start value.
3. The row opens the same form as the first-step row. When the form makes an album or a playlist, Studio links it to this publisher feed with the two-way link of ADR 0041. Then the page of the new release opens. This applies also to the first-step rows, and it changes ADR 0119 item 9.
4. When the link fails, the publisher page stays open and shows the error. The new release stays in the collection, and the field **Release** offers it.
5. The link changes the two drafts, as each link of ADR 0041 does. The user writes them through Review (ADR 0007).

## Alternatives Considered

- **A button in the toolbar of the publisher page.** Not chosen: the toolbar holds the actions of the draft, not the content of a section.
- **The large first-step row for each artist.** Not chosen: the row is a first step. It is large so that a new user sees it, and it takes space from the releases.
- **An add action on the group view of the left panel.** Not chosen: the group view shows the releases of the collection, not the page of the feed.

## Consequences

- One function of the UI gives the add row: its words and its start value, or no row.
- A new album of the artist is a release of the artist before its first write.

## Acceptance Criteria

Mechanical:

- The function gives "Add an Album" with the artist name for an artist with a release.
- The function gives "Add a Release" with no start value for a label with a release.
- The function gives no row for a feed with a first-step row, and for a feed with no file host.
- The Node tests pass.

Visual, from agent screenshots at 1280 px in the dark theme, and at 390 px in the light theme:

- The add row is at the end of the section **Releases**, in the style of the other add rows.
- The row opens the form, with the artist name for an artist.
- After **Make the Release**, the page of the new release opens, and the publisher page lists it.
