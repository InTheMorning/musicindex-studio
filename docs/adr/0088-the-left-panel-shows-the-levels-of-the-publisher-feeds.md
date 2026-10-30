# ADR 0088: The Left Panel Shows The Levels Of The Publisher Feeds

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Amended: 2026-10-30 by operator decision. The first role of a feed gives its rank in item 1, and its heading in item 4. A feed is one feed on one level.
- Changes: [ADR 0085](0085-each-artist-and-label-is-a-destination.md) items 1, 3 and 5, and [ADR 0069](0069-the-outline-head-opens-the-feed-page.md), which makes the first row of each outline "‹ Releases".
- Uses: Stophammer ADR 0069 §4 (`co_credited_feeds`) and §"A direct link between a label feed and an artist feed".
- Open: no shared musicindex ADR owns the levels. musicindex ADR 0030 R-SHELL-2 item 3 gives the stack "the root, a list, the outline of an item, then one part of the item". This record adds levels between the root and the outline. The operator decided to write the Studio record first.

## Context

- ADR 0085 gave each publisher feed one destination below Releases, A to Z. A hosting service can host many labels, and a label can have many artists. An artist can be on two labels. A flat list does not show this.
- The namespace gives one level: a publisher feed and its feeds (`docs/examples/publishers/publishers.md`). `podcast:publisher` holds "exactly one" item (`docs/tags/publisher.md`).
- Amended: 2026-10-30 by operator decision. [ADR 0089](0089-the-left-panel-shows-the-art-of-each-publisher-feed.md) shows the art of each feed of a level, with the dot on the art (item 5).
- Stophammer ADR 0069 rejected a direct link from a label feed to an artist feed: "An artist feed can name only one publisher, so an artist with releases on two labels cannot confirm both." Stophammer finds the artists of a label, and the labels of an artist, through their shared releases (`co_credited_feeds`).

## Decision

1. Each publisher feed has a rank. The hosting rank is first, then the rank of a label, then the rank of an artist. The first role of the set gives the rank, in the sequence of musicindex ADR 0036 R-ROLE-1. A feed with the first role `hosting` has the hosting rank. A feed with the first role `label` has the rank of a label. Each other publisher feed has the artist rank.

1a. A feed with the roles `artist` and `label` is one feed at the artist rank. It is not a label above itself.

2. Feed B is below feed A when B has a lower rank, and the two feeds share a release of the collection. B is directly below A when no feed C is below A with B below C. The core gives the feeds directly below and directly above each feed in the feed list, as `child_ids` and `parent_ids`. A feed can be directly below more than one feed.
3. Level 0 of the left panel shows Releases, then each top feed, then "Not linked". A top feed has no feed above it.
4. A list of feeds is in role headings: Hosting, Labels, Artists, then Producers, Hosts, Authors, Networks and Sponsors. A feed is in the heading of its first role. The feeds of a heading are A to Z.
5. A selection of a feed replaces the panel with its level. A level shows "‹" with the name of the level above it, or "‹ Releases" from level 0. Then it shows the head of the feed and the feeds directly below it in their role headings. Last, it shows its releases in the sequence of the feed. The head opens the feed page. The content area shows the releases of the feed, as ADR 0085 item 4 says.
6. A release or a head that opens from a level keeps that level. The first row of its outline is "‹" with the name of the level, and it opens that level. A release that opens from Releases or from "Not linked" keeps "‹ Releases".
7. At 760 px or less, each level is a page of the stack, with the levels above it below it. Back opens the level above.
8. The note "Also on" of a release names only the feeds that are not above or below the feed of the page. This changes ADR 0085 item 5.

## Alternatives Considered

- A direct link from a label feed to an artist feed. Not chosen by the operator, because of the cause that Stophammer ADR 0069 gives.
- A network rank between hosting and label. Not chosen by the operator.
- The rank of the highest role of the set. Not chosen by the operator: a feed with the roles `artist` and `label` then showed as a label above its own artist level.
- Level 0 with each feed in its role heading. Not chosen by the operator: level 0 shows the top feeds only.
- A tree that opens each level in the same panel position. Not chosen: the outline of a feed uses levels with "‹".

## Consequences

- An artist on two labels is below each label. Its page shows its releases of each label.
- A feed below a top feed is not at level 0. The user finds it through the levels, or through Releases.
- Two feeds of the same rank that share a release are not above or below each other. The note "Also on" names each one on the page of the other.

## Invariants

- The UI has no copy of the rules of items 1 and 2. It shows `child_ids` and `parent_ids` ([ADR 0065](0065-the-core-applies-each-feed-rule.md)).
- `crates/core/tests/publisher.rs` has the test `the_levels_follow_the_rank_and_the_shared_releases`. `crates/api/ui-tests/shell.test.mjs` tests items 3 to 8.
