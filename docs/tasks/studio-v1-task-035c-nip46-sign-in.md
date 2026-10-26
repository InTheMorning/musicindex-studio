# Studio v1 Task 035c: The NIP-46 Client And NIP-46 Sign-In

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 1, 5, 6 and 12 to 16, and [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md).
Sequence: the third packet of task 035. It depends on task 034.

## Goal

The core has a NIP-46 client. On the server, a user connects a remote signer such as Amber through a `nostrconnect://` link or QR code, or through a pasted `bunker://` address. The user then signs up, signs in and opens the vault through that signer.

## Files

Inspect: `crates/core/src/connector/relay.rs` (`fetch`, `publish`, the connection), `crates/core/src/net.rs` (034b), `crates/core/src/nostr_events.rs`, `crates/core/src/testing/relay.rs` (`TestRelay`), the `nostr` 0.45 modules `nips::nip44` and `nips::nip46`, `crates/server/src/auth.rs` (the Nostr route of 034e), `crates/api/ui/app.js` (the sign-in page and the unlock sheet), `crates/api/ui/account.js`.

Change: `Cargo.toml`, `crates/core/Cargo.toml`, the new file `crates/core/src/nip46.rs`, `crates/core/src/connector/relay.rs`, `crates/core/src/lib.rs`, the new file `crates/core/src/testing/signer.rs`, `crates/core/src/testing.rs`, `crates/api/Cargo.toml`, `crates/server/src/auth.rs`, the new file `crates/server/src/nip46_sessions.rs`, `crates/server/src/tests.rs`, `crates/api/src/lib.rs` (`WorkspaceHandle`), `crates/api/ui/app.js`, `crates/api/ui/account.js`, `crates/api/ui-tests/account.test.mjs`.

Do not change: the signing of a write. Packet 035d does it.

## Architecture

These choices are fixed. Do not change them.

### The Core Client

- The workspace `nostr` dependency gets the features `nip44` and `nip46`. The workspace adds `qrcode` with no default features and the `svg` feature. `studio-api` uses it in `pub fn qr_svg(text: &str) -> String`, so that both shells can show a QR code.
- `relay.rs` gets `pub(crate) fn wait_for(relays, filter, policy, deadline) -> Option<Event>`. It keeps one subscription open on each relay until an event that matches comes, or until the deadline.
- `nip46.rs` has `pub struct RemoteSigner`. It holds the client keys, the relays, the public key of the remote signer and the network policy. The client keys clear their memory when the value goes out of scope.
  - `RemoteSigner::nostrconnect(relays, policy) -> (Pending, String)` makes new client keys and a secret. It gives the `nostrconnect://` address with the permissions `sign_event:27235`, `sign_event:35128`, `sign_event:24242`, `nip44_encrypt` and `nip44_decrypt`, and the name "Music Index Studio".
  - `Pending::wait(deadline) -> Result<RemoteSigner, Nip46Error>` waits for the `connect` response with the same secret.
  - `RemoteSigner::bunker(uri, policy, deadline)` connects with a `bunker://` address.
  - `get_public_key()`, `sign_event(UnsignedEvent)`, `nip44_encrypt(pubkey, text)` and `nip44_decrypt(pubkey, ciphertext)`. Each request has a deadline of 60 seconds. Studio refuses an answer from a different public key, and a signed event with an incorrect signature.
- Each relay of a NIP-46 connection goes through the network policy.
- `testing/signer.rs` has `TestSigner`: a thread that answers NIP-46 requests on a `TestRelay` with one key. It can answer a `nostrconnect` address, and it gives a `bunker://` address.

### The Server

