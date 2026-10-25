# Studio v1 Task 029d: The Host Key Confirmation And The SFTP Host Route

Parent: [task 029](studio-v1-task-029-sftp-connector.md). Decisions: [ADR 0047](../adr/0047-the-sftp-connector.md) items 1 to 4 and 6, [ADR 0003](../adr/0003-user-unlocked-secret-vault.md) and [ADR 0021](../adr/0021-public-address-of-a-feed.md).
Sequence: the fourth packet of task 029. It depends on 029c.

## Goal

The API gives the fingerprint of the host key of a server before the user adds it. `POST /api/v1/connectors` adds an SFTP host with the fingerprint that the user accepted. The secrets go into the vault only. The user can accept a new host key, and can change the public address, as for an S3 host.

## Files

Inspect: `crates/core/src/connector/sftp.rs`, `crates/core/src/testing/sshd.rs`, `crates/core/src/address.rs` (`folder_url`), `crates/api/src/lib.rs` (the routes), `crates/api/src/routes.rs` (`NewConnector`, `add_s3_connector`, `check_http_address`, `update_connector`, `set_connector_base_url`), `crates/api/src/error.rs`, `crates/api/src/tests.rs`.

Change: `address.rs`, `lib.rs`, `routes.rs`, `error.rs`, `tests.rs`, and the core tests of `address.rs`.

Do not change: the UI, the pipeline, the connectors other than a small helper that the route needs.

## Architecture

These choices are fixed. Do not change them.

### The Fingerprint Route

- `POST /api/v1/connectors/fingerprint` with `{"host": "…", "port": 22}` calls `SftpHost::fingerprint_of`, and gives `{"fingerprint": "SHA256:…"}`. It needs no vault. A connection failure gives 503 `host_unavailable`. An empty host gives 422 `invalid_config`.

### The Add Route

- `NewConnector` gets a variant `Sftp` with `name`, `host`, `port`, `user`, `root`, `base_url` (optional), `login` (`"key"`, `"password"` or `"agent"`), `private_key`, `passphrase`, `password` (each optional) and `host_key`.
- The route does these steps, in this sequence:
  1. For the login `key` or `password`, with no open vault, it gives 409 `vault_required` or 423 `vault_locked`, as for S3. The login `agent` needs no vault.
  2. It checks the fields. An empty host, user, root or `host_key` gives 422 `invalid_config`. The login `key` with no `private_key`, or `password` with no `password`, gives 422 `invalid_config`. A `base_url` that is not an `http` or `https` address gives 422 `invalid_address`.
  3. It makes the `SftpHost` and connects, with `host_key` as the accepted fingerprint. Then `stat` of `root` must give a folder, or the route gives 422 `folder_not_found`.
  4. For `key` and `password`, it puts the secret JSON into the vault with the ID `connector-<uuid v4>`, and saves the vault.
  5. It adds the collection row. When step 4 or step 5 fails, it removes the secret from the vault again, as `add_s3_connector` does.
- When step 3 fails, nothing goes into the vault or the collection. A different host key gives 502 `host_unauthorized`, with the new fingerprint in the message. A server with no `posix-rename` gives 422 `no_conditions`.
- The response is the connector row, with no secret.
- The desktop shell is the only shell now, so the route accepts the login `agent`. The Server Shell phase must refuse it ([ADR 0047](../adr/0047-the-sftp-connector.md) item 2).

### A New Host Key And The Public Address

- `PUT /api/v1/connectors/{id}` accepts `host_key` for an `sftp` row. The route calls `fingerprint_of` and compares. When the server gives a different fingerprint than `host_key`, it gives 409 `host_key_mismatch`, and the row does not change. The user thus accepts only the key that the server gives now.
- `address::folder_url` and `set_connector_base_url` cover the kind `sftp`, as they cover `s3`.
- The match route works for an `sftp` host, as for an `s3` host.

## Steps

1. Add the fingerprint route, with API tests against `TestSshd`.
2. Add the `Sftp` variant of the add route, with an API test for each step and each error. The tests log in with the key of `TestSshd`.
3. Add `host_key` to `PUT /connectors/{id}`, with a test after `with_host_key_change`.
4. Change `folder_url` and `set_connector_base_url`, with tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- No password, private key or passphrase in a log, an error message, an API response, the collection row or the collection database file.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- The fingerprint route gives `TestSshd::host_key_fingerprint()`.
- `POST /connectors` with the login `key` adds the host. The vault holds the key under the `secret_ref` of the row. The private key text is not in the row, the response, the data files or the vault file as plain text.
- With a wrong `host_key`, the route gives 502 `host_unauthorized`, and the vault and the collection do not change.
- With no vault and the login `key`, the route gives 409 `vault_required`. With the login `agent`, no vault is necessary for the checks before the connection.
- A `root` that is not a folder gives 422 `folder_not_found`.
- After `with_host_key_change`, `PUT` with the old fingerprint gives 409 `host_key_mismatch`. `PUT` with the new fingerprint changes the row, and a connection then works.
- `PUT /connectors/{id}` changes the `base_url` of an SFTP host, and `folder_url` gives it.
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

Stop and report when the match route needs more than a small change for the kind `sftp`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-029d-sftp-host-route.md
- docs/adr/0047-the-sftp-connector.md
- The files that the Files section names.

Goal:
- POST /api/v1/connectors/fingerprint gives the host key fingerprint of a server. POST /api/v1/connectors adds an SFTP host with the accepted fingerprint, and its secrets go into the vault only. PUT /connectors/{id} accepts a new host key only when the server gives it.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the pipeline, the connectors other than a small helper.
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
