# Task 035 Review Checklist

- Date: 2026-10-26
- Reviewed: each packet of [task 035](../tasks/studio-v1-task-035-phone.md)
- Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md), [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md), [ADR 0013](../adr/0013-vault-format-and-cryptography.md), [ADR 0044](../adr/0044-the-nsite-signer.md) and [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 035a: a vault file of the earlier format still opens. No PRF output and no recovery key text is in a `Debug` output.
- 035g: a restricted session reaches only the routes of the packet. Only a new method removes the limit. An unknown account, an incorrect key and an account with no vault file give the same answer. No recovery key text is in a log or an error message.
- 035b: the verifier checks the origin, the challenge, the relying party ID, the UV flag and the counter on each path. No PRF output is in a log, a row or a response.
- 035c: each answer of a remote signer comes from its public key. Each relay goes through the network policy. No client key is in a log.
- 035d: the write route and the pipeline do not change. The sign route permits only the kinds 35128 and 24242.
- 035e: the import of `finish` is the import of the earlier route, with no copy. The part route does not hold the lock of the workspace during the write.
- 035f: the manifest and the icons need no session. The UI loads nothing from other servers.

## Invariants

- Each vault wrapper encrypts the same vault key. A wrapper that does not decrypt gives an error ([ADR 0013](../adr/0013-vault-format-and-cryptography.md)).
- A passkey assertion works one time only ([ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md)).
- Studio publishes no event that a key other than the key of the host signed ([ADR 0044](../adr/0044-the-nsite-signer.md)).
- An import uses the bytes that the user sent, in their positions ([ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [035a](../tasks/studio-v1-task-035a-hkdf-wrappers.md) | Pass | `hkdf` 0.13 uses the `hmac` 0.13 and `sha2` 0.11 of the workspace, so no escalation occurred. `hkdf` is a dependency of `studio-core` only, as `sha2` is, so the root `Cargo.toml` did not change. The planning model changed two new tests to `tempfile`, as the other tests do. In the manual checks, a fixed file of the earlier format, made with the code before the change, opens with its recovery key. The `Debug` output of a wrapper shows only the salt and the sealed data. The associated data of a passkey wrapper has its credential ID. A file with both recovery wrapper types tries only the HKDF wrapper, but `add_recovery_key` never makes such a file. |
