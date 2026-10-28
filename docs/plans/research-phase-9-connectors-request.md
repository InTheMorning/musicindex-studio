# Research Request: Phase 9 Connectors

## Status

Received - 2026-10-25. Answered - 2026-10-25 by [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) and [ADR 0044](../adr/0044-the-nsite-signer.md). This request comes from the research repository at
`../musicindex-research`. That repository has no Git remote, and it is private.
Its index is `AGENTS.md`.

This document binds nothing in Studio. Studio records each decision in its own
ADR.

The research examined Studio at commit `6ceaddb`.

The report is `reports/2026-10-25-studio-phase-9-connectors.md` in the research
repository. The notes are in `research_notes/connectors/`.

## 1. The Connector Interface

| # | Result | Location |
|---|---|---|
| R1 | The upload step reads the full library file into memory, and `write` takes `&[u8]`. | `crates/core/src/pipeline.rs:1681-1689` |
| R2 | The confirmation step calls `read`, so it downloads each new file again. | `crates/core/src/pipeline.rs:1725-1739` |
| R3 | The inventory reads a full host file to calculate its SHA-256. | `crates/core/src/inventory.rs:233-243` |
| R4 | `from_row` accepts only `local` and does not use the vault. The API accepts only a local folder. | `crates/core/src/connector.rs:238-250`, `crates/api/src/routes.rs:193-234` |
| R5 | No test examines a connector through the trait only. The Phase 9 plan needs a shared test set. | `crates/core/src/connector/local.rs:252-355` |
| R6 | `ConnectorErrorKind` has no type for refused credentials or for a network that is not available. | `crates/core/src/connector.rs:104-118` |

The research proposes these changes before the first remote connector:

1. A test module of the trait in `studio-core`, behind the `testing` feature.
   `LocalFolder` uses it first. Then each Phase 9 connector uses it with its
   local test server.
2. A streaming write from a reader with a known length. The write of a byte
   slice stays for the feed XML.
3. `stat` in the confirmation step for media files. The test of the public
   address becomes a different step.
4. An optional function that gives the SHA-256 of a host file when the host
   has it with no read.
5. The error types `Unauthorized` and `Unavailable`.

## 2. Each Connector

| Connector | Client | Version marker | Conditional write | Redirect | Audio |
|---|---|---|---|---|---|
| SFTP | `ssh2` 0.9.6, blocking | SHA-256 | No. Temporary file and `posix-rename`. | No | Yes |
| S3 and R2 | `rusty-s3` with `ureq` | ETag | Yes, `If-Match` and `If-None-Match` | AWS website endpoint only | Yes |
| Git pages | Forge REST API with `ureq` | Blob SHA | Yes, with the current `sha` | Codeberg only | No |
| nsite | `nostr` and `tungstenite` | Manifest event ID, SHA-256 of each path | No compare-and-swap on relays | No | Yes |
| Blossom | `ureq` and `nostr` | SHA-256 | Not necessary | No | Media only |

The important items for the ADRs are these:

- `ssh2` is the only examined SFTP crate that is blocking. The ADR 0015 cause
  for a blocking trait thus stays correct. `ssh2` has no function for
  `posix-rename@openssh.com`, so Studio must add it.
- AWS and R2 document conditional PutObject. The R2 documentation does not name
  a conditional DeleteObject. The research proposes a test of the provider when
  the user adds the host.
- The `s3s-fs` test server needs Rust 1.96, so it operates as a different
  process. It does not obey `If-Match` on DeleteObject.
- A Git pages host is not correct for audio. GitHub refuses a file of more than
  100 MiB, and a Pages site has a limit of 1 GB.
- `gix` cannot push. The forge REST APIs need no new dependency.
- A Pages site changes some time after a write. ADR 0036 item 15 does not say
  if its read is the API read or the read of the public address.
- NIP-5A has the manifest types 15128, 35128 and 5128. Type 34128 is
  deprecated. MSP 2.0 uses type 1063, which is not a replaceable event.
- The "stable pointer event" text of ADR 0006 can name the NIP-5A manifest.
- An nsite connector signs events in Rust. The signing key in the vault changes
  what the vault holds, so it needs an ADR. ADR 0013 item 19 stays correct,
  because the proposal does not use NIP-44 in Rust.

## 3. Proposed Sequence

The research proposes this sequence: the interface changes, S3 and R2, SFTP,
Git pages, then nsite and Blossom. The S3 write has conditions on the host and the
fewest open questions. nsite has the most open questions.

## 4. Open Questions For Studio

- Does Studio change the trait in one task before the first remote connector?
- Which hosts can hold audio?
- Does the vault hold a Nostr signing key for nsite?
- What is the time limit of the test of the public address for Git pages?
