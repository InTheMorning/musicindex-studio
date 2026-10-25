# ADR 0006: Connectors And The v1 Set

- Status: Accepted. [ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) moves the Git pages connector of item 3 after v1 on 2026-10-25. It also names the NIP-5A manifest, type 35128, as the stable pointer of Consequences.
- Date: 2026-10-22
- Owner: operator

## Context

Studio writes feeds on the hosts of the artist.
The research shows known write paths only for some hosts.
Wavlake hosts approximately 88% of the index, but the research found no write API for it.
Castopod makes its feed from its database, so it cannot accept a feed file.
The research is in [the baseline research note](../research/2026-10-22-studio-baseline-research.md).

## Decision

1. A connector is a core module that reads and writes the files of one host type. The files are feed files, audio and art ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
2. Each connector can:
   - read a file and give a version marker, such as an ETag, a hash or a commit.
   - write a file only if the version marker has not changed.
   - give the public URL of a file.
   - list the files of the host ([ADR 0019](0019-select-a-feed-from-the-host.md)).
3. The v1 connectors are:
   - a local folder, and a remote folder through SFTP.
   - object storage through the S3 API, which includes Cloudflare R2.
   - Git-based pages, such as GitHub Pages and Codeberg Pages.
   - Nostr hosts: Blossom and nsite.
   - the optional self-hosted file host that Studio sets up ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
4. MSP 2.0 hosting is not a v1 connector.
5. A host without researched write access has no connector. Its feeds are read-only in Studio.

## Alternatives Considered

- One generic HTTP PUT connector for all hosts. Rejected: each host has a different authentication and version method.
- An MSP 2.0 hosting connector in v1. Rejected by the operator.

## Consequences

- A Blossom upload makes a new hash for each version. The Nostr connector must update the stable pointer event after each upload.
- Each new connector needs research of its write path before a design.

## Invariants

- A connector does not write when the version marker has changed. It reports a conflict.
- A connector gets its secrets from the vault only. It does not store them.
