# ADR 0085: Each Artist And Label Is A Destination

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) items 1, 2 and 11. Releases is no longer the only destination of the left panel, and it no longer lists the publisher feeds.
- Uses: musicindex ADR 0027 R-SHELL-1 items 1 and 2, R-LIST-1, musicindex ADR 0030 R-SHELL-2 item 2 and musicindex ADR 0037 R-WORD-7.

## Context

- Releases listed each feed of the collection in one list: the albums, the playlists and the publisher feeds. ADR 0084 gave the type label of a publisher feed each of its roles, so a row could be too long for its line.
- The operator wants an artist, a label or a hosting service to see its feeds in a hierarchy. On 2026-10-30 the operator decided on one level: each publisher feed, then its releases.
- [ADR 0041](0041-publisher-feeds-and-two-way-links.md) links a publisher feed and a release. [ADR 0086](0086-a-playlist-links-to-its-artist-or-label.md) makes a playlist a release.
- musicindex ADR 0033 R-WORD-4 permits the word "publisher" only as an RSS name. Thus the interface cannot use the words "No publisher" or "Publisher order".
- Amended: 2026-10-30 by operator decision. [ADR 0089](0089-the-left-panel-shows-the-art-of-each-publisher-feed.md) shows the art of the feed in the row of item 1, with the dot on the art.
- Amended: 2026-10-30 by operator decision. [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) keeps each feed that is not an album or a playlist out of Releases and "Not linked" (items 3 and 6), and adds "Other feeds".
- Amended: 2026-10-30 by operator decision. [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) shows only the top feeds at level 0, with role headings, and gives each feed a level (items 1 and 3). It leaves the feeds above and below out of the note of item 5.

## Decision

1. Below Releases, the left panel has one destination for each publisher feed of the collection, A to Z by title. A row shows the dot of the feed (ADR 0084 item 2), its title and the number of its releases. The row Releases opens Releases from each of these destinations.
2. The core gives the links of each feed of the collection. A publisher feed has its releases, and a release has its publisher feeds. A link counts when one of the two feeds names the other. A publisher feed gives first the releases of its channel, in that sequence, then the releases that only name it. A release gives first the feeds of its `podcast:publisher`, in that sequence, then the feeds that only list it.
3. After the publisher feeds, the destination "Not linked" holds the releases with no publisher feed in the collection. It shows only when such a release exists.
4. A destination is a list page with the title of the feed. Its head shows the label of ADR 0084, with each role, and the button "Open the feed". Its sorts are "Feed order", the default, then "Newest" and "A to Z". "Feed order" is the sequence of item 2. "Not linked" has the sorts of Releases, and no "Open the feed".
5. A release of more than one publisher feed shows on the page of each feed. On each page, a line names the other feeds, for example "Also on Some Label".
6. Releases lists each album and each playlist. A publisher feed opens from its destination, not from a row of Releases.
7. At 760 px or less, the root page shows the destinations of item 1 below Releases. A destination opens as a page of the root (musicindex ADR 0030 R-SHELL-2 item 2).
8. A destination keeps its sort in the navigation record of the session. The address keeps only the sort and the view of Releases (ADR 0039 item 2a).

## Alternatives Considered

- Group the rows of Releases below a head for each publisher feed. Not chosen by the operator: the left panel holds the destinations.
- A tree of hosting, label, artist and release. Not chosen for this decision: it needs a link from a publisher feed to a publisher feed in Studio, Stophammer and musicindex.org.
- Group the feeds by the host connection. Not chosen by the operator: "host" in the hierarchy is the hosting role.

## Consequences

- A collection with many publisher feeds has a long left panel. The panel scrolls between the brand and the footer.
- The link "Releases" of an outline opens Releases (ADR 0069). It does not open the destination of a publisher feed.

## Invariants

- The UI has no copy of the link rule of item 2. It shows the fields `release_ids` and `publisher_ids` of the feed list ([ADR 0065](0065-the-core-applies-each-feed-rule.md)).
- `crates/core/tests/publisher.rs` tests item 2. `crates/api/ui-tests/shell.test.mjs` tests items 1 and 3 to 7.
