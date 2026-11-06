# ADR 0144: Recipients Use One Picker And Sort By Split

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, through the questionnaire and from the mockup in `.studio-dev/mockup-recipients/`. The operator asked that the rows sort by split, in the feed also, and that a row copies from musicindex.
- Changes: [ADR 0027](0027-the-recipient-picker.md) item 7 and [ADR 0099](0099-the-people-destination-and-the-page-of-a-person.md) item 19: only the search of musicindex selects many recipients, and a row of the user fills the form. [ADR 0137](0137-the-people-list-fills-nostr-keys-and-open-entries.md) items 4 and 6 for a recipient.
- Uses: [ADR 0123](0123-one-type-of-value-has-one-picker.md) and [ADR 0141](0141-the-form-of-add-a-wallet-comes-first.md) for the layout of the sheet. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the sort at the draft save. [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for a block that does not change.
- Source: [findings of local use, finding 27](../reviews/2026-11-06-local-use-findings.md)

## Context

An open recipient had three rows of buttons. "Add a recipient", "Pick from your people" and "Fill from a Person" did almost one job. The user had to move each row by hand.

## Decision

1. An open recipient holds its split, the check box "Fee, paid first", its address with "Change…", and "Remove".
2. "Add a recipient" and "Change…" open one sheet: first the row "Copy from a Feed on musicindex", then the form, then "Your people", then "Payees with no person". A selection of a person or a payee fills the form. The first row opens the search with check boxes for many recipients.
3. "Add to People" shows only for a row with an address that no person holds, on an open recipient and on a payee row of the sheet.
4. The rows have no move buttons and no position menu. The screen shows the biggest split first, with each fee in its own group. The rows sort when the focus leaves the open row, so that a row does not move during the typing or before a click in that row. The share bar has the same sequence.
5. The core sorts a value block that a save changes: the biggest split first, each fee after the others, and equal splits in their sequence.
6. A value block that a save does not change keeps its sequence in the feed, so a feed can differ from the sequence on the screen until the user changes that block.
7. The field view of Review matches the recipients of a block by address, custom key and custom value. A recipient at a new place shows the changes of its values, and no row tells of the new place. The detailed view shows each moved line.

## Alternatives Considered

- A sort on the screen only. Not chosen by the operator: other apps show the sequence of the file.
- A sort of each block at each save. Not chosen by the planning model: the writer changes only what the user changed (ADR 0012), so a change of a title must not move recipients.

## How This Decision Is Checked

Mechanical: `crates/core/src/pipeline.rs` examines the sort and a block that does not change. `crates/core/src/review.rs` examines a recipient at a new place. `crates/api/ui-tests/people.test.mjs` examines the sequence on the screen and the fields of the sheet.

Visual: the planning model looks at the row and the sheet at 1280 px, and at 390 px with touch, and drives an add, a change of an address and a sort.
