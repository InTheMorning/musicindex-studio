# ADR 0087: The Two Sides Of A Link Confirm A Role

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) items 5 and 6. These items give a filled dot only to a set that the user selects, or that Studio records for a link change.
- Uses: musicindex ADR 0033 R-WORD-3 item 3.

## Context

- musicindex ADR 0033 R-WORD-3 item 3 says: "When the two sides of the link give the role, the dot is filled."
- ADR 0082 item 5 confirms a set only when the user selects it, or when Studio makes a change of a link. A read-only feed has no Role field that the user can change. Thus its dot stayed hollow.
- On 2026-10-30, the HeyCitizen artist feed had a hollow dot in Studio. Its channel items and the `podcast:publisher` link of its albums gave the same `rel`: `artist host author label producer`. musicindex.org showed a filled dot for the same feed.

## Decision

1. The core finds the first role of a publisher feed on the two sides of a link. The feed has the role in its set. A release of the collection gives the role in the `rel` of its `podcast:publisher` link to the feed. The feed list gives the result as `roles_linked`. The core reads it again at each read of the list, and the collection does not keep it.
2. The dot of a publisher feed is filled when its set is confirmed (ADR 0082 item 5), or when the feed list gives `roles_linked`. In each other condition, the dot is hollow.
3. The Role section of a feed with `roles_linked` says: "The feed and its releases give the same role."

## Alternatives Considered

- Let the user confirm the roles of a read-only feed. Not chosen by the operator: Studio and musicindex.org then disagree until the user confirms each feed.

## Consequences

- The HeyCitizen feed has a filled artist dot, also when it is read-only.
- When a release removes the role from its link, the dot is hollow again.

## Invariants

- `crates/core/tests/publisher.rs` has the test `a_role_that_the_two_sides_of_a_link_give_is_linked`. `crates/api/ui-tests/words.test.mjs` tests item 2.
