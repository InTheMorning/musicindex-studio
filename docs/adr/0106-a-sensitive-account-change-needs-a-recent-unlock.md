# ADR 0106: A Sensitive Account Change Needs A Recent Unlock

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the security audit of the user data on the same day.
- Uses: [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 14 to 17 and 23 (sessions, the vault window and the limits), [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md) items 3, 5, 10 and 11 (methods, challenges and the vault key for a signer) and [ADR 0110](0110-a-connected-app-signs-each-request-with-its-own-key.md) item 5 (connect an app).
- Changes: ADR 0055 item 23 (the limits of failed sign-ins).

## Context

- On 2026-11-01, a security audit found four gaps. The [task 069](../tasks/studio-v1-task-069-account-protection.md) file gives the code of each gap.
- A stolen session can ask for the vault key (`POST /api/v1/vault/signer-key`), replace the passphrase, and add or remove a passkey. No new proof is needed.
- After such a change, the other sessions of the account continue for a maximum of 30 days.
- The routes `POST /auth/v1/challenge` and `POST /auth/v1/passkey/options` need no session. One client address can fill the table of challenges. No user can then start a Nostr sign-in or a passkey sign-in for 5 minutes.
- The counter of failed sign-ins is for each email, from all addresses. A third party can block the email sign-in of a user each hour.
- The current passphrase is not a good proof. A user after a recovery sign-in does not have it, and a user with only Nostr or a passkey has no passphrase.

## Decision

1. Each workspace records the time of the last unlock of its vault in the current vault window. Each method counts: a passphrase, a Nostr signer, a passkey or the recovery key. A sign-in that opens the vault is an unlock. The creation of a vault is an unlock. A lock clears the time.
2. A sensitive change needs an unlock in the last 5 minutes. Without it, the route gives 401 `unlock_required`. The UI then asks for an unlock and sends the request again. The sensitive changes are:
   - a new passphrase
   - the vault key for a signer (`POST /api/v1/vault/signer-key`)
   - the add or the removal of a passkey, a Nostr method or a recovery key
   - the deletion of the account
   - the add of a connected app (`POST /api/v1/apps`, ADR 0110 item 5)
3. After a new passphrase, or after the removal of a method, the server ends each other session of the account. The current session continues.
4. The text of the unlock sheet says why: "Unlock Studio again to change how you sign in."
5. ADR 0055 item 23 changes. The server permits 10 failed sign-ins for each pair of account and client address in each hour. It permits 100 failed sign-ins for each account from all addresses in each hour. The limit of 30 for each client address does not change.
6. The challenge routes permit 30 challenges for each client address in each 5 minutes. The global cap of 20,000 stays.
7. The desktop uses items 1 and 2. The desktop has no other sessions, so item 3 does nothing there.
8. The workspace takes its time from a clock that a test can replace. A test thus moves the time and does not wait.

## Alternatives Considered

- The current passphrase as the proof. Not possible for a recovery session and for an account with no passphrase.
- A new proof for each change with the method that the user chooses. More steps for the user, and the same security as an unlock in the last 5 minutes.
- A limit for each account only. A third party can then block the sign-in of a user.

## Consequences

- A stolen session cannot change how the account signs in, unless the thief can also unlock the vault.
- A change after an idle time of more than 5 minutes asks the user to unlock first.
- The user with a Nostr remote signer (NIP-46) must approve the unlock at the signer. An unlock with an open vault also asks the signer.
- A third party can still block one account from many addresses, with 100 failures in one hour. This risk is smaller than before and the operator accepts it.
- The desktop needs a vault for "Connect an App". An app with no vault has no grant vault key.
- No route removes a Nostr method yet. The rule of item 2 applies to the route when it exists.
- The vault format does not change.

## Acceptance Criteria

Mechanical:

- An API test: `set_passphrase`, `signer_key` and the add of an app give 401 `unlock_required` 6 minutes after the unlock, and succeed after a new unlock. The test uses a clock that it controls.
- An API test: the creation of a vault and a recovery unlock each count as a recent unlock.
- A server test: `passkey_remove`, the options of a new passkey and the account deletion give 401 `unlock_required` with an unlock older than 5 minutes.
- A server test: after a new passphrase, and after the removal of a passkey, a second session of the account gets 401, and the current session continues.
- A server test: 10 failed sign-ins from address A block A for that email, and a sign-in from address B succeeds.
- A server test: 100 failed sign-ins from many addresses block the email from a new address.
- A server test: the 31st challenge from one address in 5 minutes gives 429, and a challenge from a second address succeeds.
- A Node test: the UI model gives the sheet text and treats 401 `unlock_required` as a request for an unlock.

Visual: the unlock sheet shows the text of item 4 after "Add a Passkey" with an unlock older than 5 minutes.

Human: the operator signs in on two devices, changes the passphrase on one, and the other device goes to the sign-in page.
