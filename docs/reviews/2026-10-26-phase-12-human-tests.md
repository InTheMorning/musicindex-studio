# Phase 12 Human Tests

- Date: 2026-10-26
- Task: [task 037](../tasks/studio-v1-task-037-runbook-and-human-tests.md)
- Runbook: [studio-server](../runbooks/studio-server.md)

The operator does each test. The planning model records the device, the browser, the date and the result. A test that did not occur is an open gate.

## Before The Tests

Each step is done on 2026-10-27:

- The server block of `studio.musicindex.org` has `client_max_body_size 9m;` (commit `3af81a2` of `musicindex`). `pushsite.sh` installed it, and `nginx -t` passed.
- The image `ghcr.io/inthemorning/studio-server:0.1.0-rc.2` runs on the VPS, through `tools/deploy-server.sh` ([ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md)). The revision label is `af83e56`, and the log has the start line of `0.1.0-rc.2`.
- 2026-10-28: the operator deployed `0.1.0-rc.3` with `tools/deploy-server.sh`. The file `/ui/app.js` that `studio.musicindex.org` gives is the same as at `9e0eb5f`, the commit of the tag `v0.1.0-rc.3`. The human tests of tasks 046 and 047 use this version.
- 2026-10-28: the operator deployed `0.1.0-rc.4` with `tools/deploy-server.sh`. The file `/ui/app.js` that `studio.musicindex.org` gives is the same as at `3767bd2`, the commit of the tag `v0.1.0-rc.4`. The human tests of tasks 048 to 053 use this version.
- 2026-10-30: the operator made a backup of the rc.4 data folder, and deployed `0.1.0-rc.5` with `tools/deploy-server.sh`. The file `/ui/app.js` that `studio.musicindex.org` gives is the same as at `c9580b7`, the commit of the tag `v0.1.0-rc.5`. The human tests of tasks 054, 055 and 056, and test 3 of task 028, use this version.
- From outside, `/` and the manifest gave 200, and `/auth/v1/session` gave the server mode with sign-up by invite code. A `PUT` of 5 MB got 403 from Studio, not 413 from nginx.
- The operator made one invite code.

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
