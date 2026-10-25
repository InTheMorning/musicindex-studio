# Studio v1 Task 028c: The Provider Test And The Host Route

Parent: [task 028](studio-v1-task-028-s3-connector.md). Decisions: [ADR 0046](../adr/0046-the-s3-and-r2-connector.md) items 1, 2 and 4 to 6, [ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0021](../adr/0021-public-address-of-a-feed.md) and [ADR 0020](../adr/0020-first-run-asks-one-question.md).
Sequence: the third packet of task 028. It depends on 028b.

## Goal

`POST /api/v1/connectors` adds an S3 host. Before it adds the host, Studio tests that the provider refuses a conditional write that is not correct. The access keys go into the vault only. The public address of an S3 host works as the public address of a local folder. The user can change it, and the feeds of the host get it.

## Files

Inspect: `crates/core/src/connector/s3.rs`, `crates/core/src/testing/fake_s3.rs`, `crates/core/src/address.rs` (`folder_url`), `crates/api/src/routes.rs` (`NewConnector`, `add_connector`, `update_connector`, `set_connector_base_url`, `match_connector`, `import_msp`: the vault pattern), `crates/api/src/error.rs`, `crates/api/src/tests.rs`, `crates/core/src/collection.rs` (`add_connector`).

Change: `s3.rs` (the provider test), `address.rs`, `routes.rs`, `error.rs`, `tests.rs`, and the core tests of these files.

Do not change: the UI, the pipeline, the trait, `LocalFolder`.

## Architecture

These choices are fixed. Do not change them.

### The Provider Test

- `S3Bucket::test_provider(&self) -> Result<(), ConnectorError>` does the four steps of ADR 0046 item 4 with the key `.studio-probe-<uuid v4>` below the prefix. It deletes the key at the end, also after a failed step, when it can.
- When the second or the third PUT does not give 412, the function gives a new error kind `ConnectorErrorKind::NoConditions`. Its message is "the provider does not refuse a conditional write, so Studio cannot write to it safely".
- 403 gives `Unauthorized`, and a failed connection gives `Unavailable`, as in 028b.

### The Route

- `NewConnector` gets a second variant, `S3`, with `name`, `endpoint`, `region`, `bucket`, `url_style`, `prefix`, `base_url` (optional), `access_key_id` and `secret_access_key`.
- The route does these steps for `S3`, in this sequence:
  1. With no open vault, it gives 409 `vault_required` when no vault exists, and 423 `vault_locked` when the vault is locked, as `import_msp` does.
  2. It checks the fields. An endpoint or a `base_url` that is not an `http` or `https` address gives 422 `invalid_address`. An empty bucket or key gives 422 `invalid_config`. A prefix that is not empty and does not end with `/` gets the `/`.
  3. It makes the `S3Bucket` and calls `test_provider`.
  4. It puts the secret JSON into the vault with the ID `connector-<uuid v4>`, and saves the vault.
  5. It adds the collection row with that `secret_ref` and the configuration with no secret.
- When step 3 fails, nothing goes into the vault or the collection.
- `error.rs` gives `NoConditions` as 422 `no_conditions`.
- The response is the connector row, as for a local folder. It contains no secret.

### The Public Address

- `address::folder_url` gives the `base_url` of an `s3` row too.
- `set_connector_base_url` changes the `base_url` of an `s3` row too, with the same checks. The `update_connector` route thus changes it.
- The match route (`POST /connectors/{id}/match`) works for an `s3` host as for a local folder. When it needs a change for the kind `s3`, make the smallest change.

## Steps

1. Add `NoConditions` and `test_provider`, with tests against `FakeS3`: a correct provider passes, `ignore_put_conditions(true)` gives `NoConditions`, and the probe key is gone after each test.
2. Add the `S3` variant of the route, with API tests for each step and each error.
3. Change `folder_url` and `set_connector_base_url`, with tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- No test sends a request to a host outside the computer.
- The API tests use `FakeS3` through the `testing` feature.

## Acceptance Criteria

Mechanical:

- `test_provider` passes on `FakeS3`, gives `NoConditions` with `ignore_put_conditions(true)`, and leaves no probe key in both cases.
- `POST /connectors` with an S3 body adds the host. The vault holds the secret JSON under the `secret_ref` of the row. The row configuration, the response and the collection database file do not contain the secret access key.
- With `ignore_put_conditions(true)`, the route gives 422 `no_conditions`. The vault and the collection do not change.
- With no vault, the route gives 409 `vault_required`. With a locked vault, it gives 423 `vault_locked`.
- With `refuse_all(true)`, the route gives 502 `host_unauthorized`.
- An endpoint that is not an `http` or `https` address gives 422 `invalid_address`.
- `PUT /connectors/{id}` changes the `base_url` of an S3 host, and `folder_url` gives it.
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

Stop and report when the match route needs more than a small change for the kind `s3`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-028c-provider-test-and-route.md
- docs/adr/0046-the-s3-and-r2-connector.md
- The files that the Files section names.

Goal:
- POST /api/v1/connectors adds an S3 host after the provider test passes. The access keys go into the vault only. The public address of an S3 host works as for a local folder.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the pipeline, the trait, LocalFolder.
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
