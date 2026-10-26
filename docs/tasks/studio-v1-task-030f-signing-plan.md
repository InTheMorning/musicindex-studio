# Studio v1 Task 030f: The Planned Events In The Diff, And The Signed Events In The Write

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 10 to 13, [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 15 to 17, [ADR 0007](../adr/0007-draft-diff-write-and-announce.md).
Sequence: the sixth packet of task 030. It depends on 030e.

## Goal

The diff gives each event that the write signs, with no signature. The write request can send the events that the user signed. Before the first change on a host, the write makes the planned events again and checks each signed event. A missing, different or expired event stops the write.

## Files

Inspect: `crates/core/src/pipeline.rs` (`diff`, `write_draft`, `host_operations`, `feed_move`, `diff_id`, `discard_draft`, `PipelineErrorKind`), `crates/core/src/collection.rs` (`MIGRATIONS`), `crates/core/src/connector.rs` (`HostChanges`, `PlannedEvent`), `crates/core/src/nostr_events.rs`, `crates/core/tests/pipeline.rs` (the nsite test of 030e), `crates/api/src/routes.rs` (`diff_json`, `Write`, `write_feed`), `crates/api/src/workspace.rs` (`hosts_with`), `crates/api/src/error.rs`, `crates/api/src/tests.rs`.

Change: `pipeline.rs`, `collection.rs`, `crates/core/tests/pipeline.rs`, `routes.rs`, `error.rs`, `tests.rs`.

Do not change: the connectors, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Signing Plan In The Collection

- A new entry in `MIGRATIONS` adds the table `signing_plans`: `feed_id` (the primary key, a reference to `feeds` with `ON DELETE CASCADE`), `diff_id` and `plan_time` (seconds of Unix time).
- `Collection::signing_plan(feed_id) -> Option<(String, u64)>`, `put_signing_plan(feed_id, diff_id, plan_time)` and `remove_signing_plan(feed_id)`.

### The Changes Of Each Host

- `fn host_changes(...) -> BTreeMap<ConnectorId, HostChanges>` gives the changes of the write on each host:
  - each upload and each copy is a `PlannedWrite` with the path, the SHA-256 and the size of its library file.
  - each deletion is a `PlannedDelete` with the path, and the SHA-256 of the inventory record when it has one.
  - the feed XML is a `PlannedWrite` on the feed host, or on the new host of a feed move.
  - a pointer file of a feed move is a `PlannedWrite` on the host from before.
- A confirm operation gives no change.

### The Diff

- `pipeline::diff` calls `diff_at` with the time of the computer. `diff_at(..., now)` makes the diff as before, and then:
  1. It reads the signing plan of the feed. It keeps the plan time when the diff ID is the same and `plan_time + 3600 - 600 > now`. If not, the plan time is `now`, and it puts the new plan into the collection.
  2. It calls `plan_events(changes, plan_time)` on each host of `host_changes`, in the sequence of the host IDs.
- `Diff` gets `events: Vec<DiffEvent>`. A `DiffEvent` has `connector_id`, `user_signs` and the unsigned `event`.
- The diff ID does not change: it does not include the events.

### The Write

- `write_draft` calls `write_draft_signed` with no signed events and the time of the computer. `write_draft_signed(..., signed: &[nostr::Event], now)` does the steps of `write_draft`. After the check of the diff ID, and before step 1, it does these checks:
  1. It calls `plan_events` on each host of `host_changes` with the plan time. Each connector then keeps its plan for the steps that follow.
  2. When a host gives events and the feed has no plan with this diff ID, or `now >= plan_time + 3600 - 60`, the write gives the new kind `PlanExpired`.
  3. For each event with `user_signs`, `signed` must have an event with the same ID. A missing event gives the new kind `SignatureMissing`. An event with a bad signature or a different public key gives the new kind `SignatureInvalid`.
- After a write, the signing plan of the feed goes out. `discard_draft` removes it too.
- `pipeline::plan_time_for_write(collection, id, approved, now) -> Option<u64>` gives the plan time when the plan has this diff ID and has not expired. The write route uses it to make the hosts.

### The API

- `diff_json` adds `"events"`: for each `DiffEvent`, `connector_id`, `user_signs` and `event` with `pubkey`, `created_at`, `kind`, `tags`, `content` and `id`. A NIP-07 `signEvent` accepts this object.
- The body of `POST /feeds/{id}/write` gets `signed_events`, a list of JSON events, empty when it is absent. An event that does not parse gives 422 `invalid_event`.
- `write_feed` makes the hosts with `w.hosts_with(Signing { plan_time, signed })`, and calls `write_draft_signed` with the same signed events.
- The error mapping: `SignatureMissing` gives 409 `signature_required`, `SignatureInvalid` gives 422 `signature_invalid`, and `PlanExpired` gives 409 `plan_expired`. A `ConnectorErrorKind::SignatureMissing` from a connector gives `SignatureMissing`.

## Steps

1. Add the `signing_plans` table, with a test of its functions.
2. Add `host_changes`, the events of the diff and the checks of the write.
3. Add the core tests below.
4. Change the API, with the API tests below.

## Tests

Core tests, with a feed on an nsite host and the test servers:

- With `"signer"`, the diff gives planned events with `user_signs` true. A second diff gives the same event IDs.
- With `"signer"`, a write with no signed events gives `SignatureMissing`. The relays got no event, and the servers have no blob.
- With `"signer"`, a write with the planned events signed by the test key works. The relays have the new manifest.
- A planned event signed by a different key gives `SignatureInvalid`, and nothing changes on a host.
- A write at `plan_time + 3600` gives `PlanExpired`, and nothing changes on a host.
- A diff 55 minutes after the first diff, with the same diff ID, gives a new plan time.
- With `"vault"`, a write needs no signed events.
- A write to a local folder host needs no plan.

API tests:

- `GET /feeds/{id}/diff` gives `events` for an nsite host with `"signer"`.
- `POST /feeds/{id}/write` with the signed events works. With no events, it gives 409 `signature_required`.
- An event that does not parse gives 422 `invalid_event`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- The existing calls of `write_draft` and `diff` do not change.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- The tests of this packet pass.
- A write with "My signer" changes nothing on a host before each planned event has a correct signature.
- The four commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report in these cases:

- The events of a second diff do not have the same IDs as the events of the first diff.
- A check of this packet needs a change of a connector.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030f-signing-plan.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- The diff gives the planned Nostr events of each host, with a plan time that the collection keeps. The write takes the signed events, makes the planned events again, and checks each one before the first change on a host.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. The existing calls of write_draft and diff do not change.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connectors, the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
