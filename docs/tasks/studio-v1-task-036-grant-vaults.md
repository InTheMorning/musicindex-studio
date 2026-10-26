# Studio v1 Task 036: Grant Vaults And Connected Apps On The Server

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 12, milestone 12c.
Decisions: [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md), [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md), [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) and [ADR 0013](../adr/0013-vault-format-and-cryptography.md).

## Goal

A connected app does a granted write while the vault of the user is locked, in both shells. Only the token of the app opens the grant vault. On a server, the app uses the public origin.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-26-task-036-review-checklist.md) before its commit. The packets go in this sequence:

1. [036a: the grant vault](studio-v1-task-036a-grant-vault.md). It depends on 035c, for NIP-44 in the core.
2. [036b: connected apps on the server](studio-v1-task-036b-apps-on-the-server.md). It depends on 036a.

## Acceptance Criteria

Mechanical:

- A test makes a grant, locks the user vault, and does "Go live", a live value and "End the show" with the app token. The hosted feed has each change.
- The grant vault does not open with a different app token. It holds only the secret of the host of the feed XML.
- After the end of the grant, no grant vault file exists. The removal of an app removes its wrapper.
- A server test does a granted write through the public origin, and refuses the session cookie on `/app/v1/` and an app token on `/api/`.
- No file or database row contains an app token or a grant vault key as plain text.
- The four commit checks pass, in CI too.

Visual:

- The disabled grant check box for "My signer", and the app list with "Connect again". Each one in the dark theme, in the light theme and at 390 px.

Human test, open until v4vmm or the live publisher builds its side:

- A test show against a server Studio with no browser session open.
