# ADR 0198: The Review Of A Delete Names The Delete, And A Deleted Feed Shows Only What It Can Do

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire about the findings of the human test of 0.1.0-rc.22. The operator selected "Destructive" for finding 5, "in the last write" for finding 6a and "Hide them" for finding 6b.
- Technical choices: the planning model selected the label of item 1 for one file, and items 3, 5, 7 and 8.
- Changes: [ADR 0131](0131-delete-a-feed-from-its-file-host.md) items 8 and 12 for a feed with a done delete.
- Uses: [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md) items 1 and 4, and musicindex ADR 0080 R-ACT-3 items 1 and 3.
- Source: finding 5 and finding 6 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md).
- Revised: 2026-11-15 by operator decision. Items 9 to 11 give **Put the File Back**, and items 12 to 15 give the write of only deletions. The operator selected them on 2026-11-14, from drawings. These rules were in ADR 0208 and ADR 0209, which are deleted. Item 6 names item 9.
- Revised: 2026-11-17 by operator decision. Items 16 and 17 give the podping of a write of only deletions. The operator selected "No podping" on 2026-11-17, from drawings.
- Changed by: [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 27 to 30, on 2026-11-20. Items 6 and 9: each feed with a done delete shows **Put the File Back**, also when it is not hidden.

## Context

The review of a delete from the file host uses the usual review footer (`crates/api/ui/editor.js`, about line 2197). The footer has **Discard the Draft** in the destructive style, then **Write to the File Host** in the primary style. The write of a delete removes the feed file and possibly the audio and image files. The primary button does not say that.

A delete draft holds no edit. `start_delete_draft` (`crates/core/src/feed_delete.rs`) refuses a draft with a change that the user did not write. Thus **Discard the Draft** of a delete review only stops the delete. The user loses no edit.

After the delete, the feed opens as read-only from the snapshot of its delete (task 134). The Retire block then says "Hide from apps is on in the draft." The cause is in `retire_facts` (`crates/core/src/feed_retire.rs`). The feed has no connector after the delete, so `hosted_copy` gives no copy, and `written` is false. The draft is the snapshot, which hides the feed, so the state is `DraftOnly`. But the snapshot is the last write, not a draft.

The read-only feed also shows **Stop Retiring** and **Link to This Page**. Each of them starts a change of the feed. After a delete, the feed has no file on a host. Restore of ADR 0131 item 14 is the way back.

## Decision

### The Review Of A Delete

1. When the diff has an operation with `deletes_feed_file`, the write button of the review is destructive. Its label gives the count of the delete operations: **Delete the Feed File** for one, and **Delete N Files** for more than one. During the write, the button shows the usual progress label.
2. The destructive write button is the first button of the footer, at its left edge (ADR 0193 items 1 and 4).
3. In a delete review, **Discard the Draft** becomes **Cancel the Delete**, in the normal style, at the right edge. It does the same action. A delete draft holds no edit, so this button removes no work of the user. The footer thus has one destructive button.
4. A review with no delete keeps its footer. The count and the marker come from the diff of the core. The UI adds no rule about which file is the feed file.

### The Retire Block Of A Deleted Feed

5. `retire_facts` gives `Written` when the feed has a done delete and its draft hides the feed. The snapshot of the delete is the last write. Step 1 thus shows the existing text "Hide from apps is on in the last write." The UI keeps no copy of this rule.
6. A feed with a done delete does not show **Stop Retiring**. It shows **Put the File Back** (item 9), then **Check Again**, because step 2 still reads Music Index.
7. A feed with a done delete does not show the select menu "Link to an artist or label page" and **Link to This Page**. The rows of the linked pages stay. The API gives the done delete on the feed row, so that the section does not need the retire state.
8. A different read-only feed (ADR 0067) keeps these controls. Item 7 applies only to a done delete.

### Put The File Back

9. A feed with a done delete shows the button **Put the File Back** in the actions of the Retire block, before **Check Again**.
10. The button calls `POST /feeds/{id}/delete/restore`, shows the draft of the answer and opens Review. The user writes the draft as a usual write ([ADR 0007](0007-draft-diff-write-and-announce.md)). A discard of the draft gives the state after the delete again.
11. The file comes back as it was at the delete, with "Hide from apps" on. The UI adds no rule. An error of the route, for example a file at the path, shows in the block.

### A Write Of Only Deletions

12. When each operation of the diff is a deletion, and the feed file does not change and does not move, the write button is destructive. Its label is **Delete 1 File** or **Delete N Files**, with the count of the delete operations.
13. The destructive write button is the first button of the footer (ADR 0193 items 1 and 4). **Discard the Draft** then has the normal style, at the right edge. The footer thus has one destructive button.
14. A refused deletion ([ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 13) does not count. A diff with only refused deletions has no write.
15. A write that changes the feed and also deletes a file keeps **Write to the File Host**. Items 1 to 3 do not change.
16. A write of only deletions (item 12) sends no podping. The feed file does not change, so an index has nothing new to read. `plan_announcements` (`crates/core/src/pipeline.rs`) plans no announcement and gives the skip cause `feed_unchanged`.
17. The review footer shows the line "No podping: this write does not change the feed." in the position of the check box (ADR 0205 item 19). The UI shows the cause from the core and adds no rule about the operations.

## Alternatives Considered

- **Write to the File Host** in the destructive style. Rejected by the operator: the label does not say that the write deletes files.
- Keep **Discard the Draft** destructive, after the delete button. Rejected: the footer then has two red buttons, and the second one removes no work. ADR 0193 item 12 also gives a row with only destructive buttons one menu.
- Show **Stop Retiring** and **Link to This Page** dimmed, with a reason. Rejected by the operator: a dimmed button is hard to read, and the feed has no use for it.
- A red write button for each write that deletes a file. Not chosen: a usual write with one replaced image becomes red.
- A different text in the UI for a deleted feed. Rejected: the core already knows the state, and the existing text is correct when the state is correct.

## Consequences

- The user sees in the button what the write deletes.
- A deleted feed shows **Put the File Back** and **Check Again** as the actions of the Retire block. It has a way back in the UI.
- A user who deletes the old files of a move sees a red button with the count.
- A write of only deletions announces nothing, also when the announcements are on.
- The feed row of the API has one more field.

## How This Decision Is Checked

Mechanical:

- A Node test proves the label of item 1 for one and for three delete operations, and the order and styles of items 2 and 3. A review with no delete keeps **Write to the File Host** in the primary style.
- A Rust test proves that `retire_facts` gives `Written` for a feed with a done delete and a hidden draft, and `DraftOnly` for a feed with no delete.
- A Node test or an API test proves that a feed with a done delete gives the field of item 7, and that the UI hides the controls of items 6 and 7 for it.
- A Node test proves that the Retire block view gives **Put the File Back** only for a done delete, before **Check Again**.
- A Node test proves the write view of only deletions: **Delete 1 File** and **Delete N Files**. It proves no destructive view with a feed change, a feed move, an upload or only refused deletions.
- A Rust test proves that a write of only deletions gives no announcement and the skip cause `feed_unchanged`. It also proves that a write that changes the feed and deletes a file keeps its announcement.
- A Node test proves the footer line of item 17 for the skip cause `feed_unchanged`.
- The commit checks pass.

Visual, by the planning model:

- The review of a delete with the audio and image files, at 1280 px light and 390 px dark. **Delete N Files** is red and first, and **Cancel the Delete** is at the right edge.
- The page of a deleted feed. Step 1 says "in the last write". **Stop Retiring** and **Link to This Page** are not on the page.

Human, by the operator. This gate passed on 2026-11-14 in the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), check 1:

- In the desktop app, start a delete of a test feed on a file host. The review shows the red **Delete N Files** first. **Cancel the Delete** stops the delete.

The operator also examined the deleted test feed in the desktop app on 2026-11-14. Step 1 says "Hide from apps is on in the last write.", **Check Again** is the only action, and the link select menu is not on the page. This part passed.
