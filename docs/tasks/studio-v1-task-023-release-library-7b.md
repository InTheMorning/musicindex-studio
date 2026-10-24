# Studio v1 Task 023: The Release Library, Milestone 7b

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 7.
Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md), [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md), [ADR 0037](../adr/0037-button-roles.md).
Depends on: milestone 7a and [task 022](studio-v1-task-022-ui-pass-after-7a.md).

## Goal

A published feed has a record of each of its files. The user can download each file into the library. Studio finds the changes in the library folder, and the user can move the feed XML to a different host.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-23-phase-7-review-checklist.md) before its commit. The packets go in this sequence:

1. [023a: the records of remote files](studio-v1-task-023a-remote-file-records.md).
2. [023b: published feeds in the library](studio-v1-task-023b-published-feeds-in-the-library.md). It depends on 023a.
3. [023c: the download queue](studio-v1-task-023c-download-queue.md). It depends on 023b.
4. [023d: the download UI](studio-v1-task-023d-download-ui.md). It depends on 023c.
5. [023e: the folder scan](studio-v1-task-023e-folder-scan.md). It depends on 023c.
6. [023f: the folder scan UI](studio-v1-task-023f-folder-scan-ui.md). It depends on 023d and 023e.
7. [023g: the move of the feed XML](studio-v1-task-023g-feed-xml-move.md). It depends on 023b.
8. [023h: the feed XML move UI](studio-v1-task-023h-feed-xml-move-ui.md). It depends on 023f and 023g.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- A published feed that the user adds has a release folder, its feed XML, and one remote library file for each address (023b).
- A download that stops continues from the end of its partial file (023c).
- The write refuses a draft with an operation that waits for a download (023c).
- A scan reads only a file whose stamp changed (023e).
- A move of the feed XML to a host with no redirect writes the feed to the new host. Then it writes a pointer file with `itunes:new-feed-url` at the address from before. Studio refuses a deletion of the pointer file (023g).

Visual:

- The visual check of milestone 7b. The operator adds a published feed, downloads its files and puts them on two hosts. The operator changes one audio file in the library folder and uses the new file. Then the operator moves the feed XML to the second host, and writes the release.
