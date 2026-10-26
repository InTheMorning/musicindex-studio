# Studio v1 Task 030b: The Nostr Events And The Relay Client

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 5, 8, 9 and 14 to 17, [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 3, 12 and 13, [ADR 0044](../adr/0044-the-nsite-signer.md) items 2 and 4.
Sequence: the second packet of task 030. It depends on 030a.

## Goal

`studio-core` makes, parses, signs and checks the Nostr events of nsite and Blossom: the manifest (kind 35128) and the authorization event (kind 24242). A blocking relay client reads and publishes events.

## Files

Inspect: `crates/core/src/lib.rs`, `crates/core/src/connector.rs` (`ConnectorError`, `ConnectorErrorKind`), `crates/core/src/testing/relay.rs`, the `nostr` 0.45 documentation (`Keys`, `SecretKey`, `PublicKey`, `UnsignedEvent`, `Event`, `Filter`, `Tag`, `nips::nip19`).

Reference for the protocols (read only): `../musicindex-research/sources/connectors/nostr/nips/5A.md`, `../musicindex-research/sources/connectors/nostr/blossom/buds/11.md`.

Change: add `crates/core/src/nostr_events.rs` and `crates/core/src/connector/relay.rs`. Change `lib.rs` and `connector.rs` (the module lines only).

Do not change: the pipeline, the API, the UI, the other connectors, the test servers.

## Architecture

These choices are fixed. Do not change them.

### `nostr_events` (public module)

- `parse_secret_key(text) -> Result<nostr::Keys, ConnectorError>` accepts an `nsec` or 64 hexadecimal characters. Other text gives `InvalidConfig`. The error message holds no part of the text.
- `parse_public_key(text) -> Result<nostr::PublicKey, ConnectorError>` accepts an `npub` or 64 hexadecimal characters.
- `base36_public_key(&PublicKey) -> String` gives 50 characters, with `0` at the start when necessary. Test vectors:

  | Public key (hex) | Base36 |
  | --- | --- |
  | `fa984bd7dbb282f07e16e7ae87b26a2a7b9b90b7246a44771f0cf5ae58018f52` | `68ukf094zwffw6sq71wn17fst37v9rcdz4uimbb2s5gor83u4y` |
  | `0000000000000000000000000000000000000000000000000000000000000001` | `00000000000000000000000000000000000000000000000001` |
  | `ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff` | `6dp5qcb22im238nr3wvp0ic7q99w035jmy2iw7i6n43d37jtof` |

- `valid_site_name(d) -> bool`: 1 to 13 characters of `[a-z0-9-]`, with no `-` at the end (NIP-5A).
- `aggregate_hash(paths) -> String`: the SHA-256 of the lines `<sha256> <path>\n`, sorted. Test vector: the paths `/feed.xml` with 64 `a` and `/album/01.mp3` with 64 `b` give `ab4e9f81e87c04126edc5f356e57f628b9fce3d9a615924cad70cb8000902921`.
- `pub struct Manifest { d: String, paths: BTreeMap<String, String>, servers: Vec<String>, title: Option<String> }`. The keys of `paths` are absolute paths, and the values are SHA-256 texts.
  - `Manifest::unsigned(&self, public_key, created_at) -> UnsignedEvent`: kind 35128, content empty. The tags are in this sequence: `d`, `title` (when the manifest has one), one `server` tag for each server, one `path` tag for each path, and `["x", <aggregate>, "aggregate"]`. The `path` tags are in the sequence of the paths.
  - `Manifest::parse(&Event) -> Result<Manifest, ConnectorError>`. A `path` tag with a path that does not start with `/`, or with a value that is not 64 lowercase hexadecimal characters, gives `Io`.
- `pub enum AuthAction { Upload, Delete }` and `auth_event(public_key, action, sha256s, server_domain, created_at) -> UnsignedEvent`: kind 24242. The tags are `["t", "upload"]` or `["t", "delete"]`, one `x` tag for each SHA-256, `["server", <domain>]` and `["expiration", <created_at + 3600>]`. The content is `Upload with Music Index Studio` or `Delete with Music Index Studio`.
- `server_domain(url) -> Result<String, ConnectorError>`: the host name of the URL in lowercase, with no port (BUD-11).
- `auth_header(&Event) -> String`: `Nostr ` and the Base64url text of the JSON event, with no padding.
- `pub struct Signing { pub plan_time: Option<u64>, pub signed: Vec<Event> }`, with `Default`. It is the signing of one write ([ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) item 4).
- `sign(unsigned, keys: Option<&Keys>, signing: &Signing, host_key: &PublicKey) -> Result<Event, ConnectorError>`:
  - With `keys`, the function signs the event. Keys of a different public key give `Unauthorized`.
  - With no `keys`, the function finds the event in `signing.signed` with the ID of the unsigned event. It must have a correct signature and the public key of the host. A missing event gives the new error kind `SignatureMissing`. A different key or a bad signature gives `Unauthorized`.
- `ConnectorErrorKind` gets `SignatureMissing`: "the write needs an event that the user did not sign".

### The Relay Client (`connector/relay.rs`, crate-private)

- `fetch(relay_url, filter, timeout) -> Result<Vec<Event>, ConnectorError>`: connect with `tungstenite`, send `["REQ", <id>, <filter>]`, collect each `EVENT` until `EOSE` or the timeout, send `CLOSE`, and close. The client drops each event with a bad ID or a bad signature.
- `publish(relay_url, &Event, timeout) -> Result<Published, ConnectorError>`: send `["EVENT", <event>]` and wait for `OK` with the ID of the event. `Published` has `accepted: bool` and `message: String`.
- `newest(relays, filter, timeout) -> Reads`: `Reads` has the result of each relay, and the newest event of all relays. The newest is the event with the largest `created_at`. With the same `created_at`, the lowest ID is the newest (NIP-01).
- The timeout is 10 seconds for a real relay. Each function takes it as a parameter, so a test can use a short one.
- A relay that does not connect, or a relay with no answer before the timeout, gives `Unavailable` for that relay. `newest` gives `Unavailable` only when no relay answered.
- `ws://` and `wss://` addresses both work. Other addresses give `InvalidConfig`.

## Steps

1. Add `nostr_events` with unit tests: the test vectors, `valid_site_name`, a manifest that goes through `unsigned`, signing and `parse`, an authorization event, `auth_header`, and each case of `sign`.
2. Add the relay client with tests against `TestRelay`:
   - `fetch` with a filter, and an event with a bad signature that `fetch` drops.
   - `publish` with `OK true` and with `refuse_events`.
   - `newest` over two relays with different events, and a relay address with no server.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. No `unsafe` code. `nostr` keeps its default features.
- No secret key is in an error message or a `Debug` text.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- The test vectors of base36 and of the aggregate hash pass.
- A manifest goes through `unsigned`, signing and `parse` with the same paths, servers and `d`.
- `sign` with no keys and no signed event gives `SignatureMissing`. A signed event of a different key gives `Unauthorized`.
- `newest` gives the event with the largest `created_at` of two test relays, and the lowest ID when the times are the same.
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

- `nostr` 0.45 has no API to make an `UnsignedEvent` with a fixed `created_at` and custom tags.
- `nostr` 0.45 cannot sign an `UnsignedEvent` with `Keys` and the default features.
- The ID of an `UnsignedEvent` is not the ID of the event after the signature.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030b-nostr-events-and-relay-client.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- crates/core/src/nostr_events.rs makes, parses, signs and checks the nsite manifest and the Blossom authorization event. crates/core/src/connector/relay.rs reads and publishes events on a relay with tungstenite.

Constraints:
- Follow the Architecture section of the task file fully, with its test vectors.
- No new crate. No unsafe code. nostr keeps its default features.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The pipeline, the API, the UI, the other connectors, the test servers.
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
