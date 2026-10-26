# Studio v1 Task 035g: Sign-In With The Recovery Key

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 3, 8 and 29, [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 15, 22 and 23, [ADR 0013](../adr/0013-vault-format-and-cryptography.md) item 9 and [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md).
Sequence: the second packet of task 035. It depends on 035a. Packet 035b depends on it.

## Goal

On the server, a user signs in with the recovery key and the email address or the `npub` of the account. The vault opens. The session then has a limit: it can only add a new sign-in method. After the user adds a method, the session continues with no limit.

## Files

Inspect: `crates/core/src/vault.rs` (`open_with_recovery_key`, `set_passphrase`, the recovery wrappers of 035a), `crates/api/src/workspace.rs` (`Workspace`, `WorkspaceHandle`), `crates/api/src/routes.rs` (the vault routes), `crates/api/src/lib.rs`, `crates/server/src/accounts.rs`, `crates/server/src/auth.rs`, `crates/server/src/session.rs`, `crates/server/src/limits.rs`, `crates/server/src/main.rs` (the fixed wrapper of an unknown account), `crates/api/ui/app.js` (the sign-in page and the unlock sheet of 034f), `crates/api/ui/account.js`.

Change: `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/server/src/accounts.rs`, `crates/server/src/auth.rs`, `crates/server/src/session.rs`, `crates/server/src/main.rs`, `crates/server/src/tests.rs`, `crates/api/ui/app.js`, `crates/api/ui/account.js`, `crates/api/ui-tests/account.test.mjs`.

Do not change: the vault format and the wrappers of 035a. The desktop boot flow and the desktop UI.

## Architecture

These choices are fixed. Do not change them.

### The API, In Both Shells

- `Workspace::unlock_with_recovery_key(&str)` opens the vault with `open_with_recovery_key`. `WorkspaceHandle` gets the same function on a blocking thread.
- `POST /api/v1/vault/passphrase` with `{"passphrase"}` needs the open vault. It calls `OpenVault::set_passphrase` and saves the vault. The passphrase rule of ADR 0018 applies, with the same codes as the vault creation. It gives `{"vault": <state>}`.

### The Server

- A migration adds `restricted INTEGER NOT NULL DEFAULT 0` to `sessions`.
- At the start, the server adds a recovery wrapper to its fixed vault of an unknown account, and keeps no recovery key.
- `POST /auth/v1/recovery` with `{"account", "recovery_key"}`. The account is an email address or an `npub`. The server:
  1. Applies the failure counters of 034d. The account key is the email address in lowercase, or the public key as hexadecimal. Other text counts for the client address only.
  2. Takes an Argon2id permit, because a recovery wrapper of the earlier type uses Argon2id.
  3. Finds the account. For an unknown account, an account with no vault file, or an incorrect key, it gives 401 `sign_in_failed` with the message of the email sign-in. For an unknown account, it first opens the fixed vault with the given key, so that the work is the same (ADR 0056 item 8).
  4. Opens the vault through the handle, and makes a session with `restricted = 1`.
  5. Gives `{"csrf", "recovery": true, "vault"}` and the cookie.
- `GET /auth/v1/session` gives `"recovery": true` for a restricted session.
- A restricted session reaches only these routes. Each other route that needs a session gives 403 `method_required`.
  - `GET /api/v1/status`.
  - `POST /api/v1/vault/passphrase`, `POST /api/v1/vault/signer-key` and `POST /api/v1/vault/nostr-wrapper`.
  - `GET /auth/v1/session` and `POST /auth/v1/signout`.
- A response 2xx of `POST /api/v1/vault/passphrase` or `POST /api/v1/vault/nostr-wrapper` removes the limit of that session (`restricted = 0`).
- The Argon2id permit surrounds `POST /api/v1/vault/passphrase`, as it surrounds the unlock.
- The recovery key does not change. A new recovery key is not a part of this packet.

### The UI

- `account.js` gets `recoveryAccount(text)`: `{"kind": "email", "value"}` for an email address in lowercase with no spaces at the start or the end. It gives `{"kind": "npub", "value"}` for text that starts with `npub1`, and `null` for other text. It does no bech32 check. The server does that check.
- `account.js` gets `addMethodChoices(session, hasNip07)`: `"passphrase"` when the account has an email, and `"nip07"` when the account has a Nostr key and the browser has `window.nostr`.
- `authErrorText` gets a text for `method_required`.
- The sign-in page gets the link "Sign in with the recovery key". The recovery page has the account field, the recovery key field and "Sign in" with the primary role.
- After a recovery sign-in, the UI shows the page "Add a sign-in method" before the shell. It shows the methods of `addMethodChoices`:
  - "passphrase": a new passphrase with the strength meter of the vault, and "Save the passphrase" with the primary role.
  - "nip07": "Use the signer extension" with the action role. It calls `POST /api/v1/vault/signer-key`, asks `nip44.encrypt`, and sends the wrapper to `POST /api/v1/vault/nostr-wrapper`.
  - Packet 035b adds a passkey method.
- After the method is added, the UI loads the session again and opens the shell.

## Steps

1. Add the workspace function, the handle function and the passphrase route, with API tests.
2. Add the migration, the recovery route and the limit of a restricted session, with server tests.
3. Add the model functions with Node tests, the recovery page and the page "Add a sign-in method".
4. Do the visual check of the acceptance criteria against `studio-server` with a loopback origin.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. The server decodes an `npub` with the `nostr` crate.
- No recovery key text, passphrase or vault key is in a log, an error message or a `Debug` output.
- The UI gives no advice on how to make a passphrase (ADR 0018).

## Acceptance Criteria

Mechanical:

- A server test: an email account with a recovery key signs in with the recovery key. The vault is open. `GET /api/v1/feeds` gives 403 `method_required`. `POST /api/v1/vault/passphrase` with a new passphrase gives 200. `GET /api/v1/feeds` then gives 200. After sign-out, the new passphrase signs in, and the earlier passphrase does not.
- A server test: a Nostr account with a recovery key signs in with its `npub`. A new Nostr wrapper removes the limit of the session.
- A server test: an unknown account, an incorrect recovery key and an account with no vault file give the same status, code and message.
- A server test: after 10 failed recovery sign-ins for one account, the next one gets 429 `too_many_attempts`, also with the correct key.
- A server test: a restricted session gets 403 `method_required` for `POST /auth/v1/account/delete`.
- An API test in local mode: `POST /api/v1/vault/passphrase` replaces the passphrase. A weak passphrase gives the code of the vault creation.
- The Node tests examine `recoveryAccount`, `addMethodChoices` and the text of `method_required`.
- The desktop API tests pass with no change of their expected results.
- The four commit checks pass.

Visual:

- The planning model checks the recovery page and the page "Add a sign-in method" for an email account. Each one in the dark theme, in the light theme and at 390 px with touch.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a route that a restricted session needs is not in the list of the Architecture section.
- Stop and report when the fixed vault cannot give the same work for an unknown account.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035g-recovery-sign-in.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md
- The files that the Files section names.

Goal:
- Add the sign-in with the recovery key on the server.
- Limit the restricted session until the user adds a new sign-in method.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No change of the vault format.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The vault format and the wrappers of 035a.
- The desktop boot flow and the desktop UI.
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
