# Studio v1 Task 031: Announcements

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 10.
Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) and [ADR 0007](../adr/0007-draft-diff-write-and-announce.md) item 5. Also [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) item 21, [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 21 and [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md).

## Goal

After each complete write, Studio tells the Podcast Index hub the address of the feed, with a podping. The diff shows each planned announcement before the write. A queue in the collection sends each announcement 30 seconds after the write, and tries again after a failure. The feed page shows the last announcement. Two global settings turn the announcements off.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-task-031-review-checklist.md) before its commit. The packets go in this sequence:

1. [031a: the test hub, the queue and the sender](studio-v1-task-031a-queue-and-sender.md).
2. [031b: the announcements in the diff and the write](studio-v1-task-031b-diff-and-write.md). It depends on 031a.
3. [031c: the worker and the routes](studio-v1-task-031c-worker-and-routes.md). It depends on 031b.
4. [031d: the announcements in the UI](studio-v1-task-031d-announce-ui.md). It depends on 031c.

## Acceptance Criteria

Mechanical:

- An API test writes a feed, and the local test hub then gets one request with the feed address and `podping=1`.
- An API test with the Podcast Index setting off writes a feed, and the test hub gets no request.
- A core test gives the two addresses of a feed move, the new address first.
- A core test shows that a change of a setting changes the diff ID.
- Core tests give the retry times, the failed state after five failures, the replacement of a waiting announcement, and the expiry after 24 hours.
- The diff JSON has no `announce` field.
- The Node tests examine the text of the announcement lines and of the status.
- The four commit checks pass, in CI too.

Visual:

- The review sheet, the feed page and the Hosts page show the announcements and the settings, in the dark theme and in the light theme. The planning model checks them with `tools/drive-ui.mjs` against a local test hub.

Human test, open until the operator writes a real feed:

- The hub accepts the request. A podping with the feed address appears. Stophammer crawls the feed, and musicindex.org shows the change.
