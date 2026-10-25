# Studio v1 Task 027a: The Trait Changes

Parent: [task 027](studio-v1-task-027-connector-interface.md). Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) items 3 to 6.
Sequence: the first packet of task 027.

## Goal

The `Connector` trait gets a streaming write, the error types `Unauthorized` and `Unavailable`, and an optional SHA-256 from the host. `from_row` gets the secret of a connector row from the open vault. The upload step of the write uses the streaming write, so it does not read a full library file into memory. The inventory asks the connector for the SHA-256 before it reads a host file.

## Files

Inspect: `crates/core/src/connector.rs`, `crates/core/src/connector/local.rs`, `crates/core/src/pipeline.rs` (`put_uploads_and_copies`), `crates/core/src/inventory.rs` (`host_sha`), `crates/core/src/digest.rs`, `crates/core/src/vault.rs` (`OpenVault::get`), `crates/api/src/error.rs`, `crates/api/src/workspace.rs` (`hosts`).

Change: `connector.rs`, `connector/local.rs`, `pipeline.rs`, `inventory.rs`, `error.rs`, and the tests of these files.

Do not change: the UI, the collection schema, `confirm_new_addresses`, the API routes.

## Architecture

These choices are fixed. Do not change them.

- `ConnectorErrorKind` gets two kinds:
  - `Unauthorized`: the host refused the credentials.
  - `Unavailable`: Studio cannot connect to the host.
- `crates/api/src/error.rs` gives `Unauthorized` as 502 `host_unauthorized`, and `Unavailable` as 503 `host_unavailable`. Use the same code pattern as the current kinds.
- The trait gets this function:

  ```rust
  fn write_from(
      &self,
      path: &str,
      reader: &mut dyn std::io::Read,
      length: u64,
      expected: &Expected,
  ) -> Result<Version, ConnectorError>;
  ```

  It has the same version check as `write`. It reads `length` bytes. When the reader gives fewer or more bytes, it gives `ConnectorErrorKind::Io`, and the file on the host does not change.
- The default `write_from` reads the bytes into a `Vec<u8>` with the length check, and then calls `write`. The test doubles of the trait thus need no change.
- `LocalFolder` has its own `write_from`. It copies the reader into the temporary file in blocks of 64 KiB, with a SHA-256 of the bytes. Then it renames the file, as `write` does. The version is the SHA-256 of the bytes, as for `write`. For `Expected::Absent`, it uses `stat` for the version check. It does not read the current file.
- `write` stays. The pipeline uses it for the feed XML.
- The trait gets this function, with a default that gives `Ok(None)`:

  ```rust
  fn sha256(&self, path: &str) -> Result<Option<String>, ConnectorError>;
  ```

  The value is lowercase hexadecimal. `LocalFolder` uses the default.
- `inventory::host_sha` calls `connector.sha256(path)` when the row has no SHA-256. When it gives a value, `host_sha` records it with the stamp from `stat`, and it does not read the file. When it gives `None`, `host_sha` reads the file as before.
- `put_uploads_and_copies` opens the library file and gives it to `write_from` with its size from the file metadata. When the library row has no SHA-256, it uses `digest::sha256_file`. It does not call `std::fs::read`.
- `from_row` gets the secret of a row with a private function `secret_of(row, vault)`:
  - A row with no `secret_ref` gives `Ok(None)`.
  - A row with a `secret_ref` and no open vault gives `ConnectorErrorKind::Unauthorized`, with the message "the vault is locked".
  - A row with a `secret_ref` that the vault does not have gives `ConnectorErrorKind::InvalidConfig`.
  - In all other cases, it gives the secret from `OpenVault::get` as a `Zeroizing` value.
- `from_row` calls `secret_of` for each row before it makes the connector. The local folder kind ignores the value. A remote connector task uses it.
- No secret is in an error message or a log.

## Steps

1. Add the two error kinds and their API codes, with an API test for each code.
2. Add `write_from` with its default and the `LocalFolder` function, with tests.
3. Add `sha256` with its default, and change `host_sha`. Its test uses a test double that gives a SHA-256 and refuses `read`.
4. Change `put_uploads_and_copies` to `write_from`.
5. Add `secret_of` to `from_row`, with a unit test for each of its four cases.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate, and no new migration.
- The current tests pass with no change of their assertions.

## Acceptance Criteria

Mechanical:

- `LocalFolder::write_from` writes the bytes of a reader and gives the same version as `write` with the same bytes.
- `LocalFolder::write_from` with a reader that gives fewer bytes than `length` gives `Io`, and no file is at the path.
- `LocalFolder::write_from` with `Expected::Absent` gives `Conflict` when a file is at the path, and the file does not change.
- `host_sha` gives the SHA-256 of a connector that gives one, with no call to `read`.
- `secret_of` gives each result of the four cases above.
- The API gives 502 `host_unauthorized` for `Unauthorized` and 503 `host_unavailable` for `Unavailable`.
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

Stop and report when a test double in `crates/core/tests/` or `crates/core/src/inventory.rs` needs more than the default functions.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-027a-trait-changes.md
- docs/adr/0043-phase-9-connectors-and-the-interface-change.md
- The files that the Files section names.

Goal:
- The Connector trait gets write_from, sha256, and the error kinds Unauthorized and Unavailable. from_row gets the secret of a row from the open vault. The upload step uses write_from.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate, and no new migration.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, the collection schema, confirm_new_addresses, the API routes.
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
