# Studio v1 Task 030d: The Blossom Connector

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 2, 5, 10, 12 and 13, [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 3, 13, 16 and 18 to 20.
Sequence: the fourth packet of task 030. It depends on 030c.

## Goal

`studio-core` has the connector kind `blossom`. It keeps media blobs on one or more Blossom servers, with a path record for each blob in the collection. It passes the trait suite against `TestBlossom`.

## Files

Inspect: `crates/core/src/connector.rs` (the trait, `from_row_in`, `HostEnv`, `KeptBlob`, `secret_of`), `crates/core/src/connector/sftp.rs` (the form of a remote connector, its `Debug` and its list limits), `crates/core/src/nostr_events.rs`, `crates/core/src/connector/suite.rs`, `crates/core/src/testing/blossom.rs`, `crates/core/src/digest.rs`, `crates/core/src/media.rs` (a media type from a file extension, when it has one).

Change: add `crates/core/src/connector/blossom.rs` and `crates/core/src/connector/blossom_http.rs`. Change `connector.rs` (the module lines, the exports and `from_row_in`).

Do not change: the pipeline, the API, the UI, the other connectors, the suite, the test servers.

## Architecture

These choices are fixed. Do not change them.

### The Host

- `pub struct BlossomHost` with `pub const KIND: &'static str = "blossom"`. Its configuration JSON has `public_key` (64 hexadecimal characters), `servers` (one or more `http` or `https` addresses) and `signing` (`"vault"` or `"signer"`).
- With `"vault"`, the secret is `{"secret_key": "<hex>"}`. The key must give the public key of the configuration, or the row gives `InvalidConfig`. With `"signer"`, the row has no secret.
- The `Debug` text shows no secret key.
- `from_row_in` makes a `BlossomHost` with `env.blobs` as its records and `env.signing` as the signing of the write.

### The HTTP Calls (`blossom_http.rs`, crate-private)

- `Spooled`: the bytes of one write, with their SHA-256 and size. `Spooled::from_bytes(&[u8])` keeps a slice in memory. `Spooled::from_reader(reader, length)` writes the bytes to `std::env::temp_dir()` in a file named `studio-spool-<uuid v4>` and calculates the SHA-256 at the same time. A reader with a different length gives `Io`. The file goes when the value drops.
- `upload(server, &Spooled, media_type, auth_header) -> Result<Descriptor, ConnectorError>`: `PUT <server>/upload`. A descriptor with a different SHA-256 gives `Io`.
- `mirror(server, source_url, sha256, auth_header) -> Result<Descriptor, ConnectorError>`: `PUT <server>/mirror`.
- `head(server, sha256) -> Result<Option<u64>, ConnectorError>`: the size, or `None` for `404`.
- `get(server, sha256) -> Result<Option<Vec<u8>>, ConnectorError>`. Bytes with a different SHA-256 give `Io`.
- `delete(server, sha256, auth_header) -> Result<bool, ConnectorError>`: `false` for `404`.
- The status mapping: `401` and `403` give `Unauthorized`. `413` gives `Io` with "the server refused the size of the file". A connection failure gives `Unavailable`. Each other status gives `Io` with the status code. No error message contains an `Authorization` header.

### The Signing

- When the pipeline calls `plan_events` on the connector, the connector keeps the planned events for the write that follows. Each upload and each delete then uses the planned event of its server.
  - With `"vault"`, the connector signs the planned event with its key.
  - With `"signer"`, it uses `nostr_events::sign`, which finds the signed event in `env.signing`.
- With no plan, the connector makes an event for each upload or delete, with the time of the computer and one `x` tag. With `"signer"` and no plan, an upload or a delete gives `SignatureMissing` before any request.
- `plan_events(changes, plan_time)` gives, for each server in the sequence of `servers`:
  - one upload event (`AuthAction::Upload`) with an `x` tag for each SHA-256 of `changes.writes`, when the list is not empty.
  - one delete event (`AuthAction::Delete`) with an `x` tag for each blob that the write deletes from the servers, when there is one.
  - `user_signs` is true for `"signer"`.

### The Operations

- The version and the stamp of a path are `sha256:<hex>`, the form of `digest::content_version`.
- `read`: the record of the path, then `get` from each server in sequence until one gives the bytes. A path with no record gives `None`.
- `write` and `write_from`:
  1. `Expected::Absent` needs a path with no record. `Expected::Version` needs a record with that version. If not, the write gives `Conflict`.
  2. The bytes go into a `Spooled`.
  3. `upload` to the first server, with the media type from the extension of the path. Then `mirror` to each other server, from `<first server>/<sha256>`.
  4. The record of the path gets the SHA-256 and the size.
- `stat`: the record. `list`: the records, with the limits and the hidden names of `LocalFolder::list`.
- `delete(path, stamp)`: a path with no record gives `NotFound`. A different stamp gives `Conflict`. The record goes. When no other record has the same SHA-256, the connector sends `delete` to each server. A `404` is not an error.
- `sha256`: the SHA-256 of the record, with no request.
- `public_url`: `<first server>/<sha256>.<extension>` for a path with a record. `planned_url(path, Some(sha256))` gives the same form for a path with no record.
- `kept_blobs`: `Some` with each record.
- `can_redirect` uses the trait default.

## Steps

1. Add `blossom_http.rs`, with tests against `TestBlossom` for each call and for the removal of the file of `Spooled`.
2. Add `BlossomHost` and `from_row_in`, with tests of the configuration, the secret and the `Debug` text.
3. Add the operations, with one test for each check of the suite against two `TestBlossom` servers, as `connector/sftp.rs` does.
4. Add the tests below.

## Tests

- A write puts the blob on both servers.
- Two paths with the same bytes: the delete of the first path sends no `DELETE` (`TestBlossom::deletes` is empty). The delete of the second path sends one `DELETE` to each server.
- `public_url` and `planned_url` give `<first server>/<sha256>.mp3` for a path that ends in `.mp3`.
- With `"signer"` and no plan, a write gives `SignatureMissing`, and no server has a blob.
- With a plan of two writes and `"vault"`, both writes use the one planned upload event of each server.
- `write_from` with a reader of a different length gives `Io`, and no server has a blob.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. No `unsafe` code.
- No secret key is in a log, an error message or the `Debug` text.
- `write_from` does not keep the full file in memory.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- `BlossomHost` passes each check of the suite against `TestBlossom`.
- The tests of this packet pass.
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

- `BlossomHost` fails a check of the suite. Do not change the suite.
- `TestBlossom` refuses a request that follows BUD-01, BUD-02, BUD-04 or BUD-11. Do not change the test server.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030d-blossom-connector.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- crates/core/src/connector/blossom.rs has BlossomHost, the connector kind blossom. It uploads and mirrors blobs with BUD-11 tokens, and keeps a record for each path. It deletes a blob only when no other record has its SHA-256. It passes the trait suite against TestBlossom.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No unsafe code. write_from does not keep the full file in memory.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The pipeline, the API, the UI, the other connectors, the suite, the test servers.
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
