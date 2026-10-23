# Studio v1 Task 017: A New Album Feed

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decisions: [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) items 6 to 9, [ADR 0008](../adr/0008-feed-types-and-feed-identity.md) item 2 and [ADR 0007](../adr/0007-draft-diff-write-and-announce.md).

## Goal

Let the user make a new album feed in a connected folder, as a draft that the user examines and writes.

## Files

Inspect: `crates/core/src/pipeline.rs` (`start_draft`, `write_draft`, `diff`), `crates/core/src/collection.rs` (`add_feed`, `set_address`, `save_draft`, `record_state`, `DraftRow`, the `feeds` table), `crates/core/src/guid.rs` (`podcast_guid_for_url`), `crates/core/src/connector.rs` (`Connector`, `public_url`), `crates/core/src/feed.rs` (`Feed`, `write`), `crates/api/src/routes.rs` (`add_feed`), `crates/api/src/lib.rs`, `crates/api/src/error.rs`, `crates/api/src/tests.rs`, `crates/api/ui/app.js` (`AddFeeds`, `AddFeedForm`).

Change: `pipeline.rs`, `collection.rs`, `routes.rs`, `lib.rs`, `error.rs`, `tests.rs`, `app.js` and `styles.css`.

Do not change: the migrations, the connector interface, the vault, and `editor.js`.

## Architecture

These choices are fixed. Do not change them.

- `pipeline::new_feed(collection: &mut Collection, connector: &dyn Connector, connector_id: ConnectorId, path: &str, title: &str, author: &str) -> Result<FeedId, PipelineError>`.
  1. `connector.public_url(path)` gives the address. With no address, the function gives a new error of the type `NoPublicAddress`.
  2. `connector.read(path)` must give `None`. A file at the path gives a new error of the type `FileExists`.
  3. The template is a document with the `rss` root, `version="2.0"`, the `podcast` and the `itunes` namespaces, and an empty `channel`. A `Feed` with `title`, `author`, `podcast_guid = podcast_guid_for_url(address)`, `medium = "music"` and `language = "en"` writes into it.
  4. `add_feed(None, Some((connector_id, path)))`, then `set_address(Folder, Some(address))`.
  5. A new `Collection::set_new_feed(id, title, medium)` puts the title and the medium in the feed row, so that the UI opens the editor before the first write.
  6. `save_draft` with `base_version: None`, an empty `base_xml`, and the template as `draft_xml`.
- `write_draft` and `diff` work with no change. Add a test that proves it: the write puts the file with `Expected::Absent`, and a file that a different tool puts at the path first gives `Conflict`.
- Route `POST /api/v1/feeds/new` with `{ connector_id, path, title, author }`. It gives 201 and the feed row. `NoPublicAddress` maps to 422 `no_public_address`. `FileExists` maps to 409 `file_exists`. An empty title or author gives 422 `invalid_feed`.
- In the UI, the "Add feeds" screen gets a first panel, "Make a new release", when the collection has a connector. It has the host (a list when there are more than one), the feed file (placeholder `my-album.xml`), the album title and the artist. After the add, Studio opens the album page of the new feed. A notice on that page says that the folder has no file until the user reviews and writes it.

## Steps

1. Add the error types, `set_new_feed` and `new_feed`, with core tests.
2. Add the route and its tests.
3. Add the panel and the notice.

## Constraints

- No file goes to a host without a diff that the user saw (ADR 0007).
- The feed GUID is a UUIDv5 of the public address (ADR 0008 item 2). A test compares it with `podcast_guid_for_url`.
- Each control is 44 px high or more.
- The binary contains the UI files. After each UI change, stop Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- `new_feed` in a folder with a public address gives a feed with a draft. The draft has the title, the artist, the GUID, `podcast:medium` `music` and `language` `en`. The folder has no file before the write.
- `new_feed` in a folder with no public address gives `NoPublicAddress`. At a path with a file, it gives `FileExists`.
- `diff` gives the full file as added lines. `write_draft` with that diff puts the file. The read of the file gives the same feed.
- The route gives 201, 422 and 409 as the Architecture section says.
- The three commit checks pass.
- The agent runs `drive-ui.mjs`: it makes a new release in a local folder with a public address. The album page opens with the notice. The console shows no error.

Visual:

- None in this task. The visual check of Phase 6 covers it.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the editor does not open a feed with no hosted file. Stop and report when `write_draft` needs a change for a new file.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-017-new-album-feed.md
- docs/adr/0032-new-releases-formats-paths-and-new-feeds.md
- The files that the Files section names.

Goal:
- The user makes a new album feed in a connected folder, as a draft to examine and write.

Constraints:
- Follow the Architecture section of the task file fully.
- No write without a diff. The GUID is the UUIDv5 of the public address.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The migrations, the connector interface, the vault, editor.js.
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
