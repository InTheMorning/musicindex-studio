# ADR 0235: The Desktop App Can Open The Vault From The Keyring Of The Computer

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, who selected "Keyring key" from a drawing in the questionnaire. The choices were a key in the keyring, the passphrase in the keyring, and no change.
- Technical choices: the main session selected items 2 to 14. The operator did not receive them as a question.
- Bar: crosses modules. It adds a wrapper kind to the vault file in `studio-core`, a route and a setting in `studio-api`, and the keyring in `studio-desktop`. Without this record, an agent can keep ADR 0003 item 2 and refuse a keyring, or keep the passphrase itself in the keyring.
- Changes: [ADR 0003](0003-user-unlocked-secret-vault.md) item 2, the passphrase as the only unlock method of the desktop vault.
- Uses: [ADR 0142](0142-a-user-selects-the-time-before-the-vault-locks.md), the section "Lock" of Settings. [ADR 0106](0106-a-sensitive-account-change-needs-a-recent-unlock.md), the recent unlock. [ADR 0149](0149-the-desktop-app-has-many-collections.md), one vault for each collection.
- Source: [research note of 2026-11-15](../research/2026-11-15-vault-unlock-at-the-start-of-the-desktop-app.md), option 1.

## Context

The desktop app asks for the passphrase at each start. The lock time "Never" of ADR 0142 keeps the vault open only while the app runs. On a computer that only the artist uses, the login of the computer gives the same protection as the passphrase at each start.

ADR 0003 refused the keychain of the operating system, because the encrypted vault file must move between computers. The vault file can hold more than one wrapper (`crates/core/src/vault/file.rs`, `Wrapper`). A wrapper is an encrypted copy of the vault key for one unlock method. A second wrapper can use a key from the keyring, while the passphrase wrapper stays.

## Decision

### The Setting

1. Settings has the select menu "At start" in the section "Lock". Its values are "Ask for the passphrase" and "Unlock on this computer". "Ask for the passphrase" is the start value.
2. Only the desktop shell has the select menu. The server shell has no keyring, and its API does not give the choice.
3. A change to "Unlock on this computer" needs a recent unlock (ADR 0106), because it adds an unlock method. A change back needs an open vault.

### The Device Wrapper

4. "Unlock on this computer" makes a random secret of 32 bytes and a random device ID. Studio keeps the secret in the keyring of the computer, with the service `musicindex-studio` and the user `vault-{device ID}`.
5. The vault file gets the wrapper kind `device`, with the device ID. A key from HKDF-SHA-256 over the secret encrypts the vault key, as the passkey wrapper does.
6. The vault has at most one device wrapper. A new one replaces the earlier one, and Studio deletes the earlier entry from the keyring.
7. "Ask for the passphrase" removes the device wrapper from the vault file. Studio then deletes the entry from the keyring. When the delete fails, the wrapper is still removed, and the notice gives the error.
8. The passphrase wrapper does not change. A change of the passphrase keeps the device wrapper.
9. An account export and a copy of a collection do not copy the device wrapper.

### The Start Of The App

10. At its start, the desktop shell reads the keyring entry of the device wrapper and opens the vault with it. The user sees no unlock screen.
11. This unlock does not count as a recent unlock (ADR 0106). A sensitive change still asks for the passphrase.
12. When the keyring is not available, the entry is not there, or its key does not open the vault, the vault stays locked. The unlock screen shows the passphrase field and the notice "Studio could not open the vault from the keyring of this computer. Enter the passphrase." The device wrapper stays.
13. The keyring opens the vault only at the start of the app. After a lock, by the lock time or by **Lock**, Studio asks for the passphrase.
14. When "Unlock on this computer" cannot write to the keyring, the setting stays "Ask for the passphrase". The error notice says "Studio could not save a key in the keyring of this computer." and gives the detail of the keyring.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0003 item 2 | The passphrase opens the vault on each computer. On the desktop, a key in the keyring of the computer can also open it (items 4 to 13). |

## Consequences

- On a computer with "Unlock on this computer", each program that runs as the same user and can read the keyring can open the vault. The login of the computer protects the vault.
- A vault file that moves to a different computer opens with the passphrase. Its device wrapper does nothing there.
- On Linux, the computer needs a Secret Service provider, such as GNOME Keyring or KeePassXC. A window manager such as i3 does not always start one. Without it, item 12 applies.
- The vault file grows by one wrapper.

## Alternatives Considered

- The passphrase in the keyring. Not chosen by the operator: the keyring then holds the real passphrase, which can open a copy of the vault on each computer. A change of the passphrase must also change the keyring.
- A passkey at the start. Not chosen: WebKitGTK refuses WebAuthn requests (research note, option 3), and the user must touch the passkey at each start.
- No change. Not chosen by the operator.
- The keyring opens the vault again after a lock. Not chosen: a lock then has no effect.

## How This Decision Is Checked

Mechanical:

- `crates/core` tests: a device wrapper opens the vault. A second device wrapper replaces the first. The remove of the wrapper leaves the passphrase wrapper. A change of the passphrase keeps the device wrapper. An export and a copy of a collection have no device wrapper.
- `crates/api` tests of the settings, with a keyring in memory: the lock settings give the start choices on the desktop and not on the server. The change to "Unlock on this computer" needs a recent unlock. A failed keyring write leaves the vault file unchanged.
- `crates/api` tests of the start, with a keyring in memory: a start with the entry opens the vault and sets no recent unlock. A start with no entry or a wrong key leaves the vault locked and gives the notice. A lock and a new unlock do not read the keyring.

Visual, by the main session: the section "Lock" at 1280 px and at 390 px, and the unlock screen with the notice of item 12.

Human, by the operator, on the operator's computer with its own keyring: turn on "Unlock on this computer" and quit the app. Start the app again, and see no unlock screen.
