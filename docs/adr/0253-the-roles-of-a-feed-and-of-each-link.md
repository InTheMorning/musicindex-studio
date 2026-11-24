# ADR 0253: The Roles Of A Feed And Of Each Link

- Status: Accepted
- Date: 2026-11-24
- Owner: operator
- Accepted: 2026-11-24 by the operator. The operator selected roles for each link, four level roles and the screen word "Publisher" for the role `hosting` ([note of 2026-11-24](../notes/2026-11-24-roles-levels-and-words.md)). musicindex ADR 0097 records these shared rules. Items 1 to 10 and 17 to 19 restate the live rules of the four superseded records, with the words of musicindex ADR 0097.
- Operator choice, 2026-11-24: the area has one current record, because ADR 0082 had three "Changed by" lines.
- Supersedes: [ADR 0082](../archive/adr/0082-a-publisher-feed-has-a-set-of-roles.md), A Publisher Feed Has A Set Of Roles.
- Supersedes: [ADR 0083](../archive/adr/0083-hosting-is-a-first-class-role.md), Hosting Is A First-Class Role.
- Supersedes: [ADR 0084](../archive/adr/0084-the-label-of-a-feed-lists-each-role.md), The Label Of A Feed Lists Each Role.
- Supersedes: [ADR 0087](../archive/adr/0087-the-two-sides-of-a-link-confirm-a-role.md), The Two Sides Of A Link Confirm A Role.
- Changes: [ADR 0215](0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) items 1, 1a, 2 and 4. A link gives the level, curator has a rank of its own, and the heading of the role `hosting` is "Publishers" (items 15 and 16).
- Changes: [ADR 0240](0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) items 1, 2 and 3. The menu value and the Welcome row say "Publisher" (item 12).
- Restated in: [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) items 1, 2, 4, 6 and 11 restate items 1 to 5, 10, 20 and 22 of this ADR, and cite them.
- Uses: musicindex ADR 0097 R-ROLE-1 to R-ROLE-8, R-WORD-8 and R-WORD-15. musicindex ADR 0033 R-WORD-3 to R-WORD-5. musicindex ADR 0037 R-WORD-7. musicindex ADR 0067 item 82. Stophammer ADR 0101 (the `rel` of each link). ADR 0041 item 4 (a change of a link). ADR 0251 (Review writes the two sides of a link).

## Context

- Studio kept one role set for each publisher feed, and each link of the feed wrote that set in `rel` (ADR 0082 items 1 and 8). A label that hosts the EP of a friend thus put the EP on the label.
- The word "host" had three meanings: the role `hosting`, the role `host` of a person who presents, and the file host of ADR 0092. Studio showed the role `hosting` as "Hosting".
- musicindex ADR 0097 gives each link its own roles, four level roles, and the screen word "Publisher" for the role `hosting`. It supersedes musicindex ADR 0036 and ADR 0066, and it keeps their rule IDs.
- ADR 0082, ADR 0083, ADR 0084 and ADR 0087 are in `docs/archive/adr/` for research, with their context, alternatives and evidence.

## Decision

### The Role Set Of A Feed

1. A publisher feed has a set of roles from the known roles of musicindex ADR 0097 R-ROLE-1: artist, label, hosting, curator, producer, host, author, network, sponsor. Studio keeps the set in that sequence.
2. The collection keeps the set of each publisher feed, and a state: inferred or confirmed.
3. Studio reads `rel` as musicindex ADR 0067 item 82 does. A value with a comma is a comma list, and each other value is a list separated by white space. Each word is lowercased. A word that is not a known role is not recorded (musicindex ADR 0097 R-ROLE-3).
4. When Studio reads a publisher feed that has no confirmed set, it records an inferred set: each role that the `rel` of a channel `remoteItem` with `medium="music"` or `medium="musicL"` gives (ADR 0086 item 3).
5. A set becomes confirmed when the user selects it, or when Studio records it for a change of a link (ADR 0041 item 4).
6. A collection from before ADR 0082 keeps its recorded role as a confirmed set of one role.

### The Type Label And The Dot

