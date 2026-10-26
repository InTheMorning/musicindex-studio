# Task 034 Review Checklist

- Date: 2026-10-26
- Reviewed: each packet of [task 034](../tasks/studio-v1-task-034-server-core.md)
- Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0013](../adr/0013-vault-format-and-cryptography.md) and [ADR 0017](../adr/0017-local-api-access.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- The desktop API tests pass with no change of their expected results.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 034a: `local_router` gives the same response as before for each route. No vault logic is in two places.
- 034b: each client that connects to an address of a user uses the policy. The musicindex client and the hub client keep `Open`. With `Public`, no client uses a proxy from the environment.
- 034c: a session reaches only the folder of its user. No token, passphrase or vault key is in a log or an error message. The CSRF token is derived, not kept.
- 034d: each path that ends a session or the vault window removes the vault key from memory. The desktop has no server rule.
- 034e: the challenge is removed in the same database step that accepts it. Only `POST /api/v1/vault/signer-key` gives the vault key.
- 034f: the logic of the texts and the boot is in `account.js`. The desktop boot does not change.

## Invariants

- A session can read and write only the folder of its own user ([ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).
- The server keeps no vault key after a lock, a sign-out or the idle limit ([ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md)).
- A signed sign-in event works one time only ([ADR 0004](../adr/0004-server-sign-in-with-nostr-or-email.md), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md)).
- With `Public`, no outbound connection goes to a private address ([ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
