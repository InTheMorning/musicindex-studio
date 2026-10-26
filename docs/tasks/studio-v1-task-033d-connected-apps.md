# Studio v1 Task 033d: Connected Apps

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 7 to 10, and [ADR 0017](../adr/0017-local-api-access.md).
Sequence: the fourth packet of task 033. It depends on 033a.

## Goal

The user adds and removes connected apps. Each app gets a token one time. The `/app/v1/` routes accept only an app token, and the `/api/` routes do not accept one.

## Files

Inspect: `crates/api/src/access.rs` (the access token and the `Host` check), `crates/api/src/lib.rs` (`local_router`), `crates/core/src/collection.rs` (`MIGRATIONS`), `crates/core/src/vault.rs` (the random source of the vault), `crates/api/src/tests.rs`, `crates/api/ui/app.js` (`Hosts`, `LibraryPanel`, `AnnouncePanel`).

Change: `crates/core/src/collection.rs`, the new file `crates/core/src/apps.rs`, `crates/core/src/lib.rs`, `crates/api/src/access.rs`, `crates/api/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/api/ui/app.js`.

Do not change: the show grants (033e), the pipeline.

## Architecture

These choices are fixed. Do not change them.

### The Core

- A new migration adds `connected_apps (id INTEGER PRIMARY KEY, name TEXT NOT NULL, token_hash TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL)`.
- `apps.rs` has:
  - `add_app(collection, name) -> Result<(ConnectedApp, String), …>`. It gives the row and the token. The token is 32 bytes from `getrandom`, as 64 hexadecimal characters. The row keeps the SHA-256 of the token as hexadecimal.
  - `remove_app(collection, id)`.
  - `apps(collection) -> Vec<ConnectedApp>`, with `id`, `name` and `created_at`. A `ConnectedApp` never has the token or its hash in `Serialize` or `Debug`.
  - `app_for_token(collection, token) -> Option<ConnectedApp>`. It compares the SHA-256 of the token with the kept hashes.
- An empty name gives an error. The same name two times is permitted.

### The API

- `GET /api/v1/apps`, `POST /api/v1/apps` with `{"name"}` and `DELETE /api/v1/apps/{id}`. The POST response has the row and the `token`. No other response has the token.
- A second router is nested at `/app/v1`. Its layer checks the loopback `Host` as `access.rs` does, then the `Authorization: Bearer` token with `app_for_token`. A missing or unknown token gives 401 `app_unauthorized`. The layer puts the `ConnectedApp` into the request extensions.
- The `/api/` layer refuses an app token as it refuses each token that is not the access token. The `/app/v1/` layer refuses the access token.
- This packet adds one route for the test of the layer: `GET /app/v1/whoami`, which gives `{"id", "name"}` of the app. Packet 033e adds the show routes.

### The UI

- The Hosts page gets a "Connected apps" panel after the Announcements panel. It lists each app with its name and "Remove" with the destructive role. "Connect an app" with the action role opens a name field and "Make the token" with the primary role.
- After "Make the token", the panel shows the token one time in a `mono` field with "Copy". It shows the hint "Give this token to the app now. Studio does not show it again." A reload of the page does not show it.

## Steps

1. Add the migration and `apps.rs`, with unit tests.
2. Add the `/api/v1/apps` routes, with API tests.
3. Add the `/app/v1` router, its layer and `whoami`, with API tests of each refusal.
4. Add the Connected apps panel.
5. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. The core already has SHA-256 and `getrandom`.
- No token is in a log, an error message, the collection database or a response other than the POST response.

## Acceptance Criteria

Mechanical:

- A unit test adds an app, finds it with its token, and finds nothing with a different token. The database file does not contain the token text.
- API tests: `/app/v1/whoami` gives 401 with the UI access token, with no token and with a removed app's token, and 200 with the app token. `/api/v1/status` gives 401 with an app token. A non-loopback `Host` gives the refusal of `access.rs` on `/app/v1/`.
- The four commit checks pass.

Visual:

- The planning model checks the Connected apps panel with an app, and with the token after "Make the token". Each one in the dark theme, in the light theme and at 390 px.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the access layer of `/api/` cannot stay unchanged for its own routes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033d-connected-apps.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- Add the `connected_apps` table with token hashes, and the `/api/v1/apps` routes.
- Add the `/app/v1` router with its token layer and `whoami`.
- Add the Connected apps panel on the Hosts page.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- No token in a log, an error message, the database or a response other than the POST response.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The show grants, the pipeline.
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