7. The type label of a publisher feed with roles lists the screen word of each role in the sequence of item 1. It puts " · " between two words (musicindex ADR 0037 R-WORD-7). An example is "Label · Publisher". A feed with no role has the label "Feed".
8. The dot has the color of the first role (musicindex ADR 0097 R-WORD-8). The dot is filled when the feed list gives `roles_linked` (item 17). The dot is also filled when the user confirmed the set (item 5) and the feed has no link yet. When only one side of a link gives the first role, or Studio infers the set, the dot is hollow. A feed with no role has a hollow dot.
9. Each part of the UI that shows the type label uses this label: the Releases list, the left panel, the head of the page, the section title of the publisher page and the found feeds of a search. A row cuts a long label at the end of the line, as it cuts a long title.
10. The Role field of the publisher page is a select menu with "Artist", "Label", "Publisher" and "Custom". "Artist" gives the set `artist`, "Label" gives the set `label`, and "Publisher" gives the set `hosting`. "Custom" shows a check box for each known role. The menu shows "Custom" for each other set. The sheet that makes a publisher feed has the roles "Artist", "Label" and "Publisher".

### The Screen Word Of The Role `hosting`

11. The screen word of the role `hosting` is "Publisher", and a sentence says "published by" (musicindex ADR 0097 R-WORD-15). The `rel` word stays `hosting`. No screen uses "host", "hosting" or "Hosting" for this role. The role `host` keeps the word "Host". The file hosts keep the words of ADR 0092.
12. The texts of ADR 0240 change:
    - Item 1: the menu **＋ Add** has the value "Publisher". It replaces "Hosting".
    - Item 2: the Welcome row is "I publish music for other artists", with the line "Start with your publisher page. The artists and labels that you publish go on it."
    - Item 3: the row of item 2 opens the sheet of the first steps. The core makes the publisher feed with the role `hosting`, as before.
    - Each other text of ADR 0240 that says "Hosting" for the role `hosting` says "Publisher" (item 11). An example is the menus of items 12 to 15.
13. A screen does not use the words "publisher feed" for each feed with roles. Code names can stay (musicindex ADR 0097 R-WORD-15 item 3). The word "publisher" shows in an interface text only as the screen word of `hosting` or in "published by" (R-WORD-15 item 5). Studio does not show the label "Publisher" of musicindex ADR 0033 R-WORD-3 item 4 for a feed with no role. That feed has the label "Feed" (item 7). A technical view can show an RSS name, for example `podcast:publisher`.
14. The guard of musicindex ADR 0033 R-WORD-5 in Studio has the message of musicindex ADR 0097 R-WORD-15 item 6.

### The Levels Come From A Link

15. These items change ADR 0215 items 1, 1a and 2:
    - The level roles are hosting, label, curator and artist, in that rank (musicindex ADR 0097 R-ROLE-8). The other roles are credits. They show on a page and make no level.
    - The rank of a feed comes from its first level role in the sequence of item 1. A feed with the roles `artist` and `hosting` is one feed at the artist rank. A feed with the roles `artist` and `label` is also one feed at the artist rank.
    - Feed B is below feed A when A and B link the same release of the collection, and the link of A gives a level role with a higher rank than the rank of B. A kept link (ADR 0232 item 10) also puts B below A. The role set of A alone does not put B below A.
    - Example: Elm Street Duo has the roles `artist hosting`. Its link to the EP of Harbor Lights gives `hosting`, so Harbor Lights is below Elm Street Duo. Its links to its own releases give `artist hosting`, so Elm Street Duo is one row.
16. ADR 0215 item 4 changes: the heading of the role `hosting` is "Publishers", and the heading of the role `curator` comes after "Labels" and before "Artists".

### The Roles Of Each Link

17. The core finds the first role of a publisher feed on the two sides of a link. The feed has the role in its set, and the two sides of one link give it: the `rel` of the `remoteItem` of the feed, and the `rel` of the `podcast:publisher` link of the release. The feed list gives the result as `roles_linked`. The core reads it again at each read of the list, and the collection does not keep it.
18. The Role section of a feed with `roles_linked` says: "The feed and its releases give the same role."
19. When no link gives the first role on its two sides, `roles_linked` does not give it, and the dot is hollow again (item 8). An example is a feed with one release that removes the role from its link.
20. Each link writes its own roles in `rel`, on the two sides of the link. The roles of a link are a subset of the set of the feed (musicindex ADR 0097 R-ROLE-7). The words have the sequence of item 1, with one space between two roles. This replaces ADR 0082 item 8.
21. A new link gets each level role (item 15) of the set of the feed (musicindex ADR 0097 R-ROLE-7 item 2). A credit role of the set goes on a link only when the user selects it for that link. The user can remove a role from one link. A change of one link changes only the `rel` of that link, and Review writes the two sides (ADR 0251).
22. A change of the set of a feed adds a new role to each link of the feed, and removes a removed role from each link. A role that the user removed from one link stays off that link.
23. The set of a feed holds each role that the `rel` of one of its links gives, and each role that the user selects (musicindex ADR 0097 R-ROLE-7 item 2). The `rel` of a link has no role that is not in the set.

