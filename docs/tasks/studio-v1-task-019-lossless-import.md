# Studio v1 Task 019: A Lossless Import Gives An MP3 Enclosure

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decision: [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md).
Depends on: [task 018](studio-v1-task-018-import-music-files.md).

## Goal

An imported WAV file or FLAC file gives a track with an MP3 enclosure. The lossless FLAC file is a `podcast:alternateEnclosure` of the track.

## Packets

Four packets do this task. The planning model examines each packet with the [review checklist](../reviews/2026-10-23-phase-6-review-checklist.md) before its commit.

1. [019a: the alternative enclosure in the feed model](studio-v1-task-019a-alternate-enclosure-model.md). It depends on nothing.
2. [019b: the MP3 encoder](studio-v1-task-019b-mp3-encoder.md). It depends on nothing, and can go at the same time as 019a.
3. [019c: the upload route gives two files](studio-v1-task-019c-upload-two-files.md). It depends on 019a and 019b.
4. [019d: the import sheet and the track page](studio-v1-task-019d-import-sheet-and-track-page.md). It depends on 019c.

## Acceptance Criteria

Mechanical:

- An import of the WAV test file gives an MP3 enclosure and two alternative enclosures, MP3 and FLAC. An import of the FLAC test file gives the same (the criteria of 019c).
- An approved write puts the MP3 file and the FLAC file on the host before the feed file. No WAV file goes to the host.
- The corpus tests pass. A published `podcast:alternateEnclosure` keeps its bytes after a read and a write.
- The criteria of each packet.

Visual:

- The operator imports a WAV file and a FLAC file, examines the notices and the track page, and writes the release.
