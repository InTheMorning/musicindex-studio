# Studio v1 Task 045: The Page Sequence And The Artwork Sheet

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0068](../adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md), [ADR 0070](../adr/0070-a-cover-opens-the-artwork-sheet.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md).

Status: Complete - 2026-10-27. The four commit checks pass. The screenshot visual checks passed, and the [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records them.

## Goal

A click on a cover opens the Artwork sheet of its item, and each artwork field is only in that sheet. Each editor page shows its sections and fields in the sequence of ADR 0068, from one model module.

## Packets

The packets go in this sequence:

1. [045a: the Artwork sheet](studio-v1-task-045a-artwork-sheet.md). It depends on task 044.
2. [045b: the page sequence](studio-v1-task-045b-page-sequence.md). It depends on 045a, because its lists contain the row "Artwork".

The packets run one at a time, because each one changes `crates/api/ui/editor.js`.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0068 and ADR 0070. The [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records them.

- `EditableFeed` in `app.js` keeps the state of the open sheet: `null`, `{ kind: 'feed' }` or `{ kind: 'track', key }`. `AlbumEditor` shows the sheet, because it has `update`.
- The artwork summary is the new function `artworkSummary` in `artwork.js`. The sheet is the new component `ArtworkSheet` in `editor.js`, near `ArtworkList`.
- "Use the Album Cover" sets the `image` of the track to `null`, as a clear of the track image field does now.
- The page sequence is the new model module `crates/api/ui/pages.js`. Each page shows its sections through a map from the section ID to a render function.
- The group "More" of the track page holds the fields "Publication date" and "Explicit", then the row "Identity".
- The publisher page gets the row "Feed facts", so `PublisherPage` gets `onRemoved`.
- The summary of the row "Feed settings" starts with the language, for example "Language en".

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`. Each one in the dark theme, in the light theme and at 390 px:

- A click on each cover of the album page and the track page opens the sheet of the correct item. Tab reaches each cover, and Enter opens the sheet.
- The sheet shows the parts of ADR 0070 item 5 in their sequence.
- The album page, the publisher page, the playlist page, the settings page and a track page show the sequence of ADR 0068.
- `$edges()` finds no content at the edge of a group.
