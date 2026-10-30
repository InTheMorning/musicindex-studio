# ADR 0082: A Publisher Feed Has A Set Of Roles

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.
- Changes: [ADR 0041](0041-publisher-feeds-and-two-way-links.md) items 1, 4 and 11, which give one role, `artist` or `label`.
- Amended: 2026-10-29 by operator decision. [ADR 0083](0083-hosting-is-a-first-class-role.md) gives the sequence of musicindex ADR 0036 to item 1. It adds "Hosting" to the menu of item 7 and to the form of item 9.
- Amended: 2026-10-29 by operator decision. [ADR 0084](0084-the-label-of-a-feed-lists-each-role.md) replaces the label of item 6. The label lists each role.
- Amended: 2026-10-30 by operator decision. [ADR 0087](0087-the-two-sides-of-a-link-confirm-a-role.md) also fills the dot when the two sides of a link give the first role (items 5 and 6).
- Uses: musicindex ADR 0011 item 1 (`rel` is a set of role words), musicindex ADR 0012 items 5 and 8 (the sequence and the color of the roles, and the hollow dot), musicindex ADR 0033 R-WORD-3 and musicindex ADR 0035 R-WORD-6 (the label).

## Context

- On 2026-10-29 the operator opened the artist feed "HeyCitizen" in a local Studio. Each release link of the feed has `rel="artist host author label producer"`.
- `publisher::inferred_role` read `rel` as one word. It found no role, and the collection recorded none. Thus the page showed "Feed" with a hollow dot. On `studio.musicindex.org` the same feed showed "Artist". The operator selected the role there on an earlier date.
- The collection kept one role, `artist` or `label`, in the column `publisher_role`. The type label used only that recorded role.
- musicindex ADR 0012 item 5 gives eight roles with a color of their own: artist, label, host, author, producer, network, hosting, sponsor.

## Decision

1. A publisher feed has a set of roles from the eight roles of musicindex ADR 0012 item 5. Studio keeps the set in the sequence of that item.
2. The collection keeps the set of each publisher feed, and a state: inferred or confirmed.
3. Studio reads `rel` as musicindex ADR 0011 item 1 does: a value with a comma is a comma list, and each other value is a list separated by white space. Each word is lowercased. A word that is not one of the eight roles is not recorded.
4. When Studio reads a publisher feed that has no confirmed set, it records an inferred set: each role that the `rel` of a channel `remoteItem` with `medium="music"` gives.
5. A set becomes confirmed when the user selects it, or when Studio records it for a change of a link (ADR 0041 item 4).
6. The type label of a publisher feed is its first role (musicindex ADR 0035 R-WORD-6). An inferred set has a hollow dot, and a confirmed set has a filled dot (musicindex ADR 0033 R-WORD-3 item 3). A feed with no role has the label "Feed" and a hollow dot.
7. The Role field of the publisher page is a select menu with "Artist", "Label" and "Custom". "Artist" gives the set `artist`, and "Label" gives the set `label`. "Custom" shows a check box for each of the eight roles. The menu shows "Custom" for each other set.
8. A link writes the set in `rel`, with one space between two roles. A change of the set changes the `rel` of each link, as ADR 0041 item 11 says.
9. The form that makes a new publisher feed keeps the roles "Artist" and "Label".
10. A collection from before this decision keeps its recorded role as a confirmed set of one role.

## Alternatives Considered

- Record the inferred role as confirmed. Not chosen by the operator: a filled dot then does not mean a confirmed role.
- Keep "Publisher" as the label of a set with two or more roles and no artist. Not chosen by the operator (musicindex ADR 0035).
- A text field for role words that are not in the eight roles. Not chosen by the operator.

## Consequences

- The HeyCitizen feed shows "Artist" with a hollow dot until the operator confirms the set.
- Studio does not keep a role word that is not one of the eight roles. A change of the set writes `rel` with no such word.

## Invariants

- A link writes the roles of its set in the sequence of musicindex ADR 0012 item 5.
