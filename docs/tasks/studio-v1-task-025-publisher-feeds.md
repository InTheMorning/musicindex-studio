# Studio v1 Task 025: Publisher Feeds

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 8.
Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md), [ADR 0008](../adr/0008-feed-types-and-feed-identity.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0037](../adr/0037-button-roles.md).

## Goal

The user makes and edits publisher feeds for artists and labels. Each link between a publisher feed and a release of the collection goes in the two directions, with the same `rel` on the two sides.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-phase-8-review-checklist.md) before its commit. The packets go in this sequence:

1. [025a: the role and the links in the core](studio-v1-task-025a-publisher-core.md).
2. [025b: the publisher routes](studio-v1-task-025b-publisher-routes.md). It depends on 025a.
3. [025c: the publisher page](studio-v1-task-025c-publisher-page.md). It depends on 025b.
4. [025d: the publisher feeds of a release](studio-v1-task-025d-release-publishers.md). It depends on 025c.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- A link gives the two `remoteItem` elements of ADR 0041 items 6 and 7 in the two drafts, with the same `rel` (025a).
- A change of the role changes each `rel` of ADR 0041 item 11 (025a).
- Each new route needs the access token (025b).

Visual:

- The visual check of Phase 8. The operator makes a label feed and an artist feed for the releases in the collection. The operator links two releases to each, changes the sequence of the publisher feeds of one release, and writes each feed.
