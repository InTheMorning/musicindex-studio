# Studio v1 Task 073: Connected Apps With Key Pairs

Decision: [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md), with its amendment of 2026-11-01. It uses ADR 0054, ADR 0056 item 6, ADR 0058, ADR 0106 and ADR 0107.

Status: Not started. It starts after [task 068](studio-v1-task-068-key-material-on-disk.md) (the file modes) and [task 069](studio-v1-task-069-account-protection.md) (the recent unlock of ADR 0106).

## Goal

A connected app connects with its public key and a confirmation code, and signs each request. A grant applies only to the apps that it names. Studio keeps no secret of an app.

## Context

- The mockups are in `.studio-dev/mockup-110/site/grant.js`, screens 6 to 8. Git ignores that folder. Use their texts. They have no confirmation code and no app line in the grant sheet. Packet 073d adds those screens first.
- `crates/core/src/apps.rs` makes a token, keeps its SHA-256, and derives an app key from the token with HKDF-SHA-256 (`app_keys`). `crates/core/src/grant_vault.rs` adds one NIP-44 wrapper for each app key, and opens a grant vault with a token.
- `sync_one_grant` in `crates/api/src/workspace.rs` adds a wrapper for each app with a key to each grant vault. After this task, it adds a wrapper only for each active app that the grant names (ADR 0110 item 17).
- `put_show_live_value` in `crates/api/src/routes.rs` examines only that `uri` and `protocol` are not empty.
- `crates/api/src/access.rs` reads the token from `Authorization: Bearer`. The `/app/v1/` routes are in `crates/api/src/lib.rs`: `/whoami`, `/shows`, `/shows/{grant_id}/status` and `/shows/{grant_id}/live-value`.
- `crates/server/src/auth.rs` checks a signed event of kind 27235 for a Nostr sign-in, with the `nostr` crate (ADR 0056 item 6).
- `server.sqlite` keeps an index from token hashes to accounts (ADR 0058 item 18). `allow_app` in `crates/server/src/apps.rs` limits the requests for each token hash.
- The live publisher has no Studio client yet. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface for it.

## Packets

### 073a: The core

1. A migration of the collection gives `connected_apps` the columns `pubkey`, `state` (`waiting`, `active` or `token`), `code_hash`, `code_expires_at`, `code_failures`, `last_used_at` and `last_address`. It removes the token hash. An app from before the migration gets `state` `token` and no `pubkey`. Each of its wrappers goes out of each grant vault (ADR 0110 items 25 and 26).
2. `apps::add_app(collection, name, pubkey)` accepts an `npub` or 64 hexadecimal characters in each case. It keeps the key as lowercase hexadecimal. It refuses a key that is not a valid secp256k1 public key. It makes the confirmation code of ADR 0110 item 4, keeps its SHA-256, and gives the code one time. `app_keys` and the token code go out.
3. `apps::confirm(collection, pubkey, code)` makes a waiting app active when the code is correct and not expired. A wrong code adds one to `code_failures`. The fifth wrong code and an expired code remove the waiting app (ADR 0110 items 5 and 6). The function compares the hashes in constant time.
4. A grant keeps the IDs of the apps that are active when the user approves its diff. A migration gives each grant from before no app. Such a grant gets no wrapper (ADR 0110 item 17).
5. `grant_vault::make` and the changes of ADR 0058 item 7 add wrappers only for each active app that the grant names. `grant_vault::open_with_key(file, key)` opens a grant vault with the grant vault key. `grant_vault::wrapper(file, app_id)` gives the wrapper of one app. A grant vault file gets mode 0600 (ADR 0110 item 20).
6. A function `apps::short_key(pubkey)` gives the short form of ADR 0110 item 7.
7. A module in the core checks a signed request. It examines the event of kind 27235, its signature and its time within 60 seconds. It compares the `u` and `method` tags exactly with the expected address and method. It refuses a request with a body and no `payload` tag with the SHA-256 of the raw body. The function gives the public key. The server sign-in and the app routes use the same function.
8. A replay store in the collection keeps each accepted event ID for 120 seconds. The store records an ID before the route runs, and refuses an ID that it has (ADR 0110 item 13).
9. The core refuses a granted live value with a wrong scheme, a `uri` longer than 2048 characters, or a wrong `protocol`, with `live_value_invalid` (ADR 0110 item 24). The check is in the core, not in the route.
10. `crates/core/src/testing.rs` gets a test signer that signs a request as an app, for the API and server tests.

### 073b: The API

