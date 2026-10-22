# ADR 0003: Each User Unlocks A Secret Vault

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

Studio keeps the credentials that write each feed: SFTP keys, object storage keys, Git tokens and Nostr signing access.
MSP 2.0 keeps edit tokens in browser localStorage without encryption.
A public Studio server holds secrets for many users. A breach of that server must expose as little as possible.

## Decision

1. All connector secrets are in a vault. The vault is encrypted. Only its user can open it.
2. The desktop shell keeps the vault as one encrypted local file. The artist's passphrase opens it.
3. The server shell keeps one vault for each user. The key of the vault comes from the user's sign-in:
   - Email and passphrase: the key comes from the passphrase.
   - Nostr signer: the vault key is encrypted to the user's own Nostr key with NIP-44. The signer decrypts it at sign-in.
4. The server can write to a host only while the user's session is open. The server has no key that opens a vault without the user.
5. The two shells use the same vault format.

## Alternatives Considered

- The operating system keychain on the desktop. Rejected by the operator: the encrypted file can move between machines.
- One server key for all vaults. Rejected: a breach of the host exposes all users, and the mission prefers user control.

## Consequences

- A lost passphrase loses the secrets. The artist must then enter each credential again. The feeds and the collection are not lost, because they are not secrets.
- The server cannot do a scheduled write while the user is signed out.
- The NIP-44 method needs a signer that supports NIP-44 encryption.

## Invariants

- No secret is in a log, an API response, a snapshot or the collection database without encryption.
- The server keeps no vault key after the session closes.

## Follow-Up Work

- Select the key derivation function and the encryption algorithm in the vault design.
- Select a recovery method, if one is necessary.
