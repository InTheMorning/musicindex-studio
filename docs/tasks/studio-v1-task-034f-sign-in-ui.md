# Studio v1 Task 034f: The Sign-In Pages And The Account Panel

Parent: [task 034](studio-v1-task-034-server-core.md). Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 9, 12, 14 to 17 and 20, [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 1 to 11, [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md), [ADR 0020](../adr/0020-first-run-asks-one-question.md), [ADR 0037](../adr/0037-button-roles.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the sixth packet of task 034. It depends on 034d and 034e.

## Goal

The one UI works in both shells. On a server, it shows the sign-in page and the sign-up page, and sends the CSRF header. It asks for an unlock when a route gives 423, and it has an Account panel. On the desktop, nothing changes.

## Files

Inspect: `crates/api/ui/app.js` (`takeToken`, `api`, `App`, `Unlock`, `CreateVault`, `ShowRecoveryKey`, `Hosts`, the Connected apps panel), `crates/api/ui/signing.js`, `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, `crates/server/src/auth.rs`, `tools/drive-ui.mjs`.

Change: `crates/api/ui/app.js`, the new model module `crates/api/ui/account.js`, the new test file `crates/api/ui-tests/account.test.mjs`, `crates/api/ui/styles.css` when necessary, `crates/server/src/auth.rs` (one route).

Do not change: the editor, the review sheet, the desktop boot flow.

## Architecture

These choices are fixed. Do not change them.

### The Model Module

`account.js` has no Preact and no browser state. It exports:

- `bootView(session, status)`: `"local"` when `session.mode` is `"local"`, `"sign_in"` when the server has no session, and `"shell"` for a signed-in user. In the mode `"shell"` of a server, a locked vault does not stop the boot.
- `signInEvent(origin, challenge, createdAt)`: the unsigned event of kind 27235 with the tags `u`, `method` and `challenge` of ADR 0056 item 6, and an empty content.
- `normalEmail(text)`: lowercase, with no spaces at the start or the end.
- `authErrorText(code)`: one text for each error code of the auth routes: `sign_in_failed`, `email_in_use`, `invite_required`, `challenge_invalid`, `too_many_attempts`, `busy`, `session_ended`, `wrong_key`, `no_signer`. An unknown code gives a general text.
- `unlockMethods(session, hasNip07)`: the methods of the unlock sheet. `"passphrase"` when the account has an email, and `"nip07"` when the account has a Nostr key and the browser has `window.nostr`.

### The Boot And The API Function

- `App` first calls `GET /auth/v1/session` with no token. In the mode `"local"`, it does the desktop boot with no change.
- In the mode `"server"`, `api()` sends no `Authorization` header. It sends `X-Studio-CSRF` with the token of the session response. It keeps the token in memory only.
- In the mode `"server"`, a response 423 `vault_locked` opens the unlock sheet. After an unlock, `api()` sends the same request one time again. If the user closes the sheet, the request gives its error.
- A response 401 `session_required` or `session_ended` shows the sign-in page.

### The Pages

- The sign-in page has the brand, an email field, a passphrase field and "Sign in" with the primary role. When the browser has `window.nostr`, it also has "Sign in with a signer extension" with the action role. It has a link to the sign-up page.
- The sign-up page has the email field and the passphrase field with the strength meter of the vault. It also has the invite field in the mode `invite`, and the recovery key check box. After sign-up, it shows `ShowRecoveryKey` when the server gave a key. The Nostr sign-up also adds the Nostr wrapper with `nip44.encrypt` before the shell opens.
- The server gets `POST /auth/v1/strength` with `{"passphrase"}`. It gives the result of `passphrase_strength`, as the vault route of the desktop does. The failure counter of a client address applies to it.
- The unlock sheet lists the methods of `unlockMethods`. With NIP-07, it gets the wrapper from `GET /api/v1/vault/nostr-wrapper`, asks `window.nostr.nip44.decrypt`, and sends the key to `POST /api/v1/vault/unlock-key`.
- The Hosts page gets an "Account" panel in the mode `"server"` only, before the Connected apps panel. It shows the email or the `npub`, "Sign out" with the normal role, and "Delete the account" with the destructive role. The deletion asks the user to type the email or the `npub`.
- The first-run wizard of a server has no vault step (ADR 0020).

## Steps

1. Add `account.js` and its Node tests.
2. Change the boot and `api()`.
3. Add the sign-in page, the sign-up page, the unlock sheet and the strength route.
4. Add the Account panel.
5. Do the visual check of the acceptance criteria against `studio-server` with a loopback origin.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate and no new UI file other than `account.js`. The UI loads nothing from other servers ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- The UI gives no advice on how to make a passphrase (ADR 0018).
- Each control has the target size of its profile.

## Acceptance Criteria

Mechanical:

- The Node tests examine `bootView` for each case, `signInEvent` for its tags, `normalEmail`, each code of `authErrorText`, and `unlockMethods`.
- A server test: `POST /auth/v1/strength` gives the same score as the desktop route for the same passphrase.
- The desktop API tests and the desktop `drive-ui.mjs` boot pass with no change.
- The four commit checks pass.

Visual:

- The planning model checks the sign-in page, and the sign-up page in the mode `invite` with a weak passphrase. It also checks the recovery key after sign-up, the unlock sheet after a 423, and the Account panel. Each one in the dark theme, in the light theme and at 390 px with touch.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the resend after an unlock needs a change in more than `api()` and the unlock sheet.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-034f-sign-in-ui.md
- docs/adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- The files that the Files section names.

Goal:
- Add the model module `account.js` with Node tests.
- Make the UI boot in the server mode, with the sign-in page, the sign-up page and the CSRF header.
- Add the unlock sheet after 423, and the Account panel.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. The UI loads nothing from other servers. The desktop boot does not change.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The editor and the review sheet.
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
