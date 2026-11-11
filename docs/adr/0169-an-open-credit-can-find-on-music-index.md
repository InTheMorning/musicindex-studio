# ADR 0169: An Open Credit Can Find On Music Index

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire of the rc.21 human test. The operator selected a fill of the empty fields, and asked for a choice to replace a value and to add to a list.
- Technical choices: items 1, 4 and 5 are choices of the planning model. In the code, an open credit is a row of the Credits page, not a sheet.
- Changes: [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) item 8 for an open credit.
- Uses: [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md) items 3 and 5, [ADR 0168](0168-a-search-sheet-keeps-its-field-still.md).

## Context

**Find on Music Index** showed only in **Add a Credit**. A credit that exists already could not get a missing picture or Nostr key from a page on Music Index.

## Decision

1. An open credit row on the **Credits** page has the row **Find on Music Index**, under the heading "From Music Index". The sheet **Add a Credit** has the same row. The search opens in the sheet of ADR 0168.
2. A selected page fills each empty field of the credit. A field with a value keeps its value.
3. When the page gives a different value for a field with a value, the field shows the value of the page below it, with **Replace**.
4. When a field holds a list, the value of the page shows with **Add**. **Add** adds the value and keeps the values that the list has. A credit has no field that holds a list, so a credit shows no **Add**. Item 4 applies to a later place with this row and a list.
5. Each fill, each **Replace** and each **Add** is a pending change of the feed draft, as each other edit of the credit is.

## How This Decision Is Checked

Mechanical: a Node test of the fill model examines four results:

- Each empty field gets the value of the page.
- Each field with a value keeps it.
- Each different value gives a **Replace** choice.
- A list gives an **Add** choice.

Visual, by the agent: an open credit row with a page selected. The screenshots use the light and the dark theme, at the desktop width and at 390 px.

Human, by the operator: an existing credit with no picture gets the picture of a real page.
