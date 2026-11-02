# Studio v1 Task 073: Connected Apps With Key Pairs

Decision: [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md), with its amendment of 2026-11-01. It uses ADR 0054, ADR 0056 item 6, ADR 0058, ADR 0078, ADR 0106 and ADR 0107.

Status: In progress. Packet 073a is complete. It started after [task 068](studio-v1-task-068-key-material-on-disk.md) (the file modes) and [task 069](studio-v1-task-069-account-protection.md) (the recent unlock of ADR 0106).

## Goal

A connected app pairs with a pairing text from Studio, and signs each request with its own key. A grant names one app and fixes its live value origin. A client crate does the protocol for each app. Studio keeps no secret of an app.

## Context

- The mockups are in `.studio-dev/mockup-110/site/grant.js`. Git ignores that folder. The operator accepted screens 1 to 3, 6 to 6c and 7 to 11 on 2026-11-01. Use their texts.
- `crates/core/src/apps.rs` makes a token, keeps its SHA-256, and derives an app key from the token with HKDF-SHA-256 (`app_keys`). `crates/core/src/grant_vault.rs` adds one NIP-44 wrapper for each app key, and opens a grant vault with a token.
- `sync_one_grant` in `crates/api/src/workspace.rs` adds a wrapper for each app with a key to each grant vault. After this task, a grant vault has only the wrapper of the app that the grant names.
- `put_show_live_value` in `crates/api/src/routes.rs` examines only that `uri` and `protocol` are not empty.
- `crates/api/src/access.rs` reads the token from `Authorization: Bearer`. The `/app/v1/` routes are in `crates/api/src/lib.rs`: `/whoami`, `/shows`, `/shows/{grant_id}/status` and `/shows/{grant_id}/live-value`.
- `crates/server/src/auth.rs` checks a signed event of kind 27235 for a Nostr sign-in, with the `nostr` crate (ADR 0056 item 6).
- `server.sqlite` keeps an index from token hashes to accounts (ADR 0058 item 18). `allow_app` in `crates/server/src/apps.rs` limits the requests for each token hash.
- The token notice of Connected apps in `crates/api/ui/app.js` shows a `SecretInput` with Copy and Done. The pairing text uses the same pattern.
- The core makes HTTP requests with `ureq`. The workspace has the `nostr` crate with NIP-44.
- The live publisher has no Studio client yet. The [handoff note](../notes/2026-11-01-live-publisher-studio-client.md) describes the interface for it.

## Packets

### 073a: The core

1. A migration of the collection gives `connected_apps` the columns `pubkey`, `state` (`waiting`, `active` or `token`), `code_hash`, `code_expires_at`, `live_value_origin`, `last_used_at` and `last_address`. It removes the token hash. An app from before the migration gets `state` `token` and no `pubkey`. Each of its wrappers goes out of each grant vault, and each grant from before ends (ADR 0110 items 37 and 38).
2. `apps::start_pairing(collection, name)` makes a waiting app and a code of ADR 0110 item 6. It keeps the SHA-256 of the code, and gives the code one time. `app_keys` and the token code go out.
3. `apps::pair(collection, code, pubkey, live_value_origin)` finds the waiting app from the hash of the code. It refuses an expired code and a key that is not a valid secp256k1 public key. It also refuses an origin that is not `https` or `wss` with a host and no path. It keeps the key as lowercase hexadecimal, and makes the app active. A waiting app with an expired code goes out at each call and at each start.
4. `apps::normalize_code(text)` applies the rules of ADR 0110 item 9. `start_pairing` and `pair` use it before the hash.
5. After a pairing, the app keeps its code hash until the 15 minutes end. A second pairing with that code gives `app_code_used` (ADR 0110 item 11).
6. A grant keeps the ID of one app and the live value origin of that app at the approval (ADR 0110 items 19 and 25).
7. `grant_vault::make` and the changes of ADR 0058 item 7 add only the wrapper of the app that the grant names. `grant_vault::open_with_key(file, key)` opens a grant vault with the grant vault key. `grant_vault::wrapper(file, app_id)` gives the wrapper of one app. A grant vault file gets mode 0600 (ADR 0110 item 22).
8. `apps::short_key(pubkey)` gives the first 9 and the last 4 characters of the `npub`.
9. A module in the core checks a signed request. It examines the event of kind 27235, its signature and its time within 60 seconds. It compares the `u` and `method` tags exactly with the expected address and method. It refuses a request with a body and no `payload` tag with the SHA-256 of the raw body. The function gives the public key. The server sign-in and the app routes use the same function.
10. A replay store in the collection keeps each accepted event ID for 120 seconds. The store records an ID before the route runs, and refuses an ID that it has (ADR 0110 item 15). The collection also keeps the latest accepted `created_at` of each app. The check refuses an event that is more than 60 seconds earlier than that time (ADR 0110 item 13).
11. The core refuses a granted live value with `live_value_invalid` in each condition of ADR 0110 item 26. The check is in the core, not in the route.
12. While a grant is active and the feed has a draft with changes, the feed state gives `grant_blocked_by_draft`. It holds the app name, the show title and the start time (ADR 0110 item 42).
13. Each refused granted write adds an entry to the list of granted writes, with the app, the cause and the time (ADR 0110 item 43).

