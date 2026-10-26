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
| [035g](../tasks/studio-v1-task-035g-recovery-sign-in.md) | Pass after correction | The server removed the limit after any 2xx response of the two add routes. Thus an email account could add a Nostr wrapper, and a Nostr account could add a passphrase or the wrapper of a different key. None of these signs in to the account. The planning model added `usable_method`. It refuses such a method with 403 `method_not_usable`, and a new test fails without it. The coding model added the boot view `add_method`, because the recovery response has no email or public key. In the manual checks, `RESTRICTED_ROUTES` in `session.rs` is the one list of the permitted routes, and account deletion uses the same check. The fixed vault of an unknown account has a recovery wrapper, so each failure does the same work. The `drive-ui.mjs` check passes in the dark theme, in the light theme and at 390 px with touch. It shows the recovery page with an email and a key, and the page "Add a sign-in method" with a weak passphrase. At 390 px no control is smaller than 44 px. The email sign-in gives 500 for an account with no vault file, and the recovery route gives 401. Only a failed sign-up can make such an account. |
| [035b](../tasks/studio-v1-task-035b-passkeys.md) | Pass after a decision | A random PRF salt for each credential cannot work for a discoverable sign-in, because the browser can send only one salt. By operator decision, each passkey of the server uses one fixed salt, and ADR 0056 item 21 records it. The counter follows the W3C rule: when one of the two counters is not zero, the new counter must be larger. In the manual checks, the verifier compares the stored challenge bytes, the origin, the hash of the relying party ID, the flags UP and UV, the signature and the counter on each path. The assertion after a registration must come from the same credential, so its signature proves the SPKI key. Before a correct signature, a failure counts for the client address only. No PRF output is in a log, a row or a response, but the raw body buffer is not cleared. `drive-ui.mjs` gets the step keys `webauthn` and `webauthn: "replace"`. The check uses `http://localhost`, because WebAuthn refuses an IP address as the relying party ID. It passes in the dark theme, in the light theme and at 390 px with touch. It shows the Account panel with two passkeys, the sign-in page and the unlock sheet. A sign-in with a passkey and an unlock with "Use a passkey" both open the vault. |
| [035c](../tasks/studio-v1-task-035c-nip46-sign-in.md) | Pass after correction | Two corrections. A signer could give a bad ciphertext at sign-up, and a Nostr account with no recovery key could then never open its vault. The server now asks the signer to decrypt the new wrapper and compares the result with the vault key. A difference removes the new account and gives the invite back. The connect route needs no session, so the planning model limited each client address to 4 pending connections. `relay::exchange` replaces the `wait_for` of the packet, because one connection must send the request and wait for the answer. `TestRelay` now has live subscriptions. The NIP-07 route and the NIP-46 route call one function `complete_nostr`. In the manual checks, each answer of a remote signer must come from its public key, each relay goes through the network policy, and no client key is in a log. A signer belongs to one session and goes away with it. The `drive-ui.mjs` check used a temporary harness with a `TestRelay` and a `TestSigner`. It passes in the dark theme, in the light theme and at 390 px with touch. It shows the sign-in page, the page with the QR code, the bunker field and the wait state, and the unlock sheet with "Use the remote signer", which opens the vault. At narrow width the unlock sheet of 034f was 32 px too narrow, because the rule of a small sheet was stronger. The planning model corrected `styles.css`. |
| [035d](../tasks/studio-v1-task-035d-nip46-signing.md) | Pass | No correction of the code. The workspace keeps the signer in a `SignerSlot` with a pending and a failed state, and `GET /api/v1/signer` gives the failure as `error`. `TestSigner::start_with_key` was necessary for a second key. In the manual checks, the write route and the pipeline did not change. The sign route permits only the kinds 35128 and 24242, and it checks the key before a request goes to the signer. It does not hold the lock of the workspace while it waits. On the server, the session map and the workspace keep the same signer, and a sign-out, account deletion and the sweep remove it from both. The `drive-ui.mjs` check used a temporary harness with a `TestRelay`, a `TestBlossom` and two `TestSigner` keys. It passes in the dark theme, in the light theme and at 390 px with touch, with the review sheet in three states: "Connect a signer", a signer of a different key, and a connected signer of the host with "Write to the host" enabled. A write through the signer is in an API test. "DELETE /api/v1/signer" on a server does not remove the signer of the session, which `nip46/unlock` uses. |
