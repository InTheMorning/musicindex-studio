# Studio v1 Task 023g: The Move Of The Feed XML

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 16 to 21, [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) items 18 to 20, [ADR 0008](../adr/0008-feed-types-and-feed-identity.md).
Sequence: after [task 023b](studio-v1-task-023b-published-feeds-in-the-library.md).

## Goal

The user can put the feed XML on a different host through the draft. The write puts the feed on the new host, then a redirect or a pointer file at the address from before. Studio keeps each pointer file.

## Files

Inspect: `crates/core/src/connector.rs`, `crates/core/src/connector/local.rs`, `crates/core/src/collection.rs` (migration 6, `set_feed_connector`, `FeedRow`), `crates/core/src/feed.rs` (`new_feed_url`, `atom_links`, `self_url`), `crates/core/src/pipeline.rs` (`diff`, `write_draft`, `host_operations`, `BlockedDelete`, `diff_id`, `discard_draft`), `crates/core/src/address.rs`, `crates/api/src/routes.rs` (`diff_json`, `write_feed`, the draft routes), `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/core/tests/pipeline.rs`.

Change: `connector.rs`, `local.rs`, `collection.rs`, `pipeline.rs`, `routes.rs`, `lib.rs` of the API, `tests.rs`, `crates/core/tests/pipeline.rs`.

Do not change: the UI, the schema, `feed.rs` except a helper to set the self link.

## Architecture

These choices are fixed. Do not change them.

### The Connector

- `Connector` gets two functions with a default body: `fn can_redirect(&self) -> bool { false }`, and `fn set_redirect(&self, path: &str, to_url: &str) -> Result<(), ConnectorError>`, which gives the new kind `ConnectorErrorKind::Unsupported`. The local folder connector keeps the defaults (ADR 0038 item 17).

### The Records

- `Collection` gets `set_draft_feed_move(feed, connector, path)`, `draft_feed_move(feed) -> Option<DraftFeedMove>` and `remove_draft_feed_move(feed)`. `pipeline::discard_draft` also removes the feed move.
- `Collection` gets `add_feed_pointer(connector, path, feed)` and `is_feed_pointer(connector, path) -> bool`.
- `Collection` gets `set_feed_location(feed, connector, path, public_url)`. It changes the host, the path and the public address of the feed in one transaction.

### The Draft And The Diff

- `PUT /api/v1/feeds/{id}/feed-host` with `{ "connector_id": n }`. A host that is not the host of the feed puts a feed move into the draft. The path is the path of the feed XML from before (ADR 0038 item 16). It also sets the `self` link in the draft to the new address, when the draft has a `self` link. The host of the feed gives no move: the route removes the move and puts the `self` link back.
- The route refuses a host that has a file at the path, with 409 `file_exists`, from the inventory.
- `Diff` gets `feed_move: Option<FeedMove>`, with `connector_id`, `path`, `url`, `from_url` and `method`. The `method` is `redirect` when the host from before can redirect, otherwise `pointer`. It also gets `announce: false`: the diff says that Studio does not tell the musicindex API and Podcast Index yet (ADR 0038 items 18 and 21). `diff_id` includes the feed move.
- `diff_json` gives `feed_move`.

### The Write

With a feed move, `write_draft` does these steps (ADR 0038 item 19):

1. Each upload and copy, as before.
2. The draft XML on the new host with `Expected::Absent`. A file at the path gives `FileExists`, and nothing else changes.
3. With `redirect`: `set_redirect(path, new_url)` on the host from before. With `pointer`: the draft XML with `itunes:new-feed-url` set to the new address, on the host from before, with the expected version of the live file. Studio records the pointer with `add_feed_pointer`.
4. `set_feed_location`, `record_state` with the version from the new host, and the library XML with the bytes of step 2.
5. The check of the new addresses, the deletions, and the removal of the draft and its operations, as before. Also `remove_draft_feed_move`.

### The Pointer File

- A deletion of a pointer file is a blocked deletion. `BlockedDelete` gets `pointer: bool`. The diff says "Studio keeps this file. It tells the apps the new address of the feed." (ADR 0038 item 20).
- The files route marks a pointer file in "other files" with `pointer: true`.

## Steps

1. Add the connector functions and the records.
2. Add the feed-host route, and the feed move in the diff.
3. Add the steps of the write, with a core test on two local test hosts.
4. Add the blocked deletion of a pointer file.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- A write to a host occurs only through `pipeline::write_draft`, with the approved diff ID (ADR 0007).
- The feed keeps its `podcast:guid` and each item GUID (ADR 0008).

## Acceptance Criteria

Mechanical:

- A core test moves a feed from host A to host B. After the write, host B has the feed XML at the same path with the new `self` link. Host A has a pointer file with `itunes:new-feed-url` at the new address. The feed row has host B, the same path and the new public address. The library XML has the bytes of the file on host B.
- The feed keeps its `podcast:guid`.
- A test connector that can redirect gets `set_redirect`, and host A keeps its file with no pointer.
- A deletion of the pointer file is blocked, with `pointer` true.
- A move to a host with a file at the path gives 409 `file_exists`.
- A discard of the draft removes the feed move.
- The three commit checks pass.

Visual:

- The visual check of task 023 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the write needs a step that this packet does not give. Also stop when the new public address of the feed conflicts with ADR 0021.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023g-feed-xml-move.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- The files that the Files section names.

Goal:
- The user can put the feed XML on a different host through the draft. The write puts the feed on the new host, then a redirect or a pointer file at the address from before. Studio keeps each pointer file.

Constraints:
- Follow the Architecture section of the task file fully.
- A write to a host occurs only through pipeline::write_draft, with the approved diff ID. The feed keeps its podcast:guid and each item GUID.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the schema, feed.rs except a helper to set the self link.
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
