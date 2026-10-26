# Studio v1 Task 034c: Accounts, User Folders And Sessions

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 1 to 8, 10 to 13 and 18 to 20, and [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 4, 7 and 8.
Sequence: the third packet of task 034. It depends on 034b.

## Goal

`studio-server` runs. A user signs up with an email, a passphrase and an invite code, and gets a user folder and a session. The user signs in and out. Each `/api/` request goes to the workspace of the user of the session.

## Files

Inspect: `crates/api/src/lib.rs` (`workspace_router`, `ui_router`, `WorkspaceHandle` from 034a), `crates/core/src/net.rs` (034b), `crates/core/src/collection.rs` (the migration method), `crates/desktop/src/main.rs` (the options), `crates/api/src/tests.rs` (the request helpers).

Change: `crates/server/Cargo.toml`, `crates/server/src/main.rs`, the new files `crates/server/src/config.rs`, `crates/server/src/accounts.rs`, `crates/server/src/users.rs`, `crates/server/src/session.rs`, `crates/server/src/auth.rs`, `crates/server/src/tests.rs`.

Do not change: `studio-core` and `studio-api`, other than a missing public item that the server needs. Report each such item.

## Architecture

These choices are fixed. Do not change them.

### The Options

- `studio-server serve` with these options: `--data-dir PATH` (required), `--listen ADDR` (default `127.0.0.1:8080`), `--public-origin URL` (required), `--signup open|invite` (default `invite`), `--client-address-header NAME` (no default), `--musicindex URL`, `--podcastindex-hub URL`.
- The public origin must be `https`. A loopback origin, such as `http://127.0.0.1:8080`, can use `http`.
- `studio-server invite --data-dir PATH` prints one new invite code: 16 random bytes as hexadecimal. `server.sqlite` keeps its SHA-256.

### The Database

- `accounts.rs` opens `<data>/server.sqlite` with migrations, as `collection.rs` does. The tables:
  - `accounts (id INTEGER PRIMARY KEY, folder TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL)`.
  - `account_emails (account_id INTEGER NOT NULL UNIQUE, email TEXT NOT NULL UNIQUE)`.
  - `sessions (token_hash TEXT PRIMARY KEY, account_id INTEGER NOT NULL, created_at INTEGER NOT NULL, last_seen INTEGER NOT NULL)`.
  - `invites (code_hash TEXT PRIMARY KEY, created_at INTEGER NOT NULL, used_at INTEGER)`.
- An email address is kept in lowercase with no spaces at the start or the end.

### The User Folders

- `users.rs` has `Users`. It keeps one open user for each account: the router and the handle of `workspace_router`. It opens a user at the first request, with `Workspace::open(<data>/users/<folder>)`, `with_net_policy(NetPolicy::Public(PublicRules::default()))`, the musicindex address and the hub of the options. It then calls `record_all_remote_files`, `resume_downloads` and `resume_announcements`, as the desktop does.
- At the start, `Users` opens each user whose collection has a waiting announcement or a queued download.
- A new account gets a folder name of 16 random bytes as hexadecimal. The server calls `set_library_root(<user folder>/library)` for it.
- A test build of `Users` can take the `PublicRules` with test exceptions.

### Sessions And The Layers

- A session token is 32 random bytes as hexadecimal. The table keeps its SHA-256. The CSRF token is the SHA-256 of `studio-csrf:` and the session token, as hexadecimal. The server keeps no CSRF token.
- The cookie is `__Host-studio-session=<token>; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=2592000`.
- A session ends after 7 days with no request or after 30 days in total. A request after that gets 401 `session_ended`, and the server removes the row.
- The outer layer refuses a request with a `Host` that is not the host and port of the public origin, with 403 `host_not_allowed`. It refuses a request with a method other than `GET` and `HEAD` with an `Origin` that is not the public origin, with 403 `origin_not_allowed`.
- The `/api/` layer finds the session from the cookie. It refuses a missing session with 401 `session_required`, and a missing or incorrect `X-Studio-CSRF` header with 403 `csrf_required`. Then it sends the request to the router of the user with `tower::ServiceExt::oneshot`. It updates `last_seen` at most one time each minute.
- `/app/v1/` gets 404 on the server in this packet. Task 036 adds it.

### The Auth Routes

- `GET /auth/v1/session` gives `{"mode": "server", "signup": "open"|"invite", "signed_in": false}` with no session. With a session, it gives `{"mode": "server", "signed_in": true, "csrf", "email", "vault"}`.
- `POST /auth/v1/signup/email` with `{"email", "passphrase", "invite", "recovery_key"}`. In the mode `invite`, it checks the invite. It refuses a used email with 409 `email_in_use`. It makes the account and the folder, creates the vault with the passphrase through the handle, and sets the cookie. It gives `{"csrf", "recovery_key"}`. The passphrase rule of [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md) applies, through the vault.
- `POST /auth/v1/signin/email` with `{"email", "passphrase"}`. It opens the vault through the handle and sets the cookie. An unknown email address and an incorrect passphrase each give 401 `sign_in_failed` with the same message. For an unknown address, the server does one Argon2id derivation on a fixed wrapper that it makes at the start.
- `POST /auth/v1/signout` removes the session row and sends a cookie with `Max-Age=0`.
- Each body is JSON. A different content type gets 415.

## Steps

1. Add the options, `config.rs` and the `invite` command.
2. Add `accounts.rs` with its migrations and unit tests.
3. Add `users.rs`.
4. Add `session.rs` with the two layers and `auth.rs` with the routes.
5. Serve `ui_router()`, `/auth/v1/` and the `/api/` layer in `main.rs`.
6. Add the server tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- New dependencies of `studio-server`: `axum`, `tokio`, `rusqlite`, `getrandom`, `serde`, `serde_json`, `sha2` and `tower`, at the versions that the workspace already uses. No crate that the workspace does not already use.
- No session token, CSRF token, passphrase or vault key is in a log or an error message.

## Acceptance Criteria

Mechanical:

- A server test signs up two users. The first user adds a feed. The second user lists the feeds and gets none. The second user gets 404 for the feed ID of the first user.
- A test: `/api/v1/status` gives 401 with no cookie, and 403 with the cookie and no `X-Studio-CSRF` header. A different `Host` gets 403, and a POST with a different `Origin` gets 403.
- A test: sign-up with no invite gives 403 `invite_required` in the mode `invite`, and a used invite gives the same. Sign-up works in the mode `open`.
- A test: an unknown email address and an incorrect passphrase give the same status, code and message.
- A test: after sign-out, the cookie gets 401. A session with a `last_seen` older than 7 days gets 401 `session_ended`.
- A test reads the bytes of `server.sqlite` and finds no session token, CSRF token or invite code.
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

- Stop and report when a route of `studio-api` cannot work behind the `/api/` layer with no change.
- Stop and report when the server needs a change of a handler in `studio-api`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034c-accounts-and-sessions.md
- docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- The files that the Files section names.

Goal:
- Make `studio-server serve` and `studio-server invite`.
- Add `server.sqlite`, the user folders, and the email sign-up and sign-in.
- Add the sessions with the cookie and the CSRF header.
- Send each `/api/` request to the workspace of its user.

Constraints:
- Follow the Architecture section of the task file fully.
- No crate that the workspace does not already use.
- No token, passphrase or vault key in a log or an error message.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `studio-core` and `studio-api`, other than a missing public item. Report each one.
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
