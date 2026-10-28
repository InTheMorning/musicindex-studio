# ADR 0077: A New Album Can Start From An Album

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) items 5 and 6, which give the fields of the form that makes a new feed.

## Context

- MSP 2.0 makes a new feed from a template (`origin/master:src/App.tsx`, `src/utils/regenerateGuids.ts`). The template keeps each field and each track. It gives a new feed GUID and a new GUID to each track. It removes the `atom:link` with `rel="self"` and each `podcast:txt`, because a verification token names one feed.
- MSP 2.0 also removes the artist key `podcast:txt purpose="npub"`. That key names the artist, not the feed.
- Studio gives a new feed the GUID of its address ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)). A sent item with no key and no GUID gets a new random UUID ([ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md) item 7).
- One form makes each new feed. Its first field is "Type" ([ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) item 5).
- A link between a publisher feed and a release changes the two feeds ([ADR 0041](0041-publisher-feeds-and-two-way-links.md)).
- Studio deletes a file on a host only when no feed of the library refers to its address ([ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 17).

The operator selected each decision of this ADR from a questionnaire on 2026-10-28.

## Decision

### The Form

1. When "Type" is Album and the collection has an album, the form shows the select menu "Start from", after "Type". The first value is "Empty album". Each album of the collection follows, by title.
2. A selected album gives the start value of "Artist". "Album title" stays empty. "Feed file" gets its start value from the title, as ADR 0073 item 8 says.
3. A read-only album can be the source ([ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md)).

### The Copy

4. The route `POST /feeds/new` gets the optional field `from`, the ID of the source album. The core makes the copy.
5. The core copies the draft of the source. With no draft, it copies the last state that Studio read.
6. The copy keeps each field and each track, with these changes:
   - The title and the artist come from the form.
   - `podcast:guid` is the GUID of the new address (ADR 0032).
   - Each track gets a new random item GUID.
   - The `atom:link` with `rel="self"` gives the new address.
   - Each `podcast:txt` goes. The artist key `purpose="npub"` stays ([ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md)).
   - Each `podcast:publisher` and each publisher link goes. The user links the new album from the publisher page.
   - Each live item goes.
   - `itunes:new-feed-url` goes.
   - An OP3 prefix with the GUID of the source gets the GUID of the copy ([ADR 0076](0076-an-op3-switch-on-feed-settings.md)).
7. Each track keeps its audio address and its publication date. The user changes them on the track page.
8. The copy makes no copy of a file. ADR 0035 item 17 keeps each file of the source while the copy refers to its address.
9. Studio then opens the new album. The first write shows the full feed in the diff.

## Alternatives Considered

- Copy the album settings only, with no tracks. Not selected by the operator: the operator selected the copy of each track.
- A button on the album page. Not selected: one form makes each new feed (ADR 0073).
- Remove each `podcast:txt`, as MSP 2.0 does. Rejected: the artist key names the artist, and the new album has the same artist.
- Keep the publisher links. Rejected: a link changes the publisher feed too, and the user did not see that diff.
- Copy each audio file to the release folder of the new album. Not selected: the user usually replaces the tracks, and ADR 0035 item 17 keeps the files of the source.
- Give each track the date of the copy. Not selected: a `pubDate` is the release date of its track ([ADR 0024](0024-track-numbers-follow-the-track-sequence.md)). The user sets the date.

## Consequences

- An MSP 2.0 user can start a new album from an album, as with the MSP 2.0 template.
- The new album and its source share no GUID.
- A file of the source stays on the host while the copy refers to it.

## Invariants

- A copy does not keep the feed GUID or an item GUID of its source.

## Non-Goals

- A copy of a publisher feed, a show feed or a playlist.
- A template that is not an album of the collection.

## Verification

Mechanical:

- A core test: the copy has a new feed GUID, a new GUID for each track, and the new self link.
- A core test: the copy keeps the artist key and removes each other `podcast:txt`, each publisher link, each live item and `itunes:new-feed-url`.
- A core test: the copy gives an OP3 prefix the GUID of the copy.
- A core test: after the copy, the deletion check keeps each file of the source that the copy refers to.
- A Node test: the form shows "Start from" only for the type Album with an album in the collection, and the selected album gives the start value of "Artist".

Visual:

- The form with "Start from", and the new album after the copy. In the two themes, at desktop width and at 390 px.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.
