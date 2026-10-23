# Studio v1 Task 016: Cover Art In The Editor

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decisions: [ADR 0025](../adr/0025-studio-serves-cover-art.md) and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) items 13 and 14.

## Goal

Show the art of the album and of each track in the editor. The core reads each image, and the UI gets it from the local API.

## Files

Inspect: `crates/core/src/web.rs` (`Web`, `get_with_query`, `WebError`), `crates/api/src/workspace.rs`, `crates/api/src/routes.rs` (`refresh_feed`, `media_facts`), `crates/api/src/lib.rs`, `crates/api/src/error.rs`, `crates/api/src/ui.rs` (the page policy), `crates/api/src/tests.rs`, `crates/api/ui/app.js` (`api`, `EditableFeed`), `crates/api/ui/editor.js` (`AlbumPage`, `TrackPage`), `crates/api/ui/styles.css` (`.head .cover`).

Change: `web.rs`, `workspace.rs`, `routes.rs`, `lib.rs`, `error.rs`, `tests.rs`, `app.js`, `editor.js` and `styles.css`.

Do not change: the page policy in `ui.rs`, `crates/core/src/pipeline.rs`, the collection, and the feed model.

## Architecture

These choices are fixed. Do not change them.

- `Web::get_image(&self, url: &str) -> Result<WebImage, WebError>`. `WebImage { bytes: Vec<u8>, media_type: String }`. The limit is `MAX_IMAGE_BYTES = 10 * 1024 * 1024`. A response with a `Content-Type` that does not start with `image/` gives a new `WebErrorKind::NotImage`. The media type has no parameters.
- `Workspace` gets `images: ImageCache`, a small type in `workspace.rs`. It keeps at most 64 images by address. When it is full, it removes the oldest image. `refresh_feed` clears it.
- Route `GET /api/v1/images?url=<address>`. It gives the bytes with the `Content-Type` of the image and `Cache-Control: no-store`. An error gives the JSON error body. `NotImage` maps to 422 with the code `not_image`. `TooLarge` maps to 422 with the code `too_large`.
- The UI cannot put the token on an `<img>` request. A hook `useImage(url)` in `app.js` calls the route with `fetch` and the token. It reads the result as a `data:` address with `FileReader`, and gives `{ src, failed }`. It keeps each result in a `Map` for the life of the page.
- The compact header shows the album image in its `.cover`. A track page shows the track image, or the album image when the track has none. With no image, or with an image that Studio cannot read, the cover shows the current mark. For an image that Studio cannot read, the `title` attribute of the cover gives its address.
- The "Cover image address" field and the "Track image address" field each show a preview of 64 px adjacent to the field.

## Steps

1. Add `get_image`, `WebImage`, `NotImage` and their tests in `web.rs`, with a `TestServer`.
2. Add `ImageCache`, the route and the error codes. Add the API tests.
3. Add `useImage`, the covers and the previews.

## Constraints

- The UI loads nothing from other servers (ADR 0016). The test `the_ui_names_no_other_server_and_sends_the_token` must not fail.
- Each control is 44 px high or more. The styles agree with `each_background_uses_a_token`.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- `get_image` gives the bytes and the type of a PNG from a `TestServer`. A `text/html` response gives `NotImage`. A body over the limit gives `TooLarge`.
- `GET /api/v1/images` gives the bytes and the `Content-Type`. A second request for the same address does not call the server again. After a refresh of a feed, it calls the server again.
- A request with no token gives 401.
- The three commit checks pass.
- The agent runs `drive-ui.mjs` on a feed with an `itunes:image` that a local server gives. The header shows an `<img>` with a `data:` source. The console shows no error.

Visual:

- None in this task. The visual check of Phase 6 covers it.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the image needs a change of the page policy, or when a route needs the collection.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-016-cover-art.md
- docs/adr/0025-studio-serves-cover-art.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- The files that the Files section names.

Goal:
- The editor shows the art of the album and of each track, through the local API.

Constraints:
- Follow the Architecture section of the task file fully.
- No change of the page policy. Each control is 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The page policy in ui.rs, pipeline.rs, the collection, the feed model.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