### 073b: The client crate

1. Add the crate `crates/app-client` (`studio-app-client`) to the workspace. It depends on `nostr` and `ureq`, and not on `studio-core`, so that an app gets a small dependency.
2. A trait `AppSigner` signs an event and gives the public key. The crate gives `FileSigner`, which keeps a key in a file with mode 0600 in a folder with mode 0700. An app can give its own signer, for example for the key store of the operating system (ADR 0110 item 2).
3. `PairingText::parse(text)` reads the full pairing text, or the code alone with an origin that the app gives. It applies the rules of ADR 0110 item 9. `Client::pair(pairing_text, signer, live_value_origin)` sends `POST /app/v1/pair`, and gives a `Client` for that Studio.
4. A module `input` reads a pairing text only as ADR 0110 items 33 and 34 give. `input::prompt()` reads from the terminal with no echo. `input::from_clipboard()` reads the clipboard, and clears it if it still holds the pairing text. The clipboard function is behind the crate feature `clipboard`. The crate has no function that takes a pairing text from a command argument or an environment variable.
5. `PairingText::parse` and `Client` refuse a Studio origin that is not `https`, except a loopback address (ADR 0110 item 35).
6. `Client` signs each request as ADR 0110 item 12 gives, with a new event for each request. It has `whoami`, `shows`, `set_status`, `set_live_value` and `remove_live_value`.
7. `Client` corrects the clock as ADR 0110 item 36 gives. It reads the `Date` header of a 401 answer, and makes a correction of 5 minutes or less. It sends the request again one time only. It gives a warning through a callback of the app when the difference is more than 30 seconds.
8. Before the first write of a grant, `Client` gets the wrapper and decrypts it. It keeps the grant vault key in memory in a `Zeroizing` value. It clears the key when the grant ends, or when a write gives 404 or 423 (ADR 0110 item 29).
9. Each error has a type. The types are `CodeInvalid`, `CodeUsed`, `ClockWrong`, `SignatureInvalid`, `KeyRequired`, `KeyInUse`, `LiveValueInvalid`, `NotGranted`, `VaultLocked`, `TooManyAttempts` and `Network`. No error text holds a pairing text, an event or a key.
10. The crate documentation has an example of each step, from the pairing to the end of a show. It lists the rules of ADR 0110 item 32. Each example runs as a doc test.
11. The crate README tells an app developer: what the user does, what the app keeps, what the app never logs, and what to do for each error during a show.

### 073c: The API

