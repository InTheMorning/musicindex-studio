# ADR 0140: A Credit Name Shows Where It Is Used

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire, from drawings of the rows. The operator selected the rows that show the use, in a fold.
- Technical choices: items 4 and 5 are choices of the planning model.
- Changes: the rows "Credit name" and "Nostr key" of [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md).
- Uses: [ADR 0028](0028-summary-lines-and-one-open-entry.md) for a section that starts closed. [ADR 0098](0098-the-people-module.md) item 12 for the identity of a credit. [ADR 0102](0102-delete-a-person-changes-no-feed.md).
- Source: [findings of local use, finding 23](../reviews/2026-11-06-local-use-findings.md)

## Context

A credit name is a match: a credit in a feed with that identity is a credit of the person. The row showed the values of the match with the label "Credit name". It did not show what the match does or what it finds. The same Nostr key also showed in a second row.

## Decision

1. The credit names and the Nostr keys of a person are in one fold. It starts closed. Its summary line reads "Credited as {name}", with the number of credit names and Nostr keys. The name is the credit name that is the name of the person, else the first credit name, else the name of the person.
2. A text in the fold tells what the rows do.
3. Each credit name row has the name as its title. Below the title, each value has an icon, a label and its text: "Used in", "Roles", "Website" and "Nostr". "Used in" gives the number of feeds with a credit of its identity, or "No feed yet". The core gives these values in `credit_name_uses`.
4. A credit name with the same key as a Nostr key of the person is one row. A key with no credit name keeps its own row.
5. "Remove" on one row removes the credit name and its key from the person. A removal changes no feed.
6. "Add a Credit Name" and "Add a Nostr Key" stay outside the fold.
7. When a credit name has the usual role of the person, "Roles" shows that role first and in brackets. Else the sequence does not change, because the row shows only the roles of the feeds.
8. The icons are the Lucide icons `disc`, `tag`, `globe` and `zap`, all in the muted text color. No icon of a row has its own color, so no value stands out.

## Alternatives Considered

- The rows with no fold. Not chosen: the user seldom changes them.
- Role chips, two columns and a sentence. Not chosen by the operator, from drawings.
- A fold with the rows from before. Not chosen: the rows then stay hard to understand.

## Consequences

- A credit with a name and a different key has a different identity, so a name row can read "No feed yet" while "Check your feeds" finds that credit by its name (ADR 0111 item 10).

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/people.test.mjs` examines the rows and the summary line. A core test examines `credit_name_uses`.

Visual: the planning model looks at the fold, closed and open, at 1280 px, and at 390 px with touch, with `$edges()`, `$fieldText()` and `$overflow()`.