1. `/app/v1/` accepts only `Authorization: Nostr <base64 event>`. A `Bearer` header gives 401 `app_key_required`. Each other condition of ADR 0110 item 11 gives 401 `app_signature_invalid`.
2. The API makes the expected `u` from the public origin of the shell configuration, not from the `Host` header. It refuses a body larger than 64 KiB before the hash, and calculates the hash before it reads the JSON.
3. `POST /app/v1/confirm` takes `code` in the signed body, and calls `apps::confirm`. A correct code gives 200 and the state `active`. A wrong code gives 401 `app_code_invalid`.
4. A `waiting` app reaches only `GET /app/v1/whoami` and `POST /app/v1/confirm`.
5. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that signed the request. A grant that does not name the app gives 404.
6. The routes that write the feed take `grant_key` in the body, and open the grant vault with it. A wrong key gives 423 `vault_locked`. A grant that does not name the app gives 404.
7. The removal of an app removes its wrappers at once. At the next unlock, Studio makes each grant vault that named the app again with a new key (ADR 0110 item 19).
8. Each accepted request records `last_used_at` and the client address of the app.
9. `/api/` and `/auth/` refuse the header `Authorization: Nostr`.
10. `POST /api/v1/apps` takes the name and the public key. It gives the app, its short key and the confirmation code. It no longer gives a token.
11. The review sheet gets the names of the active apps for the grant check box. With no active app, it gets the state that ADR 0110 item 17 gives.
12. No log line holds a request body, an `Authorization` header, a signed event, a confirmation code or a grant vault key (ADR 0110 item 15).

### 073c: The server

1. `server.sqlite` keeps an index from each public key to its account and its state, in place of the token hashes. The server makes it again from the collections at each start. A migration removes the token hashes.
2. A waiting app does not reserve its key. `POST /app/v1/confirm` finds the waiting app from the key and the code. When an app becomes active, the server removes each waiting app of the same key in the other accounts (ADR 0110 items 8 and 9).
3. A key that is active in a different account gives 409 `app_key_in_use`. The check and the insert are one transaction, with no `INSERT OR REPLACE`.
4. A key that is the Nostr sign-in key of an account gives 400 `app_key_is_sign_in_key`.
5. `POST /api/v1/apps` needs a recent unlock (ADR 0106). Without it, it gives 401 `unlock_required`.
6. The server permits 60 requests for each app public key in each minute. For each client address, it permits 20 requests in each hour with an unknown key, a signature that is not correct or a wrong code. `allow_app` uses the public key in place of the token hash.
7. An import of an account (ADR 0107) sets each app to `token` state with no key, so that the app shows "Connect Again". The import keeps no grant (ADR 0110 item 29). If task 071 is complete, change its import. If not, add this step to task 071.

### 073d: The UI and the user guide

1. Before the code of this packet, add to the mockups: the code after "Connect an App", the waiting row with the code, and the grant sheet with the app line and the state with no app. The operator accepts these screens.
2. Connected apps shows the form of screen 6: the name and the public key of the app, with the hint.
3. After "Connect an App", Studio shows the short key and the confirmation code, with the time until the code expires. The text tells the user to compare the short key with the app and to give the code to the app.
4. Each app row shows the lines of screen 7: the name, "Key" with the short key, and the shows with the last use, or the waiting line.
5. An app with a token shows "Connect Again" and the text of screen 8.
6. The review sheet names the apps after the grant check box, or says "Connect an app first".
7. A model module gives the lines of each row and of the grant line. A Node test examines each state.
8. `docs/user-guide/connected-apps.md` explains the connection with a public key and a code, one key for each instance, a grant for the apps that are connected at the approval, "Remove", and "Last used".

## Files

Change: `crates/core/src/apps.rs`, `crates/core/src/grant.rs`, `crates/core/src/grant_vault.rs`, `crates/core/src/live.rs`, `crates/core/src/collection.rs`, `crates/core/src/lib.rs`, `crates/core/src/testing.rs`, a new core module for signed requests, `crates/api/src/access.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/workspace.rs`, `crates/server/src/auth.rs`, `crates/server/src/apps.rs`, `crates/server/src/accounts.rs`, `crates/server/src/limits.rs`, `crates/server/src/session.rs`, `crates/api/ui/app.js`, a UI model module and its Node test, `docs/user-guide/connected-apps.md`, and the handoff note if the interface changes again.

## Constraints

- Studio keeps no private key of an app, no token and no grant vault key outside the user vault. It keeps only the hash of a confirmation code.
- No secret, signed event or request body is in a log or an API response.
- The four operations of ADR 0054 item 10 do not change. Only the live value check of ADR 0110 item 24 is new.
- No feed rule in the UI (ADR 0065). The live value check is in the core.
- `POST /api/v1/apps` stays out of `RESTRICTED_ROUTES` in `crates/server/src/session.rs`.
- When a new test fails, its message names ADR 0110 and the fix.

## Acceptance Criteria

Mechanical:

- The tests of the acceptance criteria of ADR 0110.
- A core test: the shared check of a signed request refuses each condition of ADR 0110 item 11. The server sign-in tests pass with the shared function.
- A core test: the migration gives an app with a token the state `token` and no key, and removes its wrappers.
- A core test: `apps::confirm` refuses an expired code, and removes the waiting app after the fifth wrong code.
- A core test: a grant vault has a wrapper only for each active app that the grant names.
- A core test: the live value check refuses each wrong value of ADR 0110 item 24.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the two themes: screens 6 to 8 of the mockups, the screen of the code and the grant sheet. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the human test of ADR 0110, after the live publisher has its client.
