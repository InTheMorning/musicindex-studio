# ADR 0030: An Added Recipient Gets No Split

- Status: Accepted
- Date: 2026-10-23
- Owner: operator

## Context

[ADR 0027](0027-the-recipient-picker.md) item 7 says that each recipient that the picker adds keeps the split of its source. Task 013 implemented this for a recipient from musicindex.

On 2026-10-23 the operator did the visual check of task 013. The operator imported a recipient from a musicindex track, and it arrived with a split of about half the shares of the track. The operator decided that an imported recipient must not keep its split. A split in a different feed is for a different set of recipients, so it is not correct in the feed of the editor.

## Decision

1. Each recipient that the picker adds gets an empty split, from the recipients of the user and from musicindex. The user gives each new recipient its split.
2. A recipient from musicindex keeps its fee mark. The mark tells the user what type of recipient it is.
3. This ADR changes ADR 0027 item 7 only. The other items of ADR 0027 do not change.

## Alternatives Considered

- Keep the split of the source, as ADR 0027 item 7 said. Rejected by the operator: the split of a different feed is not correct here.
- Remove the fee mark also. Not chosen: with an empty split, the user must examine the recipient, and the fee mark helps the user to give it a correct split.

## Consequences

- After an add, the share bar of the list shows the new recipients with no part until the user gives them a split.
- ADR 0027 has a note that names this ADR.

## Invariants

- The picker adds each recipient with an empty split.
