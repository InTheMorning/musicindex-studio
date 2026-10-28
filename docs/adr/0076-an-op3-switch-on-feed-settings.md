# ADR 0076: An OP3 Switch On Feed Settings

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 7, which gives the fields of the page "Feed settings".

## Context

- OP3 counts the downloads of a file through a prefix on its address. The OP3 setup page (`https://op3.dev/setup`, read on 2026-10-28) gives the forms `https://op3.dev/e/` and `https://op3.dev/e,pg=GUID/`. An `http` address keeps its scheme after the prefix: `https://op3.dev/e/http://…`.
- MSP 2.0 has one switch for each feed (`origin/master:src/types/feed.ts:245`). When it is on, MSP 2.0 adds `https://op3.dev/e,pg=GUID/` to the enclosure of each track, at each write (`origin/master:src/utils/xmlGenerator.ts:443-450,497`). It does not add a second prefix.
- The corpus has OP3 addresses in 7 of 28 files, with and without `pg=`.
- `https://op3.dev/show/d5c805fb-6016-4038-9cda-2e68d2b8dcc4`, the page of a corpus feed, gave HTTP 200 on 2026-10-28.
- The core compares each enclosure address with the address of a file on a host: in the confirmation (`crates/core/src/pipeline.rs:1626`), in the inventory and in the file moves. A prefix stops each of these comparisons. [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 17 deletes a file only when no feed refers to its address.
- The audio player and the media facts of the core read the enclosure address ([ADR 0023](0023-web-addresses-and-audio-file-facts.md), route `/media/audio`). Through the prefix, each of these reads is a download for OP3.

The operator selected the scope of the switch from a questionnaire on 2026-10-28.

## Decision

### The Plain Address

1. The core has one function that gives the plain address of an enclosure address. It removes `https://op3.dev/e`, an optional `,pg=…` and the `/` after it. When the remaining text has no scheme, it adds `https://`.
2. Each comparison of a feed address with a file address uses the plain address: the confirmation, the inventory, the file moves, and the deletion check of ADR 0035 item 17.
3. Each read of an enclosure by the core uses the plain address: the media facts, the audio player and the downloads of [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md).
4. A file move changes the plain address in the feed, and keeps the prefix.

### The Switch

5. The page "Feed settings" of an album gets the switch "OP3 statistics".
6. The core gives the start value of the switch. It is on when the album has one or more tracks, and the enclosure of each track has the prefix. In each other case it is off.
7. When the album has tracks with the prefix and tracks with no prefix, the switch is off. Its hint gives the count, for example "2 of 9 tracks use OP3".
8. At the draft save, the core compares the sent value with the draft value, as [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 4 does:
   - A sent `true` adds the prefix to each track enclosure with no prefix. This includes a track that the user adds while the switch is on.
   - A sent `false` over a draft `true` removes the prefix from each track enclosure.
   - A sent `false` over a draft `false` changes no address.
9. A new prefix is `https://op3.dev/e,pg=GUID/`, with the `podcast:guid` of the feed. The core does not change a prefix that the feed has.
10. The switch changes only the enclosure of each track. It does not change an alternate enclosure, an image or a live item.
11. When the switch is on, the page shows the link "OP3 statistics" to `https://op3.dev/show/GUID`.
12. These rules are feed rules of [ADR 0065](0065-the-core-applies-each-feed-rule.md). The UI sends only the value of the switch.

## Alternatives Considered

- Add the prefix to new tracks only. Not selected by the operator: the statistics then do not include the tracks that exist.
- Add the prefix at the write, and keep the plain address in the draft. Rejected: the feed on the host then differs from the draft, and the diff does not show the real addresses ([ADR 0007](0007-draft-diff-write-and-announce.md)).
- A switch on the Settings page for each album. Not selected: OP3 counts one feed, and MSP 2.0 has one switch for each feed.
- Read the switch as on when one track has the prefix. Rejected: the next save then adds a prefix that the user did not set.

## Consequences

- A user of OP3 in MSP 2.0 keeps the statistics after the move to Studio.
- The reads of Studio do not count as downloads.
- The diff shows each address that the switch changes.

## Invariants

- Each comparison of a feed address with a file address uses the plain address.
- Studio adds or removes a prefix only after the user changes the switch, or adds a track while the switch is on.

## Non-Goals

- Other prefix services, such as Podtrac or Chartable.
- OP3 statistics in Studio. The link opens the OP3 page.

## Verification

Mechanical:

- A core test of the plain address: each form of the corpus, an `http` address, and an address with no prefix.
- A core test: the switch is on for an album with a prefix on each track, and off for an album with a mixed set.
- A core test: a sent `true` adds the prefix with the feed GUID. A sent `false` over `true` removes it. A sent `false` over a mixed set changes no address.
- A core test: a track that the user adds while the switch is on gets the prefix.
- A core test: the confirmation and the deletion check find a prefixed enclosure by its plain address.
- A core test: a file move keeps the prefix and changes the plain address.
- An API test: `/media/audio` and `/media/facts` read the plain address.

Visual:

- The page "Feed settings" in three conditions: the switch is off, the switch is on with its link, and the switch is off with the count hint. In the two themes, at desktop width and at 390 px.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.
