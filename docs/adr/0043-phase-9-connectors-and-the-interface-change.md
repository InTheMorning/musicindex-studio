# ADR 0043: The Phase 9 Connectors And The Interface Change

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

Phase 9 adds the remote connectors of [ADR 0006](0006-connectors-and-the-v1-set.md). The [research request on the Phase 9 connectors](../plans/research-phase-9-connectors-request.md) examined Studio at commit `c05cbed`. It found these limits in the connector interface:

- The upload step reads the full library file into memory, because `write` takes `&[u8]`.
- The confirmation step of [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 15 calls `read`, so it downloads each new file again.
- `from_row` accepts only `local`, and it does not use the vault.
- No test examines a connector through the trait only.
- `ConnectorErrorKind` has no type for refused credentials or for a network that is not available.

The research also found that a Git pages host is not correct for audio. GitHub refuses a file of more than 100 MiB, and a Pages site has a limit of 1 GB. A Pages site changes some time after a write, and GitHub documents no build time.

On 2026-10-25 the operator made these decisions:

- One task changes the connector interface before the first remote connector.
- Each v1 host can hold audio. A Git pages host cannot.
- The confirmation uses `stat` first, and then a different step reads the public address.
- When the confirmation does not pass, Studio does no deletion. The files to delete stay on the host.
- The Git pages connector moves after v1.
- Studio writes its own small Blossom test server. `route96` has no license, so Studio does not use it.

The planning model added the technical choices below. [ADR 0044](0044-the-nsite-signer.md) gives the signer of the nsite connector.

## Decision

### The Interface Change

1. One task changes the connector interface before the first remote connector. `LocalFolder` uses each change first.
2. `studio-core` gets a test module of the trait, behind the `testing` feature. The module examines a connector through the trait only. `LocalFolder` passes it. Each Phase 9 connector passes it against its local test server.
3. A connector gets a streaming write. It reads the bytes from a reader with a known length, and it takes the expected version marker of [ADR 0015](0015-connector-interface-and-write-pipeline.md) item 3. The upload of a library file uses the streaming write. The write of a byte slice stays for the feed XML.
4. `ConnectorErrorKind` gets `Unauthorized` for refused credentials and `Unavailable` when Studio cannot connect to the host.
5. `from_row` makes each connector type. It gets the secrets of a remote connector from the open vault ([ADR 0003](0003-user-unlocked-secret-vault.md)). A remote connector cannot open while the vault is locked.
6. A connector can give the SHA-256 of a host file with no read, when the host has it. The default gives no value, and the inventory then reads the file as before.

### The Confirmation Before The Deletions

This section replaces the confirmation of [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 15. The sequence of item 15 does not change: the uploads and the copies, then the feed XML, then the deletions.

7. Studio calls `stat` on each new file through its connector. The step passes when the size is the size of the library file. Studio does not read the file again.
8. Then Studio requests the public address of each new file and of the feed XML. The step passes when the host gives a success status. A host with no public address skips this step.
9. When step 7 or step 8 does not pass, the feed XML and the new files stay on the host. Studio does no deletion. Each deletion stays a host operation, and the user can start it again:
   - The draft then starts from the written feed XML, and it keeps each deletion.
   - The draft gets a "confirm" operation for each new file. A confirm operation keeps the host and the path of its file.
   - The next write needs a new approval. It does steps 7 and 8 again for each confirm operation, before the deletions.

### The Hosts

10. The v1 connectors are the local folder, SFTP, S3 and R2, nsite and Blossom, and the optional self-hosted file host. The Git pages connector moves after v1. This replaces the Git pages entry of ADR 0006 item 3.
11. Phase 9 builds the interface change first. Then it builds S3 and R2, then SFTP, then nsite and Blossom.
12. Each v1 connector can hold audio. A Blossom host holds media only, and it does not hold a feed XML. A Git pages connector, after v1, holds only the feed XML and the art.
13. The stable pointer of an nsite site is the NIP-5A manifest, event type 35128. This replaces the "stable pointer event" text of ADR 0006, Consequences.
14. These are the version marker of each connector, and the redirects that it can give:

| Connector | Version marker | HTTP 301 redirect |
| --- | --- | --- |
| Local folder | SHA-256 of the content | No |
| SFTP | SHA-256 of the content | No |
| S3 and R2 | ETag | AWS website endpoint only |
| nsite | Event ID of the manifest, and the SHA-256 of each path | No |
| Blossom | SHA-256 of the content | No |

15. The Blossom tests use a small Blossom server in the test code of Studio. It has only the endpoints that the connector uses.

## Alternatives Considered

- Change the interface with each connector. Not chosen by the operator: the trait and `LocalFolder` then change many times.
- Let a Git pages host hold audio, with a warning about the size limits. Not chosen by the operator.
- Confirm through the host API only. Not chosen by the operator: a host can give the old file at its public address after the API has the new file.
- Confirm through the public address only. Not chosen by the operator: a host with no public address then cannot confirm.
- Write the old feed XML again when the confirmation does not pass. Not chosen by the operator: the second write can also fail, and it is more complex.
- A Git pages connector in v1 with a time limit for the public address. Not chosen by the operator. The time limit is decided with the Git pages connector.
- `route96` as the Blossom test server. Rejected: it has no license file.

## Consequences

- The pipeline confirms a new media file with no download.
- A feed can have files that Studio wrote but could not confirm. The inventory shows them, and the deletions wait.
- The local folder connector has no public address in most collections, so it skips step 8.
- A user with a feed on GitHub Pages or Codeberg Pages cannot write it in v1. Studio reads the feed, and the feed is read-only ([ADR 0006](0006-connectors-and-the-v1-set.md) item 5).
- The research gives open items that need a live host, a container or a different computer. The [phase plan](../plans/studio-v1-phase-plan.md) lists them under Phase 9.

## Invariants

- Each connector passes the test module of the trait.
- Studio does no deletion before each new file passes the confirmation.
- A connector gets its secrets from the open vault only.
