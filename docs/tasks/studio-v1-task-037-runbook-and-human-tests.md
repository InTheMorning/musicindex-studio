# Studio v1 Task 037: The Server Runbook And The Human Tests

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 12, milestone 12d.
Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md) and [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md).
Sequence: after tasks 034, 035 and 036.

## Goal

A runbook tells an operator how to run `studio-server` behind a reverse proxy, as a public service or as a personal server. The operator deploys it on `studio.musicindex.org` and does the human tests of Phase 12 with real phones.

## Files

Inspect: `crates/server/src/config.rs` (each option), `crates/server/src/main.rs`, the nginx vhost of `studio.musicindex.org` in `../musicindex` (one proxy location to port 8080), `docs/README.md`.

Change: the new file `docs/runbooks/studio-server.md`, `docs/README.md`, the review record of the human tests in `docs/reviews/`.

Do not change: code. A defect that a human test finds gets its own packet.

## The Runbook

The planning model writes it from the code as it is at that time. It has these sections:

1. The purpose and the two uses: a public service and a personal server.
2. The build and the install of the binary, and a systemd unit with `--data-dir`, `--listen` and `--public-origin`.
3. The reverse proxy: TLS, the `Host` header, the client address header and `--client-address-header`, and a body limit of 9 MiB for the parts of ADR 0059.
4. Each option of `studio-server serve`, with its default.
5. Invite codes and open sign-up.
6. The backup of `<data>/` while the server runs, and the restore. The vault files and `server.sqlite` hold no plain secret. But a copy of the disk with an app token gives the host secret of an active grant (ADR 0058 consequences).
7. Memory: the Argon2id limit, and no core dumps for the process (`LimitCORE=0`).
8. What the guard of ADR 0057 refuses, and the error that users see.
9. The update of the binary and the effect of a restart: each vault locks, each session stays.

## Human Tests

The operator does each test. The planning model records the result of each one in a review. A test that did not occur is an open gate.

1. Deploy on `studio.musicindex.org`. Sign up with an invite code and an email address.
2. Passkey on an iPhone (Safari, iCloud Keychain): add it, sign out, sign in with it, write a change. Sign in with it on a second Apple device.
3. Passkey on an Android phone (Chrome, Google Password Manager): the same steps.
4. NIP-46 with Amber on Android: sign up, sign in, open the vault, and write to an nsite test host with "My signer".
5. NIP-46 on an iPhone with a signer that the operator selects, if one works. Record the signer and the result.
6. NIP-07 on a computer: sign up and sign in.
7. Upload a WAV file of more than 200 MiB from a phone on a mobile network. Change the network one time during the upload.
8. "Add to Home Screen" on both phone types. Studio opens with no browser bar.
9. Leave the phone for 15 minutes. The vault locks, the session stays, and the next write asks for an unlock.
10. A connected app does "Go live" and "End the show" against the server with no browser session open. This test waits for v4vmm or the live publisher.

## Acceptance Criteria

Mechanical: none. This task changes no code.

Visual and human:

- The runbook exists and agrees with the options of the binary.
- The review records the result of each human test, with the device, the browser and the date.
