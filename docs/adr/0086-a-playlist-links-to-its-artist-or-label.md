# ADR 0086: A Playlist Links To Its Artist Or Label

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0041](0041-publisher-feeds-and-two-way-links.md) items 5, 6 and 11, which link a publisher feed to music feeds only.
- Uses: the Podcasting 2.0 tags `podcast:publisher` and `podcast:medium`, in `../musicindex-research/sources/podcast-namespace` at `d4036dc`.

## Context

- ADR 0041 links a publisher feed and an album. A playlist (`musicL`, [ADR 0053](0053-playlists-are-musicl-feeds-with-remote-items.md)) had no link to an artist feed or a label feed.
- [ADR 0085](0085-each-artist-and-label-is-a-destination.md) shows the releases of each publisher feed. On 2026-10-30 the operator decided that a playlist can be in this list.
- `docs/tags/publisher.md` of the namespace puts `podcast:publisher` in `channel`, and it does not limit the `podcast:medium` of the feed. `docs/tags/medium.md` says that a publisher feed "links to other feeds a publisher owns".
- Stophammer reads `musicL` feeds and keeps the `podcast:publisher` link of each feed. A test that shows a playlist on a page of musicindex.org was not done.

## Decision

1. A release is an album (`music`) or a playlist (`musicL`). A link joins a publisher feed and a release, with the checks of ADR 0041 item 5.
2. The channel `remoteItem` in the publisher feed has the `medium` value of the release: `music` or `musicL`. The link in the release does not change: a `remoteItem` with `medium="publisher"` in `podcast:publisher`.
3. Studio reads the roles of a publisher feed from its `music` items and its `musicL` items (ADR 0082 item 4). A change of the roles changes the `rel` of the two kinds of item.
4. The playlist page has the section "Artists and labels" of the album page, with the tier "details" (ADR 0071). The publisher page offers the playlists of the collection as releases.

## Alternatives Considered

- Keep the link for albums only, and give the playlists a destination of their own. Not chosen by the operator.

## Consequences

- A playlist shows on the page of its artist or label in Studio.
- An app that reads only the `music` items of a publisher feed does not show the playlist. The album links do not change.

## Invariants

- A link writes the `medium` value of its release in the channel `remoteItem` of the publisher feed.
- `crates/core/tests/publisher.rs` has the test `a_playlist_links_to_a_publisher_feed_as_an_album_does`.
