# Studio v1 Task 111: Delete A Feed From Its File Host

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Feed Moves".
Decision: [ADR 0131](../adr/0131-delete-a-feed-from-its-file-host.md).

Status: Complete. ADR 0131 is Accepted. Packets 111a, 111b, 111c and 111d are complete. The operator confirmed the mockup of packet 111c on 2026-11-13. The footer of the final sheet puts **Delete** first, as ADR 0193 requires. The screenshot visual check of 111c passed at 1280 px in the light theme and at 390 px in the dark theme.

The human test of ADR 0131 ran on 2026-11-13 with a local build. The retire and the delete passed. The feed after the delete did not open (finding 6 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md)). [Task 134](studio-v1-task-134-deleted-feed-opens.md) corrected it, and its human check passed on the same day. A restore on a real host did not run.

## Goal

The page of a feed with a connector gets **Delete from File Host**. A sheet recommends a retire first. The delete removes the feed file, and optionally its audio and image files, through an approved diff.

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065).
- No delete occurs without a diff that the user approved (ADR 0007).
- Studio does not delete a pointer file (ADR 0038 item 20).
- Studio makes no index request unless the user opens the retire page or selects **Check Again** (ADR 0131).
- No test sends a request outside the computer (ADR 0045).
- Each text about Podcast Index states only what the sources show.

## Files To Read First

- ADR 0131, ADR 0007, ADR 0038 items 17 to 20 and ADR 0063.
- `crates/core/src/pipeline.rs`: `write_draft` and the deletions of a move.
- The `delete` operation of the connector trait, and `Collection::is_pointer_of_feed`.
- `crates/core/src/musicindex.rs`: `feed`.

## Packets

Do the packets in this sequence. Each packet is one commit, with the four commit checks.

### 111a: The Delete In The Core

ADR 0131 items 11 to 14.

- A delete draft of a feed, with the option of item 11. The draft counts each file that stays and gives the reason.
- The write goes through `pipeline::write_draft` with an approved diff. A snapshot comes first, and the conflict check stops a delete when the file changed.
- The core refuses the delete of a pointer file. It keeps a file that a different feed uses.
- After the delete, the feed is read-only with its last copy. A write to the same address brings the file back.
- Each publisher feed that links to this feed, for the line of item 13.

### 111b: The Retire State And The Routes

ADR 0131 items 5 to 9.

- The retire sets and clears "Hide from apps" in the draft (ADR 0063).
- The state of the three steps of item 6, from the draft, the last write and one read of `GET /v1/feeds/{guid}`.
- The routes of the retire state, of the delete draft and of the delete write.

### 111d: The Restore Discard And The Replaced GUID

Orchestrator decisions of 2026-11-11, on the open points of packet 111b.

- The delete record stays until a restore write is complete. A discard of the restore draft gives the feed the state after the delete again: read-only, with the restore button. A writable feed with no file on the host must not occur. Add a core test.
- A 404 with `superseded_by` from `GET /v1/feeds/{guid}` is not a removal. Step 2 stays open with the line "Music Index holds this feed with a different GUID." For that answer, the core does not let the user select **Delete the File**. Add a core test for each of the three answers: a 404, a 404 with `superseded_by`, and a 200.

### 111c: The Sheets And The Retire Page

ADR 0131 items 1 to 10.

- First a mockup of the first sheet, the retire page and the final sheet, at 1280 px and at 390 px. Stop until the operator confirms it.
- The UI model gives the steps of a retire from the state of the core, and the button that waits for the exact title. Add a Node test.
- The page shows the action, the sheets and the retire page.
- The last sheet calls `POST /feeds/{id}/delete/draft` with `with_files: false` when it opens.
- It calls the route again when the check box changes.
- The route gives the preview counts and names.
- The sheet shows the route error and keeps **Delete** off when the route refuses.
- **Delete** opens Review on the delete draft.
- **Cancel**, the close button and Escape discard the draft with `DELETE /feeds/{id}/draft`.
- Add a page to `docs/user-guide/`.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0131.
- The four commit checks.

Visual, by the planning model:

- The visual criteria of ADR 0131.

Human, by the operator. On 2026-11-13 the retire and the delete passed. The restore did not run:

- The human criteria of ADR 0131. The human gate of [ADR 0208](../adr/0208-the-delete-and-move-sheets-give-goals-state-points-and-the-domain-line.md) has the restore, with its button **Put the File Back**.

## Stop And Ask When

- A delete would remove a file that a different feed uses, or a pointer file.
- The index API does not give the fact that a step of item 6 needs.
- A text needs a fact about Podcast Index that no source shows.

## Final Report

For each packet, give the commit, the results of the four commit checks and the screenshots of a UI packet. Report each gate that did not run as open.
