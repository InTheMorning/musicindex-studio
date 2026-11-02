# Studio v1 Task 073: Connected Apps With Key Pairs

Decision: [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md). It uses ADR 0054, ADR 0056 item 6 and ADR 0058.

Status: Not started.

## Goal

A connected app connects with its public key and signs each request. Studio keeps no secret of an app.

## Context

- The mockups are in `.studio-dev/mockup-110/site/grant.js`, screens 6 to 8. Git ignores that folder. Use their texts.
- `crates/core/src/apps.rs` makes a token, keeps its SHA-256, and derives an app key from the token with HKDF-SHA-256 (`app_keys`). `crates/core/src/grant_vault.rs` adds one NIP-44 wrapper for each app key, and opens a grant vault with a token.
- `crates/api/src/access.rs` reads the token from `Authorization: Bearer`. The `/app/v1/` routes are in `crates/api/src/lib.rs`: `/whoami`, `/shows`, `/shows/{grant_id}/status` and `/shows/{grant_id}/live-value`.
- `crates/server/src/auth.rs` checks a signed event of kind 27235 for a Nostr sign-in, with the `nostr` crate (ADR 0056 item 6).
- `server.sqlite` keeps an index from token hashes to accounts (ADR 0058 item 18).
- The live publisher has no Studio client yet. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface for it.

## Packets

### 073a: The core

1. A migration of the collection gives `connected_apps` the columns `pubkey`, `state` (`waiting` or `active`), `last_used_at` and `last_address`. It removes the token hash. An app from before the migration gets `state` `token`, and each of its wrappers goes out of each grant vault (ADR 0110 item 12).
2. `apps::add_app(collection, name, pubkey)` accepts an `npub` or 64 hexadecimal characters, and refuses a key that is not a valid secp256k1 public key. `app_keys` and the token code go out.
3. `grant_vault::make` and the changes of ADR 0058 items 7 and 8 add wrappers only for `active` apps. `grant_vault::open_with_key(file, key)` opens a grant vault with the grant vault key. `grant_vault::wrapper(file, app_id)` gives the wrapper of one app.
4. A module in the core checks a signed request: the event of kind 27235, its signature, its time within 60 seconds, and its `u`, `method` and `payload` tags against the request. It gives the public key. The server sign-in and the app routes use the same function.
5. A replay store keeps each accepted event ID for 120 seconds, in memory, for each workspace.
6. `crates/core/src/testing.rs` gets a test signer that signs a request as an app, for the API and server tests.

### 073b: The API

1. `/app/v1/` accepts only `Authorization: Nostr <base64 event>`. A `Bearer` header gives 401 `app_key_required`. Each other condition of ADR 0110 item 6 gives 401 `app_signature_invalid`.
2. The first correct request of a `waiting` app makes it `active`. Studio adds its wrapper to each active grant vault at the next time that the user vault is open (ADR 0058 item 8).
3. A `waiting` app reaches only `GET /app/v1/whoami`.
4. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that signed the request.
5. The routes that write the feed take `grant_key` in the body, and open the grant vault with it. A wrong key gives 423 `vault_locked`.
6. Each accepted request records `last_used_at` and the client address of the app.
7. `/api/` and `/auth/` refuse the header `Authorization: Nostr`.
8. `POST /api/v1/apps` takes the name and the public key. It no longer gives a token.

### 073c: The server

1. `server.sqlite` keeps an index from each app public key to its account, in place of the token hashes. A migration removes the token hashes.
2. A key that another account has gives 409 `app_key_in_use`.
3. The server permits 60 requests for each app in each minute. For each client address, it permits 20 requests in each hour with an unknown key or a signature that is not correct.

### 073d: The UI and the user guide

1. Connected apps shows the form of screen 6: the name and the public key of the app, with the hint.
2. Each app row shows the lines of screen 7: the name, "Key" with the short key, and the shows with the last use, or "Waiting for the app. Start it once to confirm this key."
3. An app with a token shows "Connect Again" and the text of screen 8.
4. A model module gives the lines of each row. A Node test examines each state.
5. `docs/user-guide/connected-apps.md` explains the connection with a public key, one key for each instance, "Remove", and "Last used".

## Files

Change: `crates/core/src/apps.rs`, `crates/core/src/grant_vault.rs`, `crates/core/src/collection.rs`, `crates/core/src/lib.rs`, `crates/core/src/testing.rs`, a new core module for signed requests, `crates/api/src/access.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/workspace.rs`, `crates/server/src/auth.rs`, `crates/server/src/apps.rs`, `crates/server/src/accounts.rs`, `crates/server/src/limits.rs`, `crates/api/ui/app.js`, a UI model module and its Node test, `docs/user-guide/connected-apps.md`.

## Constraints

- Studio keeps no private key of an app, no token and no grant vault key outside the user vault.
- No secret is in a log or an API response.
- The four operations of ADR 0054 item 10 and the grant check box do not change.
- When a new test fails, its message names ADR 0110 and the fix.

## Acceptance Criteria

Mechanical:

- The tests of the acceptance criteria of ADR 0110.
- A core test: the shared check of a signed request refuses each condition of ADR 0110 item 6. The server sign-in tests pass with the shared function.
- A core test: the migration gives an app with a token the state `token`, and removes its wrappers.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the two themes: screens 6 to 8 of the mockups. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the human test of ADR 0110, after the live publisher has its client.
