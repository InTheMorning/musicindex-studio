# Studio v1 Task 035a: The HKDF Wrappers

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 22 and 28, and [ADR 0013](../adr/0013-vault-format-and-cryptography.md).
Sequence: the first packet of task 035. It depends on task 034.

## Goal

The vault gets two wrapper types with a key from HKDF-SHA-256: a recovery wrapper and a passkey wrapper. New recovery keys use the HKDF wrapper. A recovery wrapper of the earlier type still opens.

## Files

Inspect: `crates/core/src/vault.rs` (`add_recovery_key`, `open_with_recovery_key`, `derived_wrapper`, `open_derived`, `wrapper_aad`), `crates/core/src/vault/file.rs` (`Wrapper`), `crates/core/src/vault/crypto.rs`, `Cargo.toml`, `crates/core/Cargo.toml`.

Change: `crates/core/src/vault.rs`, `crates/core/src/vault/file.rs`, `crates/core/src/vault/crypto.rs`, `crates/core/Cargo.toml`, `Cargo.toml`.

Do not change: the passphrase wrapper, the Nostr wrapper, the records.

## Architecture

These choices are fixed. Do not change them.

- The workspace adds `hkdf` at the version that matches `sha2` 0.11. `studio-core` uses it.
- `crypto.rs` gets `Key::expand(input: &[u8], salt: &[u8], info: &str) -> Key`. It uses HKDF-SHA-256 and gives 32 bytes.
- `Wrapper` gets two variants:
  - `RecoveryHkdf { salt, sealed }`, with the `kind` `recovery_hkdf`. The info text is `musicindex-studio recovery wrapper v1`. The input is the 16 bytes of the recovery key, not its text.
  - `Passkey { credential_id, salt, sealed }`, with the `kind` `passkey`. The info text is `musicindex-studio passkey wrapper v1`. The input is the 32-byte PRF output.
- The associated data of each wrapper is that of `wrapper_aad` with its kind, as for the other wrappers. For a passkey, it also has the credential ID.
- `add_recovery_key` makes a `RecoveryHkdf` wrapper and removes each earlier recovery wrapper. `open_with_recovery_key` tries a `RecoveryHkdf` wrapper first, and then a `Recovery` wrapper.
- `OpenVault::add_passkey_wrapper(credential_id: &str, salt: &[u8; 32], prf_output: &[u8; 32])`, `OpenVault::remove_passkey_wrapper(credential_id)` and `VaultFile::open_with_passkey(credential_id, prf_output)`.
- `VaultFile::methods() -> Vec<VaultMethod>` lists the wrapper kinds, with the public key of a Nostr wrapper and the credential ID of a passkey wrapper. It gives no salt and no ciphertext.
- The salt of a passkey wrapper is the PRF salt of its credential. The server of 035b gives it.

## Steps

1. Add `hkdf` and `Key::expand`.
2. Add the two wrapper variants and the functions.
3. Add the unit tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- One new crate: `hkdf`, a RustCrypto crate with the license MIT or Apache-2.0.
- No PRF output and no recovery key text is in a `Debug` output.

## Acceptance Criteria

Mechanical:

- A unit test makes a vault with a recovery key, saves it, loads it and opens it with the recovery key. The file has the kind `recovery_hkdf` and no kind `recovery`.
- A unit test loads a fixed vault file of the earlier format with a `recovery` wrapper and opens it with its recovery key.
- A unit test adds a passkey wrapper and opens the vault with the same PRF output. It refuses a different PRF output and a different credential ID.
- A unit test removes a passkey wrapper. The vault then does not open with it.
- `methods()` gives no salt and no ciphertext.
- The four commit checks pass.

Visual: none.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `hkdf` needs a version of `sha2` or `hmac` that the workspace does not use.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035a-hkdf-wrappers.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- docs/adr/0013-vault-format-and-cryptography.md
- The files that the Files section names.

Goal:
- Add `Key::expand` with HKDF-SHA-256, the wrappers `RecoveryHkdf` and `Passkey`, and `VaultFile::methods`.

Constraints:
- Follow the Architecture section of the task file fully.
- One new crate: `hkdf`.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The passphrase wrapper, the Nostr wrapper and the records.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
