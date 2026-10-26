# Studio v1 Task 035: The Phone

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 12, milestone 12b.
Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md), [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md), [ADR 0013](../adr/0013-vault-format-and-cryptography.md), [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) and [ADR 0044](../adr/0044-the-nsite-signer.md).

## Goal

An artist changes a feed from a phone. The artist signs in and opens the vault with a passkey, or with a NIP-46 signer such as Amber. A user who has only the recovery key signs in with it and adds a new method. A NIP-46 signer also signs nsite and Blossom writes. A large music file goes to Studio in parts. Studio opens from the home screen.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-26-task-035-review-checklist.md) before its commit. The packets go in this sequence:

1. [035a: the HKDF wrappers](studio-v1-task-035a-hkdf-wrappers.md). It depends on task 034.
2. [035g: sign-in with the recovery key](studio-v1-task-035g-recovery-sign-in.md). It depends on 035a.
3. [035b: passkeys with PRF](studio-v1-task-035b-passkeys.md). It depends on 035a and 035g.
4. [035c: the NIP-46 client and NIP-46 sign-in](studio-v1-task-035c-nip46-sign-in.md). It depends on task 034.
5. [035d: "My signer" through NIP-46](studio-v1-task-035d-nip46-signing.md). It depends on 035c.
6. [035e: uploads in parts](studio-v1-task-035e-uploads-in-parts.md). It depends on task 034.
7. [035f: the home screen app](studio-v1-task-035f-home-screen-app.md). It depends on task 034.

## Acceptance Criteria

Mechanical:

- Unit tests open a vault with each wrapper type, and with a recovery wrapper of the earlier type.
- A test with fixed WebAuthn data adds a passkey and signs in. It refuses an incorrect origin, an incorrect challenge and a counter that does not increase.
- A test signer on the test relay does a NIP-46 sign-in, a NIP-46 unlock and the signature of a planned nsite event.
- A recovery sign-in opens the vault. The session then gets 403 `method_required` until the user adds a new method.
- An upload in parts in a random sequence gives the same SHA-256 as the source file.
- The manifest and the icons load with no session.
- The four commit checks pass, in CI too.

Visual:

- The passkey list, the passkey sign-in and the NIP-46 connection with its QR code.
- The recovery page and the page "Add a sign-in method".
- The review sheet with a NIP-46 signer, and the import sheet during an upload in parts.
- Each one in the dark theme, in the light theme and at 390 px.

Human tests, open until the operator does them in task 037:

- A passkey on an iPhone and on an Android phone, each with a second synced device.
- A NIP-46 sign-in and an nsite write with Amber on Android. A NIP-46 sign-in on an iPhone, if a signer works.
- A large upload from a phone on a mobile network. "Add to Home Screen" on both phone types.
