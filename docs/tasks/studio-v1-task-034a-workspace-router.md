# Studio v1 Task 034a: The Router Of One Workspace

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 4, 5 and 20, and [ADR 0017](../adr/0017-local-api-access.md).
Sequence: the first packet of task 034.

## Goal

The API crate gives the routes of one workspace as a router with no access layer. It also gives a handle that the server uses for the vault. The desktop keeps the same behavior. The local router answers `GET /auth/v1/session` with the local mode.

## Files

Inspect: `crates/api/src/lib.rs` (`local_router`), `crates/api/src/access.rs`, `crates/api/src/ui.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs` (`create_vault`, `unlock_vault`, `lock_vault`, `blocking`), `crates/api/src/tests.rs` (`app_with_index`), `crates/desktop/src/main.rs`.

Change: `crates/api/src/lib.rs`, `crates/api/src/ui.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`.

Do not change: the handlers of the routes, other than to move the vault logic of item 5 below. The desktop binary, other than its call of the new functions.

## Architecture

These choices are fixed. Do not change them.

1. `pub fn workspace_router(workspace: Workspace) -> (Router, WorkspaceHandle)` gives the `/api/v1` router and the `/app/v1` router of `local_router`, with the workers attached, and with no access layer and no UI routes.
2. `pub fn ui_router() -> Router` gives `/` and `/ui/{*path}` with the security headers of `ui.rs`.
3. `local_router(workspace, access)` becomes `ui_router()`, the route `GET /auth/v1/session`, and the router of item 1, under the layer of `access.rs`. Its behavior for each existing route does not change. `GET /auth/v1/session` needs the loopback `Host` and no token. It gives `{"mode": "local"}`.
4. `WorkspaceHandle` is a clone of the shared workspace. It has these public functions. Each one runs on a blocking thread:
   - `vault_state() -> VaultState`.
   - `create_vault(passphrase: Option<&str>, recovery_key: bool) -> Result<Option<String>, ApiError>`. It gives the recovery key when one was asked. With no passphrase, it makes a vault with no wrapper and keeps it in memory only. Packet 034e uses this.
   - `unlock_with_passphrase(&str)`, `unlock_with_key(&str)` and `lock()`.
   - `set_library_root(&Path)`.
5. The routes `create_vault`, `unlock_vault` and `lock_vault` call the same functions of `Workspace` as the handle. No logic is in two places.
6. `OpenVault` gets `create_with_no_wrapper()` in `studio-core`. `Workspace::save_vault` does not write a vault with no wrapper.

## Steps

1. Add `ui_router`, `workspace_router` and `WorkspaceHandle`. Make `local_router` from them.
2. Move the vault logic of the three routes into `Workspace` functions, and call them from the routes and the handle.
3. Add `GET /auth/v1/session` to `local_router`.
4. Add `OpenVault::create_with_no_wrapper` with a unit test, and the save rule of item 6.
5. Add API tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- The UI does not change in this packet.

## Acceptance Criteria

Mechanical:

- Each existing API test passes with no change of its expected result.
- A test: `GET /auth/v1/session` gives `{"mode": "local"}` with a loopback `Host` and no token, and 403 with a different `Host`.
- A test calls `workspace_router` and sends `GET /api/v1/status` with no token and no `Host`. It gets 200. This shows that the router has no access layer.
- A test: the handle creates a vault with a passphrase, locks it and unlocks it. A vault with no wrapper is not written to the disk.
- The four commit checks pass.

Visual: none. The UI does not change.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a route needs the access layer for a reason other than the checks of `access.rs`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034a-workspace-router.md
- docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md
- The files that the Files section names.

Goal:
- Split `local_router` into `ui_router`, `workspace_router` and the access layer, with no change of behavior.
- Add `WorkspaceHandle` with the vault functions, and `GET /auth/v1/session` in local mode.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No UI change.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The handlers, other than the vault logic of Architecture item 5.
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
