# ADR 0013: Vault Format And Cryptography

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0003](0003-user-unlocked-secret-vault.md) puts each connector secret in a vault that only its user can open.
It leaves three items open: the key derivation function, the encryption algorithm and a recovery method.
[ADR 0004](0004-server-sign-in-with-nostr-or-email.md) lets one user sign in with a Nostr signer, with an email and a passphrase, or with both. Each method must open the same vault.

## Decision

### Keys

1. Each vault has one vault key: 32 random bytes from the random source of the operating system.
2. The vault key encrypts each secret. It does not change when the user changes a passphrase or adds a sign-in method.
3. Each unlock method has a wrapper: a copy of the vault key that the method encrypts. A vault can have more than one wrapper.

### Secrets

4. Each secret is one record. The vault key encrypts the record with XChaCha20-Poly1305.
5. Each record has a new random 24-byte nonce for each write.
6. The associated data of each record is the vault ID, the record ID and the format version. A record that moves to a different vault or ID does not decrypt.

### Wrappers

7. Passphrase wrapper:
   - Argon2id makes a key from the passphrase and a random 16-byte salt.
   - The parameters are 64 MiB of memory, 3 passes and 1 lane. The wrapper records them, so that a new version can make them stronger.
   - That key encrypts the vault key with XChaCha20-Poly1305.
   - A passphrase must have a minimum of 12 characters. Studio sets no other rule for its characters.
8. Nostr wrapper:
   - The user's signer encrypts the vault key to the user's own public key with NIP-44 version 2.
   - The wrapper records the public key and the ciphertext.
   - At sign-in, the signer in the browser decrypts the wrapper. The server does not get the private key.
9. Recovery wrapper (optional):
   - At setup, Studio offers a recovery key: 16 random bytes, shown one time as text.
   - Studio uses the recovery key like a passphrase with the same Argon2id parameters.
   - A user who does not want a recovery key can decline it. Then the loss of all unlock methods loses the secrets ([ADR 0003](0003-user-unlocked-secret-vault.md)).

### Storage

10. The desktop vault is one JSON file. It contains the format version, the vault ID, the wrappers and the records. The file contains no plain secret.
11. Studio writes the desktop file to a temporary file, flushes it to disk and then renames it. An interrupted write does not damage the vault.
12. The server keeps the same structure for each user in its database.

### Memory

13. The vault key and each opened secret are in types that clear their memory when they go out of scope.
14. These types do not show their content in debug output or in logs.
15. The server keeps the vault key only in the memory of the open session ([ADR 0003](0003-user-unlocked-secret-vault.md)).

### Sign-In With Email

16. For email sign-in, the passphrase opens the vault, and a vault that opens is the proof of the sign-in. Studio keeps no second passphrase hash.
17. The server gives the same error and takes the same time for an unknown email and for an incorrect passphrase.

### Dependencies

18. Studio uses the RustCrypto crates `argon2` and `chacha20poly1305`, and `zeroize`. Their licenses (MIT or Apache-2.0) are compatible with AGPL-3.0 ([ADR 0011](0011-agpl-license.md)).
19. The Rust code does not use NIP-44. The signer in the browser does the NIP-44 encryption and decryption.

## Alternatives Considered

- AES-256-GCM. Rejected: its 12-byte nonce is too short for random nonces over many writes. XChaCha20-Poly1305 has a 24-byte nonce, and it is fast without hardware support.
- scrypt or PBKDF2. Rejected: Argon2id is the current recommendation of RFC 9106 for a password key, and it resists GPU attacks better.
- The `age` file format. Rejected: it cannot use NIP-44 for a wrapper, and it cannot mix a passphrase with other recipients.
- A separate passphrase hash for sign-in. Rejected: it gives an attacker a second target with no new function.

## Consequences

- Each email sign-in uses 64 MiB of server memory for the duration of the key derivation. A server operator must limit the number of sign-ins that occur at the same time.
- A change of passphrase writes only one wrapper again. The records do not change.
- A Nostr user's browser sees the vault key at sign-in, because the signer decrypts it there.
- A recovery key that the user loses is of no use. Studio cannot make a new one without an open vault.

## Invariants

- No plain secret and no plain vault key is in a file, a database row, a log or an API response.
- Each record write uses a new nonce.
- A wrapper or record that does not decrypt gives an error. It does not give partial data.

## Open Questions

- Does the server keep one vault file for each user, or rows in a database table? This ADR requires only the same structure.
