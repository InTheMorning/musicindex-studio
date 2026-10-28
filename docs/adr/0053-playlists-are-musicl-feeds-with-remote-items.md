# ADR 0053: Playlists Are musicL Feeds With Remote Items

- Status: Accepted
- Date: 2026-10-26
- Owner: operator
- Amended: 2026-10-27 by operator decision. [ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) item 4 puts "Language" of item 3 on the page "Feed settings". [ADR 0070](0070-a-cover-opens-the-artwork-sheet.md) puts the image of item 3 in the Artwork sheet.

## Context

[ADR 0008](0008-feed-types-and-feed-identity.md) item 1 makes playlists (`podcast:medium` `musicL`) a v1 feed type. Studio reads them, but the UI shows them as read-only.

The research of 2026-10-26 gives these observations:

- The namespace says that a list feed such as `musicL` has no `<item>` and no `<podcast:liveItem>`. It holds only `podcast:remoteItem` elements. The example has a channel `podcast:guid`, a value block, and remote items with `feedGuid`, `feedUrl`, `itemGuid` and `medium="music"` (`musicindex-research`, `sources/podcast-namespace/docs/tags/medium.md`).
- In `podcast:remoteItem`, `feedGuid` is required. `feedUrl`, `itemGuid`, `medium` and `title` are optional (`docs/tags/remote-item.md`).
- Stophammer keeps `itemGuid` and `title` of each remote item and the value block of a playlist. It does not list `musicL` feeds in public listings (stophammer ADR 0038, archived ADR 0060).
- MSP 2.0 does not edit `musicL` feeds.
- The Studio core already models `remote_items` of the channel. The editor sends the model with entry keys ([ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md)). Thus an add, a removal or a new sequence of remote items is a change of the model.
- The collection keeps the GUID and the title of each item of each feed (`feed_items`). The musicindex client gives the tracks and the address of an indexed feed.

On 2026-10-26 the operator selected the scope and the track sources below.

## Decision

1. Studio edits a `musicL` feed that it can write, as it edits an album feed.
2. The user can make a new playlist. It is a new feed with `podcast:medium` `musicL`, made as a new album feed is made ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)). Its GUID comes from its address ([ADR 0008](0008-feed-types-and-feed-identity.md) item 2).
3. The playlist page edits the channel fields that apply: title, author, description, image, link and language. It has the value routes of the channel, with the value editor of the album page.
4. Each track of a playlist is a `podcast:remoteItem` of the channel, with `feedGuid`, `itemGuid` and `medium="music"`. It has `feedUrl` and `title` when Studio knows them.
5. The track picker has two sources:
   - A search of musicindex. The user selects a feed, and then a track of that feed. musicindex gives the feed address and the track titles.
   - The releases of the collection. Each release with a feed GUID gives its tracks from the last read, and its public address.
6. A new track goes last. The user changes the sequence and removes a track on the playlist page. The same track (the same `feedGuid` and `itemGuid`) cannot be in a playlist two times.
7. A playlist has no `<item>`. Studio does not add one, and the import of music files is not available on a playlist page.
8. A playlist does not change the feeds of its tracks. It has no two-way link ([ADR 0041](0041-publisher-feeds-and-two-way-links.md) does not apply).
9. A playlist has the same draft, diff, write and announcement as each other feed ([ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0052](0052-announcements-through-the-podcast-index-hub.md)).

## Alternatives Considered

- A search of musicindex only. Not chosen by the operator. The tracks of the user's own new releases are often not in the index yet.
- A third source, a pasted feed address or GUID. Not chosen by the operator. It adds UI, and the two sources cover the need now.
- A two-way link from a release to each playlist that holds it. Rejected: a playlist does not belong to the artist of its tracks, and the namespace defines no such link.

## Consequences

- The shell treats `musicL` as an editable medium.
- A playlist row with no `title` shows the feed GUID and the item GUID. Studio does not read each remote feed to find its title.
- Phase 11 builds the playlist editor in milestone 11a, before the live items of [ADR 0054](0054-live-items-connected-apps-and-show-grants.md).

## Verification

Mechanical:

- A test makes a new `musicL` feed, adds two remote items through the draft, and writes it. The written feed has the two `podcast:remoteItem` elements in sequence, and no `<item>`.
- A test gives the tracks of the collection for the picker, with the feed GUID, the item GUID, the title and the public address.
- The Node tests give the rules of the picker: no second copy of a track, a new track last, and the summary of a row.
- The corpus tests of the three `musicL` feeds still give the same bytes after a read and a write.

Visual:

- The playlist page, the picker with its two sources, and a new playlist. Each one in the dark theme, in the light theme and at 390 px.

## Invariants

- A playlist has no `<item>`.
- A change of a playlist does not change a different feed.