## Item Map

| Superseded item | ADR 0253 item | Change |
| --- | --- | --- |
| ADR 0082 item 1, ADR 0083 item 1 (the sequence) | 1 | The known roles of musicindex ADR 0097 R-ROLE-1. |
| ADR 0083 item 1 (the label of the first role) | 7 | The label lists each role, as ADR 0084 gave. |
| ADR 0082 items 2, 3 and 5 | 2, 3, 5 | None. |
| ADR 0082 item 4 | 4 | Also `musicL` links, as ADR 0086 item 3 gives. |
| ADR 0082 item 10 | 6 | None. |
| ADR 0082 item 6, ADR 0084 items 1 and 2, ADR 0087 item 2 | 7, 8 | The screen word "Publisher" for the role `hosting`. A confirmed set with links fills the dot only when the two sides of a link give the first role (musicindex ADR 0097 R-WORD-8 item 2). |
| ADR 0084 item 3 | 9 | None. |
| ADR 0082 items 7 and 9, ADR 0083 items 2 and 3 | 10 | "Publisher" in place of "Hosting". |
| ADR 0082 item 8 | 20 to 23 | Each link writes its own roles (musicindex ADR 0097 R-ROLE-7). |
| ADR 0087 items 1 and 3 | 17, 18 | The two sides of one link give the role. |
| ADR 0087 consequence | 19 | Each link of the feed is examined. |
| (new) | 11 to 16 | musicindex ADR 0097 R-ROLE-8 and R-WORD-15. |

## Alternatives Considered

- Keep one role set for each feed. Not chosen by the operator: a label that hosts the EP of a friend then puts the EP on the label.
- Keep the word "Hosting", or use "Hosted by". Not chosen by the operator: "host" has three meanings in Studio.
- Show a feed with the roles `artist` and `hosting` as a publisher row with an artist row below it. Not chosen by the operator: the same feed shows two times.
- The other alternatives are in the archived records.

## Consequences

- A link to the EP of a friend can give only `hosting`, so the EP is not on the label.
- The type label of a feed with `label hosting` is "Label · Publisher".
- A feed with the roles `hosting` and `producer` has the label "Publisher · Producer".
- A link writes `rel` in the sequence of item 1. A `rel` that Studio wrote before keeps its sequence until the next change of that link.
- A curator feed is not at the artist rank. Its heading is "Curators", after "Labels".

## How This Decision Is Checked

The tests below exist, but they check the rules of the superseded records. The code tasks of this ADR change them as each line gives. Until then, the mechanical check of items 8, 11, 15, 16 and 20 to 23 is open.

Mechanical:

- `crates/api/ui-tests/words.test.mjs`: the type label of each role gives "Publisher" for `hosting` (it gives "Hosting" now), and the guard has the message of item 14.
- `crates/core/tests/publisher.rs`: `a_role_that_the_two_sides_of_a_link_give_is_linked` (item 17) also reads a feed with two links, where only one link gives the role on its two sides (item 19). A new test writes a link with a subset of the roles of the feed, and reads the two sides (items 20 and 21).
- `crates/core/tests/publisher.rs`: `the_levels_follow_the_rank_and_the_shared_releases` gets a feed `artist hosting` with another artist below it (item 15).
- `crates/api/ui-tests/shell.test.mjs`: the heading "Publishers", and "Curators" after "Labels" and before "Artists" (item 16). It puts "Curators" after "Artists" now.

Visual (a person, or the main session with screenshots):

- A feed with the roles `label hosting` shows "Label · Publisher" in the left panel and in the head of its page.
