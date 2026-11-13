# ADR 0195: Restore The Hosted Copy Is Destructive

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, through musicindex ADR 0020 R-ACT-3, with the technical choices of the planning model
- Accepted: 2026-11-12. The orchestrator session gave the request, and cited musicindex ADR 0020 R-ACT-3 and ADR 0064 R-ACT-8.
- Technical choices: items 3 and 5 are choices of the planning model.
- Applies: musicindex ADR 0020 R-ACT-3 items 1, 3 and 4, and musicindex ADR 0064 R-ACT-8.
- Source: the review of ADR 0193 on 2026-11-12.

## Context

A file of a feed that changed in the local folder shows a row in the list of library changes (`crates/api/ui/editor.js`, `ChangeRow`). The row has **Use the New File** and **Restore**. **Restore** downloads the hosted copy and writes it over the changed local file. The local bytes are lost, and the user cannot undo this in the same view. The button has the normal style and no confirmation.

## Decision

1. **Restore** becomes **Restore the Hosted Copy…**. The label names the result (R-ACT-3 item 3), and "…" shows that a confirmation follows.
2. The button has the destructive style (R-ACT-3 item 1). It is the first button of its row, before **Use the New File** (R-ACT-8, ADR 0193).
3. A click opens a confirmation through `askConfirm`, from a new entry of `CONFIRMS` in `crates/api/ui/confirms.js`. The text says that the changed local file is lost. The action button is **Restore the Hosted Copy**.
4. A button that writes over local data or deletes local data with no confirmation gets the same treatment, or this ADR lists it with the reason. On 2026-11-12, a search found no other such button:
   - **Forget the File** removes the record of a missing file. It removes no local bytes.
   - **Download Again** writes to a missing local path. It writes over no local bytes.
   - **Use the New File** puts the changed local file into the draft. It removes no local bytes.
   - **Delete** of a hosted copy adds a deletion to the change set, which the user reviews before the write.
   - **Read Again** reads the feed again and keeps a draft with edits.
5. A Node test proves that the confirmation entry exists, has the destructive role, and that the restore action of the row calls `askConfirm`.

## How This Decision Is Checked

Mechanical:

- The Node tests of item 5, and the scan tests of ADR 0193.
- The four commit checks pass.

Visual, by the planning model: a screenshot of the confirmation sheet, if the walk data has a changed file. If it has none, the test of item 5 is the check, and the task says so.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, change a local audio file of a hosted feed. Select **Restore the Hosted Copy…**. A confirmation opens. **Cancel** keeps the changed file.
