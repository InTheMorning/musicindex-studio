# Studio v1 Task 034e: NIP-07 Sign-In And The Nostr Wrapper

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 1, 2, 5, 6 and 9 to 11, [ADR 0004](../adr/0004-server-sign-in-with-nostr-or-email.md) item 2, and [ADR 0013](../adr/0013-vault-format-and-cryptography.md) item 8.
Sequence: the fifth packet of task 034. It depends on 034c.

## Goal

A user signs up and signs in with a NIP-07 signer. Each signed event answers a single-use challenge, bound to the URL and the method. The vault of a Nostr account has a Nostr wrapper. The signer decrypts it to open the vault.

## Files

Inspect: `crates/core/src/vault.rs` (`key_for_signer`, `add_nostr_wrapper`, `open_with_key`), `crates/core/src/vault/file.rs` (`nostr_ciphertext`), `crates/core/src/nostr_events.rs` (the verification of an event), the server files of 034c and 034d, `crates/api/src/routes.rs` (the vault routes), `crates/api/src/lib.rs`.

Change: `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/server/src/accounts.rs`, `crates/server/src/auth.rs`, `crates/server/src/tests.rs`.

Do not change: the vault format.

## Architecture

These choices are fixed. Do not change them.

### The API, In Both Shells

- `GET /api/v1/vault/nostr-wrapper` gives `{"pubkey", "ciphertext"}` of the first Nostr wrapper, or 404 `no_nostr_wrapper`.
- `POST /api/v1/vault/unlock-key` with `{"key"}` opens the vault with `open_with_key`. A key that does not open it gives 401 `wrong_key`.
- `POST /api/v1/vault/nostr-wrapper` with `{"pubkey", "ciphertext"}` adds a Nostr wrapper and saves the vault. It needs the open vault.
- `POST /api/v1/vault/signer-key` gives `{"key"}` from `key_for_signer`, so that a signer can encrypt it. It needs the open vault. It is the only route that gives the vault key.

### The Server

- A migration adds `challenges (challenge TEXT PRIMARY KEY, expires_at INTEGER NOT NULL)` and `account_nostr (account_id INTEGER NOT NULL, pubkey TEXT NOT NULL UNIQUE)`.
- `POST /auth/v1/challenge` gives `{"challenge", "expires_at"}`: 32 random bytes as hexadecimal, valid for 5 minutes. The server removes expired rows at each call.
- `POST /auth/v1/nostr` with `{"event", "invite"}`. The server:
  1. Checks that the event has kind 27235, a correct ID and a correct signature. The `u` tag is `<public origin>/auth/v1/nostr`, and the `method` tag is `POST`. The `created_at` is within 60 seconds of the server time.
  2. Removes the challenge of the `challenge` tag with `DELETE ... RETURNING`. A missing or expired challenge gives 401 `challenge_invalid`.
  3. With a known public key, it makes a session. With an unknown key, it applies the sign-up rule of 034c and makes the account and the folder. It makes a vault with no wrapper through `create_vault(None, recovery_key)`. The vault stays open in memory.
  4. It gives `{"csrf", "new_account", "recovery_key"}` and the cookie.
- The failure counters of 034d apply. The account of a Nostr sign-in is the public key.
- A new Nostr account must add the Nostr wrapper in the same visit. The UI of 034f calls `POST /api/v1/vault/signer-key`, asks the signer for `nip44.encrypt`, and calls `POST /api/v1/vault/nostr-wrapper`. Until then, the vault file does not exist. A sign-in to an account with no vault file makes a new vault with no wrapper again. The account has no secret yet, so nothing is lost.

## Steps

1. Add the four vault routes to `studio-api`, with API tests.
2. Add the migration, the challenge route and the Nostr route to the server.
3. Add the server tests with keys of the `nostr` crate.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. The `nostr` crate verifies the events.
- No vault key is in a log. Only `POST /api/v1/vault/signer-key` gives it.

## Acceptance Criteria

Mechanical:

- A server test signs up with a signed event and adds a Nostr wrapper with a test NIP-44 ciphertext. It signs out, signs in with a new event, and opens the vault with `unlock-key`.
- Server tests refuse with 401 `challenge_invalid`: a used challenge, an expired challenge and an unknown challenge. They refuse an event with a different `u` tag, a different `method` tag, a `created_at` 120 seconds old, and an incorrect signature.
- An API test: `unlock-key` with an incorrect key gives 401 and the vault stays locked.
- A test: an account with no vault file gets a new vault with no wrapper at its next sign-in.
- The four commit checks pass.

Visual: none. Packet 034f adds the pages.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the `nostr` crate cannot verify an event of kind 27235 with its default features.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034e-nip07-sign-in.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- The files that the Files section names.

Goal:
- Add the vault routes for the Nostr wrapper and the unlock with the vault key, in both shells.
- Add the challenges and the NIP-07 sign-up and sign-in to the server.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No vault key in a log.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The vault format.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
