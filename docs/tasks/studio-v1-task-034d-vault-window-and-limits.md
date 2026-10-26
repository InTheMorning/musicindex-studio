# Studio v1 Task 034d: The Vault Window, The Limits And The Server Rules

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 3, 9, 14 to 17 and 21 to 25, and [ADR 0013](../adr/0013-vault-format-and-cryptography.md) item 15.
Sequence: the fourth packet of task 034. It depends on 034c.

## Goal

The server locks the vault of a user after 10 minutes with no use of a secret. It limits the storage of each user, the Argon2id derivations and the failed sign-ins. It refuses the functions of the desktop only. A user can delete the account.

## Files

Inspect: `crates/api/src/workspace.rs` (`hosts_with`, `vault`), `crates/api/src/routes.rs` (`add_connector` with `Local`, `put_library`, `upload_file`), `crates/api/src/downloads.rs`, `crates/core/src/download.rs` (the free space check), the server files of 034c.

Change: `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/downloads.rs`, `crates/api/src/lib.rs` (`WorkspaceHandle`), `crates/api/src/tests.rs`, `crates/server/src/*.rs`.

Do not change: the vault format, the core pipeline.

## Architecture

These choices are fixed. Do not change them.

### Server Rules In The Workspace

- `Workspace` gets `pub fn with_server_rules(self, rules: ServerRules) -> Self`. `ServerRules` has `storage_limit: u64` and the fixed library folder. The desktop does not call it. With server rules:
  - `add_connector` with a `Local` host gives 403 `not_on_server`.
  - `put_library` with a folder other than the fixed folder gives 403 `not_on_server`.
  - The upload route and each download check the size of the user folder before they start, and after each 8 MiB of written bytes. Over the limit, they stop, remove the partial file and give 507 `storage_full`. A download records the same code as its error.
- `Workspace` records the time of the last use of the vault. `hosts_with` sets it when it gives a host a secret. The vault routes set it when they open the vault.
- `WorkspaceHandle` gets `lock_if_idle(limit: Duration) -> bool`, `storage_used() -> u64` and `delete_folder()`.

### The Server

- New options: `--session-idle-days` (default 7), `--session-max-days` (default 30), `--vault-idle-minutes` (default 10), `--storage-limit-gib` (default 10), `--argon2-concurrency` (default 2).
- A task runs each 30 seconds. It calls `lock_if_idle` for each open user. It also closes the user (the router and the handle) when the user has no session, no open vault, no waiting announcement and no queued download.
- Sign-out locks the vault when it ends the last session of the user.
- A semaphore with the `--argon2-concurrency` permits surrounds each sign-up, sign-in with a passphrase, and unlock with a passphrase. A request that waits more than 10 seconds for a permit gets 503 `busy`.
- An in-memory counter keeps the failed sign-ins of each account and of each client address for one hour. After 10 for an account or 30 for an address, a sign-in gets 429 `too_many_attempts`, with no Argon2id derivation. The client address comes from the header of `--client-address-header`. With no option, the server uses the address of the connection.

### Account Deletion

- `POST /auth/v1/account/delete` with `{"confirm": "<email or npub>"}` needs a session and an open vault. It removes the session rows, the email row and the account row, closes the user, and removes the user folder. It gives 204 and a cookie with `Max-Age=0`.

## Steps

1. Add `ServerRules`, the refusals and the storage checks to `studio-api`, with API tests.
2. Add the time of the last vault use and the handle functions.
3. Add the options, the idle task, the semaphore and the counters to the server.
4. Add account deletion.
5. Add the server tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- The desktop behavior does not change. The desktop tests pass with no change.

## Acceptance Criteria

Mechanical:

- An API test with server rules refuses a `Local` host and a different library folder with 403 `not_on_server`.
- An API test with a storage limit of 1 MiB refuses an upload of 2 MiB with 507 `storage_full`, and leaves no partial file.
- A server test with an idle limit of 1 second: the vault is open after sign-in, and after 2 seconds a route that needs a secret gives 423 `vault_locked`.
- A server test: with `--argon2-concurrency 1` and a held permit, a sign-in gets 503 `busy`. The test can set a short wait.
- A server test: after 10 failed sign-ins for one account, the eleventh gets 429 with the correct passphrase too.
- A server test: account deletion removes the user folder and the rows, and the cookie then gets 401.
- The four commit checks pass.

Visual: none.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the size check of a folder of 10 000 files takes more than one second.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034d-vault-window-and-limits.md
- docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md
- The files that the Files section names.

Goal:
- Add the server rules of the workspace: no local folder host, a fixed library folder and a storage limit.
- Lock an idle vault, limit the Argon2id derivations and the failed sign-ins, and add account deletion.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. The desktop behavior does not change.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The vault format and the pipeline.
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
