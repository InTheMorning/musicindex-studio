# Studio v1 Task 069: Account Protection

Decision: ADR 0055 item 23 (limits), ADR 0056 items 3, 5, 10 and 11 (methods, challenges and the vault key for a signer). Packet a writes ADR 0106 first. ADR 0106 changes ADR 0055 item 23 and adds the rule of a recent unlock.

Status: Not started.

## Goal

A stolen session cannot change the sign-in methods of an account or get the vault key. A third party cannot block the sign-in of a different user.

## Context

On 2026-11-01, a security audit of the user data found these items. The report is not in the repository. Each item below gives the code that shows it.

- **SEC-01.** `POST /auth/v1/challenge` and `POST /auth/v1/passkey/options` need no session. Each call adds one row to `challenges`. `create_challenge` in `crates/server/src/accounts.rs` has one global cap of 20,000 rows, and each row lasts 300 seconds. No limit applies to each client address. One client can fill the table, and then no user can start a Nostr or passkey sign-in for 5 minutes.
- **SEC-02.** `signer_key` in `crates/api/src/routes.rs` gives the vault key while the vault is open, at each call. The UI uses it at sign-up and when the user adds a Nostr method (`app.js`). A restricted session after a recovery sign-in can call it (`RESTRICTED_ROUTES` in `crates/server/src/session.rs`). The restricted session needs it, because it must add a new method.
- **SEC-03.** `set_passphrase` in `crates/api/src/routes.rs` replaces the passphrase wrapper with no proof. `passkey_remove` in `crates/server/src/auth.rs` asks for no new proof. After a change, the other sessions of the account continue for a maximum of 30 days.
- **SEC-06.** `Attempts` in `crates/server/src/limits.rs` counts failed sign-ins for each email, from each address. Ten failures in one hour block that email from all addresses. The counter also runs for an unknown email. A third party can thus block the email sign-in of a user each hour.

The current passphrase is not a good proof. A user after a recovery sign-in does not have it, and a user with only Nostr or a passkey has no passphrase. ADR 0106 thus uses a recent unlock with one of the methods.

## Packets

### 069a: The decision

Write ADR 0106 "A sensitive account change needs a recent unlock". It states these rules:

1. Each workspace records the time of the last unlock of its vault in the current vault window. The unlock can use each method: a passphrase, a Nostr signer, a passkey or the recovery key. A sign-in that opens the vault is an unlock. The creation of a vault is an unlock.
2. A sensitive change needs an unlock in the last 5 minutes. Without it, the route gives 401 `unlock_required`. The UI then asks for an unlock, and sends the request again. The sensitive changes are:
   - a new passphrase
   - the vault key for a signer (`POST /api/v1/vault/signer-key`)
   - the add or the removal of a passkey, a Nostr method or a recovery key
   - the deletion of the account
   - the add of a connected app (`POST /api/v1/apps`, [ADR 0110](../adr/0110-a-connected-app-signs-each-request-with-its-own-key.md) item 2)
3. After a new passphrase, or after the removal of a method, the server ends each other session of the account. The current session continues.
4. ADR 0055 item 23 changes. The server permits 10 failed sign-ins for each pair of account and client address in each hour. It permits 100 failed sign-ins for each account from all addresses in each hour. The limit of 30 for each client address does not change.
5. The challenge routes permit 30 challenges for each client address in each 5 minutes. The global cap stays.

Add ADR 0106 to `docs/README.md` and to AGENTS.md in the same commit. Mark ADR 0055 item 23 as changed by ADR 0106.

### 069b: The recent unlock

1. `Workspace` keeps `unlocked_at: Option<Instant>`. Each unlock route and each vault creation sets it. A lock clears it.
2. Add `require_recent_unlock(w)` adjacent to `require_open_vault` in `crates/api/src/routes.rs`. It gives 401 `unlock_required` when the unlock is older than 5 minutes.
3. `set_passphrase` and `signer_key` call it. The route of a new recovery key does not exist yet. Task 071 adds it with this check. On the server, `passkey_add`, `passkey_remove`, the routes of a Nostr method and the account deletion call it.
4. The UI handles `unlock_required` as it handles 423 `vault_locked`: it asks for an unlock, then it sends the request again. The text of the unlock sheet says why: "Unlock Studio again to change how you sign in." Put the text in the module of the sign-in texts.
5. The desktop uses the same rule. The desktop has no other sessions, so item 3 of ADR 0106 does nothing there.

### 069c: Sessions after a change

1. Add `Accounts::delete_other_sessions(account_id, keep_token_hash)`.
2. Call it after a new passphrase and after the removal of a method, on the server.

### 069d: The limits

1. `Attempts` keeps two counters: one for each pair of account and address, with the limit 10, and one for each account, with the limit 100. The two counters use the window of one hour. The counter of each address does not change.
2. `challenge` and `passkey_sign_in_options` use a new counter for each client address: 30 in 5 minutes. Above the limit, they give 429 `too_many_attempts`.
3. Update `docs/runbooks/studio-server.md` section 5 with the new limits.

## Files

Add: `docs/adr/0106-a-sensitive-account-change-needs-a-recent-unlock.md`.

Change: `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/ui/app.js`, `crates/server/src/auth.rs`, `crates/server/src/accounts.rs`, `crates/server/src/limits.rs`, `crates/server/src/session.rs`, `docs/runbooks/studio-server.md`, `docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md`, `docs/README.md`, `AGENTS.md`.

## Constraints

- No change to the vault format.
- A restricted session after a recovery sign-in can add a method. Its sign-in is a recent unlock.
- No feed rule in the UI (ADR 0065). This task changes no feed.
- When a new test fails, its message names ADR 0106 and the fix.

## Acceptance Criteria

Mechanical:

- An API test: `set_passphrase` and `signer_key` give 401 `unlock_required` 6 minutes after the unlock, and succeed after a new unlock. The test uses a clock that it controls.
- An API test: the sign-up and a recovery sign-in each count as a recent unlock.
- A server test: `passkey_remove` and the removal of a Nostr method give 401 `unlock_required` with an unlock older than 5 minutes.
- A server test: after a new passphrase, a second session of the account gets 401, and the current session continues.
- A server test: 10 failed sign-ins from address A block A for that email, and a sign-in from address B succeeds.
- A server test: 100 failed sign-ins from many addresses block the email from a new address.
- A server test: the 31st challenge from one address in 5 minutes gives 429, and a challenge from a second address succeeds.
- Each new test fails when its fix is removed.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 1100 px and at 390 px with touch:

- The unlock sheet with the text of 069b item 4, after "Add a Passkey" with an unlock older than 5 minutes.
- `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human, on `studio.musicindex.org`:

- The operator signs in on two devices, changes the passphrase on one, and the other device goes to the sign-in page.
