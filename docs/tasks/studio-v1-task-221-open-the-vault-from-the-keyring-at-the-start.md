# Studio v1 Task 221: Open The Vault From The Keyring At The Start

Decision: [ADR 0235](../adr/0235-the-desktop-app-can-open-the-vault-from-the-keyring-of-the-computer.md), all items.

Status: Open.

## Problem

The desktop app asks for the passphrase at each start. ADR 0235 adds a device wrapper to the vault file, and a key in the keyring of the computer opens it. The setting "At start" in the section "Lock" turns it on and off.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Vault wrappers | extend | crates/core/src/vault/file.rs | `Wrapper` near line 33 gets the kind `Device`. |
| Vault open and HKDF wrapper | extend | crates/core/src/vault.rs | `add_passkey_wrapper` near line 296, `hkdf_wrapper` near line 378 and `open_with_passkey` near line 472 are the pattern for the device wrapper. |
| Account export | extend | crates/core/src/account_archive.rs | The export appends `vault.json` near line 281. It must leave out the device wrapper. |
| Collection copy | extend | crates/core/src/collections.rs | The copy near line 253 keeps the vault. It must leave out the device wrapper. |
| Workspace unlock and lock | extend | crates/api/src/workspace.rs | `unlock_with_passphrase` near line 1465, `lock` near line 1610, `require_recent_unlock` near line 1170, `lock_choices` near line 1204. `rules == None` is the desktop shell. |
| Lock settings route | extend | crates/api/src/routes.rs | `get_lock_settings` and `put_lock_settings` near line 4200. The route is `/settings/lock` in `crates/api/src/lib.rs` near line 552. |
| Desktop start | extend | crates/desktop/src/lib.rs | `start` opens the `Workspace` near line 298. The unlock at the start goes after it. |
| Settings section Lock | extend | crates/api/ui/app.js | `LockPanel` near line 5060. Settings saves it near line 3116. |
| Unlock screen | extend | crates/api/ui/app.js | `Unlock` near line 1980. |
| Keyring access | new | crates/desktop/src/keyring.rs | Searched crates/*, ~/build/musicindex-lib and Cargo.lock: no crate uses a keyring. `crates/desktop-app` has `zbus` for the tray only. Use the crate `keyring` (MIT OR Apache-2.0) from the research note. |

## API Contract

- `GET /api/v1/settings/lock` gives also `start` ("ask" or "device") and `start_choices`. `start_choices` is `["ask", "device"]` on the desktop with a keyring store, and `[]` on the server.
- `PUT /api/v1/settings/lock` accepts also `start`. "device" needs a recent unlock (401 `unlock_required`). A failed keyring write gives 502 `keyring_failed` with the detail of the keyring. The vault file does not change.
- `GET /api/v1/status` (or the route that the unlock screen reads now) gives `start_unlock_failed: true` after item 12 of the ADR, until the next unlock.

## Changes

1. Core, items 4, 5, 6 and 8: `Wrapper::Device { device_id, inner: HkdfWrapper }` with its own HKDF info string. `OpenVault::set_device_wrapper(device_id, secret)` replaces each earlier device wrapper and gives back the earlier device ID. `OpenVault::remove_device_wrapper()` gives back the removed device ID. `LockedVault::device_id()` gives the ID of the device wrapper. `LockedVault::open_with_device_secret(secret)` opens the vault.
2. Core, item 9: one function gives the bytes of a vault file with no device wrapper. The account export and the collection copy use it.
3. API: a trait `DeviceKeyStore` with `get`, `set` and `delete` of a secret by device ID. `Workspace::with_device_store` sets it. The server sets none. The tests use a store in memory, with a switch that makes each call fail.
4. API, items 1 to 3, 7 and 14: the lock settings routes of the API contract. "device" makes the secret and the device ID with the random source of the vault, writes the keyring first, then the vault file. When the keyring write fails, the vault file does not change. "ask" removes the wrapper, saves the vault file, then deletes the entry. A failed delete gives a warning in the answer, and the setting is "ask".
5. API, items 10 to 13: `Workspace::unlock_from_device_store()` opens the vault with the secret of the device wrapper. It does not call `mark_unlocked`. When it fails, it sets `start_unlock_failed`. Only the desktop start calls it. The 423 unlock sheet and `/vault/unlock` do not call it.
6. Desktop: `crates/desktop/src/keyring.rs` implements `DeviceKeyStore` with the crate `keyring`, service `musicindex-studio`, user `vault-{device ID}`. `start` sets the store and calls `unlock_from_device_store()` after `Workspace::open`. Add the dependency to `crates/desktop/Cargo.toml` only.
7. UI, item 1: `LockPanel` shows the select menu "At start" with "Ask for the passphrase" and "Unlock on this computer" when `start_choices` is not empty. On 401 `unlock_required`, use the unlock sheet of ADR 0106 that other sensitive changes use, then send again. On 502, show "Studio could not save a key in the keyring of this computer." with the detail.
8. UI, item 12: `Unlock` shows "Studio could not open the vault from the keyring of this computer. Enter the passphrase." above the passphrase field when `start_unlock_failed` is true.

## Checks

Mechanical:

- The core tests and API tests that ADR 0235 section "How This Decision Is Checked" names.
- UI test: the model of `LockPanel` gives the select menu "At start" for `start_choices` `["ask", "device"]`, and no select menu for `[]`.
- `tools/gate.sh` passes.
- A syntax check of each changed `ui/*.js` file as a `.mjs` copy passes.

Visual, by the main session: the checks of ADR 0235.

Real keyring, by the main session: the release binary on a private Xvfb with its own D-Bus and `gnome-keyring-daemon`. Turn on "Unlock on this computer", quit, start again, and see no unlock screen. `secret-tool search service musicindex-studio` shows one entry. Then start with no keyring daemon and see the notice of item 12.

Human, by the operator: the check of ADR 0235.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments, UI text and the Result section in Simplified Technical English.
- Read `/home/citizen/.agents/skills/rust-dev/SKILL.md` before the Rust changes.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above, the test files of those crates, `crates/api/ui-tests/`, `crates/desktop/Cargo.toml`, `Cargo.lock` and this task file.
- Read the documentation of the crate `keyring` on docs.rs for the version that you add. Use its default features unless the build on Linux fails. Report the version and the features.
- `node` is not on the PATH. Use `tools/node`.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.
