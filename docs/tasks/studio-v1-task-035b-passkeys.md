# Studio v1 Task 035b: Passkeys With PRF

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 2, 3, 5 and 20 to 27, and [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md).
Sequence: the third packet of task 035. It depends on 035a and 035g.

## Goal

A signed-in user with an open vault adds a passkey. The user then signs in and opens the vault with that passkey, on each device where the passkey manager syncs it. The server checks each assertion itself, with ES256 only.

## Files

Inspect: `crates/core/src/vault.rs` (the passkey wrapper of 035a), `crates/server/src/accounts.rs`, `crates/server/src/auth.rs`, `crates/server/src/session.rs`, `crates/api/ui/app.js` (the sign-in page, the unlock sheet, the Account panel of 034f), `crates/api/ui/account.js`.

Change: `Cargo.toml`, `crates/server/Cargo.toml`, the new file `crates/server/src/webauthn.rs`, `crates/server/src/accounts.rs`, `crates/server/src/auth.rs`, `crates/server/src/tests.rs`, `crates/api/src/lib.rs` (`WorkspaceHandle`), `crates/api/ui/app.js`, `crates/api/ui/account.js`, `crates/api/ui-tests/account.test.mjs`.

Do not change: the other sign-in methods.

## Architecture

These choices are fixed. Do not change them.

### The Verifier

- The workspace adds `p256` with the features `ecdsa` and `pkcs8`. `studio-server` uses it, with `sha2`.
- `webauthn.rs` has two functions. Each one checks `clientDataJSON`: the `type`, the `challenge` (base64url of the challenge bytes), and the `origin` equal to the public origin. Each one checks the authenticator data: the SHA-256 of the relying party ID, the flags UP and UV.
  - `verify_registration(client_data, authenticator_data, public_key_spki, algorithm)`: the `type` is `webauthn.create`, and the algorithm is -7 (ES256). It reads the credential ID from the attested credential data. It gives the credential ID, the public key and the counter.
  - `verify_assertion(client_data, authenticator_data, signature, public_key, stored_counter)`: the `type` is `webauthn.get`. The ECDSA signature in DER form covers the authenticator data and the SHA-256 of `clientDataJSON`. A counter that is not zero must be larger than the stored counter.
- The relying party ID is the host of the public origin.

### The Server

- A migration adds `account_passkeys (credential_id TEXT PRIMARY KEY, account_id INTEGER NOT NULL, public_key BLOB NOT NULL, counter INTEGER NOT NULL, prf_salt TEXT NOT NULL, name TEXT NOT NULL, created_at INTEGER NOT NULL)` and a column `user_handle` of 16 random bytes for each account.
- A restricted session of [035g](studio-v1-task-035g-recovery-sign-in.md) reaches `POST /auth/v1/passkeys/options` and `POST /auth/v1/passkeys`. A passkey that the server adds removes the limit of that session. The page "Add a sign-in method" of 035g gets the passkey method.
- `POST /auth/v1/passkeys/options` needs a session and an open vault. It gives `{"challenge", "assertion_challenge", "rp_id", "user_handle", "user_name", "prf_salt", "exclude"}`. The registration uses `challenge`, and the assertion after it uses `assertion_challenge`. The PRF salt is the fixed salt of ADR 0056 item 21. The server keeps the two challenges.
- `POST /auth/v1/passkeys` with `{"name", "registration": {...}, "assertion": {...}, "prf_output"}`. The server checks the registration and then the assertion of the same credential. It refuses a missing PRF output with 422 `passkey_no_prf`. It adds the passkey wrapper through the handle and keeps the row.
- `GET /auth/v1/passkeys` lists the name, the creation time and the credential ID. `DELETE /auth/v1/passkeys/{credential_id}` needs the open vault. It removes the wrapper and the row. It refuses the last method of the account (ADR 0056 item 3).
- `POST /auth/v1/passkey/options` gives a challenge for a sign-in with no session, or for an unlock with a session.
- `POST /auth/v1/passkey` with `{"assertion", "prf_output"}`. The server finds the account from the credential ID, checks the assertion, opens the vault with `open_with_passkey`, and updates the counter. With no session, it makes a session and sets the cookie. The failure counters of 034d apply.
- The PRF output goes from the browser to the server in the JSON body. The server clears it from memory after use.

### The UI

- `account.js` gets `toBase64Url`, `fromBase64Url`, `creationOptions(options)` and `requestOptions(options)`. The two option functions give the `publicKey` objects for `navigator.credentials`. They set `residentKey: "required"` and `userVerification: "required"`. They set `pubKeyCredParams` with the algorithm -7 only, and the `prf` extension with `eval.first` set to the salt.
- To add a passkey, the UI calls `navigator.credentials.create`, then `navigator.credentials.get` with the PRF extension, and sends both results. It uses `response.getPublicKey()` and `response.getAuthenticatorData()` of the registration.
- The sign-in page gets "Sign in with a passkey" with the action role. The unlock sheet gets "Use a passkey". The Account panel lists the passkeys, with "Add a passkey" (action role) and "Remove" (destructive role).
- A browser with no `PublicKeyCredential` does not show the passkey controls. The error `passkey_no_prf` shows the text "This passkey manager cannot open the vault. Use iCloud Keychain or Google Password Manager, or a different method."

## Steps

1. Add `p256` and `webauthn.rs`, with unit tests that use a `p256` signing key to make test data.
2. Add the migration and the routes, with server tests.
3. Add the model functions and their Node tests.
4. Add the UI controls.
5. Do the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- One new crate: `p256`, a RustCrypto crate with the license MIT or Apache-2.0. No CBOR crate.
- No PRF output is in a log, a database row or a response.

## Acceptance Criteria

Mechanical:

- Unit tests accept a correct registration and a correct assertion. They refuse a different origin, a different challenge and a different relying party ID. They refuse a missing UV flag, an incorrect signature, an algorithm other than -7, and a counter that does not increase.
- A server test adds a passkey with test data, signs out, signs in with the passkey, and finds the vault open.
- A server test refuses a registration with no PRF output with 422 `passkey_no_prf`, and refuses the removal of the last method.
- The Node tests examine the base64url functions and the two option functions.
- The four commit checks pass.

Visual:

- The planning model checks the Account panel with two passkeys, the sign-in page with "Sign in with a passkey", and the unlock sheet with "Use a passkey". Each one in the dark theme, in the light theme and at 390 px. Headless Chromium has a virtual authenticator in the DevTools protocol (`WebAuthn.addVirtualAuthenticator`), which the check can use for a registration.

Human test, in task 037:

- A passkey on an iPhone and on an Android phone, each with a second synced device.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a browser gives the registration public key in a form other than SPKI.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035b-passkeys.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- The files that the Files section names.

Goal:
- Add the ES256 WebAuthn verifier, the passkey routes and the passkey sign-in and unlock.
- Add the passkey controls of the UI and their model functions.

Constraints:
- Follow the Architecture section of the task file fully.
- One new crate: `p256`. No CBOR crate.
- No PRF output in a log, a database row or a response.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The other sign-in methods.
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
