# ADR 0161: A Live Item Needs A Stream, And A Content Link Is Recommended

- Status: Accepted
- Date: 2026-11-09
- Owner: operator
- Accepted: 2026-11-09 through the orchestrator session, on its research request.
- Item 9 changed: 2026-11-12, after the human check of ADR 0187. The operator selected a text that gives the next step for a show with no stream type.
- Changes: [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 4 and its mechanical check of the content link.
- Uses: the [research request on the live stream](../plans/research-live-item-stream-request.md), musicindex ADR 0025 R-STATUS-1, [ADR 0065](0065-the-core-applies-each-feed-rule.md).

## Context

ADR 0054 item 4 refuses a write of a live item with no `podcast:contentLink`. The namespace text says that a live item must have a content link. Studio has no check of the stream of a live item.

The research note `research_notes/podcast-apps/live-item-stream-and-content-link.md` at `musicindex-research` commit `82f56fe` examined 14 published live items in 10 feeds:

- Each live item has a stream, an `enclosure` with a URL and a type.
- 8 of the 14 live items have no content link. Studio refuses a write of each of these 8 feeds.
- Partytime, the parser of Podverse, drops a live item with no enclosure. Podverse and Boost Me Bitch play the enclosure. The search found no app that plays a content link.
- Two snapshots of Pixel Palace had an HLS manifest URL with the type `audio/mpeg`. Thus a publisher can write a stream type that is not correct.

Thus the stream is the part that apps use, and Studio does not check it. The content link is a page for a person, and publishers use it for different purposes.

The UI has a copy of the rule of ADR 0054 item 4 in `live.js`. ADR 0065 does not let the UI keep a copy of a feed rule.

## Decision

1. Studio does not write a live item with no stream. A live item has a stream when its `enclosure` has a URL that is not blank. This replaces the content link part of ADR 0054 item 4. The refusals of no `start` and of a bad `status` do not change.
2. Studio does not write a live item with a stream type that is blank.
3. Studio does not write a live item with a stream type that does not agree with the stream URL. The core reads the extension of the last path part of the URL. It ignores the query, the fragment and the case. When the extension is in this table, the type must be one of its types:

   | Extension | Types |
   | --- | --- |
   | `m3u8` | `application/x-mpegurl`, `application/vnd.apple.mpegurl`, `audio/mpegurl`, `audio/x-mpegurl` |
   | `mp3` | `audio/mpeg`, `audio/mp3` |
   | `aac` | `audio/aac`, `audio/aacp` |
   | `m4a` | `audio/mp4`, `audio/x-m4a` |
   | `ogg` | `audio/ogg`, `application/ogg` |
   | `opus` | `audio/ogg`, `audio/opus` |
   | `mpd` | `application/dash+xml` |

   The comparison ignores the case of the type and each parameter after `;`. A URL with no extension in the table, for example an Icecast mount `/live`, accepts each type that is not blank.
4. A live item with no content link gets a warning, not a refusal. The write continues. The warning text is "This show has no content link."
5. The core gives each problem of a live item with its level, `refusal` or `warning`. Only a refusal stops the write. The refusal stops the write before any change on a host, as ADR 0054 item 4 states.
6. The draft response gives the problems of the live items as `live_problems`. The live show page shows each problem of its show in the Status section. A refusal uses the Danger color, and a warning uses the Warning color of musicindex ADR 0025 R-STATUS-1. The UI keeps no copy of the rule (ADR 0065), and the function `liveProblems` of `live.js` goes.
7. A granted write uses the same check, because it goes through the same write. A show with no stream gets no granted write until the user adds a stream and writes the feed.
8. A read of a hosted feed does not change. Studio reads and shows a live item with each problem. The check runs only at a write.
9. The texts of the refusals are "This show has no stream address.", "Choose a stream type, or click Check to get the type from the stream server." and "The stream type TYPE does not agree with the stream address, which ends in .EXT.".

## Consequences

- Studio can write each of the 8 published feeds with no content link that the research found.
- A new show that Studio makes has no stream. The user must add a stream address and a stream type before the first write.
- The table of item 3 can refuse a correct stream with an unusual type. A change of the table needs a change of this record.

## Alternatives Considered

- Keep the content link refusal. Rejected: most published live items have no content link, and no examined app uses it to play a show.
- Check the stream type from the bytes of the stream. Rejected: a write must not need a live stream. A stream can be offline before the show.
- Make the type check a warning. Rejected: the research found an incorrect type in a real feed, and an app possibly cannot play a stream with an incorrect type.

## How This Decision Is Checked

Mechanical:

- `studio-core`, `live` tests: each of these live items gives a refusal: no enclosure, a blank URL, a blank type, and an `.m3u8` URL with the type `audio/mpeg`. A URL with the extension in the query only, or with no extension, gives no refusal. No content link gives a warning.
- `crates/core/tests/pipeline.rs`: a write of a live item with no stream stops with `BadLiveItem` before any change on a host. A write of a live item with no content link continues.
- `crates/core/tests/corpus.rs`: each corpus live item gives no refusal. `behindthesch3m3s-music-pixel-palace.xml` gives one warning, for its content link.
- `crates/api/src/tests.rs`: the draft response gives `live_problems` with the level of each problem.

Visual, by the agent: the Status section of a live show with one refusal and one warning. The agent examines it in the light and the dark theme, at 1280 px and at 390 px with touch.

Visual result: passed on 2026-11-09. Eight screenshots in `.studio-dev/v-1109-live/out/` show the refusal in `--error` and the warning in `--warning`, in both themes, at 1280 px and at 390 px with touch. With a stream, only the warning stays. `$edges()` and `$overflow()` gave no result.
