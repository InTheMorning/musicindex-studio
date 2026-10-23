# Studio v1 Task 018: Import Music Files

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decisions: [ADR 0026](../adr/0026-import-music-files-through-the-draft.md) and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md).
Depends on: [task 017](studio-v1-task-017-new-album-feed.md).

## Goal

Let the user import audio files and image files into the draft of an album. The approved write puts the files on the host, and then the feed file.

## Packets

Four packets do this task. Each packet has a prompt for a coding model. The planning model examines each packet with the [review checklist](../reviews/2026-10-23-phase-6-review-checklist.md) before its commit.

1. [018a: the pending files and the write](studio-v1-task-018a-pending-files.md). It depends on task 017.
2. [018b: the tags and the WAV conversion](studio-v1-task-018b-tags-and-conversion.md). It depends on nothing, and can run with task 016.
3. [018c: the upload route](studio-v1-task-018c-upload-route.md). It depends on 018a and 018b.
4. [018d: the import sheet](studio-v1-task-018d-import-sheet.md). It depends on 018c.

## Acceptance Criteria

Mechanical:

- An import of tagged test files gives one track for each file, with its title, track number, duration, size and type (the mechanical check of Phase 6).
- An approved write puts each file on the host before the feed file, and does not replace a file that exists (the mechanical check of Phase 6).
- The criteria of each packet.

Visual:

- The visual check of Phase 6: the operator makes a new release from a folder of music files and writes it.
