# Studio v1 Task 013: The Recipient Picker

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0014](../adr/0014-collection-in-sqlite.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md) and [ADR 0027](../adr/0027-the-recipient-picker.md).
Depends on: [task 012](studio-v1-task-012-value-editor.md).

Status: complete. The [visual check](../reviews/2026-10-23-task-013-visual-check.md) passed.

## Packets

Three packets do this task, one after the other. Each packet has a prompt for a coding model. The planning model examines each packet with the [review checklist](../reviews/2026-10-22-task-013-review-checklist.md) before its commit.

1. [013a: the recipient store and the musicindex routes](studio-v1-task-013a-recipient-store.md): steps 1, 2 and the core of step 4.
2. [013b: the recipient routes](studio-v1-task-013b-recipient-routes.md): step 3 and the API of step 4.
3. [013c: the picker sheet](studio-v1-task-013c-picker-sheet.md): steps 5 and 6.

The picker goes into the Recipients page and the recipients section of each track page of [task 014](studio-v1-task-014-editor-layout.md).

## Goal

Let the user add recipients from a list of used and saved recipients, and from the feeds and tracks in musicindex.

## Steps

1. Add a migration with a table for the recipients of each feed and a table for the saved recipients (ADR 0027 items 4 and 5).
2. Keep the recipients of each feed and of its tracks after each read and each write.
3. Add API routes: list the recipients of the collection and the saved recipients, save a recipient, and remove a saved recipient.
4. Add an API route that gives the `payment_routes` of a musicindex feed or track (ADR 0027 item 6).
5. Add the picker to the recipients of the album and of each track. It shows the recipients with checkboxes, and adds each selected recipient after the recipients of the editor (ADR 0027 item 7).
6. Let the user save a recipient from the editor, and remove a saved recipient.

## Constraints

- The constraints of [task 011](studio-v1-task-011-album-editor.md) apply.
- Tests use a local test server for the musicindex API. They use no network.

## Acceptance Criteria

Mechanical:

- A database at the previous schema version opens and gets the new tables.
- After a read, the list has each recipient of the feed and of its tracks one time, by address and custom pair.
- When no feed has a recipient, the recipient goes out of the list. A saved recipient stays until the user removes it.
- The musicindex route gives the routes of a feed and of a track from the test server.
- The database file contains no secret after these steps.

Visual:

- The operator adds two recipients from the list to a track, and the recipients of the editor stay.
- The operator imports selected recipients from a musicindex track, and only the selected ones are added.
- The operator saves a recipient, and it is in the list of a different feed.
