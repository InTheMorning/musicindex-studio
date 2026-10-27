# Studio v1 Task 039: The File Picker And The Audio Player

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md), [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md), [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md), [ADR 0019](../adr/0019-select-a-feed-from-the-host.md), [ADR 0025](../adr/0025-studio-serves-cover-art.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md), [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) and [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md).

Status: Complete - 2026-10-27. The four commit checks are Green. The screenshot visual check and the human tests passed. The [review checklist](../reviews/2026-10-27-tasks-038-039-review-checklist.md) records the review.

## Goal

Each image field and each audio field has the button "Choose…". It opens one sheet with the files of this release, the file tree of each host, and an upload from the computer. A field also accepts a dropped file. The audio section of a track has a player, and its size and media type are in the closed disclosure "Properties".

## Packets

The packets go in this sequence:

1. [039a: the file sources in the API](studio-v1-task-039a-file-sources.md). It depends on task 038.
2. [039b: the picker sheet and the drop](studio-v1-task-039b-picker-sheet.md). It depends on 039a.
3. [039c: the audio player and "Properties"](studio-v1-task-039c-audio-player.md). It depends on 039a. It can go before or after 039b.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The sheet of the album cover with the three sources.
- A track with no image gets an image from "This computer", by the button and by a drop. The review lists the upload.
- A track gets an image from "Hosts". The review lists no upload.
- A track image is replaced. The review lists the deletion of the image from before, and "Undo" keeps it.
- The audio file of a track is replaced from "This release".
- The audio file of a track is replaced from "Hosts" with a file that only the host holds. The duration changes with no press of the play button.
- The audio section of a track: the player plays, the position moves, and "Properties" is closed with its summary. With an empty media type, "Properties" is open.
