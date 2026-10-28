# Studio v1 Task 046: A Read-Only Feed Opens In The Editor

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0067](../adr/0067-a-read-only-feed-opens-in-the-editor.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md), [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) and [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md).

Status: Complete - 2026-10-27. The four commit checks pass. The screenshot visual check passed, and the [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records it. The human test with a real public feed passed on 2026-10-28, on `0.1.0-rc.3`.

## Goal

A read-only album, publisher feed or playlist opens in the editor with a draft. Review shows the diff. The write needs a host first: the user connects the host of the file, or moves the feed to a connected host.

## Packets

The packets go in this sequence:

1. [046a: the draft with no host in the core](studio-v1-task-046a-core-draft-with-no-host.md). It depends on task 045.
2. [046b: the API routes](studio-v1-task-046b-api-routes.md). It depends on 046a.
3. [046c: the editor and Review](studio-v1-task-046c-editor-and-review.md). It depends on 046b.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0067. The [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records them.

- The core starts a read-only draft from the feed XML of the release folder. The base version is `last_version` of the feed row, the SHA-256 of the last public read.
- A read-only draft keeps its host choice as the draft feed move that ADR 0038 item 16 has now. The feed row stays read-only until the write succeeds.
- `FeedMoveMethod` gets the variant `None`. Its text for the diff ID is `none`. The write puts no redirect and no pointer file for it.
- The path on the new host is the release folder, `/`, then the file name of the public address (ADR 0067 item 7, ADR 0035 item 13).
- The write of a read-only draft reads the public address through the `Web` of the workspace, so that the network policy of ADR 0057 applies. A different SHA-256 gives a conflict.
- After a write, the feed row gets the new connector, the path, the address and the default host of the files.
- A link by a folder match keeps the draft. When the host file has the SHA-256 of the draft base, the base version becomes the version of the host. Otherwise the base version stays, and the write gives a conflict. The link removes the draft feed move.
- The import of a file into a read-only draft is refused until the feed has a host. With a host choice, each upload goes to the new host.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`. Each one in the dark theme, in the light theme and at 390 px:

- A read-only album opens in the editor with its cover, its tracks and "read-only" in the eyebrow.
- Review of a read-only draft shows the diff, the two choices and no button "Write to the host".
- After "Move to a host", Review shows the warning of ADR 0067 item 9 and the button "Write to the host".

Human, for the operator, because it needs a real public feed:

- Open a read-only feed from a remote host. Change the title, move it to a connected host, and write it. The new address gives the new title, and the old address gives the old title.
