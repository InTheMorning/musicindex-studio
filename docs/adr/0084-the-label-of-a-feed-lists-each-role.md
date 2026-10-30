# ADR 0084: The Label Of A Feed Lists Each Role

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.
- Changes: [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) item 6 and [ADR 0083](0083-hosting-is-a-first-class-role.md) item 1, which give a feed the label of its first role.
- Uses: musicindex ADR 0037 R-WORD-7.

## Context

- ADR 0082 and ADR 0083 gave a publisher feed with roles the word of its first role. The artist feed "HeyCitizen" showed "Artist", and its other roles showed only in the Role section of its page.
- On 2026-10-29 the operator decided that each app shows all the roles of a feed. musicindex ADR 0037 records the shared rule.

## Decision

1. The type label of a publisher feed with roles lists the word of each role, with " · " between two words. The words are in the sequence of musicindex ADR 0036 R-ROLE-1.
2. The dot has the color of the first role. An inferred set has a hollow dot, as ADR 0082 item 6 says.
3. Each part of the UI that shows the type label uses this label: the Releases list, the left panel, the head of the page, the section title of the publisher page and the found feeds of a search.

## Alternatives Considered

- Keep the first role as the label. Not chosen by the operator.

## Consequences

- The HeyCitizen feed shows "Artist · Label · Producer · Host · Author".
- A long label can fill its line. A row cuts it at the end of the line, as it cuts a long title.

## Invariants

- The type label of a publisher feed names each of its roles.