- New option: `--nip46-relay URL`, more than one time. The default is `wss://relay.nsec.app`.
- `nip46_sessions.rs` keeps the pending connections and the connected signers in memory, by a random connection ID. A pending connection expires after 5 minutes. A connected signer moves to the session after the sign-in, and goes away with the session.
- `POST /auth/v1/nip46/connect` with `{}` starts a `nostrconnect` connection. It gives `{"connection", "uri", "qr_svg"}`. With `{"bunker"}`, it connects to that address and gives `{"connection"}`.
- `GET /auth/v1/nip46/{connection}` waits a maximum of 25 seconds. It gives `{"state": "pending"|"connected"|"failed", "pubkey"}`.
- `POST /auth/v1/nip46/{connection}/signin` with `{"invite"}`. The server gets a challenge and makes the event of ADR 0056 item 6. It asks the signer to sign it, and then does the steps of the Nostr route of 034e. For a new account, it also asks the signer for `nip44_encrypt` of the vault key and adds the Nostr wrapper. The vault stays open after a sign-up and is locked after a sign-in.
- `POST /auth/v1/nip46/unlock` needs a session with a connected signer. It gets the wrapper, asks for `nip44_decrypt`, and opens the vault through `unlock_with_key`.
- A relay that the policy refuses gives 422 `address_not_public`. A signer that does not answer gives 504 `signer_timeout`.

### The UI

- `account.js` gets `nip46State(answer)` and `nip46ErrorText(code)`.
- The sign-in page gets "Sign in with Amber or a bunker" with the action role. It shows the QR code, the link "Open in the signer app", and a field for a `bunker://` address. While the state is `pending`, it shows "Waiting for the signer…" with a "Stop" button.
- The unlock sheet gets "Use the remote signer" when the session has a connected signer.
- The QR code is the SVG text of the server, in an `img` with a `data:` address. The UI makes no QR code.

## Steps

1. Add the features, `wait_for`, `nip46.rs` and `TestSigner`, with core tests.
2. Add the server option, `nip46_sessions.rs` and the routes, with server tests.
3. Add the model functions, their Node tests and the UI.
4. Do the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- One new crate: `qrcode`, with the license MIT or Apache-2.0. The `nostr` crate gets two features.
- No test sends a request to a relay outside the computer. The tests use `TestRelay` with a test exception of the network policy.
- No client key and no vault key is in a log or a response.

## Acceptance Criteria

Mechanical:

- A core test: `TestSigner` answers a `nostrconnect` address. The client gets the public key, signs an event, and does `nip44_encrypt` and `nip44_decrypt` of a text.
- A core test refuses an answer from a different public key, and an event with an incorrect signature.
- A server test: a NIP-46 sign-up makes an account with a Nostr wrapper. A sign-out and a NIP-46 sign-in follow, and `nip46/unlock` opens the vault.
- A server test with the policy `Public` and no exception refuses a loopback relay with 422.
- A server test: a signer that does not answer gives 504 `signer_timeout`, with a short deadline.
- The Node tests examine `nip46State` and each code of `nip46ErrorText`.
- The four commit checks pass.

Visual:

- The planning model checks the sign-in page with the QR code and the bunker field, the wait state, and the unlock sheet with "Use the remote signer". Each one in the dark theme, in the light theme and at 390 px. The check uses `TestSigner`.

Human test, in task 037:

- A NIP-46 sign-in with Amber on Android. A NIP-46 sign-in on an iPhone, if a signer works.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the `nip46` feature of `nostr` 0.45 needs an async runtime in the core.
- Stop and report when Amber or nsec.app needs a permission text other than the list above. The human test finds this.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035c-nip46-sign-in.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- docs/adr/0057-the-private-network-guard-of-the-server.md
- The files that the Files section names.

Goal:
- Add the NIP-46 client to the core, with `TestSigner`.
- Add the NIP-46 connection, sign-up, sign-in and unlock to the server, and their controls to the UI.

Constraints:
- Follow the Architecture section of the task file fully.
- One new crate: `qrcode`. The `nostr` crate gets `nip44` and `nip46`.
- No test sends a request outside the computer. No client key and no vault key in a log or a response.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The signing of a write.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
