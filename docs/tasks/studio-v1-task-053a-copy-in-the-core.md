# Studio v1 Task 053a: The Copy In The Core And The Route

Parent: [task 053](studio-v1-task-053-album-from-album.md). Decision: [ADR 0077](../adr/0077-a-new-album-can-start-from-an-album.md) items 4 to 9.

Status: Open.

## Goal

`POST /feeds/new` with `from` makes a new album from the draft of the source album, with the changes of ADR 0077 item 6.

## Files

Inspect: `crates/core/src/pipeline.rs` (`new_feed`, `current_draft`, `replace_model`), `crates/core/src/feed.rs` (`Feed`, `items`, `live_items`, `txts`, `remote_items`, `new_feed_url`, the self link), `crates/core/src/guid.rs` (`podcast_guid_for_url`), `crates/core/src/media.rs` (`plain_address` and `op3_prefix` of task 052), `crates/core/src/publisher.rs` (the publisher reference of an album), `crates/api/src/routes.rs` (`NewAlbumFeed`, `new_album_feed`, `get_draft`, `ensure_draft`).

Change: `crates/core/src/pipeline.rs`, `crates/api/src/routes.rs`, and their tests. Change `crates/core/src/media.rs` only to add a function for the `pg=` part, and then say so.

Do not change: the UI, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `NewAlbumFeed` gets `from: Option<i64>`. With `from`, the sent medium and the medium of the source are `music`. When one is not `music`, the route gives 422 `invalid_feed`.
- The route calls `ensure_draft` for the source, then `pipeline::copy_feed`.
- `pipeline::copy_feed(collection, connector, connector_id, path, title, author, source)` has the checks of `new_feed`: a public address, and no file at the path. It reads the source draft XML into a `Document` and a `Feed`, and changes the model. It saves the result as the draft of a new feed with no hosted file.
- The changes are those of ADR 0077 item 6:
  - `title` and `author` from the form.
  - `podcast:guid` from `podcast_guid_for_url` of the new address.
  - Each item gets a new random UUID (version 4) as its GUID.
  - The `atom:link` with `rel="self"` gives the new address. When the source has none, add none.
  - Each `txts` entry goes, except an entry with `purpose` `npub`.
  - The publisher reference of the album goes.
  - Each live item goes.
  - `new_feed_url` goes.
  - An enclosure with the prefix `https://op3.dev/e,pg=SOURCE_GUID/` gets the GUID of the copy. Another prefix does not change.
- Each other element stays, with the bytes of the source.
- The copy makes no library file and no host operation. Each track keeps its audio address.

## Steps

1. Write `copy_feed` with core tests.
2. Change the route, with API tests.
3. Run the four commit checks.

## Constraints

- No write to a host.
- The source draft does not change.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- A core test: the copy has the new feed GUID, a new GUID for each track, the new self link, and the sent title and author.
- A core test: the copy keeps the artist key, and removes each other `podcast:txt`, the publisher reference, each live item and `itunes:new-feed-url`.
- A core test: an OP3 prefix with the source GUID gets the copy GUID. A prefix with no `pg=` does not change.
- A core test: each other element of the source is in the copy with the same bytes.
- A core test: the copy refuses a path with a file, as `new_feed` does.
- A core test: after the copy, the deletion check of ADR 0035 item 17 keeps a source file that the copy refers to.
- An API test: `from` with a publisher feed or a playlist gives 422. A copy leaves the source draft with no change.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `ensure_draft` of the source needs a host that is not connected.
- Stop and report when the model has no field for the publisher reference of an album.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0077-a-new-album-can-start-from-an-album.md
- docs/tasks/studio-v1-task-053-album-from-album.md
- docs/tasks/studio-v1-task-053a-copy-in-the-core.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- POST /feeds/new with from makes a new album from the draft of the source album. The copy has new GUIDs and the changes of ADR 0077 item 6.

Constraints:
- Follow the Architecture section of the task file fully.
- No write to a host. The source draft does not change.

Do not touch:
- The UI, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
