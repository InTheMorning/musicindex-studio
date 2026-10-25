# Studio v1 Task 029: The SFTP Connector

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 9, the third task.
Decisions: [ADR 0047](../adr/0047-the-sftp-connector.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md).

## Goal

A user can add an SFTP server as a host, with a private key, a password or ssh-agent. Studio connects only to a host key that the user accepted. It writes each file with a temporary file and an OpenSSH rename.

## Packets

The planning model writes the packets after task 028 is complete, so that they use what task 028 shows. The packets go in this sequence:

1. 029a: the `sshd` test harness. A test starts the system `sshd` on a free loopback port with a temporary configuration, and CI installs `openssh-server`.
2. 029b: the SFTP version 3 requests over an `ssh2` channel, with the OpenSSH extensions.
3. 029c: the SFTP connector, with the trait suite against `sshd`.
4. 029d: the host key confirmation and the host route.
5. 029e: the SFTP host form.

## Acceptance Criteria

Mechanical:

- The SFTP connector passes the test module of the trait against `sshd`.
- A server with no `posix-rename@openssh.com` cannot be added.
- A changed host key gives `Unauthorized`, and Studio does not log in.
- No password, private key or passphrase is in the collection row, a log or an API response.
- The four commit checks pass, in CI too.

Visual:

- The Hosts page shows the SFTP form, the fingerprint confirmation and an added SFTP host, in the dark theme and in the light theme. The planning model checks it with `tools/drive-ui.mjs`.

Human test, open until the operator can use a real SFTP host:

- The operator adds a web host with SFTP and a public address. Studio writes a release to it, and a podcast app plays a track.
