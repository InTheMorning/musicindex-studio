# Studio v1 Task 054b: The Routes Of A Change Set

Parent: [task 054](studio-v1-task-054-recipient-change-sets.md). Decision: [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 2, 3, 9, 12 and 17 to 26.

Status: Not started. It needs task 054a.

## Goal

The API lists the uses of a recipient, makes a change set, gives its state and discards it. The UI writes each feed through the existing write route.

## Files

Inspect: `crates/api/src/lib.rs` (the routes), `crates/api/src/routes.rs` (`list_recipients`, `list_feeds`, `files_feed`, `ensure_draft`, `writer`, `get_diff`, `write_feed`), `crates/core/src/change_set.rs` of task 054a, `crates/core/src/collection.rs` (`known_recipients`, `saved_recipients`).

Change: `crates/api/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`. Change `crates/core/src/collection.rs` only to change a saved recipient (item 5 below).

Do not change: the UI, the ADRs, the write route.

## Architecture

These choices are fixed. Do not change them.

1. `POST /api/v1/recipients/uses` with a `RecipientKey` gives each feed that has the recipient. A feed row has `feed_id`, `title`, `medium`, `channel`, `items`, `live_items` and `writable`. The route reads each model with `files_feed`. A body is necessary, because an address can have each character.
2. `POST /api/v1/change-sets` with `recipient`, `operation` and `feed_ids` makes a change set. The `start` closure of task 054a calls `ensure_draft`. For Add, the UI gets `feed_ids` from `GET /api/v1/feeds/{id}/level` (item 3).
3. `GET /api/v1/feeds/{id}/level` gives `feed_ids` from `level_feeds` of task 054a. A feed that is not a publisher feed gives 422 `not_publisher`.
4. `GET /api/v1/change-sets` lists the open change sets. `GET /api/v1/change-sets/{id}` gives the operation, the recipient, and each feed with its outcome, its `written_at` and its cause when Studio cannot write it now. The causes are `read_only_no_host` (ADR 0067) and `vault_locked`.
5. A change set with Change address or Rename also changes the saved recipient of the old key, when one exists (ADR 0091 item 12).
6. `DELETE /api/v1/change-sets/{id}` calls `discard_change_set`.
7. The UI writes each feed with `GET /feeds/{id}/diff`, then `POST /feeds/{id}/write`. No new write route exists. Thus each write uses the ID of its own diff (ADR 0091 item 23), and the signer flow of the review sheet applies to each feed.
8. The change set routes need the vault of the user. A connected app token gets 401, as for each other `/api/` route. A show grant does not cover a change set (ADR 0091 item 26).

## Steps

1. Write the routes and their API tests.
2. Run the four commit checks.

## Constraints

- No write to a host in a change set route.
- No secret in a response.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- An API test: `recipients/uses` gives the places of a recipient in two feeds, and not a feed that has it only in a value time split.
- An API test: a change set over three feeds gives one `Changed` feed, one `NotChanged` feed with its cause, and one read-only feed with `read_only_no_host`.
- An API test: after `POST /feeds/{id}/write` for a changed feed, the change set gives its `written_at`.
- An API test: `DELETE` puts back the draft of a feed with an earlier draft, byte for byte.
- An API test: Rename changes the name of the saved recipient.
- An API test: `feeds/{id}/level` of an album gives 422.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `ensure_draft` of a feed needs a host that is not connected, and that is not a read-only feed.
- Stop and report when the server shell needs a different route for the vault check.
