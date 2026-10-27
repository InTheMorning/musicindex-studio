# Studio v1 Task 039c: The Audio Player And "Properties"

Parent: [task 039](studio-v1-task-039-file-picker.md). Decisions: [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) items 16 and 23 to 29, [ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md) items 4 and 5, [ADR 0025](../adr/0025-studio-serves-cover-art.md) item 2, [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0017](../adr/0017-local-api-access.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the third packet of task 039. It depends on 039a. It can go before or after 039b.

Status: Complete - 2026-10-27. The four commit checks are Green. The screenshot visual check and the human tests passed. The [review checklist](../reviews/2026-10-27-tasks-038-039-review-checklist.md) records the review.

## Goal

The audio section of a track shows the address, a player and the closed disclosure "Properties" with the size and the media type. The player plays through Studio with a media ticket. A file that only a host holds gets its duration from the metadata that the player reads.

## Files

Inspect: `crates/api/src/routes.rs` (`get_image` and its cache, `media_facts`), `crates/api/src/lib.rs` (the routes and the access layer), `crates/api/src/access.rs`, `crates/api/src/ui.rs` (the page policy), the web client of the core with the guard of ADR 0057, `crates/api/ui/app.js` (`api`), `crates/api/ui/editor.js` (the audio section of the track page, the duration field), `crates/api/src/tests.rs`.

Change: a new file `crates/api/src/media.rs`, `crates/api/src/lib.rs`, the web client of the core (only for the `Range` header), the new model module `crates/api/ui/audio.js`, the new test file `crates/api/ui-tests/audio.test.mjs`, `crates/api/ui/editor.js`, `crates/api/ui/styles.css`, `crates/api/src/tests.rs`.

Do not change: the page policy. `default-src 'self'` already permits audio from Studio. Do not change the access rules of the other routes.

## Architecture

These choices are fixed. Do not change them.

### The Media Ticket

- `POST /media/tickets` with `{"url"}` gives `{"ticket"}`. It uses the usual access of the API. It refuses an address that is not correct by ADR 0023 item 2.
- A ticket is 16 random bytes as hexadecimal. It holds the address, the user (in the server shell) and the time it expires, one hour after it was made. `media.rs` keeps the tickets in memory, in a map behind a lock, and removes expired tickets at each new ticket.
- `GET /media/audio?ticket=` is outside the access layer. It accepts only a ticket that exists, has not expired, and belongs to the user of the session when the server shell has a session. It needs no other access.

### The Audio Route

- When the library holds the bytes of the address, the route reads them from the release folder. Otherwise it reads the address from its host through the web client of the core. The server shell applies the guard of ADR 0057 to the address and to each redirect.
- The route accepts one `Range: bytes=a-b` header. It gives 206 with `Content-Range` for a range, 200 for no range, and 416 for a range outside the file. It gives the media type of the file.
- The route sends the bytes as a stream. It does not keep a full file in memory and does not write it to disk.

### The Model Module

- `audio.js` has no Preact and no browser state. It has these functions:
  - `propertiesSummary(enclosure)`: the size in MB with one decimal and the short type, for example "3.8 MB · MP3". A missing value gives "—" for that part.
  - `propertiesOpen(enclosure, errors)`: true when the size or the media type is empty, or when either one has an error.
  - `durationFromSeconds(seconds)`: the duration value of the track, in the form that the duration field uses now.
  - `needsMetadataRead(source)`: true for each selection from "This release" or "Hosts". False for an upload, because the import reads the duration from the file.

### The Audio Section

- The section shows, in this sequence: the address field with "Choose…" of task 039b, the player, and the disclosure "Properties". The disclosure uses the `details` pattern of the track page. It holds the size field and the media type field. Its summary is `propertiesSummary`. It is open when `propertiesOpen` gives true.
- The player has a play and pause button, a position bar that the user can move, the position and the duration. It uses one `audio` element with `preload="none"`, with tokens for each color and size. It gets its address with a new ticket when the user presses play, or when the address of the field changes.
- After a selection that `needsMetadataRead` marks, the UI gets a ticket and sets `preload="metadata"` on the element. At the `loadedmetadata` event, the UI puts `durationFromSeconds(duration)` into the track. When the element gives an error or no finite duration, the duration field is empty and shows "Type the duration of the new file".

## Steps

1. Add `media.rs` with the tickets and the audio route, with API tests.
2. Add `audio.js` with its Node tests.
3. Change the audio section of the track page.
4. Run the four checks. Give the operator the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. When the web client of the core cannot send a `Range` header, extend it. Do not add a second HTTP client.
- No logic in two places: the guard of ADR 0057 is the existing guard.

## Acceptance Criteria

Mechanical:

- An API test makes a ticket and reads the range `bytes=0-99` of an audio file of the release folder. The response is 206 with 100 bytes and the correct `Content-Range`.
- An API test reads a range of an audio file of a local test web server. The bytes are the bytes of that range.
- An API test gives 416 for a range outside the file.
- An API test refuses an unknown ticket, an expired ticket and, in the server shell, a ticket of a different user.
- A server shell test refuses a ticket for an address in a private network.
- An API test confirms that `GET /media/audio` with no ticket gives an error, and that `POST /media/tickets` with no access gives an error.
- The Node tests examine `propertiesSummary`, `propertiesOpen`, `durationFromSeconds` and `needsMetadataRead`.
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The audio section of a track: the player plays, the position moves, and "Properties" is closed with its summary.
- With an empty media type, "Properties" is open.
- The audio file of a track is replaced from "Hosts" with a file that only the host holds. The duration changes with no press of the play button.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the access layer cannot leave out one route.
- Stop and report when the web client of the core cannot give a response as a stream.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-039c-audio-player.md
- docs/adr/0062-a-file-picker-for-each-file-field.md
- The files that the Files section names.

Goal:
- Add the media tickets and the audio route with byte ranges.
- Add the model module `audio.js`, and the player and the disclosure "Properties" on the track page.
- Put the duration from the player metadata into the track.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No second HTTP client. No logic in two places.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The page policy and the access rules of other routes.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The operator does the visual check.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
