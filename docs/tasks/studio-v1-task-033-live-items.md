# Studio v1 Task 033: Live Items, Connected Apps And Show Grants

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 11, milestone 11b.
Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0017](../adr/0017-local-api-access.md), [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md) and [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md).

## Goal

A user adds a live show to an album feed or to a new show feed, and edits it. The user starts and ends it through the review sheet. An ended show can get a recording. The user can connect an app such as v4vmm. With a show grant, that app can start and end the show and set its live value, with no new diff.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-26-task-033-review-checklist.md) before its commit. The packets go in this sequence:

1. [033a: the live item rules in the core](studio-v1-task-033a-live-item-rules.md).
2. [033b: the live item page](studio-v1-task-033b-live-item-page.md). It depends on 033a.
3. [033c: the recording of an ended show](studio-v1-task-033c-recording.md). It depends on 033b.
4. [033d: connected apps](studio-v1-task-033d-connected-apps.md). It depends on 033a.
5. [033e: show grants and granted writes](studio-v1-task-033e-show-grants.md). It depends on 033d.
6. [033f: the grants in the UI](studio-v1-task-033f-grants-ui.md). It depends on 033b and 033e.

## Acceptance Criteria

Mechanical:

- A test adds a live item, writes it, then writes "Go live" and "End the show". The written feed has each status in sequence and the same `guid`.
- A test refuses a live item with no `contentLink`, before any change on a host.
- A test confirms that `/app/v1/` refuses the UI access token, and that `/api/` refuses an app token.
- A test client with an app token goes live, sets the live value and ends the show. The grant ends after the last change.
- Tests refuse a change that the grant does not cover, and a change after the end of the grant. A test refuses a change while the feed has a draft with changes.
- The collection keeps no app token as plain text.
- The Node tests examine the live item models and the grant texts.
- The four commit checks pass, in CI too.

Visual:

- The live item page, the status buttons, the recording, the grant check box, the list of granted writes and the Connected apps panel. Each one in the dark theme, in the light theme and at 390 px.

Human test, open until v4vmm or the live publisher builds its side:

- A test show with a real Studio, a real host and the other app.
