# Studio v1 Task 022a: The API Marks And The Host Address

Parent: [task 022](studio-v1-task-022-ui-pass-after-7a.md). Decision: [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 11 to 14.
Sequence: the first packet of task 022.

## Goal

The files route tells the UI which copy and which host file the feed uses, and which operation waits for the write. The user can change the public address of a host.

## Files

Inspect: `crates/api/src/routes.rs` (`list_release_files`, `add_connector`, `list_connectors`), `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/core/src/collection.rs` (`set_connector_config`), `crates/core/src/connector/local.rs` (`config`), `crates/core/src/pipeline.rs` (`library_file_in_use`, the addresses of a draft).

Change: `routes.rs`, `lib.rs`, `tests.rs`. Change `crates/core` only to make a private function public, such as the function that gives the addresses of a draft.

Do not change: the UI.

## Architecture

These choices are fixed. Do not change them.

- `GET /api/v1/feeds/{id}/files`:
  - Each copy of a library file gets `in_use: bool`: true when its URL is an address of the draft.
  - Each host file in "other files" gets `in_use: bool` and `url`.
  - Each copy and each other file gets `pending: null | { kind: "copy" | "delete", operation_id }` from the draft operations with the same host and path.
  - Each library file gets `pending_copies`: each draft copy operation of the file, with its host, its path and its `operation_id`.
- `PUT /api/v1/connectors/{id}` with `{ name?, base_url? }` changes the name and the public address of a local folder host. An empty `base_url` removes the address. A `base_url` that is not an http or https address gives 422 `invalid_address`. It sets `base_url_inferred` to false. Each feed address of the folder source changes as after a folder match ([ADR 0021](../adr/0021-public-address-of-a-feed.md)).

## Steps

1. Add the marks to the files route.
2. Add the host route.
3. Add the tests.

## Constraints

- Each route needs the access token (ADR 0017).
- No route writes to a host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- After a write, the copy that the enclosure refers to has `in_use: true`, and a second copy has `in_use: false`.
- A delete operation on an other file gives that file `pending.kind = "delete"` with the operation ID. A copy operation gives the library file a `pending_copies` entry.
- `PUT /connectors/{id}` with a new `base_url` changes the address in `GET /connectors`, and `GET /feeds/{id}/files` gives URLs on the new address.
- `PUT /connectors/{id}` with `ftp://x` gives 422.
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

Stop and report when the host route needs a change of the connector interface.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-022a-api-marks-and-host-address.md
- docs/adr/0036-library-records-host-inventory-and-host-operations.md
- The files that the Files section names.

Goal:
- The files route tells the UI which copy and which host file the feed uses, and which operation waits for the write. The user can change the public address of a host.

Constraints:
- Follow the Architecture section of the task file fully.
- Each route needs the token. No route writes to a host.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI.
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
