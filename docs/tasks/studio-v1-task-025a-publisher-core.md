# Studio v1 Task 025a: The Role And The Links In The Core

Parent: [task 025](studio-v1-task-025-publisher-feeds.md). Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) items 1, 2, 4 to 7 and 9 to 14, [ADR 0008](../adr/0008-feed-types-and-feed-identity.md), [ADR 0014](../adr/0014-collection-in-sqlite.md).
Sequence: the first packet of task 025.

## Goal

The collection keeps the role of each publisher feed. The core makes a new publisher feed, and adds, removes and changes the links in the drafts of the two feeds of a link.

## Files

Inspect: `crates/core/src/collection.rs` (`MIGRATIONS`, `FeedRow`, `FEED_SELECT`, `feed_row`, `set_new_feed`), `crates/core/src/pipeline.rs` (`new_feed`, `NEW_FEED_TEMPLATE`, `edit_draft`, `draft_model`, `PipelineError`), `crates/core/src/feed.rs` (`Feed`, `Publisher`), `crates/core/src/feed/elements.rs` (`RemoteItem`), `crates/core/src/feed/sync.rs` (`Entry`), `crates/core/src/lib.rs`, `crates/core/tests/pipeline.rs`.

Change: `collection.rs`, `pipeline.rs`, `lib.rs`, a new module `crates/core/src/publisher.rs`, `crates/core/tests/pipeline.rs` and a new test file `crates/core/tests/publisher.rs`. The callers of `new_feed` in `crates/api/src/routes.rs` change only to give the medium `music`.

Do not change: the UI, the API routes other than the `new_feed` call, the connector interface, `crates/core/src/connector/`, `crates/core/src/xml.rs`.

## Architecture

These choices are fixed. Do not change them.

### The Role

- A new migration adds `publisher_role TEXT CHECK (publisher_role IN ('artist', 'label'))` to `feeds`.
- New type `collection::PublisherRole` with `Artist` and `Label`. It serializes as `"artist"` and `"label"`, and gives its `rel` text with `as_str`.
- `FeedRow` gets `publisher_role: Option<PublisherRole>`. `FEED_SELECT` and `feed_row` read it.
- New function `Collection::set_publisher_role(id, Option<PublisherRole>)`.

### The New Publisher Feed

- `pipeline::new_feed` gets a last argument `medium: &str`. The draft and `set_new_feed` use it, not the fixed `music`. Each current caller gives `"music"`.
- The function does not set the role. The route of packet 025b sets it after the call.

### The Links

New module `publisher`, with a new error type `PublisherError` and its kind enum. It has these kinds: `NotPublisher` (the first feed is not a publisher feed), `NotRelease` (the second feed is not a music feed), `ReadOnly` (a feed has no connector), `NoRole`, `NoGuid` (a draft has no `podcast:guid`), `NoAddress` (a feed has no public address), and `Pipeline` (a wrapped `PipelineError`).

```rust
pub fn inferred_role(feed: &Feed) -> Option<PublisherRole>;
pub fn role(collection: &Collection, publisher: FeedId) -> Result<PublisherRole, PublisherError>;
pub fn linked_releases(collection: &Collection, publisher: FeedId) -> Result<Vec<FeedId>, PublisherError>;
pub fn link(collection: &Collection, publisher: FeedId, release: FeedId) -> Result<(), PublisherError>;
pub fn unlink(collection: &Collection, publisher: FeedId, release: FeedId) -> Result<(), PublisherError>;
pub fn set_role(collection: &Collection, publisher: FeedId, role: PublisherRole) -> Result<(), PublisherError>;
```

