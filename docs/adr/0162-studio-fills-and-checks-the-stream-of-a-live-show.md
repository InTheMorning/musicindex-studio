# ADR 0162: Studio Fills And Checks The Stream Of A Live Show

- Status: Accepted
- Date: 2026-11-09
- Owner: operator
- Accepted: 2026-11-09 by the operator. The operator selected the drawings of items 1, 4 and 5. The agent recommended item 7, and the operator did not select an option for it.
- Item 9 accepted: 2026-11-10 by the operator, after the human check found an Icecast stream with the type HLS. The operator selected the drawing with the warning and the **Use** button.
- Uses: [ADR 0161](0161-a-live-item-needs-a-stream-and-a-content-link-is-recommended.md), [ADR 0054](0054-live-items-connected-apps-and-show-grants.md), [ADR 0057](0057-the-private-network-guard-of-the-server.md), [ADR 0065](0065-the-core-applies-each-feed-rule.md).

## Context

ADR 0161 refuses a live item with no stream type, or with a stream type that does not agree with the stream address. The user types the stream type as text. The user cannot see if the stream works before the show goes live.

The operator accepted ADR 0161 on 2026-11-09, and asked for three changes:

- Studio fills the stream type.
- A Check button finds if the stream is live, and finds its type.
- When the stream is not live, Studio gives a warning, and it refuses Go Live. The stream type is a select menu with the common types.

`musicindex-live-publisher` does not call Studio. It sends value payloads to a relay, and it does not send the audio stream. Thus no app sets a show to live through a grant now.

## Decision

1. The Stream section has the row **Check Stream** below **Stream type**. The result of the check shows on the next line: "Live · TYPE · checked TIME" when the stream is live, or a warning with the cause when it is not live.
2. The core checks a stream. The check reads the stream address for at most 8 seconds and at most 256 KiB. It uses the network policy of the shell (ADR 0057), so the server refuses a private address. The check writes nothing. The result gives `live`, the type that the stream server sends, and the cause when the stream is not live.
3. The core decides `live` by the form of the stream:
   - An error, a timeout or an HTTP status that is not 2xx gives not live.
   - An HLS playlist is live when it has a segment and no `#EXT-X-ENDLIST`. A playlist with `#EXT-X-ENDLIST` is a recording. For a main playlist, the core reads the first variant playlist.
   - A DASH manifest is live when its `type` is `dynamic`.
   - An audio answer is live when bytes arrive and the answer has no `Content-Length`. An answer with a `Content-Length` is a file, not a live stream.
4. The stream type is a select menu. It holds HLS (`application/x-mpegURL`), MP3 (`audio/mpeg`), AAC (`audio/aac`), M4A (`audio/mp4`), Ogg (`audio/ogg`), DASH (`application/dash+xml`) and **Other…**. **Other…** shows a text field for a different type. A type that is not in the menu shows as **Other…** with its text. The core gives the menu, so the UI has no copy of the types.
5. Studio fills the stream type until the user selects one. The core gives the type from the extension of the address, with the table of ADR 0161 item 3. A check gives the type that the stream server sends. When that type is a different name of a menu type in the table of ADR 0161 item 3, Studio fills the menu type. For example, `application/vnd.apple.mpegurl` fills HLS. A type that the user selects stays, and the refusal of ADR 0161 item 3 shows when it does not agree.

   The page keeps the "selected by the user" state only while it is open. After a new load, a type that is not blank counts as selected.
6. **Go Live** refuses when the stream is not live. The core checks the stream before it changes the status to `live`, and the draft does not change. The text is "Studio cannot go live, because the stream is not live: REASON". A change to `pending` or `ended` has no check.
7. A granted change to `live` from a connected app gets the same check, and the same refusal.
8. The check of item 2 is not a feed rule of ADR 0161. A write of a feed does not check the stream, because a stream can be offline before the show.
9. The user can select a stream type that does not agree with the type that the stream server sends. Then the result line also shows a warning. The text is "The stream server sends SERVER TYPE, but the stream type is LABEL." A button **Use LABEL** sets the type of the server.

   Two names of one type in the table of ADR 0161 item 3 agree. The core gives the warning. The warning does not stop **Go Live** or a write. A change of the type removes the warning.

## Consequences

- A user can see if the stream works before the show.
- A show cannot go live while its stream server is down. It also cannot go live when the computer or the server of Studio cannot reach the stream server.
- A future app that sends its own stream and must go live before the stream answers needs a new ADR.

## Alternatives Considered

- The Check button beside the address. Rejected: on a phone, the address field becomes short.
- A fixed menu with no **Other…**. Rejected: a stream with an unusual type could not be written from Studio.
- Always replace the type. Rejected: it replaces a type that the user selected on purpose.
- Check only the Go Live of Studio, not the app. Rejected: no app uses the granted change to `live` now, and one rule in one place is simpler.
- A warning, with **Go Live Anyway**. Rejected: the operator asked for a refusal.

## How This Decision Is Checked

Mechanical:

- `studio-core` stream check tests with a local test server. The cases are a live HLS playlist, an HLS playlist with `#EXT-X-ENDLIST` and a main playlist. They are also a dynamic and a static DASH manifest, and an audio answer with no `Content-Length`. The last cases are an audio file with a `Content-Length`, a 404 and a timeout. Each gives the `live` value and the type of item 3.
- `studio-core`: the network policy `Public` refuses a check of a private address.
- `studio-core`: the type of each extension of ADR 0161 item 3, and no type for an address with no extension in the table.
- `studio-core` or `studio-api`: a change to `live` with a stream that is not live gives the refusal of item 6. The draft does not change. A granted change to `live` gives the same refusal (item 7). A change to `ended` has no check.
- `crates/api/ui-tests`: the model of the Stream section fills the type until the user selects one. It shows a type that is not in the menu as **Other…**.
- `studio-core` and `crates/api/src/tests.rs`: a server type that does not agree with the stream type gives the warning of item 9. Two names of one type give no warning. `crates/api/ui-tests`: the page shows the warning only for a type that the user selected.

Visual, by the agent: the Stream section with the menu, and **Other…** with its text field. Also a live result, a warning result and the Go Live refusal. The agent examines it in the light and the dark theme, at 1280 px and at 390 px with touch.

Visual result: passed on 2026-11-09. 24 screenshots in `.studio-dev/v-1162/out/`, and the 1280 px walk again in `.studio-dev/v-1162/out2/` after the menu type fill. They show the autofill to HLS, a live result, the warning for a recording, **Other…** with its text field, and the Go Live refusal. The streams came from a local test server. `$edges()` and `$overflow()` gave no result.

Visual result of item 9: passed on 2026-11-10. Screenshots in `.studio-dev/v-1162/out3/` show the warning and **Use MP3** in both themes, at 1280 px and at 390 px with touch. After **Use MP3**, the menu shows MP3 and the warning goes. `$edges()` and `$overflow()` gave no result.

Human, by the operator: **Check Stream** and **Go Live** with a real HLS stream and a real Icecast stream. The operator does this on the desktop and on the server.
