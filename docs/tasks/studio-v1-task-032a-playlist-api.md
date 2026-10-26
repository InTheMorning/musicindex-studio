# Studio v1 Task 032a: The Playlist Feed And The Track Sources In The API

Parent: [task 032](studio-v1-task-032-playlists.md). Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md) items 2, 4 and 5, and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md).
Sequence: the first packet of task 032.

## Goal

`POST /api/v1/feeds/new` makes a `musicL` feed. Two routes give the tracks for the picker: the tracks of the collection, and the tracks and the address of a feed of musicindex.

## Files

Inspect: `crates/core/src/pipeline.rs` (`new_feed`), `crates/core/src/collection.rs` (`feed_items`, `item_guids`, `FeedRow`, `feeds`), `crates/core/src/musicindex.rs` (`feed_url`, `IndexRoutes`, `IndexTrack`, the function that gives the tracks of a feed), `crates/api/src/routes.rs` (`new_album_feed`, `search`, the `musicindex/feeds/{guid}/recipients` route), `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Change: `crates/core/src/collection.rs`, `crates/core/src/musicindex.rs` (only when a new function is necessary), the new file `crates/core/src/playlist.rs`, `crates/core/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`.

Do not change: the feed model, the writer, the pipeline other than a call, the UI.

## Architecture

These choices are fixed. Do not change them.

### The New Feed

- `new_album_feed` accepts `medium` `musicL`. It needs no role. The error text for an empty title is "give the name of the playlist".
- `pipeline::new_feed` needs no change.

### The Tracks Of The Collection

- `playlist.rs` has:

  ```rust
  pub struct PickTrack { pub feed_guid: String, pub feed_url: Option<String>, pub feed_title: Option<String>,
      pub item_guid: String, pub title: Option<String> }   // serde::Serialize
  pub fn own_tracks(collection: &Collection) -> Result<Vec<PickTrack>, CollectionError>;
  ```

- `own_tracks` gives the tracks of each feed with `medium` `music` and a `podcast_guid`, from `feed_items`. The sequence is that of the feeds, and then of the items. `feed_url` is the public address of the feed row.
- A collection function gives the GUID and the title of each item of a feed, if no function gives the titles now.
- `GET /api/v1/playlist/tracks` gives `own_tracks` as a JSON list.

### The Tracks Of A Feed Of musicindex

- `GET /api/v1/musicindex/feeds/{guid}/tracks` gives `{"feed_guid", "feed_url", "title", "tracks": [{"track_guid", "title"}]}`.
- It uses the musicindex client that the recipients route uses for the tracks, and `MusicIndex::feed_url` for the address. A feed that musicindex does not know gives 404 `not_in_index`. A network failure gives the error that the recipients route gives.

## Steps

1. Accept `musicL` in the new feed route, with a test.
2. Add `playlist.rs` and its unit tests. The test collection has two music feeds, a publisher feed and a `musicL` feed.
3. Add the two routes, with API tests. The musicindex test uses the local `TestServer`, as the recipients route tests do.
4. Add an API test that makes a `musicL` feed and writes it. It puts two `remote_items` into the draft model with `PUT /feeds/{id}/draft` first.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- `POST /feeds/new` with `medium` `musicL` gives 201, and the draft has `podcast:medium` `musicL`.
- `own_tracks` gives the tracks of the two music feeds only, with their feed GUID, address, item GUID and title.
- `GET /musicindex/feeds/{guid}/tracks` gives the address and the tracks from the test server, and 404 for an unknown GUID.
- The written playlist has the two `podcast:remoteItem` elements with `feedGuid`, `itemGuid`, `feedUrl`, `medium="music"` and `title`, in sequence, and no `<item>`.
- The three commit checks of Rust pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when the writer does not write the `title` or the `medium` of a remote item.
- Stop and report when musicindex gives no route for the tracks of a feed that the client can use.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-032a-playlist-api.md
- docs/adr/0053-playlists-are-musicl-feeds-with-remote-items.md
- The files that the Files section names.

Goal:
- The new feed route makes a `musicL` feed. `GET /api/v1/playlist/tracks` gives the tracks of the collection, and `GET /api/v1/musicindex/feeds/{guid}/tracks` gives the address and the tracks of a feed of musicindex.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The feed model, the writer, the UI.
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
