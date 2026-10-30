# ADR 0083: Hosting Is A First-Class Role

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.
- Changes: [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) items 1, 7 and 9.
- Amended: 2026-10-29 by operator decision. [ADR 0084](0084-the-label-of-a-feed-lists-each-role.md) replaces the label of item 1. The label lists each role.
- Uses: musicindex ADR 0036 R-ROLE-1 (the sequence of the roles) and R-ROLE-2 (the first-class roles).

## Context

- ADR 0082 gave a publisher feed a set of roles in the sequence of musicindex ADR 0012 item 5: artist, label, host, author, producer, network, hosting, sponsor. The Role menu had "Artist", "Label" and "Custom". The form that makes a publisher feed had "Artist" and "Label".
- On 2026-10-29 the operator decided that hosting is a first-class role, like artist and label. musicindex ADR 0036 gives the new sequence: artist, label, hosting, producer, host, author, network, sponsor.

## Decision

1. Studio keeps the roles of a set in the sequence of musicindex ADR 0036 R-ROLE-1. The type label is the first role in that sequence. This replaces the sequence of ADR 0082 item 1.
2. The Role menu of the publisher page has "Artist", "Label", "Hosting" and "Custom". "Hosting" gives the set `hosting`. This replaces the menu of ADR 0082 item 7.
3. The form that makes a publisher feed has the roles "Artist", "Label" and "Hosting". This replaces ADR 0082 item 9.

## Alternatives Considered

- A hue of its own for hosting. Not chosen by the operator (musicindex ADR 0036).

## Consequences

- A feed with the roles "hosting" and "producer" has the label "Hosting".
- A link writes `rel` in the new sequence. A `rel` that Studio wrote before keeps its sequence until the next change of the set.

## Invariants

- A link writes the roles of its set in the sequence of musicindex ADR 0036 R-ROLE-1.
