# Studio v1 Task 005: Vault

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 2.
Decisions: [ADR 0003](../adr/0003-user-unlocked-secret-vault.md) and [ADR 0013](../adr/0013-vault-format-and-cryptography.md).

## Goal

Make the vault in `studio-core`: the vault key, the wrappers, the encrypted records and the desktop vault file.

## Files Likely To Change

- `crates/core/Cargo.toml`, the workspace `Cargo.toml`
- `crates/core/src/vault.rs` and its submodules

## Constraints

- Use the algorithms and parameters of ADR 0013 only.
- The Rust code does not do NIP-44. It stores the ciphertext that the signer makes, and it opens a vault with the vault key that the signer gives.
- No public function lets a caller choose weaker Argon2id parameters.

## Acceptance Criteria

Mechanical:

- A test opens a vault with each unlock method: passphrase, recovery key and a vault key from a Nostr signer.
- A test opens no vault with an incorrect passphrase, recovery key or vault key.
- A passphrase shorter than 12 characters gives an error.
- A record that moves to a different record ID or vault does not decrypt.
- Each write of a record uses a new nonce.
- A change of passphrase keeps the vault key and each record.
- The JSON of a vault file contains no plain secret and no plain vault key.
- The debug output of an open vault and of a vault key contains no secret and no key.
- A saved desktop vault file loads again with the same content.

Visual: none.

The plan also asks for a test that finds no secret in an API response or in the collection database. Those parts do not exist yet. Phase 3 and Phase 4 add those tests.

## Test Commands

```sh
cargo test -p studio-core vault
```
