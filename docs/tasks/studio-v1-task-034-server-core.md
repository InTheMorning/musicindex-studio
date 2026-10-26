# Studio v1 Task 034: The Server Core

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 12, milestone 12a.
Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 1 to 11, [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0003](../adr/0003-user-unlocked-secret-vault.md), [ADR 0013](../adr/0013-vault-format-and-cryptography.md) and [ADR 0017](../adr/0017-local-api-access.md).

## Goal

`studio-server` serves the Studio API and UI to many users on a public origin. A user signs up and signs in with an email and a passphrase, or with a NIP-07 signer. Each user has a data folder, a long session and a short vault window. The server refuses addresses in private networks.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-26-task-034-review-checklist.md) before its commit. The packets go in this sequence:

1. [034a: the router of one workspace](studio-v1-task-034a-workspace-router.md).
2. [034b: the private-network guard](studio-v1-task-034b-private-network-guard.md). It depends on 034a.
3. [034c: accounts, user folders and sessions](studio-v1-task-034c-accounts-and-sessions.md). It depends on 034b.
4. [034d: the vault window, the limits and the server rules](studio-v1-task-034d-vault-window-and-limits.md). It depends on 034c.
5. [034e: NIP-07 sign-in and the Nostr wrapper](studio-v1-task-034e-nip07-sign-in.md). It depends on 034c.
6. [034f: the sign-in pages and the account panel](studio-v1-task-034f-sign-in-ui.md). It depends on 034d and 034e.

## Acceptance Criteria

Mechanical:

- The desktop API tests pass with no change of their expected results.
- An API test signs in two users. A session of the first user cannot read or change a feed, a draft, a connector or a file of the second user.
- `/api/` refuses a request with no session and a request with no correct `X-Studio-CSRF` header. A `Host` that is not the public host gets 403.
- With the policy `Public`, tests refuse a loopback address for a feed, an image, a redirect, a relay, an S3 endpoint and an SFTP host.
- With a short idle limit, the vault locks, and a route that needs a secret gives 423.
- A test refuses a used challenge and a Nostr event with a different `u` tag.
- `server.sqlite` contains no session token and no vault key.
- The four commit checks pass, in CI too.

Visual:

- The sign-in page, the sign-up page, the unlock sheet and the account panel. Each one in the dark theme, in the light theme and at 390 px.
