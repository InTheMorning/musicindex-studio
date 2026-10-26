# Studio v1 Task 030a: The Test Relay And The Test Blossom Server

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) items 1 to 3, [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) item 15, [ADR 0051](../adr/0051-the-core-signs-nostr-events-with-secp256k1.md).
Sequence: the first packet of task 030.

## Goal

The `testing` module of `studio-core` has a small blocking Nostr relay and a small Blossom server. The connectors of 030d and 030e test against them. The core gets the crates `nostr`, `tungstenite` and `secp256k1`, and the function that signs an event with a key.

## Files

Inspect: `crates/core/src/testing.rs`, `crates/core/src/testing/fake_s3.rs` (the form of a test server), `crates/core/src/testing/sshd.rs`, `Cargo.toml`, `crates/core/Cargo.toml`, the `nostr` 0.45 documentation for `Keys`, `Keys::sign_schnorr_with_aux_rand`, `Event::verify` and `UnsignedEvent`, the `tungstenite` documentation for `accept` and `connect`.

Reference for the protocols (read only, do not copy text): `../musicindex-research/sources/connectors/nostr/nips/01.md` when it exists, and `../musicindex-research/sources/connectors/nostr/blossom/buds/01.md`, `02.md`, `04.md` and `11.md`.

Change: add `crates/core/src/nostr_events.rs`, `crates/core/src/testing/relay.rs` and `crates/core/src/testing/blossom.rs`. Change `lib.rs` and `testing.rs` (the module lines), `Cargo.toml` and `crates/core/Cargo.toml`.

Do not change: the connectors, the pipeline, the API, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Crates

- The root `Cargo.toml` gets `nostr = "0.45"` and `tungstenite = { version = "0.30", features = ["rustls-tls-webpki-roots"] }` in `[workspace.dependencies]`, as `ssh2` is. It also gets `secp256k1 = { version = "0.30", default-features = false, features = ["std"] }`. `crates/core/Cargo.toml` uses the three crates with `.workspace = true`.
- `nostr` has its default features only. Do not enable `nip44`, `nip46`, `rand` or `os-rng` ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 2).
- A test key comes from fixed bytes, for example `SecretKey::from_slice(&[1; 32])`. A test does not need random keys.

### The Signature

- `nostr` 0.45 signs with its default features only through `Keys::sign_schnorr_with_aux_rand` ([ADR 0051](../adr/0051-the-core-signs-nostr-events-with-secp256k1.md)). `EventBuilder` functions that sign, `Keys::sign_schnorr` and `SignEvent for Keys` need `os-rng`. Do not use them.
- Add the public module `nostr_events` (`crates/core/src/nostr_events.rs`) with one function. Packet 030b adds the other functions to it.
- `pub fn sign_with_keys(unsigned: UnsignedEvent, keys: &Keys) -> Result<Event, ConnectorError>`:
  - It makes a context with `secp256k1::Secp256k1::signing_only()`.
  - It gets 32 bytes with `getrandom::fill`. If it fails, the function gives `ConnectorErrorKind::Io`.
  - It signs `unsigned.compute_id()` with `keys.sign_schnorr_with_aux_rand`, then calls `unsigned.add_signature`. If `add_signature` fails, the function gives `Io`.
  - The error message holds no part of the key.
- Make a test event with `UnsignedEvent::new(public_key, Timestamp::from(t), Kind::Custom(n), tags, content)`. The test relay, the test Blossom server and their tests sign with `sign_with_keys`.

### The Test Relay

- `pub struct TestRelay` with `TestRelay::start() -> TestRelay` and `url() -> String` (`ws://127.0.0.1:<port>`). It runs in threads of the test process, one thread for each connection.
- It reads the NIP-01 messages `EVENT`, `REQ` and `CLOSE`:
  - `EVENT`: the relay checks the ID and the signature with `Event::verify`. It stores the event and answers `["OK", <id>, true, ""]`. A check that fails gives `["OK", <id>, false, "invalid: ..."]`.
  - `REQ`: the relay sends each stored event that matches one of the filters, the newest first, then `["EOSE", <subscription>]`. The filter fields are `ids`, `authors`, `kinds`, `#d`, `since`, `until` and `limit`.
  - `CLOSE`: the relay stops the subscription.
