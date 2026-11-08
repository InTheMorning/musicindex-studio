# ADR 0158: A Hidden Suggestion Can Come Back

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator. The label "Not This Person" did not name a person or a step, and a dismissal had no undo.
- Changes: [ADR 0098](0098-the-people-module.md) item 15, and the button "Not This Person" of [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) and [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md).
- Uses: [ADR 0156](0156-the-page-of-a-person-lists-where-the-person-is.md) item 7, for the credit rows of the sheets.

## Context

The sheet of a suggestion had the button "Not This Person". It hid the suggestion and kept the dismissal, and it changed no person and no feed. The label named no person, so the user could not tell what the button did. A suggestion that the user hid by mistake did not come back until a new item with the same name came into a feed.

## Decision

1. The button "Not This Person" becomes "Hide This Suggestion". It does what it did before: the core keeps the dismissal, and the suggestion goes from the list.
2. The People page has the fold "Hidden suggestions" at its end, with its count. It starts closed (ADR 0146 item 10). It shows only when one or more suggestions are hidden.
3. Each row of the fold shows the suggestion as the list of suggestions shows it, with the normal button "Show Again". "Show Again" removes the dismissal immediately, and the suggestion goes back into the list of suggestions. It changes no person and no feed.
4. A suggestion that has a new item comes back by itself, as before (ADR 0098 item 15). It then has no row in the fold.
5. A credit row of the sheets "Check", "Make a Person" and "Add to a Person" has the form of ADR 0156 item 7: the release as its title, then one line for each role with its places. The core gives the places of each role of a suggested credit.

## How This Decision Is Checked

Mechanical:

- `studio-core`: the hidden suggestions give each dismissed suggestion that has no new item. "Show Again" removes the dismissal, and the suggestion comes back.
- `studio-api`: the route of the hidden suggestions and the route of "Show Again".
- `crates/api/ui-tests/`: the UI text has no "Not This Person". The title case test examines "Hide This Suggestion" and "Show Again". The fold is the last section, and a credit row of a sheet has the form of item 5.

Visual, by the agent: the fold with one hidden suggestion, and the list after "Show Again", at 1280 px and at 390 px with touch.
