# ADR 0264: Overview Groups The Releases Below Label, Artist And Curator Headings

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25 by the operator. The operator selected the mockup option A2, a heading for each artist and each label, through the orchestrator session. The operator then selected these values in the questionnaire. A label heading has an artist heading below it. The headings follow the Catalog tree, and the newest release is first in a group. A release shows as a tile. Curator headings for the playlists go after the release headings.
- Technical choices: the main session selected items 4, 6 and 8 and the heading names "Independent", "No artist yet" and "No curator yet" of items 1 and 5. The operator did not receive them as a question.
- Bar: opposite choice by mistake. Without this record, an agent can build the one list of ADR 0263.
- Source: the operator choice of 2026-11-25, sent by the orchestrator session.
- Changes: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 5 and 7. Overview shows headings and tiles, not one list of rows. The rows of ADR 0263 item 7 stay on the list pages of ADR 0263 item 9a.
- Uses: musicindex ADR 0098 R-LIST-16 and R-LIST-17. [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 3 and 5 for the sequence of the tree. [ADR 0257](0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) for the one artist page and the one label page of a release. ADR 0263 item 6 for the release date.

## Context

- ADR 0263 gave Overview one list of each album and each playlist, with the newest release date first.
- The operator then selected the option A2 of the mockup `.studio-dev/design-brief/prototype/overview.html`: Overview separates the releases below a heading for each label and each artist. The A2 mockup grouped only by artist and showed tiles. The operator also named the labels.

## Decision

1. Below the filters, Overview shows the release part, then the playlist part. The release part has these headings, in this sequence:
    - A heading for each label page of the collection that holds a release, in the sequence of the label rows of the Catalog tree (ADR 0254 items 3 and 5).
    - The heading "Independent", for the releases with an artist page and no label page.
    - The heading "No artist yet", for the releases with no artist page.
2. Below a label heading and below "Independent", a second heading names each artist page, in the sequence of the artist rows of the Catalog tree. The releases are below that second heading. A release with a label page and no artist page goes below the second heading "No artist yet" of its label.
3. A release goes below the label page and the artist page of its fields (ADR 0257). Each release shows one time in the release part.
4. Below a heading, the newest release date (ADR 0263 item 6) is first. A release with no release date goes after the releases with a date. Releases with the same instant, or with no date, go A to Z.
5. The playlist part has a heading for each curator page with a playlist, in the sequence of the curator rows of the Catalog tree. A playlist that links to two curator pages shows below each one. The heading "No curator yet" holds the playlists of ADR 0263 item 11. Item 4 gives the sequence below a heading.
6. A heading shows the name and the number of the tiles below it. The name of a label, artist or curator heading opens the page of that feed. The headings "Independent", "No artist yet" and "No curator yet" open nothing. The headings are headings for assistive technology, one level for item 1 and item 5, and the next level for item 2.
7. A release or a playlist shows as a tile: the art, the title and the release date. A click on the tile opens the release or the playlist. A tile has no menu **⋯**. The tiles of a heading fill the width of the page, at 1280 px and at 390 px.
8. The filter Releases shows only the release part. The filter Playlists shows only the playlist part. A search (ADR 0263 item 4) shows only the matching tiles. A heading with no tile that shows does not show. When no tile matches, the page shows "Nothing has “{text}”.".

## Alternatives Considered

- One list with the newest release date first (ADR 0263 item 5). Replaced by the operator.
- A heading for each artist only, with the label name on each tile. Not selected by the operator: a label does not group its artists.
- A part "Labels" and a part "Artists". Not selected by the operator: a signed release shows two times.
- The headings A to Z, or the heading with the newest release first. Not selected by the operator: Overview and the left panel then have different sequences.
- Rows in place of tiles. Not selected by the operator.
- One heading "Playlists". Not selected by the operator.

## Consequences

- Overview and the left panel give the labels and the artists in the same sequence.
- A tile shows fewer values than a row of ADR 0263 item 7. The release page gives the type, the number of tracks and the state of the draft.

## Open Points

- None. musicindex ADR 0098 R-LIST-16 adds the R-LIST-9 row "A catalog grouped by label, artist and curator, in an editor", and Overview is a list of that row. R-LIST-17 gives the filters with no count.

## How This Decision Is Checked

Mechanical:

- A Node test of the Overview model: the headings and the second headings of items 1 and 2 in the sequence of the tree. The test also has a release with no label, a release with no artist, and a release with a label and no artist. Each release shows one time (item 3).
- A Node test: the sequence below a heading (item 4), a playlist with two curators and "No curator yet" (item 5), the count of a heading (item 6).
- A Node test: the filters and the search hide the parts and the empty headings, and the empty text (item 8).

Visual (the main session with screenshots, then the operator):

- Overview at 1280 px and at 390 px, in Light and in Dark, with label, artist and curator headings and tiles.