1. `/app/v1/` accepts only `Authorization: Nostr <base64 event>`. A `Bearer` header gives 401 `app_key_required`. Each other condition of ADR 0110 item 13 gives 401 `app_signature_invalid`.
2. The API makes the expected `u` from the public origin of the shell configuration, not from the `Host` header. It refuses a body larger than 64 KiB before the hash, and calculates the hash before it reads the JSON.
3. `POST /app/v1/pair` takes `code` and `live_value_origin` in the signed body, and calls `apps::pair`. It is the only route that accepts a key that is not active. An unknown or expired code gives 401 `app_code_invalid`. A used code gives 409 `app_code_used`.
4. `GET /app/v1/shows/{grant_id}/wrapper` gives the wrapper of the app that the grant names. Each other app gets 404.
5. The routes that write the feed take `grant_key` in the body, and open the grant vault with it. A wrong key gives 423 `vault_locked`. An app that the grant does not name gets 404.
6. The removal of an app removes its wrappers at once and ends its grants. At the next unlock, Studio makes each of those grant vaults again with a new key (ADR 0110 item 21).
7. Each accepted request records `last_used_at` and the client address of the app.
8. `/api/` and `/auth/` refuse the header `Authorization: Nostr`.
9. `POST /api/v1/apps` takes the name. It gives the app and its pairing text. It no longer gives a token. `GET /api/v1/apps/{id}` gives the state of one app, with no code. `DELETE /api/v1/apps/{id}` removes a waiting app and its code, as Done needs.
10. The review sheet gets the active apps with their live value origins, for screens 1 to 3.
11. No log line holds a request body, an `Authorization` header, a signed event, a pairing text or a grant vault key (ADR 0110 item 17).
12. The API tests of `/app/v1/` use `studio-app-client` as their client (ADR 0110 item 30).

### 073d: The server

1. `server.sqlite` keeps an index from each public key to its account, and from each code hash to its waiting app. The server makes it again from the collections at each start. A migration removes the token hashes.
2. A pairing with a key that is active in a different account gives 409 `app_key_in_use`. The check and the insert are one transaction, with no `INSERT OR REPLACE`.
3. A pairing with the Nostr sign-in key of an account gives 400 `app_key_is_sign_in_key`.
4. `POST /api/v1/apps` needs a recent unlock (ADR 0106). Without it, it gives 401 `unlock_required`.
5. The server permits 60 requests for each app public key in each minute. For each client address, it permits 20 requests in each hour with an unknown key, a signature that is not correct or a wrong code. `allow_app` uses the public key in place of the token hash.
6. An import of an account (ADR 0107) sets each app to `token` state with no key, so that the app shows "Connect Again". The import keeps no grant (ADR 0110 item 41). If task 071 is complete, change its import. If not, add this step to task 071.

### 073e: The UI and the user guide

1. "Connect an App" shows the form of screen 6: only the name.
2. After "Make Pairing Text", Connected apps shows the notice of screen 6b: the pairing text in a `SecretInput`, Copy, Done and the hint. The waiting row shows the time until the code expires, and no part of the code. The rules of ADR 0110 item 7 apply:
   - Copy writes the pairing text to the clipboard while it stays hidden, and then shows "Copied".
   - Studio writes to the clipboard only from Copy.
   - The pairing text is not in the address, the page title, a notification or an error.
   - While the notice is open, the page reads `GET /api/v1/apps/{id}` every 2 seconds. When the app pairs, the notice shows screen 6c, and the pairing text goes out of the page state.
   - Done while the app waits sends `DELETE /api/v1/apps/{id}` (ADR 0110 item 8). Done after the pairing only closes the notice.
3. Each app row shows the lines of screen 7: the name, "Key" with the short key, the shows with the last use, and "Live value address".
4. An app with a token shows "Connect Again" and the text of screen 8.
5. The review sheet shows screens 1 to 3: the check box with the name of the one app and its live value address, the select menu for two or more apps, and the disabled menu with no app.
6. A model module gives the lines of each row and of the grant control. A Node test examines each state.
7. `docs/user-guide/connected-apps.md` gives these parts, each as a short procedure or a list:
   - How to connect an app: the name, the pairing text, the paste into the app, and the time limit of 15 minutes.
   - The pairing text is a secret for 15 minutes. Do not send it in a chat or an email. Paste it only into the app on your own computer.
   - Use Copy. Do not select the eye button while a person can see your screen, for example during a screen share or a recording.
   - Select Done only after the app says that it is connected. Done before that ends the pairing text.
   - When the app says that the pairing text was already used, a different program possibly used it. Remove that app in Studio, and make a new pairing text.
   - Use one pairing for each computer or copy of the app.
   - A grant gives one app one show. To use a different copy, approve the show again and select that copy.
   - What the app can change, and what it cannot change: only the status and the live value of the granted show, and only on its live value address.
   - What to do when a computer is lost or a key is stolen: select "Remove" on that app. The grants of the app end at once.
   - How to read "Last used" and the list of granted writes on the live show page.
   - How to change the live value address: remove the app and connect it again.
