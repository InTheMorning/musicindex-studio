# ADR 0198: The Review Of A Delete Names The Delete, And A Deleted Feed Shows Only What It Can Do

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire about the findings of the human test of 0.1.0-rc.22. The operator selected "Destructive" for finding 5, "in the last write" for finding 6a and "Hide them" for finding 6b.
- Technical choices: the planning model selected the label of item 1 for one file, and items 3, 5, 7 and 8.
- Changes: [ADR 0131](0131-delete-a-feed-from-its-file-host.md) items 8 and 12 for a feed with a done delete.
- Uses: [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md) items 1 and 4, and musicindex ADR 0069 R-ACT-3 items 1 and 3.
- Source: finding 5 and finding 6 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md).
- Changed by: [ADR 0208](0208-the-delete-and-move-sheets-give-goals-state-points-and-the-domain-line.md) item 13: a deleted feed also shows **Put the File Back**.

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
6. A feed with a done delete does not show **Stop Retiring**. **Check Again** stays, because step 2 still reads Music Index.
7. A feed with a done delete does not show the select menu "Link to an artist or label page" and **Link to This Page**. The rows of the linked pages stay. The API gives the done delete on the feed row, so that the section does not need the retire state.
8. A different read-only feed (ADR 0067) keeps these controls. Item 7 applies only to a done delete.

## Alternatives Considered

- **Write to the File Host** in the destructive style. Rejected by the operator: the label does not say that the write deletes files.
- Keep **Discard the Draft** destructive, after the delete button. Rejected: the footer then has two red buttons, and the second one removes no work. ADR 0193 item 12 also gives a row with only destructive buttons one menu.
- Show **Stop Retiring** and **Link to This Page** dimmed, with a reason. Rejected by the operator: a dimmed button is hard to read, and the feed has no use for it.
- A different text in the UI for a deleted feed. Rejected: the core already knows the state, and the existing text is correct when the state is correct.

## Consequences

- The user sees in the button what the write deletes.
- A deleted feed shows only **Check Again** as an action of the Retire block. A write to the same address brings the file back (ADR 0131 item 14).
- The feed row of the API has one more field.

## How This Decision Is Checked

Mechanical:

- A Node test proves the label of item 1 for one and for three delete operations, and the order and styles of items 2 and 3. A review with no delete keeps **Write to the File Host** in the primary style.
- A Rust test proves that `retire_facts` gives `Written` for a feed with a done delete and a hidden draft, and `DraftOnly` for a feed with no delete.
- A Node test or an API test proves that a feed with a done delete gives the field of item 7, and that the UI hides the controls of items 6 and 7 for it.
- The four commit checks pass.

Visual, by the planning model:

- The review of a delete with the audio and image files, at 1280 px light and 390 px dark. **Delete N Files** is red and first, and **Cancel the Delete** is at the right edge.
- The page of a deleted feed. Step 1 says "in the last write". **Stop Retiring** and **Link to This Page** are not on the page.

Human, by the operator. This gate passed on 2026-11-14 in the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), check 1:

- In the desktop app, start a delete of a test feed on a file host. The review shows the red **Delete N Files** first. **Cancel the Delete** stops the delete.

The operator also examined the deleted test feed in the desktop app on 2026-11-14. Step 1 says "Hide from apps is on in the last write.", **Check Again** is the only action, and the link select menu is not on the page. This part passed.
