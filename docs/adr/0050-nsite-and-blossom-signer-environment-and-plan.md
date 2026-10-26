# ADR 0050: The nsite And Blossom Signer, Host Environment And Signing Plan

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

[ADR 0048](0048-the-nsite-and-blossom-connectors.md) gives the nsite and Blossom connectors. [ADR 0044](0044-the-nsite-signer.md) gives two signing methods: "Key in the vault" and "My signer". Task 030 builds them.

The planning model found these facts in the code of Studio:

- The API makes a new set of connectors for each request (`Workspace::hosts`). A connector can thus keep a change in memory until the end of one write.
- `from_row` makes a connector from its collection row and the vault only. A connector has no access to the collection.
- `inventory::planned_path` gets the public address of a new file from `Connector::public_url` before the upload. The address of a Blossom blob comes from its SHA-256, not from its path.
- A Blossom upload needs an authorization event with the SHA-256 of the blob in an `x` tag (BUD-11). A streaming write gets a reader, and it knows the SHA-256 only after the last byte.
- Each check of the trait suite uses one connector on a new empty host.

On 2026-10-25 the operator made these decisions:

- "My signer" uses a NIP-07 browser extension only. NIP-46 comes in a later task.
- Studio reads the relay list and the server list of a key from `wss://purplepag.es` and `wss://user.kindpag.es`.
- With "Key in the vault", the user pastes a private key. Studio does not make a key.

The planning model added the technical choices below.

## Decision

### The Signer And The Key

1. "My signer" uses `window.nostr` of a NIP-07 browser extension: `getPublicKey` and `signEvent`. This replaces "NIP-07 or NIP-46" in ADR 0044 item 3. A NIP-46 signer needs its own record.
2. The lookup relays are `wss://purplepag.es` and `wss://user.kindpag.es`. Studio reads the newest kind 10002 and kind 10063 event of the key from them.
3. With "Key in the vault", the user pastes an `nsec` or 64 hexadecimal characters. The vault holds `{"secret_key": "<hex>"}`. Studio derives the public key and makes no key.

### The Host Environment

4. `connector::from_row_in(row, env)` makes a connector with a `HostEnv`: the open vault, the kept blobs of a Blossom host, and the `Signing` of a write. `from_row(row, vault)` stays, with an empty environment.
5. `Workspace::hosts` uses `from_row_in` with the kept blobs of each Blossom host. The write route adds the `Signing` of the write.

### The Trait

6. The trait gets four functions. Each has a default, so the local folder, S3 and SFTP do not change:
   - `finish`: publish the changes of this write step. The default does nothing.
   - `plan_events`: the unsigned Nostr events that a write with these changes signs. The default gives none.
   - `planned_url`: the public address of a planned file, from its path and its SHA-256. The default gives `public_url`.
   - `kept_blobs`: the blob records that Studio keeps for the host. The default gives `None`.
7. The pipeline calls `finish` for each touched host after the feed XML, before it records the state, and again after the deletions. A failed first `finish` changes no record, and the draft stays.
8. After a write, the pipeline puts the `kept_blobs` of each touched host into the collection, before the inventory scan.
9. The trait suite gets one check: a write and a deletion, each followed by `finish`, keep their effect.

### The Signing Plan

10. A planned event has a fixed time. The collection keeps one signing plan for each feed: the diff ID and the time. A diff of the same ID reuses the time while 10 minutes or more remain before its expiration. If less time remains, the diff makes a new time.
11. The diff gives each planned event of each host that the write touches, with no signature. The diff ID does not include the events, because the time and the diff give the same events again.
12. Before the first change on a host, the write makes the planned events again with the time of the plan:
    - For "My signer", each planned event must have a signed event with the same ID, a correct signature and the public key of the host. If one event is missing or different, the write stops.
    - For "Key in the vault", the core signs the planned events.
    - A missing or expired plan stops the write. The user reviews again.
    - Each connector keeps the events of its `plan_events` call, and uses them in the steps of the write. One `write` call does not know the other files of the write, so the connector cannot make a planned event again in that call.
13. An nsite manifest has the time `max(plan time, newest manifest time + 1)`. A manifest after the deletions has one second more. An authorization event has the plan time and an expiration one hour later. It names each blob of the write in `x` tags, and the domain of one server in a `server` tag (BUD-11).

### The nsite Connector

14. The connector shows the newest manifest with the changes of the write. A Studio path `a/b.mp3` is the manifest path `/a/b.mp3`. `finish` publishes one manifest with the `path` tags, the `server` tags, the `d` tag and the aggregate `x` tag of NIP-5A.
15. `finish` reads the newest manifest from each relay first. A manifest that is not the manifest of the start of the write gives `Conflict`, and Studio publishes nothing.
16. A write puts the bytes into a temporary file and calculates the SHA-256. Then it uploads the file to the first server and mirrors it to the other servers. A file stays in memory only when the bytes came as a slice.
17. The stamp and the version of a path are `sha256:<hex>`, the form of `digest::content_version`. The size comes from the write, or from a HEAD request to a server.

### The Blossom Connector

18. The collection keeps the kept blobs of a Blossom host: the path, the SHA-256 and the size.
19. The public address of a path is `<first server>/<sha256>.<extension>`.
20. A deletion removes the record of the path. Studio deletes the blob from each server only when no other record of the host has the same SHA-256.

## Alternatives Considered

- NIP-46 in task 030. Not chosen by the operator: it needs NIP-44 in the browser and a relay connection from the UI.
- The relays of MSP 2.0 as lookup relays. Not chosen by the operator.
- Studio makes a new key. Not chosen by the operator.
- The diff ID includes the planned events. Rejected: the events then change with each new diff time, and a user signature is not correct after a reload of the diff.
- A Blossom path that is the SHA-256 of the file. Rejected: the trait suite writes different bytes to one path.
- A `from_row` with the collection as a parameter. Rejected: each read route then needs the collection, and the connector could change the collection outside the pipeline.

## Consequences

- A diff that is open for more than 50 minutes gives a new plan time. A user with "My signer" then signs again.
- A write of a large file to nsite or Blossom needs free space for one temporary copy.
- The manifest of a site with many paths can be larger than the event limit of a relay. A test with 500 paths stays an open research item.

## Invariants

- Studio publishes no event that a key other than the key of the host signed ([ADR 0044](0044-the-nsite-signer.md)).
- A write with "My signer" changes nothing on a host before it has each planned event with a correct signature.
- Studio does not delete a blob that a manifest or a kept record of the host refers to ([ADR 0048](0048-the-nsite-and-blossom-connectors.md)).
