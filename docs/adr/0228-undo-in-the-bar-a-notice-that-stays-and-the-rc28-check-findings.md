# ADR 0228: Undo In The Bar, A Notice That Stays, And The rc.28 Check Findings

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, in the human check of rc.28 (`docs/reviews/2026-11-20-rc28-human-check.md`, findings 2 to 9 and 11). The operator selected each part from drawings in the questionnaire. The operator gave the direction of items 1 to 7 and item 9 in the chat.
- Technical choices: the main session selected items 4, 5, 7, 12, 19 and 21. The operator did not receive them as a question.
- Bar: crosses modules (the UI pages, the notices and the core delete rule). An agent can also keep a delete blocked or put back the bottom notice by mistake.
- Changes: [ADR 0227](0227-undo-and-redo.md) items 13 to 19. The notice with **Undo** is replaced by steps in the bar (items 1 to 7).
- Changes: [ADR 0175](0175-the-check-shows-what-music-index-holds.md) item 1, the notice after a write with a podping. [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 4, for the confirmation only (items 8 to 10).
- Changes: [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) item 8, the button "Sections" (item 11).
- Changes: [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) item 17, the tab label (item 13).
- Changes: [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) item 19, the order of the Review sheet (items 14 to 16).
- Changes: [ADR 0117](0117-the-page-of-a-person-finds-values-on-musicindex.md) item 1, for a linked person (item 18).
- Changes: [ADR 0200](0200-studio-deletes-a-feed-file-only-after-music-index-removes-the-entry.md) items 4 and 6, for a leftover copy (items 19 to 21).
- Uses: [ADR 0227](0227-undo-and-redo.md) items 1 to 12 and 20 to 22: the history of a page and the operations with no undo.

## Context

The operator did the human check of rc.28 on 2026-11-20. These findings need a decision:

- Finding 2: the operator saw the **Undo** of "{name} is deleted." only after the 10 seconds. The operator expected **Undo** at the top right, where the bar has **Undo** and **Redo**. People data must not look different from feed data.
- Finding 3: the release rows of an artist page have the same indent as the artist row.
- Finding 4: the confirmation of a write shows **Send the Podping Again**, and the notice then becomes smaller.
- Finding 5: the text "Added by you" after a Nostr key gives no useful fact.
- Finding 6: the large button "Find on Music Index" is too prominent on a person that has a linked page.
- Finding 7: the name "Sections" and its position at the top right do not tell what the button does.
- Finding 8: the tab label "Check 3" does not tell what the tab shows.
- Finding 9: Studio does not let the user delete a leftover copy of a feed. Music Index holds the GUID of the feed at a different address, so the core reads the answer as "Music Index holds the feed".
- Finding 11: in a long Review sheet, the user must scroll through the file operations to see the XML changes. An error of the write shows at the top, far from the button.

## Decision

### Undo In The Bar (Findings 2)

1. Delete Person, Remove from Collection, the menu **Kind** of a release and the nickname of a person show no notice. Their undo is in the buttons **Undo** and **Redo** of the bar and in the keys of ADR 0227 item 3.
2. A change of the nickname is one step of the history of the person page. A change of the menu **Kind** is one step of the history of the feed editor. Each change continues to save at the time of the change (ADR 0194, ADR 0213 items 13 and 16). **Undo** sends the value before the change through the same route, and **Redo** sends the new value again.
3. After Delete Person or Remove from Collection, the page that opens has **Undo** and **Redo** in its bar. The position of the draft status shows "{name} is deleted." or "{title} is removed from your collection." **Undo** puts back the rows with their IDs (ADR 0227 item 16). **Redo** does the delete again with no confirmation sheet, because the user confirmed it one time.
4. The undo of a delete stays while the app is open. The next operation of item 3 replaces it. When the core deleted the record (ADR 0227 item 16), **Undo** shows the error of ADR 0227 item 19.
5. The page of item 3 is the page that Studio opens after the delete: People after Delete Person, and Releases after Remove from Collection. On a different page, the bar shows the buttons of that page.
6. The confirmation sheets stay (ADR 0227 item 14). Remove from Collection with pending files or pending host operations stays with no undo (ADR 0227 item 17).
7. One base component keeps the history of each page (ADR 0227 item 12). A step of items 2 and 3 is a step of that component. A page does not keep a second history.

### The Notice After A Write With A Podping (Finding 4)

8. The notice after a write with a podping has the button **Dismiss** in each state: waiting, no reply and confirmed. The notice stays until the user selects **Dismiss**, or until the user leaves the page.
9. The confirmation "Music Index has your write of {time}." has no podping line and no **Send the Podping Again**. A waiting line and a line with no reply keep that button (ADR 0191 item 4).
10. The notice keeps its height when its state changes. The confirmation keeps the height of the notice before it.

### The Feed Page On A Phone (Findings 7 And 8)

11. The button "Sections" of ADR 0223 item 8 has the name "‹ Overview". It is not in the bar. It is in the row of the select menu of the tabs, at the left of that menu. It opens the outline page as before.
12. The stack of ADR 0223 item 8 does not change.
13. The tab of the findings has the label "Issues ({n})", for example "Issues (3)". With no finding, the label is "Issues". The crumb of the page and the select menu of the tabs use the same name.

### The Review Sheet (Finding 11)

14. The XML changes come before the file operations.
15. The file operations are in a fold that is closed when the sheet opens. The fold head is "File operations ({n})".
16. The error of a write shows in the footer of the sheet, directly above the check box "Send a podping" and the write button. The other notices keep their positions.

### The Person Head And The Artist Page (Findings 3, 5 And 6)

17. A Nostr key that Studio found shows its source, for example "Found in a Nostr profile". A key that the user added shows no source text.
18. A person with a linked page shows "Find Another Page" as a quiet text button in the role line. A person with no linked page keeps the button "Find on Music Index" below the role line (ADR 0117 item 1).
19. On the page of an artist, the release rows have an indent below the artist row.

### A Leftover Copy Of A Feed (Finding 9)

20. When Music Index holds the GUID of the feed at an address that is not the address of this file, the file is a leftover copy. The core gives `removal_done: true` for it. The sheet then shows the row of the delete, with the text "Music Index reads this feed from {address}. This file is a leftover copy."
21. Item 20 does not apply when Studio moved the feed from the address of Music Index to this file. Music Index then reads the earlier address until it follows the move (ADR 0184), and this file is the current feed.
22. The core compares the two addresses after the same normalization that the core uses for a feed address. The core refuses the delete draft as before for each other answer (ADR 0200 item 6).

## Consequences

- People data and feed data have one undo: the buttons in the bar. The bottom notice and its 10 seconds go.
- A delete undo stays longer than 10 seconds. The core record stays until the next delete. Thus a record can stay in the collection after the app stops (task 197, Result).
- A leftover copy can be deleted from Studio. An incorrect comparison of the addresses can delete the file that Music Index reads. Item 21 and the tests of task 200 control that risk.

## Alternatives Considered

- Finding 2: a notice at the top right, below **Undo**, with **Dismiss**. It keeps two forms of undo.
- Finding 2: only the nickname and **Kind** in the bar, and the bottom notice for a delete. The operator selected the bar for each operation.
- Finding 4: a confirmation that closes after 10 seconds. The operator selected **Dismiss**.
- Finding 7: "Overview" in the bar, or a first item "Overview" in the select menu.
- Finding 8: "Problems (3)" or "Check (3)".
- Finding 11: two tabs in the sheet, or only a move of the error.
- Finding 5: no source text for each key.
- Finding 6: no Find button for a linked person.
- Finding 9: a delete that is always available, with a warning. Or no change.

## Checks

Mechanical:

- UI test: the nickname and **Kind** add a step to the history of the page, and **Undo** sends the value before (item 2).
- UI test: the People page and the group page get a delete step, and the bar shows the text of item 3. No undo notice component remains (item 1).
- UI test: `writeNotice` gives **Dismiss** in each state, and the confirmation has no podping line (items 8 and 9).
- UI test: `tabLabel` gives "Issues (3)" and "Issues" (item 13).
- UI test: `keyEvidenceText` gives no text for a key that the user added (item 17).
- Core test: a GUID at a different address gives `removal_done: true`. A GUID at the same address and a GUID at the source address of a move by Studio give `false` (items 20 and 21).
- `bash tools/gate.sh` passes.

Visual (the main session, walks at 1280 px and 390 px):

- The bar after Delete Person and after Remove from Collection, and after **Undo**.
- The notice after a write: waiting and confirmed with the same height.
- "‹ Overview" at the left of the tab menu on a phone.
- The Review sheet with the closed fold and an error above the button.
- The person head with "Find Another Page", and the artist page with the indent.

Human:

- Delete Person on the real collection, then **Undo** from the bar.
- A write with a podping on a real file host, until the confirmation, then **Dismiss**.
- The delete of the leftover copy of finding 9 on the real file host.
