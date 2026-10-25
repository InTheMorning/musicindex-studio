# Task 029 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 029](../tasks/studio-v1-task-029-sftp-connector.md)
- Decisions: [ADR 0047](../adr/0047-the-sftp-connector.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 029a: no `sshd` child process stays after the tests, and CI installs `openssh-server`.
- 029b: the code has no `unsafe` block, and each response with a different ID gives an error.
- 029c: Studio does not log in after a host key that is not the accepted key. A failed write leaves no temporary file.
- 029c and 029d: no password, private key or passphrase is in a log, an error message, an API response, the collection row or the `Debug` text.
- 029d: a failed connection changes neither the vault nor the collection.

## Invariants

- Studio does not log in to a host whose key the user did not accept ([ADR 0047](../adr/0047-the-sftp-connector.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- The core has no async runtime ([ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [029a](../tasks/studio-v1-task-029a-sshd-harness.md) | Pass | The planning model added a retry to `TestSshd::start`. When `sshd` stops because a different process took the free port, the harness tries a new port, a maximum of five times. Packet 029c runs many SFTP tests at the same time, so this race is more probable there. The manual check passes: no `sshd` child process stays after three runs, and CI installs `openssh-server`. |
| 029b | Open | |
| 029c | Open | |
| 029d | Open | |
| 029e | Open | |
