# Studio v1 Task 036b: Connected Apps On The Server

Parent: [task 036](studio-v1-task-036-grant-vaults.md). Decisions: [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md) items 16 to 19, and [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 19 and 23.
Sequence: the second packet of task 036. It depends on 036a.

## Goal

The server serves `/app/v1/` on the public origin. An app token finds its user and goes to the workspace of that user. The server limits the requests of each app.

## Files

Inspect: `crates/api/src/lib.rs` (`workspace_router`), `crates/api/src/routes.rs` (`app_check`, `add_app`, `remove_app`), `crates/core/src/apps.rs`, the server files of tasks 034 and 035, especially `session.rs` and `users.rs`.

Change: `crates/server/src/accounts.rs`, `crates/server/src/session.rs`, the new file `crates/server/src/apps.rs`, `crates/server/src/main.rs`, `crates/server/src/tests.rs`, `crates/api/src/lib.rs` (`WorkspaceHandle`), `crates/api/src/routes.rs`.

Do not change: the `/app/v1/` routes and their contract (ADR 0054).

## Architecture

These choices are fixed. Do not change them.

- A migration of `server.sqlite` adds `app_tokens (token_hash TEXT PRIMARY KEY, account_id INTEGER NOT NULL, app_id INTEGER NOT NULL)`.
- `WorkspaceHandle` gets a hook for the app list: the server gives a function that the workspace calls after `add_app` and `remove_app`, with the token hash and the app ID. The server adds or removes the row. The desktop gives no function.
- At the start, the server makes the rows again from the `connected_apps` table of each user folder. Thus a lost row does not stop an app.
- The `/app/v1/` layer of the server:
  1. Applies the `Host` rule of the outer layer. It does not apply the `Origin` rule, because an app is not a browser.
  2. Refuses a request with the session cookie and no `Authorization` header with 401 `app_unauthorized`.
  3. Finds the account from the SHA-256 of the bearer token. An unknown token gives 401 `app_unauthorized` and counts against the client address.
  4. Sends the request to the router of that user with `oneshot`. The `app_check` of the workspace then checks the token again.
- Limits: 60 requests for each app in each minute, and 20 unknown tokens for each client address in each hour. After that, 429 `too_many_attempts`.
- The `/api/` layer refuses a request with a bearer token that is an app token, as the session layer refuses each request with no session.

## Steps

1. Add the migration, the hook and the rows at the start.
2. Add the `/app/v1/` layer with the limits.
3. Add the server tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No app token is in a log or an error message.

## Acceptance Criteria

Mechanical:

- A server test: a user adds an app and makes a grant, locks the vault and signs out. A test client sends "Go live" to `/app/v1/` on the public origin with the app token. The hosted feed has the change.
- A server test: the session cookie on `/app/v1/whoami` gives 401. The app token on `/api/v1/status` gives 401. A different `Host` gives 403.
- A server test: the 61st request of an app in one minute gives 429. The 21st unknown token of an address in one hour gives 429.
- A server test: the app of one user cannot list the shows of a different user.
- A server test: after a removal of `app_tokens` rows and a restart, the app still works.
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

- Stop and report when the hook needs a change of the `/api/v1/apps` routes other than the call of the hook.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-036b-apps-on-the-server.md
- docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md
- The files that the Files section names.

Goal:
- Add the index of app tokens to `server.sqlite`.
- Add the `/app/v1/` layer that sends each app request to the workspace of its user.
- Add the request limits of the apps.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No app token in a log or an error message.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The `/app/v1/` routes and their contract.
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
