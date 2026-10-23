# Studio v1 Task 021: The Release Library, Milestone 7a

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 7.
Decisions: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md).
Depends on: Phase 6.

## Goal

The library folder is the master of each release. Each file of a release can be on a different host. The user can copy, move, reference, dereference and delete files on hosts, and the write does each operation after the diff.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-23-phase-7-review-checklist.md) before its commit. The packets go in this sequence:

1. [021a: the records and the library folder](studio-v1-task-021a-records-and-library-folder.md).
2. [021b: the host inventory and the addresses](studio-v1-task-021b-inventory-and-addresses.md). It depends on 021a.
3. [021c: the import into the library, and the diff](studio-v1-task-021c-library-import-and-diff.md). It depends on 021b.
4. [021d: the write with host operations](studio-v1-task-021d-write-with-host-operations.md). It depends on 021c.
5. [021e: the move of the current collection](studio-v1-task-021e-move-into-the-library.md). It depends on 021d.
6. [021f: the API of the library](studio-v1-task-021f-library-api.md). It depends on 021e.
7. [021g: the library UI](studio-v1-task-021g-library-ui.md). It depends on 021f.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- A release with files on two local test hosts writes a feed with the correct address for each file (021d).
- A changed library file gets a hash name on the host. The write deletes the file from before only after the feed is written (021d).
- Studio refuses a deletion of a host file that a feed of the library refers to (021d).

Visual:

- The visual check of milestone 7a: the operator sets the library folder, finds the current release in it, puts one file on a second host, moves one file, deletes one copy, and writes the release.
