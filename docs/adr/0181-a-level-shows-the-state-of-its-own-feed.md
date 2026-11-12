# ADR 0181: A Level Shows The State Of Its Own Feed

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through a drawing in the questionnaire of the human checks.
- Technical choices: items 2 and 4 are choices of the planning model. The operator selected the design for an artist. Item 2 applies it to each level.
- Changes: [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 5, the content area of a level.
- Uses: [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) items 1 to 4 for the marks.
- Source: finding 6 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, the row "HeyCitizen" in the left panel showed the pencil and the number 4. The operator read it as "4 changes not written". The number is the count of releases. The pencil came from a draft of the artist feed "HeyCitizen".

The operator opened the level. Each of the four release tiles showed "No changes". The head row "HeyCitizen · Artist" had no pencil, but the page of the artist feed showed "Saved, not written". Thus the open level did not show the mark of the feed with the draft, as ADR 0165 item 2 gives. The user could not find the change.

## Decision

1. In an open level, the head row shows the mark of the publisher feed of the level only. A release row of the level does the same for its feed (ADR 0165 item 3).
2. The content area of a level shows the publisher feed of the level as the first tile, above the releases. This applies to an artist, a label and a hosting feed.
3. The tile shows the state of the feed with the same text and marks as a release tile. Examples are "No changes" and "Saved, not written".
4. A click on the tile opens the feed page, as the head row does. The tile has the accessible name of the feed, with its type and its state.
5. The row of a closed level keeps the mark that covers its feed and its releases (ADR 0165 item 3).

## Alternatives Considered

- Correct only the head row, and keep the content area as it is. Not chosen by the operator: the drawing that the operator selected also has the tile.

## Consequences

- From the pencil of a closed level, the user opens the level. The left panel and the content area then show the feed with the draft.

## Acceptance Criteria

Mechanical:

- A Node test: the mark of the head row of an open level comes from the draft state of the level feed only. A release with a draft does not give the head row a pencil.
- A Node test: the tile model of a level gives the level feed first, with its state, then the releases.

Visual, by the planning model:

- An artist level with a draft of the artist feed only, at 1280 px and at 390 px, in the light and dark themes. The head row has the pencil, and the first tile shows "Saved, not written".

Human, by the operator. This gate stays open until it runs:

- None. The screenshots of the planning model prove the change.
