# ADR 0209: The Review Sheet Keeps The Podping Box In View And Names Each Delete

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire, from drawings. The operator selected "In the footer" for the podping check box and "Labels grey, values dark" for the rows. The operator selected "Warning color, kind 'Not deleted'" for a refused deletion. The operator selected "Red 'Delete N Files', first" for a write of only deletions.
- Technical choices: the planning model selected items 2 to 5, 8, 11, 12, 15 and 16.
- Changes: [ADR 0136](0136-one-announcement-choice-at-the-write.md) item 1 and [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) item 19: the position and the label of the check box in the review sheet.
- Changes: [ADR 0198](0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md) items 1 to 4: a write of only deletions also has the destructive write button.
- Uses: [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) items 13 and 14 for the refused deletion and the rows. [ADR 0193](0193-a-destructive-button-is-the-first-button-of-its-row.md) items 1 and 4. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1 and musicindex ADR 0068 R-STATUS-1 for the style of a warning. [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 10 to 12 for the deletion of the old files.
- Source: design items 1, 2, 3 and 5 of the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md).

## Context

The human check walk of 2026-11-14 gave four faults of the review sheet:

- The row **Announce** is at the top. Its check box "Tell Podcast Index and send a podping" is at the end of the body, so the user sees it only after a scroll.
- Each row of a file has many grey lines: the addresses, the file host and the notes. A block of grey text is tiring to read.
- When the user adds the deletion of a pointer file, Studio refuses it. The row is red, has the kind "Delete" and says "Studio keeps this file." The kind and the text do not agree, and nothing is wrong.
- A write that only deletes the old files of a move has the blue button **Write to the File Host**. The review of a feed delete has a red **Delete N Files** (ADR 0198).

## Decision

### The Podping Check Box

1. The check box of the announcement is in the footer of the review sheet, in its own row above the buttons. It is in view at each scroll position.
2. Its label in the footer is "Send a podping". The row **Announce** at the top keeps the full words and the address.
3. The hint of the check box moves to the row **Announce**, as a second line: the time of the send. When the check box is off, the row **Announce** says "No announcement. No index learns about this write until it reads the feed by itself."
4. The check box shows in the same conditions as before. A change of it saves the choice and gets the diff again (ADR 0205 items 25 and 26).
5. The review page of a change set does not change.

### The Text Of A Row

6. In a row of the review, a value has the normal text color. A value is an address, a path or the name of a file host. A label, such as "From" and "To", and a note have the muted color.
7. The row of a feed move shows "From" and "To", each with its address. The name of the file host is on its own line below the address that it belongs to.
8. The UI shows the file host of the address before the move only when the diff gives it. The UI adds no rule.

### A Refused Deletion

9. The row of a refused deletion has the style of a warning: the triangle icon and the Warning color. Its kind is "Not deleted". It is not an alert.
10. For a pointer file, the text is "Studio does not delete this file. It tells the apps the new address of the feed."
11. For a file that feeds refer to (ADR 0036 item 13), the row has the same style and kind. The text is "Studio does not delete this file. These feeds use it: {titles}".
12. **Undo** stays. It removes the refused deletion from the draft.

### A Write Of Only Deletions

13. When each operation of the diff is a deletion, and the feed file does not change and does not move, the write button is destructive. Its label is **Delete 1 File** or **Delete N Files**, with the count of the delete operations.
14. The destructive write button is the first button of the footer (ADR 0193 items 1 and 4). **Discard the Draft** then has the normal style, at the right edge. The footer thus has one destructive button.
15. A refused deletion does not count. A diff with only refused deletions has no write.
16. A write that changes the feed and also deletes a file keeps **Write to the File Host**. The review of a feed delete does not change (ADR 0198 items 1 to 3).

## Alternatives Considered

- The row **Announce** becomes the check box at the top of the sheet. Recommended by the planning model. Not chosen by the operator.
- A fold for the note of each row. Not chosen: the user must click to learn what a row does.
- The kind "Keep" in a neutral color for a refused deletion. Recommended by the planning model. Not chosen by the operator: the user asked for a deletion that does not occur.
- A red write button for each write that deletes a file. Not chosen: a usual write with one replaced image becomes red.

## Consequences

- The footer of the review sheet has two rows when the check box shows. On a phone, the footer is higher.
- The address of the podping and the check box are in two places: the top and the footer.
- A user who deletes old files sees a red button with the count.
- It is not decided if a write of only deletions needs a podping. Design item 6 of the walk stays open.

## How This Decision Is Checked

Mechanical:

- A Node test proves the label of the footer check box.
- A Node test proves the lines of the row **Announce** for the check box on, off and with a wait.
- A Node scan test proves that the check box is in the footer of the review sheet, and not in its body.
- A Node test proves the view of a refused deletion: the kind "Not deleted", the warning tone and the two texts.
- A Node test proves the write view of only deletions: **Delete 1 File** and **Delete N Files**.
- The same test proves no destructive view with a feed change, a feed move, an upload or only refused deletions.
- The five commit checks pass.

Visual, by the planning model:

- Four reviews at 1280 px light and 390 px dark, with the fit checks.
- The reviews are a usual write, a move, a refused deletion and a write of only deletions.
- The footer at 390 px: the check box and each button are fully in view.

Human, by the operator. This gate is open:

- In the desktop app, open Review of a long draft. The check box "Send a podping" is in view with no scroll.
- Move a test release and select **Delete Now…**. The review has a red **Delete N Files** at the left.
