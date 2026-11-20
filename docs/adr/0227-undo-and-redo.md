# ADR 0227: Undo And Redo

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, in the human check of rc.27 (`docs/reviews/2026-11-19-rc27-human-check.md`, finding 14). The operator selected the scope, the place of the buttons, the life of the history and the step size from drawings.
- Technical choices: the main session selected items 5, 6, 9 to 12, 14, 16 and 18 to 22. The operator did not get them as a question.
- Bar: crosses modules (the feed editor, the person page, Settings, the desktop app and the core). An agent can also add an undo to an operation that reaches an outside service by mistake.
- Changes: [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) item 25. On a page with an undo history, the undo keys go to the history of the page. This also applies in a text field (item 9).
- Uses:
  - [ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md) item 9: the automatic save of the draft.
  - [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 1 to 3: the toolbar of the editor.
  - [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md): Save and Discard of the person page.
  - [ADR 0166](0166-settings-has-tabs-and-one-save-bar.md) items 7 to 13: the draft and the bar of Settings.
  - [ADR 0217](0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 14 to 17: the undo of a merge.
  - [ADR 0203](0203-people.md) item 51 and musicindex ADR 0085 R-CONFIRM-1: the confirmation sheets.
- Changed by: [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 1 to 7, on 2026-11-20. Items 13 to 19: the four operations show no notice. Their undo is a step in the bar.

## Context

Finding 14 of the rc.27 check: the operator wants undo and redo where possible.

Today only a merge of two persons has **Undo** (ADR 0217 items 14 to 17). The draft of a feed saves the full model 700 ms after each edit and keeps no history. The person page and Settings keep their pending values in the page, with **Save** and **Discard**. A delete of a person and a removal of a feed delete the rows of the collection, with no way back.

Some operations reach an outside service: a write to a file host, a podping and a Nostr publish. Studio cannot take them back. **Versions** can only prepare a new write.

## Decision

### Phase 1: Edits Before Save Or Write

1. The feed editor, the person page and Settings have an undo history of their edits. An edit is a change of the draft model of the feed, or of the pending values of the person page or of Settings.
2. The bar of each of these pages has two icon buttons, **Undo** and **Redo**, at the left of the draft status. They have the accessible names "Undo" and "Redo", and a tooltip with the key. A button with no step is disabled. In a narrow window, the buttons stay in the bar, in the row of the draft status.
3. Ctrl+Z gives Undo. Ctrl+Shift+Z and Ctrl+Y give Redo. On a Mac, Cmd takes the place of Ctrl.
4. Typing in one field is one step. A new step starts when the focus goes to a different field, or after 1 second with no typing. A change from a button, a check box, a select menu or a drag is one step.
5. Undo puts the model or the values of the step before back on the page. The page then saves as after an edit: the feed editor sends the draft, and the person page and Settings mark the values as pending. Redo does the opposite.
6. A new edit after an Undo deletes the steps that Redo can give.
7. The history stays while the app is open. It stays when the user opens a different page and comes back.
8. These events clear the history of a page: a write of the feed, **Discard**, and **Save** of the person page or of Settings. An end of the app also clears it. Studio does not store the history.
9. On a page with an undo history, the keys of item 3 go to that history, also when the focus is in a text field. The page takes the key before the script of the Linux desktop app, so that the script does not give the undo of the text field (ADR 0124 item 25). A page with no history keeps the undo of the text field.
10. An operation that changes the draft through a route other than the draft save clears the history of that feed. Examples are a role, a link of a release, the listing, a live show, a repair, a merge of the hosted file, a host operation and a move. Studio cannot put back the other records of these operations, so an Undo would make the draft and the records disagree.
11. The history keeps at most 100 steps for each page. The oldest step goes first.
12. One base component in the UI keeps the history. The three pages use it.

### Phase 2: An Undo Notice After An Immediate Local Operation

13. After each operation in this list, Studio shows a notice with **Undo** for 10 seconds:
    - **Delete Person**: "{name} is deleted."
    - **Remove from Collection**: "{title} is removed from your collection."
    - The menu **Kind** of a release: "The kind is now {kind}."
    - The nickname of a person: "The nickname is now {nickname}."
14. The confirmation sheets of Delete Person and Remove from Collection stay (ADR 0203 item 51, musicindex ADR 0085 R-CONFIRM-1). A notice of 10 seconds is easy to miss, so it does not replace a confirmation.
15. Only one notice shows at a time. A new operation of item 13 replaces the notice and its undo.
16. For Delete Person and Remove from Collection, the core stores the deleted rows as one undo record in the collection, in the same transaction as the delete. **Undo** puts back the rows with their IDs in one transaction. The core keeps the record of the last operation only. It deletes the record at the next operation of item 13 and when the notice ends. The undo of a merge (ADR 0217 items 14 to 17) does not change and keeps its own record.
17. Remove from Collection gives no notice when the feed has pending files or pending host operations. The removal deletes those files, and the core cannot put them back. The confirmation sheet then says "This cannot be undone."
18. For the menu Kind and the nickname, **Undo** sends the value before the change through the same route.
19. When an undo fails, the notice shows the error text of the core and no **Undo**.

### Operations With No Undo

20. These operations get no undo: a write to a file host, a delete of a hosted file, a podping, a Nostr publish, a write of a change set, a removal of a connected app and an end of a show grant. **Versions** stays the way back for a written feed.
21. These local operations also get no undo now: an import, a refresh, a scan, a download, a change of a file host, a change of the vault, the library folder and the operations of desktop collections. A subsequent decision can add one of them to item 13.
22. An operation that is not in item 13 shows no Undo notice. An agent does not add an Undo to an operation of item 20.

## Alternatives Considered

- Scope: undo and redo for edits only, with no undo after a delete. The operator selected the larger scope.
- Scope: the feed editor only. The operator selected the larger scope.
- Buttons: the keys only, with no buttons. A phone has no keys. The operator selected the buttons.
- Buttons: Undo and Redo in a menu. It needs two taps. The operator selected the buttons in the bar.
- History: kept after a restart. More storage and more work. The operator selected a history that ends with the app.
- Step size: one step for each word. The operator selected one step for each field change.
- Phase 2: an Undo notice in the place of the confirmation sheet. Not selected (item 14).
- Phase 2: a delete that waits until the notice ends. A quit of the app in those 10 seconds would lose the delete. Not selected.

## Consequences

- The feed editor keeps up to 100 copies of the draft model in memory for each feed that the user edited. A feed with many tracks uses more memory.
- After an operation of item 10, Undo is disabled until the next edit. The user can still use **Discard**.
- The core has a second undo record, next to the record of a merge.

## How This Decision Is Checked

Mechanical:

- UI test of the history component: a step for each field, a new step after 1 second or a different field, Redo after Undo, no Redo after a new edit, at most 100 steps, and clear (items 4, 6, 8 and 11).
- UI test: the key handler gives Undo for Ctrl+Z and Cmd+Z, and Redo for Ctrl+Shift+Z, Cmd+Shift+Z and Ctrl+Y (items 3 and 9).
- UI test: each of the three bars has the buttons "Undo" and "Redo" before the draft status (item 2).
- Core tests: Delete Person and Remove from Collection, then the undo, give the same rows with the same IDs. A second operation deletes the first record. Remove from Collection with pending files stores no record (items 16 and 17).

Visual, by screenshots of the main session before each commit, at 1280 px and 390 px:

- The buttons in each bar, enabled and disabled (item 2).
- An Undo and a Redo of a field in the feed editor, the person page and Settings (items 4 and 5).
- The notice after Delete Person and after Remove from Collection, and the page after **Undo** (items 13 and 16).

Human:

- In the Linux desktop app, Ctrl+Z in a text field of the feed editor gives the undo of the page, and Ctrl+Z in a text field of a page with no history gives the undo of the field (item 9). A screenshot walk does not run the script of the desktop app.
