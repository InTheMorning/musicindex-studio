# ADR 0045: The Phase 9 Crates, Test Servers And Host Forms

- Status: Accepted. [ADR 0051](0051-the-core-signs-nostr-events-with-secp256k1.md) adds `secp256k1` to item 1 on 2026-10-25.
- Date: 2026-10-25
- Owner: operator

## Context

[ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) sets the v1 connectors and the sequence of Phase 9: S3 and R2, then SFTP, then nsite and Blossom. Task 027 changed the connector interface. Each connector needs a client crate, a local test server and a form that adds the host.

The [research request on the Phase 9 connectors](../plans/research-phase-9-connectors-request.md) compared the crates and the test servers. `s3s-fs` needs Rust 1.96. At the time of the research, Studio built with Rust 1.93. `route96` has no license file.

On 2026-10-25 the operator selected the items below. The operator named no real host for a human test.

## Decision

1. The client crates are these:
   - S3 and R2: `rusty-s3` for the request signing and the addresses, with `ureq` for the HTTP requests.
   - SFTP: `ssh2`, with its `vendored-openssl` feature.
   - nsite and Blossom: `nostr` with its default features, `tungstenite` for the relays, and `ureq` for Blossom.
2. The core does not enable the `nip44` or the `nip46` feature of `nostr` ([ADR 0013](0013-vault-format-and-cryptography.md) item 19).
3. Each connector passes the test module of the trait ([ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) item 2) against these local test servers:
   - S3 and R2: a small S3 server in the `testing` module of `studio-core`. It has only the routes that the connector uses.
   - SFTP: the OpenSSH `sshd` of the computer. Each test starts it on a free loopback port, with a temporary configuration and key. CI installs `openssh-server`. When the computer has no `sshd`, the SFTP tests stop with a message that names the missing program.
   - Blossom: a small Blossom server in the `testing` module ([ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) item 15).
   - nsite: the same Blossom server, and a small blocking relay in the `testing` module.
4. Each connector task adds the form of its host type to the Hosts page. The form puts each secret into the vault and the other settings into the collection row ([ADR 0003](0003-user-unlocked-secret-vault.md)).
5. A test against a real host is a human test. Each connector task names it. It stays an open item of Phase 9 until the operator can use a real host of that type.

## Alternatives Considered

- No `ssh2`, so no C code. Not chosen by the operator: no blocking SFTP client in pure Rust exists, and an own SSH transport is a large risk.
- `s3s-fs` as a different process. Not chosen by the operator: it needs a second toolchain, and it ignores `If-Match` on DeleteObject.
- A change of the Studio toolchain to Rust 1.96 or later for `s3s-fs`. Not chosen by the operator. [ADR 0049](0049-the-rust-toolchain-is-pinned.md) later pinned Rust 1.99.1 for a different reason, and the choice of the fake S3 server stays.
- A Docker SFTP server. Not chosen by the operator: it needs Docker on each computer and in CI.
- One UI task at the end of Phase 9 for all the host forms. Not chosen by the operator: no connector can then be used before the end of the phase.

## Consequences

- The core compiles libssh2 and OpenSSL. A build on Windows and macOS is an open item until the Desktop Shell phase.
- CI installs `openssh-server`.
- A fake server finds only the behavior that it has. A defect that only a real provider shows waits for the human test.

## Invariants

- The core has no async runtime ([ADR 0015](0015-connector-interface-and-write-pipeline.md) item 1).
- No test sends a request to a host outside the computer.
