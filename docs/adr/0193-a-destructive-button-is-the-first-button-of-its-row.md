# ADR 0193: A Destructive Button Is The First Button Of Its Row

- Status: Accepted
- Date: 2026-11-12
- Owner: operator, through musicindex ADR 0064, with the technical choices of the planning model
- Accepted: 2026-11-12. The operator accepted musicindex ADR 0064 (musicindex `51d94d1`) through the orchestrator session. The orchestrator session asked Studio to apply it.
- Technical choices: items 2, 4, 6, 9 and 10 are choices of the planning model.
- Applies: musicindex ADR 0064 R-ACT-8 to R-ACT-12. Musicindex `ab11758` added R-ACT-10 to R-ACT-12 and the scope of R-ACT-8 on 2026-11-12, as operator decisions.
- Changes: [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) item 8, the button order of "Leave without saving?". [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md) item 14, the order of "Merge with a Person" and "Delete Person". [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) item 25, the order of "Change" and "Remove". [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md), the order of the recipient actions, when they show as a row of buttons.
- Source: the screenshot of "Leave without saving?" on 2026-11-12. "Discard Changes" was the rightmost button of the sheet.

## Context

The confirmation sheet of `askConfirm` (`crates/api/ui/app.js`, `ConfirmHost`) shows "Cancel" or "Keep Editing" on the left and the destructive button on the right. Many rows of buttons on the pages of Studio also put the destructive button last, for example "Read Again" and then "Remove from Collection". A person expects the safe or usual action in the rightmost position. Thus a person can start the destructive action by habit.

Musicindex ADR 0064 gives the rule for each app of the ecosystem. The destructive button is the leftmost button of a row, a larger space follows it, and the safe action is the rightmost button. In a dialog, the safe action gets the first focus, and Enter does not start the destructive action.

## Decision

### The Rows

1. A row of buttons with a destructive button and one or more other buttons follows musicindex ADR 0064 R-ACT-8. The destructive button is the first button of the row. A select menu or a text in the row is not a button.
2. Item 1 applies to each row of buttons in Studio. It applies to the footer of a sheet or a dialog and to a bar. It also applies to a row `actions` of a page or a form, and to the buttons of a list entry. One rule for all rows is easier to learn and to test than a list of exceptions.
3. The **Discard** button of a save bar is a destructive button for item 1. The save bars are on the page of a person (ADR 0116 item 2) and in Settings (ADR 0166 item 8). It keeps the normal role and color. Its place does not change, because it is already the first button.
4. The space after the destructive button is 12 px larger than the gap of the row. In the footer of a sheet or a dialog, the destructive button goes to the left edge. The other buttons stay at the right edge.

### The Sheets And Dialogs

5. A confirmation sheet with a destructive action shows the destructive button first and "Cancel" or "Keep Editing" last. "Cancel" or "Keep Editing" gets the first focus (R-ACT-9 item 1). A confirmation with a primary action, for example "Download this file first?", keeps "Cancel" first and the action last.
6. A pure function of `crates/api/ui/confirms.js` gives the buttons of a confirmation sheet in their order, with the button that gets the first focus. `ConfirmHost` shows the buttons of that function.
7. Enter does not start a destructive action (R-ACT-9 item 2). A destructive button is never a `submit` button. Enter on a focused button starts that button.
8. Escape closes a sheet or a dialog with no action (R-ACT-9 item 3). The `dialog` element of Studio already does this.
9. The sheet of a destructive collection action gets **Cancel** as the last button of its footer, with the first focus. Examples are "Replace the Collection" and "Delete the Collection" (ADR 0176). Enter in a field of that sheet does not start the action. The order is the destructive button, "Export First…", then **Cancel**.
10. The account deletion form has a typed confirmation. Enter in that field does not delete the account. The user must click "Yes, Delete the Account".

### The Amendment Of Musicindex ADR 0064

11. Musicindex `ab11758` gives R-ACT-8 the scope of item 2: each horizontal row of buttons, also the buttons of a list entry.
12. A row with only destructive buttons shows one "Remove…" button that opens a menu (R-ACT-10). On 2026-11-12, Studio has no such row. Each row with one destructive button and no other button keeps that button.
13. A vertical group of controls puts its destructive control last, after a divider. When the group wraps into one row, R-ACT-8 applies (R-ACT-11). On 2026-11-12, Studio has no vertical group of buttons with a destructive button. A narrow window puts the text of a `.link-row` above its `.actions` row, and the buttons of that row stay in one horizontal row.
14. A button that replaces the saved choices of the person is destructive, with the red tint and a confirmation (R-ACT-12). On 2026-11-12, Studio has no such button. Settings has no "Use Defaults" button.

### Out Of Scope

- A menu keeps musicindex ADR 0020 R-ACT-3 item 2: the destructive item is the last item, after a divider.
- The native quit dialog of the desktop app (ADR 0124 item 18) has no destructive button in a row of the web UI. It does not change.

## Alternatives Considered

- Item 1 applies only to the footers of sheets and to bars. The rows of a page and of a list entry keep their order. Not chosen: a test then needs a list of exceptions, and the user sees two orders.
- Give **Discard** of a save bar the destructive role. Not chosen: musicindex ADR 0059 R-SET-3 item 2 gives that bar its buttons, and the order already agrees.

## Consequences

- Each row with a destructive button changes its order, for example "Remove from Collection" then "Read Again", "Delete Person" then "Merge with a Person", and "Remove" then "Edit".
- The order of a row is a guard. A new row that puts a destructive button after another button fails a test.

## How This Decision Is Checked

Mechanical, by `crates/api/ui-tests/confirms.test.mjs`. Each failure message names musicindex ADR 0064 and the fix:

- For each entry of `CONFIRMS` with the destructive role, the buttons of item 6 give the destructive button first and the safe button last, and the safe button gets the first focus.
- A scan of `crates/api/ui/*.js` finds each row of buttons with a destructive button, and finds that no button comes before the destructive button in that row.
- A scan finds no destructive button of the type `submit`.
- A scan finds no row with only destructive buttons (R-ACT-10).
- The four commit checks pass.

Visual, by the planning model, with screenshots at 1280 px and 390 px:

- "Leave without saving?": "Discard Changes" is at the left edge, "Keep Editing" is at the right edge and has the focus.
- The bar of the change set, the last row of the page of a person, the actions of a feed and the account actions.
- `$edges()` and `$overflow()` are empty.

Human, by the operator. This gate passed on 2026-11-12 in the desktop app:

- In the desktop app, open "Leave without saving?". "Discard Changes" is on the left, with a space after it. "Keep Editing" is on the right and has the focus. Enter keeps the page open.

Manual, by the planning model, because no Studio control exists for a test to examine:

- R-ACT-11 and R-ACT-12 (items 13 and 14). A change that adds a vertical group of buttons with a destructive button, or a button that replaces saved choices, examines the rule and adds a test.
