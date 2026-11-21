# Studio v1 Task 221: Open The Vault From The Keyring At The Start

Decision: [ADR 0235](../adr/0235-the-desktop-app-can-open-the-vault-from-the-keyring-of-the-computer.md), all items.

Status: Done.

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

## Result

- Added the device wrapper and filtered vault bytes in [vault.rs](../../crates/core/src/vault.rs:330) and [file.rs](../../crates/core/src/vault/file.rs:238).
- Excluded the device wrapper from exports and collection copies in [account_archive.rs](../../crates/core/src/account_archive.rs:287) and [collections.rs](../../crates/core/src/collections.rs:737).
- Added the key store, startup unlock, lock settings and status fields in [workspace.rs](../../crates/api/src/workspace.rs:888) and [routes.rs](../../crates/api/src/routes.rs:4204).
- Added the desktop keyring with `keyring` 4.2.0 and default `v1` features in [keyring.rs](../../crates/desktop/src/keyring.rs:1). The docs.rs source lists `get_secret`, `set_secret` and `delete_credential`.
- Connected the desktop start in [lib.rs](../../crates/desktop/src/lib.rs:292). Added the “At start” menu and the keyring unlock error notice in [app.js](../../crates/api/ui/app.js:5063).
- Added the device wrapper cases, API startup cases and UI menu model in [vault.rs](../../crates/core/src/vault.rs:990), [device_unlock.rs](../../crates/api/src/tests/device_unlock.rs:1) and [settingsdraft.test.mjs](../../crates/api/ui-tests/settingsdraft.test.mjs:25).
- Updated `Cargo.lock` for the keyring dependency.
- Checks: `tools/gate.sh` passed. It passed the Rust checks, ADR guard and 947 UI checks. UI syntax checks and the component guard passed.
- STE check: 25 dictionary findings remain in the task file.
- Visual: the Lock section at 1280 px passed its fit check. The 390 px Lock section check remains open.
- Keyring check of the delegate: open. Xvfb exited after a segmentation fault.
- Human check: open for the operator.

## Main Session Changes

- `crates/api/src/workspace.rs`, `set_start_mode`: the same start value changes nothing. The UI sends the saved start value with each change of the lock time. Before this change, such a change made a new keyring key and needed a recent unlock. On the server, the value "ask" gave 409.
- `crates/desktop/src/keyring.rs`: a read waits at most 60 seconds. A keyring such as KeePassXC can ask the user to unlock it, and the start must not wait with no limit. Technical choice of the main session: the limit is 60 seconds.
- `crates/api/ui/vaultlock.js` and `crates/api/ui/app.js`: "At start" has the hint `START_HINT`, as "Lock Studio after" has `LOCK_HINT`.
- `crates/api/src/tests/device_unlock.rs`: tests for ADR 0235 items 3, 12 and 13, for the same start value after 20 minutes, and for a server lock time with `start: "ask"`.
- `crates/desktop/src/keyring.rs`, `while_dumpable`: the keyring read at the start runs while the process is dumpable (ADR 0235 item 15, from finding 2 of the human check). `crates/desktop/src/lib.rs`: the not dumpable test also examines this function.

Main session checks:

- `tools/gate.sh` passes.
- Real keyring: `target/debug/studio-desktop` ran on a private D-Bus with `gnome-keyring-daemon`.
- "Unlock on this computer" wrote one entry (`secret-tool search` gives service `musicindex-studio`, user `vault-{device ID}`) and one wrapper with the kind `device`. The next start gave `"vault":"open"`. A start with no D-Bus and a start after `secret-tool clear` gave `"vault":"locked"` and `"start_unlock_failed":true`.
- Screenshots with `tools/drive-ui.mjs`: the unlock screen with the notice of item 12 at 1280 px and 390 px. Settings > Lock with "At start" is also at 1280 px and 390 px. `$edges()` gives no finding, and the fit check gives no finding at 390 px and 761 px.
- The Tauri binary was not started for this task. It calls the same `studio_desktop::start`.

## Findings Of The Human Check

The operator used KeePassXC as the Secret Service provider.

1. KeePassXC shows "unknown executable" for Studio. The window "Access Request" gives the warning "Non-existing/inaccessible executable path". Cause: the desktop start makes the process not dumpable (`crates/desktop/src/lib.rs` near line 83). Then other processes of the same user cannot read `/proc/PID/exe`.
2. "Remember" in that window has no effect after a restart. KeePassXC asks again at each start of Studio. Cause: KeePassXC keeps each decision only in memory, in the `DBusClient` of one D-Bus connection (`src/fdosecrets/dbus/DBusClient.h` and `.cpp` of KeePassXC). Item 15 does not change this. The operator kept ADR 0235 item 15, so that the window shows the program path of Studio with no warning, and a click at each start.
3. The first Studio process gave "SS error: prompt dismissed" with no window, and KeePassXC kept 4 entries with no title. A new Studio process wrote the key with no error. After "Allow Selected", the start opened the vault with no unlock screen (ADR 0235 item 10). Inference: KeePassXC kept a refused condition for the connection of the first process. The test programs `kdiag` (the crate `secret-service`, all four cases) and `kdiag2` (the crate `keyring`, as Studio uses it) passed from a terminal.

Open:

- None. The human check of ADR 0235 passed with KeePassXC: item 10 opened the vault with no unlock screen, and with item 15 the access request shows `/home/citizen/build/musicindex-studio/target/release/musicindex-studio-desktop` with no warning.
