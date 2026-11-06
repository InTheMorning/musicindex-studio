# ADR 0141: The Form Of Add a Wallet Comes First

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 8, the payees first in the sheet "Add a Wallet" and the row that adds a wallet.
- Source: [findings of local use, finding 24](../reviews/2026-11-06-local-use-findings.md)

## Context

The sheet "Add a Wallet" showed a long list of payees above its form, so the form was out of view. A select of a payee added the wallet at once and closed the sheet, with no chance to check the values.

## Decision

1. The form is first in the sheet, has no heading, and stays in view when the list scrolls. At 760 px or less it does not stay in view, because it would take most of the window.
2. A select of a payee fills the form, with its type, address, custom key and account. It adds nothing and does not close the sheet.
3. "Add the Wallet" adds the values of the form.
4. A payee row has no chevron, because it opens no page.
5. The payee row that the form holds stays selected, with the accent tint and outline. A change of the address, the custom key or the account ends the selection.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines the sequence of the sections and the values from a payee.

Visual: the planning model looks at the sheet with a long list at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