- The storage follows NIP-01. Kinds 0, 3 and 10000 to 19999 keep one event for each public key and kind. Kinds 30000 to 39999 keep one event for each public key, kind and `d` value. A newer `created_at` replaces an older one. With the same `created_at`, the lowest ID stays. An event that is older than the stored one gets `["OK", <id>, false, "duplicate: a newer event exists"]`.
- The switches:
  - `refuse_events(Some(message))`: each `EVENT` gets `OK false` with the message. `None` stops this.
  - `drop_events(true)`: each `EVENT` gets `OK true`, but the relay does not store it. This is a relay that did not get the newest manifest.
  - `insert(event)`: the relay stores an event with no message. This is a write by a different client.
  - `events() -> Vec<Event>`: the stored events.
  - `received() -> usize`: the number of `EVENT` messages that the relay got.

### The Test Blossom Server

- `pub struct TestBlossom` with `TestBlossom::start() -> TestBlossom` and `url() -> String` (`http://127.0.0.1:<port>`). It is an HTTP/1.1 server in threads of the test process, in the form of `FakeS3`.
- The routes:

  | Request | Result |
  | --- | --- |
  | `PUT /upload` | Stores the body with its `Content-Type`. `201` for a new blob, `200` for a blob that exists. The body is a blob descriptor: `url`, `sha256`, `size`, `type` and `uploaded`. The `url` has the extension of the `Content-Type`. |
  | `PUT /mirror` | The body is `{"url": "..."}`. The server gets the blob with `ureq`, checks its SHA-256 and stores it. The result is as for `PUT /upload`. |
  | `HEAD /<sha256>` and `HEAD /<sha256>.<ext>` | `200` with `Content-Length` and `Content-Type`, or `404`. |
  | `GET /<sha256>` and `GET /<sha256>.<ext>` | The bytes, or `404`. |
  | `DELETE /<sha256>` | `200` and the blob goes, or `404`. |

- Each `PUT` and each `DELETE` needs the header `Authorization: Nostr <token>`. The token is the Base64url text of a JSON event, with no padding. The server refuses the request with `401` and an `X-Reason` header when one of these checks fails (BUD-11):
  - The event has kind 24242, a correct ID and a correct signature.
  - The `expiration` tag is in the future.
  - The `t` tag is `upload` for `PUT` and `delete` for `DELETE`.
  - An `x` tag is the SHA-256 of the blob.
  - When the event has `server` tags, one of them is the host name of the request, with no port.
- The switches: `refuse_uploads(true)` gives `403` for each `PUT`. `blobs() -> Vec<String>` gives the SHA-256 of each stored blob. `deletes() -> Vec<String>` gives the SHA-256 of each `DELETE` request that the server got.

## Steps

1. Add the crates and `sign_with_keys`, with a unit test: the signed event passes `Event::verify`, and its ID is the ID of the unsigned event. Make sure that `cargo tree -e features -p studio-core` shows no `nip44` and no `nip46`.
2. Add `TestRelay`, with unit tests that use a `tungstenite` client: the replaceable rule, the addressable rule with `d`, the filters, `refuse_events`, `drop_events` and an event with a bad signature.
3. Add `TestBlossom`, with unit tests that use `ureq`:
   - An upload with a correct token gives `201` and then `200`.
   - A token with a different `x` gives `401`. An expired token gives `401`.
   - A mirror between two servers, and a delete.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No crate other than `nostr`, `tungstenite` and `secp256k1`.
- No `unsafe` code.
- No test sends a request to a host outside the computer.
- A test server stops its threads when it drops, as `FakeS3` does.

## Acceptance Criteria

Mechanical:

- The unit tests of the steps pass.
- `cargo tree -e features -p studio-core` shows no `nip44`, `nip46`, `rand` or `os-rng` feature of `nostr`.
- The four commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
cargo tree -e features -p studio-core | grep -E 'nostr feature' | sort -u
```

## Escalation

Stop and report in these cases:

- `nostr` with its default features cannot make a key from fixed bytes.
- `secp256k1` 0.30 is not the version that `nostr` uses, so `sign_schnorr_with_aux_rand` does not accept its context.
- `tungstenite` 0.30 does not build with Rust 1.99.1.
- The TLS feature of `tungstenite` needs a crate with a license other than MIT, Apache-2.0, ISC, BSD or CC0.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030a-test-relay-and-blossom.md
- docs/adr/0048-the-nsite-and-blossom-connectors.md
- docs/adr/0051-the-core-signs-nostr-events-with-secp256k1.md
- The files that the Files section names.

Goal:
- crates/core/src/testing has TestRelay, a blocking NIP-01 relay, and TestBlossom, a Blossom server with the BUD-11 token checks. The core depends on nostr, tungstenite and secp256k1, and nostr_events::sign_with_keys signs an event.

Constraints:
- Follow the Architecture section of the task file fully.
- No crate other than nostr, tungstenite and secp256k1. nostr has its default features only. No unsafe code.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connectors, the pipeline, the API, the UI.
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
