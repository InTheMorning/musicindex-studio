# Studio v1 Task 053: A New Album From An Album

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "MSP 2.0 Parity".
Decisions: [ADR 0077](../adr/0077-a-new-album-can-start-from-an-album.md), [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md), [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md), [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Complete - 2026-10-28, except the human test. The four commit checks pass. The screenshot visual check passed in the two themes and at 390 px. The human test is open.

## Goal

The form that makes a new feed has the select menu "Start from". A selected album gives a new album with each field and each track of the source, and new GUIDs.

## Packets

The packets go in this sequence. 053b needs 053a.

1. [053a: the copy in the core and the route](studio-v1-task-053a-copy-in-the-core.md). Core and API.
2. [053b: "Start from" in the form](studio-v1-task-053b-start-from.md). UI only.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0077.

- The copy uses the draft of the source. When the source has no draft, the route starts one, as `GET /feeds/{id}/draft` does. That draft has no change, so the source shows no change and gets no write.
- The copy is `pipeline::copy_feed`, adjacent to `pipeline::new_feed`. It makes the same kind of draft with no hosted file.
- The UI reads the start value of "Artist" from `GET /api/feeds/{id}/draft` of the selected album.

## Acceptance Criteria

Mechanical: the mechanical criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The form shows "Start from" for the type Album when the collection has an album. It does not show it for the other types.
- After a copy, Studio opens the new album with the tracks of the source.
- The diff of the first write shows the full feed.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: make a new album from an album, replace one track, and write it. The source album does not change.
