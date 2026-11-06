# ADR 0137: The People List Fills Nostr Keys And Open Entries

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire, from drawings of the sheet. The operator selected one field with the list below it, and "Fill from a Person".
- Technical choices: items 2, 3, 7 and 8 are choices of the planning model.
- Changes: [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) item 20, the text of the field.
- Uses: [ADR 0098](0098-the-people-module.md), a value from a person is a start value. [ADR 0123](0123-one-type-of-value-has-one-picker.md), one sheet for each Nostr key field. [ADR 0065](0065-the-core-applies-each-feed-rule.md), an edit aid in the UI.
- Source: [findings of local use, finding 3](../reviews/2026-11-06-local-use-findings.md)

## Context

The sheet "Find a Nostr profile" searched only outside the People list, so the user had to search for a key that Studio already had. "Pick From Your People" adds a new row, but it does not fill a row that exists.

An inventory found these other fields that can hold a value of a person: the recipients, the credits and the support links. "Pick From Your People" adds a row for each of them. An artist is its publisher feed and not a person (ADR 0098), so the artist name is not in this set. A person has no email field, so the owner email is not in this set.

## Decision

1. The sheet "Find a Nostr profile" shows "Your people" below its one field, before the first search. The text in the field narrows the list. "Search" adds "Search results" below the list. The field has the label "Name, npub or NIP-05 address".
2. The list has one row for each Nostr key of each person. The key of the field has no row.
3. The sheet on the page of a person does not show the list, because a key of a different person is a merge ("Merge with a Person" of ADR 0103).
4. Removed by [ADR 0145](0145-each-sheet-that-adds-a-value-has-one-layout.md). An open credit has no "Fill from a Person".
5. Removed by ADR 0145.
6. Removed by ADR 0144.
7. Removed by ADR 0145.
8. Removed by ADR 0145.
9. The function is `nostrPickRows` of `crates/api/ui/people.js`.

## Alternatives Considered

- Two tabs in the sheet. Not chosen: a search of the people and of Nostr needs a change of tab.
- A row that opens the picker of the recipients. Not chosen: it needs one more click, and a second sheet opens on the first.
- "Fill from a Person" replaces each value. Not chosen: it can remove a value that the user typed, with no warning.
- A button on a support link. Not chosen: a support link has only two fields, and a person can have many links. "Pick From Your People" adds them.

## Consequences

- A long list of people moves the search results down until the user types in the field.
- A person with many wallets fills the address of the default wallet, as "Pick From Your People" does.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines the rows, each fill and the condition of the button.

Visual: the planning model looks at the two sheets at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
