# Studio v1 Task 117: A Level Shows The State Of Its Own Feed

Decision: [ADR 0181](../adr/0181-a-level-shows-the-state-of-its-own-feed.md).

Status: Complete on 2026-11-12. The four commit checks are green. The screenshot visual check passed at 1280 px in the light and dark themes, in the list view and the tile view. The level feed uses the view of the releases. At 390 px the level is one stack, so the level feed shows as the head row.

## Goal

In an open level, the head row shows the mark of the level feed only. The first tile of the level shows the level feed and its state. A click on the tile opens the feed page.

## Constraints

- No feed rule in the UI (ADR 0065). The marks come from the draft states that the API gives.
- The row of a closed level keeps the mark that covers its feed and its releases (ADR 0165 item 3).
- Each level gets the tile: hosting, label and artist.

## Files To Read First

- ADR 0181, ADR 0165 items 1 to 4, and ADR 0088 items 5 and 6.
- `crates/api/ui/outsidecheck.js`: the feeds that a row covers, and the mark of a set of feeds.
- `crates/api/ui/app.js`: the marks that the shell gives to the left panel (ADR 0165 items 1 to 4).
- `crates/api/ui/shell.js`: `levelFeed`, the left panel of a level, and the tile model with `DRAFT_LABELS`.
- `crates/api/ui-tests/shell.test.mjs` and `crates/api/ui-tests/outsidecheck.test.mjs`.

## Work

- The head row of an open level gets its mark from the level feed only.
- The tile model of a level gives the level feed first, then the releases. The level tile uses the same state text and marks as a release tile.
- The tile opens the feed page, as the head row does. Its accessible name gives the feed name, its type and its state.
- Add the two Node tests of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0181.

Visual, by the planning model:

- The visual criterion of ADR 0181: an artist level with a draft of the artist feed only, at 1280 px and at 390 px, in the light and dark themes.

Human: none (ADR 0181).

## Stop And Ask When

- The first tile changes the keys of the list of ADR 0172.
