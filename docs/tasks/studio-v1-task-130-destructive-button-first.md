# Studio v1 Task 130: A Destructive Button Is The First Button Of Its Row

Decision: [ADR 0193](../adr/0193-a-destructive-button-is-the-first-button-of-its-row.md), which applies musicindex ADR 0064 R-ACT-8 and R-ACT-9.

Status: Complete on 2026-11-12. The mechanical checks passed. The two new scan tests fail on the code before the change.

The screenshot check of "Leave without saving?" and of the page of a person passed at 1280 px light and 390 px dark. "Discard Changes" is at the left edge, and "Keep Editing" is at the right edge and has the focus. `$edges()` and `$overflow()` are empty.

The walk data has no read-only feed page, no change set and no account, so these rows have no screenshot. The scan test proves their order. The human check is open.

## Goal

Each row of buttons with a destructive button and another button shows the destructive button first, with a larger space after it. A confirmation sheet gives the first focus to its safe button. Enter never starts a destructive action. A Node test guards the order.

## Rules For The Delegate

- Read ADR 0193 and the code below first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff and commits.
- Do not change a text of `confirms.js` or a button label.

## Work

1. `crates/api/ui/confirms.js`: add an exported pure function `confirmButtons(entry)`. It gives an array of buttons in their order, left to right. Each button is `{ label, role, kind, focus }`. `kind` is `'action'` or `'cancel'`. For `role: 'destructive'`, the order is the action, then the cancel button (`entry.cancel || 'Cancel'`), and the cancel button has `focus: true`. For `role: 'primary'`, the order is the cancel button, then the action, and the cancel button has `focus: true`.
2. `crates/api/ui/app.js`, `ConfirmHost` (about line 5270 to 5310): show the buttons of `confirmButtons(ask.entry)`. The button with `focus: true` gets the ref that gets the first focus. The cancel button closes the sheet. The action button sets the result to true and closes the sheet. Each button stays `type="button"`.
3. `crates/api/ui/styles.css`: add the space of ADR 0193 item 4.
   - A rule for each row: a destructive button that is the first button and is not the last button gets `margin-inline-end: 12px`. Use `.button.destructive:first-of-type:not(:last-of-type)` and the same rule for `.button.discard`.
   - A rule for `.sheet footer`: the same button gets `margin-inline-end: auto`, so that it goes to the left edge.
   - Make sure that the narrow layout at 390 px still wraps without overflow.
4. The save bars: add the class `discard` to the **Discard** button of the Settings bar (`app.js`, about line 2826). Add it also to the bar of the page of a person (`peoplepages.js`, about line 1071). Do not change their order or their color.
5. Put the destructive button first in each row below. Keep each button, label, handler and `disabled` value.
   - `app.js` about 1993: the feed actions. "Remove from Collection", then "Read Again".
   - `app.js` about 2595: the file host card. "Remove", then "Move All Releases…", then "Edit".
   - `app.js` about 3019: the bar of the change set. "Discard", then the write button.
   - `app.js` about 4598: the account actions. "Delete the Account", then "Sign Out".
   - `app.js` about 2467: the connected app. "Remove", then "Connect Again".
   - `peoplepages.js` about 1065: "Delete Person", then "Merge with a Person".
   - `peoplepages.js` about 86 and about 941 to 1042: the credit name entry and the support link entry. "Remove", then "Edit".
   - `editor.js` about 778: the Nostr profile. "Remove", then "Change".
   - `editor.js` about 3878 and 3942: the hosted copy and the pending copy. "Delete", then "Use This Copy".
   - `editor.js` about 4036: the missing file. "Forget the File", then "Download Again".
   - `recipients.js` about 427: if the wallet actions show as a row of buttons, the remove action comes first. If they show as a menu, do not change them (ADR 0193 "Out Of Scope").
   - Each other row that the scan of item 8 finds.
6. `app.js` about 4590: the account deletion form. "Yes, Delete the Account" becomes `type="button"` and calls the delete with its `onClick`. The `onSubmit` of the form calls `preventDefault()` and does nothing else. Thus Enter in the field does not delete the account (ADR 0193 item 10).
7. `app.js` about 4900 to 4925: the sheet of a collection action. When `words.destructive` is true:
   - The footer is the destructive button, then "Export First…" when it shows, then a new **Cancel** button that closes the sheet.
   - **Cancel** gets the first focus when the sheet opens.
   - The destructive button is `type="button"` and starts the action with its `onClick`.
   - Enter in a field of the form does not start the action.
   When `words.destructive` is false, keep the current footer, focus and submit.
8. `crates/api/ui-tests/confirms.test.mjs`: add these tests. Each failure message starts with "musicindex ADR 0064" and the rule, then the fix, for example: "musicindex ADR 0064 R-ACT-8: a destructive button is never the rightmost button. Make it the first button of the row in {file}, and make the safe action the last button."
   - `a destructive confirmation shows the destructive button first and focuses the safe button`: for each entry of `CONFIRMS`, build the entry, call `confirmButtons`. For the destructive role, the first button has `kind: 'action'`, the last button has `kind: 'cancel'`, and the focused button is the last button. For the primary role, the focused button has `kind: 'cancel'`.
   - `no button comes before a destructive button in its row`: a scan of each `crates/api/ui/*.js`. For each button with the class `destructive` or `discard` (a static class or a template class that contains `destructive`), find the start of its row: the nearest opening before it of `<footer`, `class="actions"`, `class="draft-toolbar"` or `` <${Bar}> ``. Fail when a `<button` occurs between that start and the destructive button. Skip a destructive button that has no row start before it in the same template. If a real row uses another container, add that container to the list.
   - `no destructive button is a submit button`: a scan finds no `<button` tag that has `destructive` in its class and `type="submit"`.
   - Make sure that each new test fails on the code before the change. Report how you proved it.

## Checks

Mechanical:

- The tests of item 8 pass.
- The four checks of `AGENTS.md` pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.
- For each changed file in `crates/api/ui/`, copy it to `$TMPDIR/x.mjs` and run `tools/node --check $TMPDIR/x.mjs`.

Visual, by the main session: ADR 0193 "How This Decision Is Checked".

Human, by the operator: ADR 0193 "How This Decision Is Checked".

## Stop And Ask When

- A row needs a new component or a change of a label.
- The scan of item 8 needs a list of exceptions.
- The sheet of a collection action has a form that Enter must submit for a reason that ADR 0193 does not give.

## Report

- The list of the changed rows, with file and line.
- The test names, and the proof that each new test fails before the change.
- The output of the four checks.
