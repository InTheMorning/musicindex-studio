# Task 125: One Podping After A Move

- Decision: [ADR 0189](../adr/0189-a-move-sends-one-podping-of-the-old-address.md)

Status: Complete on 2026-11-12. The mechanical checks passed. The screenshot visual check passed: the notice gives one line, for the address before the move, with **Send the Podping**. The human test is open.

## Goal

A move write announces only the address before the move. The notice after a move gives one podping line, for that address. **Send the Podping** sends that address.

## Work

- `crates/core/src/pipeline.rs`, `announce_addresses`: with a feed move, give `from_url` when it has a value, or else the new `url`. Write one address, not two (ADR 0189 items 1 and 2).
- `crates/core/src/outside.rs`, `last_written_move`: give one announcement line. The address is `movement.from_url` when it has a value, or else `row.public_url` (ADR 0189 item 3).
- `crates/api/src/routes.rs`, `send_announcement`: when the feed has a last written move, queue the address of the line of `last_written_move`. Without a move, keep the current address. Keep the 422 `no_public_address` when no address is available (ADR 0189 item 4).
- Change each Rust test and each API test that expects two announcements of a move. Give each new or changed assertion a failure message that names ADR 0189 and the fix.
- `crates/api/ui/outsidecheck.js` takes its lines from the API. Change it only if a Node test fails. Keep the four-state Node tests.

## Acceptance Criteria

Mechanical:

- A Rust test: the diff and the write of a move plan one announcement, of the address before the move.
- A Rust test: a move with no address before the move plans one announcement of the new address.
- A Rust test: `last_written_move` gives one line, for the address before the move.
- An API test: `POST /feeds/{id}/announcements` after a move queues the address before the move.
- The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the planning model:

- The notice with one line, at 1280 px, in the light theme.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0189, after a real move between two file hosts.

## Stop And Ask When

- A different caller of `announce_addresses` needs two addresses.
- An announcement with no feed move changes.
