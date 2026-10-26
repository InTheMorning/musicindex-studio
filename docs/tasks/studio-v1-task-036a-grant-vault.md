# Studio v1 Task 036a: The Grant Vault

Parent: [task 036](studio-v1-task-036-grant-vaults.md). Decisions: [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md) items 1 to 15, [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 12 to 16, and [ADR 0013](../adr/0013-vault-format-and-cryptography.md).
Sequence: the first packet of task 036. It depends on 035c.

## Goal

Each active show grant has a grant vault with the secret of the host of the feed XML. A granted write opens it with the app token and works while the user vault is locked. This applies in both shells.

## Files

Inspect: `crates/core/src/apps.rs`, `crates/core/src/grant.rs`, `crates/core/src/pipeline.rs` (`write_granted`, the `grant::record` call near line 2493), `crates/core/src/vault.rs`, `crates/core/src/vault/file.rs`, `crates/core/src/collection.rs` (the connector rows, `secret_ref`, the feed host), `crates/api/src/workspace.rs` (`hosts_with`), `crates/api/src/routes.rs` (`granted_change`, `app_check`, `add_app`, `remove_app`, `update_connector`, `set_feed_host`, `write_feed`, `unlock_vault`), `crates/api/ui/live.js`, `crates/api/ui/app.js` (the Connected apps panel), the review sheet in `crates/api/ui/editor.js`.

Change: the new file `crates/core/src/grant_vault.rs`, `crates/core/src/apps.rs`, `crates/core/src/collection.rs`, `crates/core/src/vault.rs`, `crates/core/src/vault/file.rs`, `crates/core/src/lib.rs`, `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/api/ui/live.js`, `crates/api/ui-tests/live.test.mjs`, `crates/api/ui/app.js`, `crates/api/ui/editor.js`.

Do not change: `pipeline::write_granted` and `covered`. The grant rules of ADR 0054 items 13 and 14 do not change.

## Architecture

These choices are fixed. Do not change them.

### The Core

- A migration adds the column `app_pubkey TEXT` to `connected_apps`. `add_app` derives the app key: HKDF-SHA-256 of the 32 token bytes with the info `musicindex-studio app key v1`, as a secp256k1 secret key. If the 32 bytes are not a valid key, it makes a new token. It keeps the public key as hexadecimal. `ConnectedApp` gets `has_key: bool`.
- `pub fn app_keys(token: &str) -> Keys` derives the same key from a token.
- `Wrapper` gets `App { app_id, ciphertext }`: the grant vault key as base64, encrypted with NIP-44 version 2 from a new random key to the app public key. The wrapper keeps the public key of the random key with the ciphertext.
- `grant_vault.rs` has:
  - `make(user_vault: &OpenVault, secret_ids: &[&str], apps: &[ConnectedApp]) -> Result<(VaultFile, Zeroizing<String>), …>`. It copies the records, adds one wrapper for each app with a key, and gives the file and the grant vault key.
  - `open(file: VaultFile, app_id: i64, token: &str) -> Result<OpenVault, …>`.
  - `add_wrapper(file, grant_key, app)` and `remove_wrapper(file, app_id)`.
- The secret IDs of a grant are the `secret_ref` of the connector of the feed XML of that feed. A host with no secret gives an empty list.

### The Workspace

- The grant vault of grant `n` is `grants/<n>.json` in the data folder. The user vault keeps its key as the record `grant-vault/<n>`.
- `Workspace::sync_grant_vaults(refresh: Option<FeedId>)` does each step that is possible now, and nothing else:
  - With the user vault open: make a missing grant vault for each open grant. Make again the grant vaults of the `refresh` feed. Add a missing wrapper for each app with a key.
  - Always: remove each wrapper of a removed app. Remove the grant vault and the record of each grant that is not active (`grant::active`).
- Studio calls it after each write, after an unlock, after `add_app` and `remove_app`, after `update_connector` (with the feed of each grant that uses the connector), after `set_feed_host` (with the feed), at the start, and each hour.
- `granted_change` takes the token from the request extensions of `app_check`. It opens the grant vault and makes the hosts with `hosts_with` and that vault in place of the user vault. The user vault can be locked. A missing grant vault gives 423 `vault_locked`.
- `app_check` keeps the token in the request extensions in a type that clears its memory.

### The UI

- `live.js` gets `grantBoxState(feedHostSigner)`: disabled with the text "Connected apps cannot write to a host with My signer, because no person is present to sign." when the host of the feed XML uses "My signer".
- The review sheet uses it for the grant check box.
- The Connected apps panel shows "Connect again" with the action role for an app with `has_key` false. It removes the app and makes a new one with the same name, and shows the new token.

## Steps

1. Add the migration, the app key and the `App` wrapper, with unit tests.
2. Add `grant_vault.rs`, with unit tests.
3. Add `sync_grant_vaults` and its calls.
4. Change `granted_change` to use the grant vault.
5. Add `grantBoxState` with its Node tests, the check box state and "Connect again".
6. Do the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. NIP-44 comes from the `nostr` feature of 035c, and HKDF from 035a.
- No app token and no grant vault key is in a log, an error message, a file or a database row as plain text.

## Acceptance Criteria

Mechanical:

- An API test: the user makes a grant for a show on an SFTP host of the sshd harness and locks the vault. A test client with the app token does "Go live", sets a live value and does "End the show". The hosted feed has each change. After the end, `grants/` has no file and the user vault has no `grant-vault/` record.
- A unit test: `grant_vault::open` refuses a different token and a removed app. The grant vault has only the records of the secret IDs.
- An API test: an app that the user adds during a grant gets 423 before the next unlock and writes after it.
- An API test: after `update_connector` with a new password, a granted write uses the new password.
- An API test reads each file in the data folder and the collection database, and finds no app token and no grant vault key.
- The Node tests examine `grantBoxState`.
- The four commit checks pass.

Visual:

- The planning model checks the disabled grant check box for an nsite host with "My signer". It also checks the Connected apps panel with "Connect again". Each one in the dark theme, in the light theme and at 390 px.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a granted write needs a secret of a second host.
- Stop and report when `hosts_with` needs a change of the connector trait for the grant vault.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-036a-grant-vault.md
- docs/adr/0058-grant-vaults-and-connected-apps-on-the-server.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- Add the app key, the `App` wrapper and the grant vault to the core.
- Keep the grant vaults current with `sync_grant_vaults`, and make each granted write use the grant vault.
- Disable the grant check box for "My signer", and add "Connect again".

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No app token or grant vault key as plain text in a log, a file or a row.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `pipeline::write_granted` and `covered`.
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