8. Each editor page of a feed shows the notice of screens 9 and 10 when the feed state gives `grant_blocked_by_draft`. The notice uses the neutral `notice` style.
9. The list of granted writes on the live show page shows a refused write with its cause, as screen 11.
10. `docs/user-guide/live-shows.md` links to the grant part of the connected apps page. It tells the user to write or discard each draft of the feed before a show, and explains the notice and the refused writes.

## Files

Add: `crates/app-client/` with its `Cargo.toml`, `src/lib.rs`, `README.md` and tests.

Change: `Cargo.toml`, `crates/core/src/apps.rs`, `crates/core/src/grant.rs`, `crates/core/src/grant_vault.rs`, `crates/core/src/live.rs`, `crates/core/src/collection.rs`, `crates/core/src/lib.rs`, a new core module for signed requests, `crates/api/Cargo.toml`, `crates/api/src/access.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/workspace.rs`, `crates/server/src/auth.rs`, `crates/server/src/apps.rs`, `crates/server/src/accounts.rs`, `crates/server/src/limits.rs`, `crates/server/src/session.rs`, `crates/api/ui/app.js`, a UI model module and its Node test, `docs/user-guide/connected-apps.md`, `docs/user-guide/live-shows.md`, the handoff note if the interface changes again, and `AGENTS.md` (the layout table gets `crates/app-client`).

## Constraints

- Studio keeps no private key of an app, no token and no grant vault key outside the user vault. It keeps only the hash of a pairing code.
- No secret, signed event or request body is in a log or an API response.
- The four operations of ADR 0054 item 10 do not change. Only the live value check of ADR 0110 item 26 is new.
- No feed rule in the UI (ADR 0065). The live value check is in the core.
- `POST /api/v1/apps` stays out of `RESTRICTED_ROUTES` in `crates/server/src/session.rs`.
- `studio-app-client` does not depend on `studio-core`, `studio-api` or a server crate.
- When a new test fails, its message names ADR 0110 and the fix.

## Acceptance Criteria

Mechanical:

- The tests of the acceptance criteria of ADR 0110.
- A core test: the shared check of a signed request refuses each condition of ADR 0110 item 13. The server sign-in tests pass with the shared function.
- A core test: the migration gives an app with a token the state `token` and no key, removes its wrappers and ends its grants.
- A core test: `apps::pair` refuses an expired code, a used code, a wrong origin and a key that is not valid.
- A core test: a grant vault has only the wrapper of the app that the grant names.
- A core test: the live value check refuses each condition of ADR 0110 item 26.
- A client test: `PairingText::parse` refuses a text with no origin or no code, and reads `k7m2 q9xd`, `K7M2-Q9XD` and `K7M2Q9XO` as the same code. `FileSigner` makes its file with mode 0600.
- A core test: a used code gives `app_code_used`, and a code after Done gives `app_code_invalid`.
- A core test: with an active grant and a draft with changes, the feed state gives `grant_blocked_by_draft`. A refused granted write adds its cause to the list.
- A Node test: the pairing text uses `SecretInput`, and the Connected apps model gives the lines of screen 6c after the pairing.
- A client test against the test API: with a clock that is 3 minutes wrong, a request succeeds after one correction and gives the warning. With a clock that is 10 minutes wrong, the client gives `ClockWrong` and sends no second request. A forged `Date` header on a second 401 does not cause a third request.
- A client test: `Client` refuses an `http` origin that is not a loopback address.
- A core test: after a change of the server clock back by one hour, an event that Studio accepted before is refused.
- `cargo test --doc -p studio-app-client` passes.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the two themes: screens 1 to 3, 6 to 6c and 7 to 11 of the mockups. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the human test of ADR 0110, after the live publisher has its client.
