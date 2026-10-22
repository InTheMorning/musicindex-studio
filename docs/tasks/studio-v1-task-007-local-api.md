# Studio v1 Task 007: Local JSON API

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 4.
Decisions: [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md), [ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0016](../adr/0016-ui-without-a-build-step.md) and [ADR 0017](../adr/0017-local-api-access.md).

## Goal

Give the core functions of the first run as a JSON API in local mode. Serve the UI files from the same binary. The desktop binary runs this API until the Tauri window exists (Phase 10).

## Routes

- `GET /api/v1/status`: the vault state and the counts of connectors and feeds.
- `POST /api/v1/vault`: make the vault. Optional recovery key.
- `POST /api/v1/vault/unlock` and `POST /api/v1/vault/lock`.
- `GET` and `POST /api/v1/connectors`: list and add connectors. The first kind is `local`.
- `GET` and `POST /api/v1/feeds`: list and add feeds.
- `POST /api/v1/feeds/{id}/refresh`: read the live feed through its connector.

The draft, diff and write routes come with the album editor (Phase 5).

## Acceptance Criteria

Mechanical:

- A contract test for each route gives the status code and the JSON shape.
- An `/api/` request without the token gives 401. A request with a `Host` that is not a loopback name gives 403.
- A request body that is not JSON gives 415.
- No `GET` response contains a connector secret, the passphrase or the vault key.
- The UI files load without the token.

Visual: none.

## Test Commands

```sh
cargo test -p studio-api
```
