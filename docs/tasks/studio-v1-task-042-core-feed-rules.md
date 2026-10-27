# Studio v1 Task 042: The Core Applies Each Feed Rule

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md), [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md), [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md) and [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md).
Audit: [UI feed rule audit](../reviews/2026-10-27-ui-feed-rule-audit.md).

Status: Complete - 2026-10-27. The four commit checks pass. The screenshot visual checks passed, and the [review checklist](../reviews/2026-10-27-task-042-review-checklist.md) records them.

## Goal

The core applies the four feed rules that the audit found in the UI. After this task, a client with no UI gets the same feed as the UI after the same user action. An API call and a connected app are such clients. The UI has no copy of these rules.

## Packets

The packets go in this sequence:

1. [042a: the cover step](studio-v1-task-042a-cover-step.md). It depends on task 040.
2. [042b: the listing routes](studio-v1-task-042b-listing-routes.md). It depends on 042a.
3. [042c: the lossless enclosures and the guard](studio-v1-task-042c-lossless-and-guard.md). It depends on 042b, because its guard test examines the removals of 042a and 042b.

The packets run one at a time, because each one changes `crates/api/ui/editor.js`. Task 041 starts after this task.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0065. The [review checklist](../reviews/2026-10-27-task-042-review-checklist.md) records them.

- The step `keep_cover_form` runs after `number_tracks` in `pipeline::replace_model`. Packet 041a later adds `keep_explicit_form` after it.
- After a save, the UI takes the cover of the saved model through a new function in `cover.js`. `draft.js` has Preact, so a Node test cannot examine it (ADR 0042).
- The draft response gives `hidden` in each route that gives the draft. `useDraft` keeps it as state.
- A listing route for one platform uses the `id` of the block, not its position in the list.
- The core makes the two alternative enclosures in `crates/core/src/feed/enclosures.rs`. The upload facts give them in `alternate_enclosures`, also as an empty list.
- The guard test is the new file `crates/api/ui-tests/feed-rules.test.mjs`.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The cover field after an edit and a save, in three feeds: a DeMu feed, a feed with only `itunes:image` and a feed with both tags. The field shows the new address, and no note about a different address shows. The diff keeps the form of the feed.
- The switch "Hide from apps" and "Advanced", after a hide and after a show. The diff gives the tags of ADR 0063 items 3 and 4.
- An import of a WAV file gives the two alternative enclosures in the track.
