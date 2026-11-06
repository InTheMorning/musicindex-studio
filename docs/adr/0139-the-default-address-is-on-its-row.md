# ADR 0139: The Default Address Is On Its Row

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire, from drawings of the rows. The operator selected the button on each row.
- Changes: [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 4, the select menu "Default address", and [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for that menu.
- Source: [examples of a default among many addresses](../research/2026-11-06-default-address-examples.md), and [findings of local use, finding 21](../reviews/2026-11-06-local-use-findings.md)

## Context

The page of a person had a select menu "Default address" above the rows of the same addresses. Gmail, Amazon and Wingspan put the control on the row of each item.

## Decision

1. The section "Payment" has no select menu for the default address.
2. The row of the default address has the tag "Default" directly after its title, with a check icon, in the accent color and with the radius of a badge (musicindex ADR 0024).
3. Each other row has the button "Make Default", with the role normal.
4. The button makes a pending change. "Save" keeps it and "Discard" removes it, as ADR 0116 gives. The row of a pending default has the pending mark.
5. A text above the wallets tells that "Pick from your people" adds the default address.
6. At 760 px or less, the button goes on a line below the address.
7. The default address is always first in its wallet, and its wallet is first in the section. A selection of "Make Default" moves the row at once.

## Alternatives Considered

- A fixed sequence of the rows. Not chosen: on 2026-11-06 the operator asked for the default to be first, after seeing the rows with real data.
- A menu of three dots on each row. Not chosen: the action needs one more click, and a user finds it less easily.
- A shorter select menu. Not chosen: it still repeats the rows.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines which row is the default.

Visual: the planning model looks at the rows and a pending change at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
