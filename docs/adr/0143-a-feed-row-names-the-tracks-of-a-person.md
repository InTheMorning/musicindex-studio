# ADR 0143: A Feed Row Names The Tracks Of A Person

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, who asked that the row names the track. The form of the line is a choice of the planning model.
- Uses: [ADR 0120](0120-the-check-of-a-person-shows-each-difference.md) items 1 and 2 for the places and their text.
- Source: [findings of local use, finding 26](../reviews/2026-11-06-local-use-findings.md)

## Context

A feed row of a person gave the address and the role, but not the track that has them. The credit index kept the places of each credit. The payment index kept none.

## Decision

1. `feed_recipients` keeps the places of each recipient, in the form of the places of `feed_credits`.
2. `GET /people/{id}` gives `pay_places` and `role_places` for each feed.
3. Each "Pays" line and the "Credited as" line of a feed row end with the places: the channel or the title of each track. The text is the text of ADR 0120 item 2: the first two places and "and N more".
4. A migration adds the column, and a run that applies it refills the index from the library copies. A feed with no library copy has no places until its next read, and its lines then have no places.

## How This Decision Is Checked

Mechanical: `crates/core/src/collection.rs` examines the places and the index of the migration. `crates/api/ui-tests/people.test.mjs` examines the lines.

Visual: the planning model looks at a row of "Check your feeds" at 1280 px, and at 390 px with touch.
