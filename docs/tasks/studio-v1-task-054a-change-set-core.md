# Studio v1 Task 054a: The Operations And The Change Set In The Core

Parent: [task 054](studio-v1-task-054-recipient-change-sets.md). Decision: [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 5 to 7, 9, 10 and 13 to 20.

Status: Not started.

## Goal

The core applies one recipient operation to a set of feeds as one change set. It gives the outcome of each feed, keeps the draft of each feed from before the change, and can put back these drafts.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed::recipients`, `push_recipients`), `crates/core/src/feed/value.rs` (`ValueBlock`, `ValueTimeSplit`), `crates/core/src/feed/elements.rs` (`Recipient`), `crates/core/src/feed/sync.rs` (`Entry`), `crates/core/src/pipeline.rs` (`edit_draft`, `current_draft`, `discard_draft`, `write_draft_signed`), `crates/core/src/collection.rs` (`MIGRATIONS`, `drafts`, `save_draft`, `delete_draft`, `known_recipients`), `crates/core/src/publisher.rs` (`Catalog`, `catalogs`).

Change: a new module `crates/core/src/change_set.rs`, `crates/core/src/lib.rs`, `crates/core/src/collection.rs` (one migration and its functions), `crates/core/src/pipeline.rs` (the mark of item 6 below), and their tests.

Do not change: the API, the UI, the ADRs.

## Architecture

These choices are fixed. Do not change them.

1. `RecipientKey { address, custom_key, custom_value }` identifies a recipient, as ADR 0027 item 3 does. An empty custom key is the same as no custom key.
2. `RecipientOperation` has four variants:
   - `ChangeAddress { address, custom_key, custom_value, recipient_type }`.
   - `Rename { name }`.
   - `Remove`.
   - `Add { name, recipient_type, address, custom_key, custom_value, percent }`, with `percent` from 1 to 99. The core refuses a different value.
3. `apply(feed: &mut Feed, key: &RecipientKey, operation: &RecipientOperation) -> Outcome` is a pure function on the model. It changes each value block of the channel, of each item and of each live item. It does not change `time_splits`.
   - Change address, Rename and Remove change each block that has the key.
   - Add changes each block of the feed. A block that has the key is "not changed" for that block.
4. `Outcome` is `Changed { channel: bool, items: usize, live_items: usize }` or `NotChanged(Cause)`. `Cause` has these values:
   - `NoRecipient`: no block has the key.
   - `HasRecipient`: each block has the recipient before the change (Add).
   - `HasNewAddress`: a block has the new key before the change (Change address). The core does not change that block. When no block changes, the outcome is `NotChanged(HasNewAddress)`.
   - `NoValueBlock`: the feed has no value block (Add).
   - `OnlyFees`: each other split of a block is a fee (Add and Remove).
   - `SplitNotNumber`: a non-fee split of a block is not a non-negative integer (Add and Remove). The core does not change that block.
5. The split rules of ADR 0091 items 13 to 15 are one function `scale(splits: &[u64], total: u64) -> Vec<u64>`. It keeps the relative shares, rounds down, and gives the remainder to the largest split. With two equal largest splits, the first one gets the remainder. A fee split is not an input.
   - Add: the new split is `percent`. The other non-fee splits scale to `100 - percent`.
   - Remove: the other non-fee splits scale to the non-fee total before the change.
6. The migration adds two tables:
   - `change_sets (id INTEGER PRIMARY KEY, operation TEXT NOT NULL, recipient TEXT NOT NULL, made_at INTEGER NOT NULL)`. `operation` and `recipient` are JSON.
   - `change_set_feeds (change_set_id, feed_id, outcome TEXT NOT NULL, before_xml BLOB, before_base_xml BLOB, before_base_version TEXT, had_draft INTEGER NOT NULL, written_at INTEGER, PRIMARY KEY (change_set_id, feed_id))`.
   - `write_draft_signed` sets `written_at` for each open change set row of the feed, after a write that succeeds.
7. `make_change_set(collection, start, key, operation, feeds) -> ChangeSet`. `start` is a closure that makes a draft for a feed with no draft, so that the core needs no connector. For each feed, the function keeps the draft before the change, calls `start` when the feed has no draft, and then calls `edit_draft` with `apply`. A feed with `NotChanged` keeps its draft as it was. When the feed had no draft, the function deletes the new draft. A failure of `start` is the outcome `NotChanged(Cause::CannotStart(reason))` for that feed.
8. `discard_change_set(collection, id)` puts back the kept draft of each feed with no `written_at`, byte for byte. A feed that had no draft then has no draft. Then the change set goes.
9. `level_feeds(catalogs, publisher) -> Vec<FeedId>` gives the publisher feed, each feed below it through `children` (all levels), and the releases of each of these feeds (`releases`). The list has each feed one time.
10. A change set is open while a changed feed has no `written_at`. `change_sets()` lists the open change sets.

## Steps

1. Write `scale` and `apply`, with unit tests.
2. Write the migration and the collection functions.
3. Write `make_change_set`, `discard_change_set` and `level_feeds`, with core tests.
4. Add the mark of `written_at` to `write_draft_signed`.
5. Run the four commit checks.

## Constraints

- No write to a host.
- A draft that a change set does not change keeps its bytes.
- `apply` does not change an attribute that the operation does not name.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- A core test: Change address replaces the address, the custom pair and the type in the channel block, an item block and a live item block. Each split, each name and each fee stays the same. A value time split does not change.
- A core test: Rename changes only `name`.
- A core test: Remove of a split of 20 from the non-fee splits 50, 30 and 20 gives 63 and 37 (total 100). A fee split stays the same.
- A core test: Add of 10 % to the non-fee splits 50 and 50 gives 45, 45 and 10. Add of 10 % to the splits 2 and 1 gives 60, 30 and 10.
- A core test: `scale` gives the remainder to the largest split, and to the first of two equal largest splits.
- A core test: each `Cause` of item 4.
- A core test: a feed with an open draft keeps its earlier edits after the change. A feed with no draft gets a draft.
- A core test: `discard_change_set` puts back each kept draft byte for byte, and deletes a draft that the change set made.
- A core test: after a write of one feed, `discard_change_set` does not change that feed.
- A core test: `level_feeds` gives a label, its artist feed below it, and the releases of both, each one time.
- The corpus tests pass with no change.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `edit_draft` cannot add a value block entry with no source element.
- Stop and report when a feed in the corpus has a split that is not an integer.
