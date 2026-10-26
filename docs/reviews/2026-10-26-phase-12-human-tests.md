# Phase 12 Human Tests

- Date: 2026-10-26
- Task: [task 037](../tasks/studio-v1-task-037-runbook-and-human-tests.md)
- Runbook: [studio-server](../runbooks/studio-server.md)

The operator does each test. The planning model records the device, the browser, the date and the result. A test that did not occur is an open gate.

## Before The Tests

- Add `client_max_body_size 9m;` to the server block of `studio.musicindex.org` in `../musicindex/nginx-site` (runbook section 3).
- Install the binary and the unit of runbook section 2, with `--client-address-header X-Real-IP`.
- Make one invite code (runbook section 5).

## Results

| Test | Device and browser | Date | Result |
| --- | --- | --- | --- |
| 1. Deploy on `studio.musicindex.org`. Sign up with an invite code and an email address. | | | Open |
| 2. Passkey on an iPhone with Safari and iCloud Keychain: add, sign out, sign in, write. Sign in on a second Apple device. | | | Open |
| 3. Passkey on an Android phone with Chrome and Google Password Manager: the same steps. | | | Open |
| 4. NIP-46 with Amber on Android: sign up, sign in, open the vault, write to an nsite test host with "My signer". | | | Open |
| 5. NIP-46 on an iPhone with a signer of the operator, if one works. | | | Open |
| 6. NIP-07 on a computer: sign up and sign in. | | | Open |
| 7. Upload a WAV file of more than 200 MiB from a phone on a mobile network, with one change of the network. | | | Open |
| 8. "Add to Home Screen" on the two phone types. Studio opens with no browser bar. | | | Open |
| 9. Do not use the phone for 15 minutes. The vault locks, the session stays, and the next write asks for an unlock. | | | Open |
| 10. A connected app does "Go live" and "End the show" with no browser session open. It waits for v4vmm or the live publisher. | | | Open |
