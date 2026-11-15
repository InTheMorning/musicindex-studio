# ADR 0208: The Delete And Move Sheets Give Goals, State Points And The Domain Line

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire, from drawings. The operator selected "Goal, then one line" for the choice sheet, "Name the takeover" for the domain line, "Three points and the domain line" for the last sheet, and "Add a button, and name it" for the restore.
- Technical choices: the planning model selected items 3, 6, 9, 11, 12 and 14 to 16.
- Changes: [ADR 0131](0131-delete-a-feed-from-its-file-host.md) items 2, 3, 4, 7 and 10, the texts of the two sheets and of the Retire block.
- Changes: [ADR 0198](0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md) item 6: a deleted feed also shows **Put the File Back**.
- Uses: [ADR 0200](0200-studio-deletes-a-feed-file-only-after-music-index-removes-the-entry.md) for the rule that a delete comes after the removal of the entry. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1 and musicindex ADR 0068 R-STATUS-1 for the style of a warning. [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) for the move sheet. [ADR 0007](0007-draft-diff-write-and-announce.md) for the review of the restore.
- Source: check 1, step 2 of the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), text 3 of its list.

## Context

The last sheet "Delete the Feed File" has the list "You lose these things:" with three long items. The operator found that the list is long and misleading. It tells the user what to fear, and not what to do.

The list came from ADR 0131 item 10, when Studio could delete the file of a feed that an index holds. ADR 0200 removed that delete. Now the user gets to the last sheet only after Music Index removed the entry. Thus the first item of the list is false, and the second item is not an effect of a delete.

The real facts for a user are these:

- Music Index changes an entry only from the file that it reads at the feed address. Thus the address is the control of the feed.
- A move keeps a pointer file at the address from before. The entry, the GUID and the payment recipients stay.
- "Hide from apps" asks the indexes to remove the entry. The file stays, and the user can stop it.
- A delete after the removal of the entry loses nothing in Music Index. Studio keeps the last copy of the feed.
- The one risk that stays is the loss of the domain of the address. The next owner of the domain can publish a feed with the same GUID and different payment recipients. This is a conclusion from the rules of Stophammer in ADR 0131. No test has shown it.

The last sheet also said "Only a write to this address brings the file back." The core and the API can do this (`POST /feeds/{id}/delete/restore`, ADR 0131 item 14). No control of the UI starts it.

## Decision

### The Domain Line

1. The domain line is one warning with these words: "Keep the domain {host}. If it expires, its new owner can take over this feed and its payments." `{host}` is the host of the public address of the feed.
2. The domain line has the style of a warning: the triangle icon and the Warning color (ADR 0191 item 1).
3. The core gives the host as `address_host` on the feed row of the API. The UI does not get a host from an address. A feed with no public address, or with a placeholder address, has no `address_host`, and the line does not show.
4. The domain line shows in four places, with the same words: the choice sheet, the last sheet, the Retire block, and the move sheet when the plan moves the feed file.

### The Choice Sheet

5. The sheet "Delete from File Host" has the title "What do you want to do?". It has no paragraph before its rows.
6. Each row is a goal with one line of text. The rows keep their sequence and their actions:
    - "Put the feed on a different file host", with "Apps and Music Index follow. Nothing is lost." It opens the move.
    - "Take the feed out of the apps", with "The file stays. You can stop this later." It starts the retire.
    - "Remove the file", with "Music Index removed the entry." It opens the last sheet. It is destructive, and it shows only when `removal_done` is true (ADR 0200 item 4).
7. No row has the tag "Recommended".
8. The domain line is below the rows.

### The Last Sheet

9. The sheet "Delete the Feed File" keeps the file host, the path, the check box with its counts, the lines of the linked publisher feeds, the title field and its buttons.
10. The list "You lose these things:" goes. In its place are three state points:
    - "✓ Music Index removed the entry"
    - "✓ Studio keeps a copy of the feed"
    - "↺ You can put the file back later with **Put the File Back** on this feed"
11. The domain line is below the points.

### The Retire Block

12. The head of the Retire block says "Studio cannot check Podcast Index." The domain line replaces the other sentences of the head.

### Put The File Back

13. A feed with a done delete shows the button **Put the File Back** in the actions of the Retire block, before **Check Again**.
14. The button calls `POST /feeds/{id}/delete/restore`, shows the draft of the answer and opens Review. The user writes the draft as a usual write. A discard of the draft gives the state after the delete again (task 111, packet 111d).
15. The file comes back as it was at the delete, with "Hide from apps" on. The UI adds no rule. An error of the route, for example a file at the path, shows in the block.

### The Move Sheet

16. When the plan moves the feed file (`plan.feed.moves`), the sheet "Move This Release…" shows the domain line below the plan. The host is the host of the address before the move.

## Alternatives Considered

- Two short paragraphs in the place of the list. Rejected by the operator: paragraphs are the wrong form.
- "Keep the old address." as the rule line. Rejected by the operator: too vague. The worst case is the loss of the domain, and the line must name it.
- The point "A write to this address brings the file back." Rejected by the operator: it names a mechanism, not an action. It also names an action with no control.
- The three points with no domain line on the last sheet. Not chosen: a user who only deletes then does not see the one risk that stays.
- The rows keep their action names (**Move It to a Different File Host**, **Retire It**, **Delete the File Only**). Not chosen: "Retire" is a word that the user must learn.

## Consequences

- The user reads one sentence about the one real risk, in four places, with the name of the domain.
- The sheets no longer explain Podcast Index or the stale entry. ADR 0200 makes that case not possible in Studio.
- A deleted feed has a way back in the UI.
- The words "take over this feed and its payments" give a conclusion that no test has shown.

## How This Decision Is Checked

Mechanical:

- A Node test proves the rows of the choice sheet: the titles and texts of item 6, no tag, and the third row only with `removal_done: true`.
- A Node test proves the domain line of item 1 for a host, and no line for no host.
- A Node test proves the three points of item 10.
- A Node scan test proves that the two sheets, the Retire block and the move sheet use the domain line function.
- The same test proves that the UI has no "You lose these things".
- A Rust test proves that the feed row gives `address_host` for a public address, and none for a placeholder address.
- A Node test proves that the Retire block view gives **Put the File Back** only for a done delete.
- The five commit checks pass.

Visual, by the planning model:

- Four views at 1280 px light and 390 px dark, with the fit checks.
- The views are the choice sheet, the last sheet, the Retire block of a deleted feed and the move sheet.

Human, by the operator. This gate is open:

- In the desktop app, open **Delete from File Host…** of a test feed. The sheet shows the goals and the domain line with the real domain.
- Delete a test feed that Music Index does not hold. Then select **Put the File Back** and write. The file host gives the file again.
