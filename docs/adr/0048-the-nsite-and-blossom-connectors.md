# ADR 0048: The nsite And Blossom Connectors

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

[ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) makes the NIP-5A manifest, type 35128, the stable pointer of an nsite site. A Blossom host holds media only. [ADR 0044](0044-the-nsite-signer.md) lets the user select "Key in the vault" or "My signer" for each nsite host. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) selects the crates and the test servers.

The research found these facts:

- An nsite manifest maps each path to the SHA-256 of a blob. A change of one path is a new manifest for the whole site.
- A relay has no compare-and-swap. The relays of one user can disagree.
- A Blossom upload and a Blossom delete need an authorization event, type 24242. One event can name more than one blob with `x` tags.
- A gateway has a default size limit of 128 MB.
- MSP 2.0 uses the first 8 hexadecimal characters of the feed GUID as the `d` tag of a site.

On 2026-10-25 the operator made these decisions:

- A host gets its relays and its Blossom servers from the lists of its key, and the user can change them.
- A new site uses the `d` tag form of MSP 2.0.
- Blossom uses the same two signing methods as nsite.
- The default gateway is `nsite.lol`, and the user can change it for each host.

The planning model added the technical choices below.

## Decision

### The Hosts

1. The kind `nsite` holds the feed XML and its media in one named site. Its collection row has the public key, the `d` tag, the relays, the Blossom servers, the gateway and the signing method.
2. The kind `blossom` holds media only. Its collection row has the public key, the Blossom servers and the signing method. A new collection table keeps the path, the SHA-256 and the size of each blob that Studio put on the host.
3. When the user adds a host, Studio reads the NIP-65 relay list (kind 10002) and the Blossom server list (kind 10063) of the key. It fills the form with them.
4. The `d` tag of a new site is the first 8 hexadecimal characters of the feed GUID. A site that exists keeps its `d` tag.
5. The public address of a path is `https://<pubkeyB36><d>.<gateway>/<path>` for nsite, and `<server>/<sha256>.<extension>` for Blossom.

### One Change Of The Host For Each Write Step

6. The trait gets `finish`, with a default that does nothing. The pipeline calls it for each host that a write step touched. An nsite connector collects the changes of the paths in memory, and `finish` publishes them as one manifest.
7. The pipeline calls `finish` after the uploads and the feed XML, before the confirmation. It calls `finish` again after the deletions.

### The Operations

8. The version and the stamp of an nsite path are the SHA-256 in its `path` tag. A write first reads the newest manifest from each relay. It compares the path with `Expected`, and gives `Conflict` when they are different.
9. `finish` signs the new manifest with a `created_at` that is larger than each current one. It publishes the manifest to each relay, and then reads it again from each relay. A newer manifest from a different client gives `Conflict`.
10. Each blob goes to the first Blossom server, and then to each other server of the host with a mirror request. The size of an nsite path comes from a HEAD request to a Blossom server.
11. An nsite deletion removes the path from the manifest. Studio does not delete a blob from a Blossom server when a manifest refers to it.
12. A Blossom write uploads the blob and records it in the collection table. A Blossom deletion sends a delete request with an authorization event, and removes the record.
13. `sha256` gives the SHA-256 from the manifest or from the collection table, with no read. `can_redirect` gives false for both kinds.

### The Signer

14. With "Key in the vault", the core signs each event in Rust.
15. With "My signer", the diff gives each event that the write will sign, with no signature: each manifest and one authorization event for the blobs of each Blossom server. The UI asks the signer to sign them when the user starts the write. The write request sends the signed events with the diff ID.
16. The write uses a signed event only when its ID is the ID of the planned event. A missing event or a different event stops the write before any change on a host.
17. A planned event has a fixed `created_at` and, for an authorization event, an expiration one hour later. A diff that is older than the expiration needs a new review.

## Alternatives Considered

- One manifest for each path. Rejected: a release with many files would publish many manifests, and "My signer" would need a signer step for each one.
- A signer step in the middle of a write. Rejected: the write must then stop and continue, with a state that the collection must keep.
- The user types each relay and each server. Not chosen by the operator.
- A `d` tag that the user selects. Not chosen by the operator.
- Blossom with "Key in the vault" only. Not chosen by the operator.

## Consequences

- A write with deletions publishes two manifests.
- A site can be inconsistent for some time, when a relay did not get the newest manifest. Studio shows it as inconsistent until each relay gives the same manifest ([ADR 0044](0044-the-nsite-signer.md) item 6).
- An audio file larger than the limit of a gateway cannot play through that gateway.
- Whether podcast apps and Podcast Index accept a feed address on a gateway stays an open research item.

## Invariants

- Studio publishes no manifest that a key other than the key of the host signed ([ADR 0044](0044-the-nsite-signer.md)).
- Studio does not delete a blob that a manifest of the host refers to.