- `inferred_role`: the role when each channel `remoteItem` with `medium="music"` has the same `rel`. That `rel` is `artist` or `label` after a trim and an ASCII lowercase. `None` when the feed has no such item, or when two values differ (ADR 0041 item 4).
- `role`: the recorded role. With no recorded role, the inferred role of the draft of the publisher feed, which the function then records. `NoRole` when the two give no role.
- Each function, but not `inferred_role`, reads the drafts with `pipeline::draft_model` and changes them with `pipeline::edit_draft`. It does not start a draft. A feed with no draft gives the `Pipeline` kind with `NoDraft`. Packet 025b starts each draft before the call.
- A feed of a link matches a `remoteItem` when the `feedGuid` is the `podcast:guid` of the draft of that feed. When the item has no `feedGuid`, a `feedUrl` that is the public address of that feed matches.
- `link`:
  - checks the two rows: the publisher row has the medium `publisher` and a connector, and the release row has the medium `music` and a connector.
  - gets the role with `role`.
  - In the draft of the publisher feed, it adds a channel `remoteItem` at the end of `remote_items` (ADR 0041 item 6). The item has `medium` `music`, the `feedGuid`, the `feedUrl` and the title of the release, and `rel` the role.
  - In the draft of the release, it makes `podcast:publisher` when the draft has none. It adds a `remoteItem` at the end (ADR 0041 items 7 and 9). The item has `medium` `publisher`, the `feedGuid`, the `feedUrl` and the title of the publisher feed, and `rel` the role.
  - When a matching item is in the draft before the call, the function changes its `feedGuid`, `feedUrl`, `title`, `medium` and `rel`, and keeps its position. It adds no second item.
- `unlink` removes each matching item from the two drafts. A `podcast:publisher` with no item goes out of the release draft.
- `linked_releases`: each feed of the collection with the medium `music` and a connector that a `medium="music"` item of the publisher draft matches.
- `set_role` records the role. It sets `rel` of each `medium="music"` item of the publisher draft. It sets `rel` of the matching `medium="publisher"` item in the draft of each feed of `linked_releases` (ADR 0041 item 11).
- No function changes an element that ADR 0041 item 14 does not name. Each other `remoteItem` stays at its position.

## Steps

1. Add the migration, `PublisherRole`, the column and `set_publisher_role`, with a migration test.
2. Add the `medium` argument to `new_feed`, with a test that makes a publisher feed draft.
3. Add the `publisher` module, with the tests below.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- A schema change is a new migration ([ADR 0014](../adr/0014-collection-in-sqlite.md)).
- The writer keeps each element that the model does not change ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).

## Acceptance Criteria

Mechanical:

- A database of the version before this packet opens at the new version, and each feed has no role.
- `new_feed` with the medium `publisher` gives a draft with `podcast:medium` `publisher`, and a row with that medium.
- After `link`, the publisher draft has one `medium="music"` item for the release, and the release draft has one `medium="publisher"` item in `podcast:publisher`. Each has the GUID, the address, the title and `rel` of ADR 0041 items 6 and 7.
- A second `link` of the same two feeds adds no item.
- A release with a publisher link from before gets the new link after it.
- After `unlink`, the two drafts do not have the link. A release with no other link has no `podcast:publisher`.
- `set_role` changes `rel` in the publisher draft and in each linked release draft.
- `role` records the inferred role of a publisher draft with items that give `rel="label"`, and gives `NoRole` for items with no `rel`.
- `link` gives `NotPublisher`, `NotRelease`, `ReadOnly` and `NoRole` in their cases.
- A `remoteItem` of the publisher draft for a feed that is not in the collection keeps its position after `link` and `unlink`.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the model of `podcast:publisher` or of `remoteItem` cannot give a rule of this packet, or when a corpus test fails.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-025a-publisher-core.md
- docs/adr/0041-publisher-feeds-and-two-way-links.md
- The files that the Files section names.

Goal:
- The collection keeps the role of each publisher feed. The core makes a new publisher feed, and adds, removes and changes the links in the drafts of the two feeds of a link.

Constraints:
- Follow the Architecture section of the task file fully.
- A schema change is a new migration. No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the connector interface, crates/core/src/connector/, crates/core/src/xml.rs.
- The API routes, but not the callers of new_feed.
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
