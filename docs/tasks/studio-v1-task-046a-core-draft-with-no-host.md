# Studio v1 Task 046a: The Draft With No Host In The Core

Parent: [task 046](studio-v1-task-046-read-only-editor.md). Decisions: [ADR 0067](../adr/0067-a-read-only-feed-opens-in-the-editor.md) items 3 to 11 and 13, [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md) items 3 and 8, [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) items 13 and 18 to 21, [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 16 to 19, [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) item 8 and [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md).
Sequence: the first packet of task 046. It depends on task 045.

Status: Complete - 2026-10-27. The four commit checks pass, and the [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records the review.

## Goal

The core keeps a draft for a feed with no connector. The diff works for it. The write refuses it until it has a host. A move from a read-only feed writes the feed on the new host with no redirect and no pointer file. A link by a folder match keeps the draft.

## Files

Inspect: `crates/core/src/pipeline.rs` (`writable`, `start_draft`, `refresh_public`, `set_feed_host`, `feed_move`, `FeedMove`, `FeedMoveMethod`, `feed_move_bytes`, `diff_at`, `host_operations`, `host_changes`, `plan_announcements`, `write_draft`, `write_draft_signed`, `finish_hosts`, `put_moved_feed_file`, `pointer_bytes`), `crates/core/src/collection.rs` (`FeedRow`, `DraftRow`, `set_feed_connector`, `draft_feed_move`, `set_draft_feed_move`, `remove_draft_feed_move`, `set_files_connector` or its equivalent), `crates/core/src/library.rs` (`Library::path`, `write_feed_xml`, `give_published_folder`), `crates/core/src/web.rs`, `crates/core/tests/pipeline.rs`, the feed move tests of milestone 7b.

Change: `crates/core/src/pipeline.rs`, `crates/core/src/library.rs`, `crates/core/tests/pipeline.rs`. Change `crates/core/src/collection.rs` only when a step needs a new query.

Do not change:

- The write of a feed with a connector. Each test of it passes with no change.
- `write_granted` and the grant vaults. A granted write of a feed with no connector stays refused.
- The corpus tests and the XML writer.
- `crates/api`, the UI and each other crate.

## Architecture

These choices are fixed. Do not change them.

### The Draft

- Add `Library::read_feed_xml(&self, folder: &str, file_name: &str) -> io::Result<Vec<u8>>`, the reverse of `write_feed_xml`.
- Add `pub fn start_public_draft(collection: &Collection, library: &Library, id: FeedId) -> Result<DraftRow, PipelineError>`.
  - It refuses a feed with a connector (`ReadOnly` is not correct here: use `Conflict` with the text "feed N has a host; start the draft from the host").
  - It refuses a feed with no `folder` or no `public_url` with `NotFound`.
  - The file name is the last segment of `public_url`, as in `give_published_folder`.
  - When the SHA-256 of the bytes is not `last_version`, it refuses with `Conflict` and the text "the release folder has a different feed XML; read the feed again".
  - `base_version` is `last_version`. `base_xml` and `draft_xml` are the bytes.
- Add `fn public_target(collection: &Collection, row: &FeedRow) -> Result<Option<(ConnectorId, String)>, PipelineError>`. It gives the draft feed move of a feed with no connector, or `None`.

### The Host Choice

- `set_feed_host` for a feed with no connector: the path is `folder`, `/`, then the file name of `public_url`. A feed with no folder gives `NotFound`. Each other check of the function stays: a file at the path on the target gives `FileExists`, and no public address gives `NoPublicAddress`. It records the draft feed move and sets the self link of the draft to the new address.
- Add `pub fn clear_feed_host(collection: &Collection, id: FeedId) -> Result<(), PipelineError>` for a feed with no connector. It removes the draft feed move. It sets the self link of the draft back to the self link of the base, as `set_feed_host` does for the same host.

### The Diff

- Add `FeedMoveMethod::None`. Its text is `none`. Its doc comment says that Studio writes nothing at the address before the move (ADR 0067 item 8).
- `feed_move` for a feed with no connector: with a draft feed move, it gives `FeedMove` with the target, its path and its URL, `from_url` from `public_url`, and `method` `None`. With no draft feed move, it gives `None`.
- `diff_at` works for a feed with no connector. `host_operations` plans each upload to the target of the draft feed move. With no target, each upload is in `blocked` with the reason "waits for a host".
- `plan_announcements` gives the new address, then the address before the move, as for each other move (ADR 0052 item 8).

### The Write

- `write_draft_signed` gets a new parameter `public: Option<&Web>`. Each caller that writes a feed with a connector passes `None`.
- For a feed with no connector:
  1. With no draft feed move, it refuses with `ReadOnly` and the text "choose a host for this feed first".
  2. With `public` `None`, it refuses with `ReadOnly`.
  3. Each blocked upload refuses the write, as now.
  4. It reads `public_url` through `public`, with the limit of `refresh_public`. When the SHA-256 is not `draft.base_version`, it refuses with `Conflict` and the text "the feed changed at its public address after the draft started".
  5. When the target has a file at the path, the write refuses the draft (ADR 0015 item 3).
  6. It puts the uploads, then the feed XML on the target. It writes nothing at the address before the move.
  7. It records the connector, the path and the public address of the feed, and sets the default host of the files to the target. Then it records the announcements and clears the draft, as each other write does.
- `write_draft` keeps its signature and passes `None`. Add `pub fn write_public_draft(collection, hosts, library, addresses, web: &Web, id, approved)` that passes `Some(web)`.

### The Link

- Add `pub fn rebase_linked_draft(collection: &Collection, connector: &dyn Connector, id: FeedId) -> Result<(), PipelineError>`. Call it after `set_feed_connector` links a read-only feed.
  - With no draft, it does nothing.
  - It removes the draft feed move, and sets the self link of the draft back, as `clear_feed_host` does.
  - It reads the host file. When the SHA-256 of its bytes is the SHA-256 of `draft.base_xml`, it sets `base_version` to the version of the host file. Otherwise `base_version` stays, so the write gives a conflict.

## Steps

1. Add `read_feed_xml` and `start_public_draft`, with their tests.
2. Change `set_feed_host` and add `clear_feed_host`, with their tests.
3. Add `FeedMoveMethod::None`, and change `feed_move`, `host_operations` and `diff_at`, with their tests.
4. Change `write_draft_signed` and add `write_public_draft`, with their tests.
5. Add `rebase_linked_draft`, with its tests.
6. Run the four checks.

## Constraints

- Each feed rule is in the core (ADR 0002, ADR 0065).
- The core has no HTTP server code. It can make HTTP requests (ADR 0002).
- The write of `public_url` goes only through the `Web` that the caller gives, so that the network policy of ADR 0057 applies. Do not call `Web::new()` in the write.
- No write occurs without the approved diff ID (ADR 0007).
- A read and a write with no edit give the same bytes. The corpus tests must pass with no change.
- Each code comment and doc comment is in ASD-STE100 Simplified Technical English, as the files are.
- Apply the `rust-dev` skill. No `unwrap` outside tests.

## Acceptance Criteria

Mechanical, each one a test in `crates/core/tests/pipeline.rs` with a local folder connector and a local HTTP test server:

- `start_public_draft` gives a draft whose base is the release folder XML and whose base version is `last_version`.
- `start_public_draft` refuses a feed with a connector, and a feed whose release folder XML has a different SHA-256.
- The diff of a read-only draft with an edit gives the unified diff and no feed move.
- The write of a read-only draft with no host choice refuses with `ReadOnly`.
- `set_feed_host` on a read-only draft gives the path of ADR 0067 item 7 and the self link of the new address. The diff gives a feed move with the method `none`.
- `clear_feed_host` removes the move and gives the self link of the base.
- The write of a moved read-only draft puts the feed on the target. The test server at the old address gets only the read request. The feed row then has the connector, the path, the address and the default host of the files. The feed keeps its `podcast:guid` and each item GUID.
- When the public file changed after the draft started, the write refuses with `Conflict`, and the target has no file.
- The announcements of the move give the new address, then the old address.
- `rebase_linked_draft` with the same bytes on the host gives the version of the host as the base version. With different bytes, a write gives `Conflict`.
- A granted write of a feed with no connector stays refused.
- The tests of the write of a feed with a connector pass with no change.
- The four commit checks pass.

Visual: none in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `host_operations` or `host_changes` needs the connector of the feed row for a different step.
- Stop and report when the new parameter of `write_draft_signed` changes more than its callers in `crates/core` and `crates/api`.
- Stop and report when the release folder XML has a name that is not the end of `public_url`.
- Stop and report when a test of the write of a feed with a connector fails.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-046a-core-draft-with-no-host.md
- docs/adr/0067-a-read-only-feed-opens-in-the-editor.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- docs/adr/0035-a-release-library-is-the-master-of-each-feed.md
- The files that the Files section names.

Goal:
- The core keeps, diffs and writes a draft of a feed with no connector.
- A move from a read-only feed writes no redirect and no pointer file. A folder match keeps the draft.

Constraints:
- Follow the Architecture section of the task file fully.
- Read the public address only through the `Web` that the caller gives.
- Apply the rust-dev skill. Write each comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The write of a feed with a connector, `write_granted`, the grant vaults, the corpus tests, `crates/api` and the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
