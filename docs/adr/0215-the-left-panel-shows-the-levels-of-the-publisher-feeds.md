# ADR 0215: The Left Panel Shows The Levels Of The Publisher Feeds

- Status: Accepted
- Date: 2026-11-17
- Owner: operator
- Accepted: 2026-11-17. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR restates each live rule of ADR 0088, with each change from a subsequent ADR. It keeps the item numbers of ADR 0088.
- Supersedes: [ADR 0088](../archive/adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md), The Left Panel Shows The Levels Of The Publisher Feeds.
- Changed by: [ADR 0232](0232-each-group-below-an-open-feed-has-an-add-row.md) items 9 to 12 add a link that only Studio keeps to item 2. Items 1 to 7 add the add rows to item 5. Item 8 gives the level of a new feed to item 6.
- Changed by: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) items 15 and 16, on 2026-11-24. Items 1, 1a and 2: a link gives the level, and curator has a rank between label and artist. Item 4: the heading of the role `hosting` is "Publishers".
- Uses: Stophammer ADR 0069 §4 (`co_credited_feeds`) and §"A direct link between a label feed and an artist feed".
- Uses: musicindex ADR 0097 R-ROLE-1 and musicindex ADR 0081 R-CURATOR-1 item 1 for the sequence of the roles.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0085](0085-each-artist-and-label-is-a-destination.md) (the destinations), [ADR 0069](0069-the-outline-head-opens-the-feed-page.md) (the outline head), [ADR 0089](0089-the-left-panel-shows-the-art-of-each-publisher-feed.md) (the art of a row), [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) ("Other feeds" and "No role"), [ADR 0181](0181-a-level-shows-the-state-of-its-own-feed.md) (the tile of the level feed).
- Open: no shared musicindex ADR owns the levels. musicindex ADR 0080 R-SHELL-2 item 3 gives the stack "the root, a list, the outline of an item, then one part of the item". This record adds levels between the root and the outline. The operator decided to write the Studio record first.

## Context

- ADR 0085 gave each publisher feed one destination below Releases, A to Z. A hosting service can host many labels, and a label can have many artists. An artist can be on two labels. A flat list does not show this.
- The namespace gives one level: a publisher feed and its feeds (`docs/examples/publishers/publishers.md`). `podcast:publisher` holds "exactly one" item (`docs/tags/publisher.md`).
- Stophammer ADR 0069 rejected a direct link from a label feed to an artist feed: "An artist feed can name only one publisher, so an artist with releases on two labels cannot confirm both." Stophammer finds the artists of a label, and the labels of an artist, through their shared releases (`co_credited_feeds`).
- ADR 0181, ADR 0089 and ADR 0090 changed items 3, 4 and 5 of ADR 0088. The global decision of 2026-11-14 (musicindex-research `df7b823`) sets one current ADR for an area after three changes. musicindex ADR 0081 adds the role curator, and that change needs item 4 again. This ADR is the current ADR for the levels.
- ADR 0088 is in `docs/archive/adr/` for research, with its alternatives and its checks.

## Decision

1. Each publisher feed has a rank. The hosting rank is first, then the rank of a label, then the rank of an artist. The first role of the set gives the rank, in the sequence of musicindex ADR 0097 R-ROLE-1. A feed with the first role `hosting` has the hosting rank. A feed with the first role `label` has the rank of a label. Each other publisher feed has the artist rank.

1a. A feed with the roles `artist` and `label` is one feed at the artist rank. It is not a label above itself.

2. Feed B is below feed A when B has a lower rank, and the two feeds share a release of the collection. B is directly below A when no feed C is below A with B below C. The core gives the feeds directly below and directly above each feed in the feed list, as `child_ids` and `parent_ids`. A feed can be directly below more than one feed.
3. Level 0 of the left panel shows Releases, then each top feed, then "Not linked". A top feed has no feed above it. After "Not linked", the destination "Other feeds" shows when a feed that is not a release or a publisher feed exists (ADR 0090 item 2).
4. A list of feeds is in role headings. The first headings are Hosting, Labels and Artists. Then each other role has a heading, in the sequence of musicindex ADR 0097 R-ROLE-1. With the sequence of musicindex ADR 0081 R-CURATOR-1 item 1, these are Curators, Producers, Hosts, Authors, Networks and Sponsors. A feed is in the heading of its first role. The feeds of a heading are A to Z.
    - A publisher feed with no role is in the heading "No role" (ADR 0090 item 2).
5. A selection of a feed replaces the panel with its level. A level shows "‹" with the name of the level above it, or "‹ Releases" from level 0. Then it shows the head of the feed and the feeds directly below it in their role headings. Last, it shows its releases in the sequence of the feed. The head opens the feed page.
    - Each row of a feed shows the art of the feed and then its title. The head shows the art at 40 px. The type dot is on the art (ADR 0089 items 1 to 4).
    - The row "Not linked" has no art. A space of 28 px keeps its title in line with the other titles (ADR 0089 item 5).
    - In an open level, the head row shows the mark of the publisher feed of the level only. The row of a closed level keeps the mark that covers its feed and its releases (ADR 0181 items 1 and 5).
    - The content area shows the publisher feed of the level as the first tile (ADR 0181 items 2 to 4). Then it shows the releases of the feed (ADR 0085 item 4).
6. A release or a head that opens from a level keeps that level. The first row of its outline is "‹" with the name of the level, and it opens that level. A release that opens from Releases or from "Not linked" keeps "‹ Releases".
7. At 760 px or less, each level is a page of the stack, with the levels above it below it. Back opens the level above.
8. The note "Also on" of a release names only the feeds that are not above or below the feed of the page. This changes ADR 0085 item 5.

## Item Map

| ADR 0088 item | ADR 0215 item | Change |
| --- | --- | --- |
| 1, 1a, 2 | 1, 1a, 2 | None. |
| 3 | 3 | ADR 0090 item 2 adds "Other feeds". |
| 4 | 4 | ADR 0090 item 2 adds "No role". The other headings follow the role sequence, with Curators from musicindex ADR 0081. |
| 5 | 5 | ADR 0089 gives the art of each row and the space of "Not linked". ADR 0181 gives the marks of a level and the tile of the level feed. |
| 6, 7, 8 | 6, 7, 8 | None. |

## Consequences

- An artist on two labels is below each label. Its page shows its releases of each label.
- A feed below a top feed is not at level 0. The user finds it through the levels, or through Releases.
- Two feeds of the same rank that share a release are not above or below each other. The note "Also on" names each one on the page of the other.
- A curator feed has the artist rank. Its heading is "Curators".

## Invariants

- The UI has no copy of the rules of items 1 and 2. It shows `child_ids` and `parent_ids` ([ADR 0065](0065-the-core-applies-each-feed-rule.md)).
- `crates/core/tests/publisher.rs` has the test `the_levels_follow_the_rank_and_the_shared_releases`. `crates/api/ui-tests/shell.test.mjs` tests items 3 to 8, and the heading "Curators" of item 4.
