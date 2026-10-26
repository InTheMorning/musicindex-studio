# ADR 0044: The nsite Signer

- Status: Accepted. [ADR 0050](0050-nsite-and-blossom-signer-environment-and-plan.md) limits "My signer" of item 3 to NIP-07 on 2026-10-25.
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

An nsite write publishes a new NIP-5A manifest, event type 35128 ([ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) item 13). Each manifest is a Nostr event that the key of the site signs.

[ADR 0003](0003-user-unlocked-secret-vault.md) lets the vault hold Nostr signing access. [ADR 0013](0013-vault-format-and-cryptography.md) item 19 says that the Rust code does not use NIP-44. Signing an event is not NIP-44, so that item does not change.

The research found that a relay has no compare-and-swap. A newer replaceable event replaces an older one. The relays of one user can disagree, and a gateway can show a manifest that is not current.

On 2026-10-25 the operator selected two signing methods. The user selects one for each nsite host.

The planning model added the technical choices below.

## Decision

1. When the user adds an nsite host, the user selects its signing method: "Key in the vault" or "My signer". The host keeps the public key of the site.
2. With "Key in the vault", the vault holds the private key of the site as a secret of the connector. The core signs the manifest in Rust with the `nostr` crate, with its default features only.
3. With "My signer", the core makes the manifest event with no signature. The UI asks the user's NIP-07 or NIP-46 signer to sign it. The API gives the signed event to the pipeline, and the write then continues.
4. Before Studio publishes a manifest, it makes sure that the public key of the signed event is the public key of the host. Studio refuses a different key.
5. A manifest write reads the current manifest from each relay of the host. It signs a new manifest with a `created_at` that is larger than each current one. It publishes the manifest to each relay, and then it reads it again from each relay.
6. Studio shows a site as inconsistent until each relay gives the same manifest.

## Alternatives Considered

- Only the key in the vault. Not chosen by the operator: a user who keeps the key in a signer cannot then use nsite.
- Only the user's signer. Not chosen by the operator: each write then needs a signer step.

## Consequences

- The vault can hold a Nostr private key. A breach of a server with an open vault can expose that key ([ADR 0003](0003-user-unlocked-secret-vault.md)).
- A write with "My signer" has a step in the UI between the approval of the diff and the publication of the manifest. The approved diff ID of [ADR 0007](0007-draft-diff-write-and-announce.md) stays the condition of the write.
- A concurrent write by a different client can replace the manifest with no error. The read after the write finds it, and Studio shows the site as inconsistent.
- Each signing method needs its own tests.

## Open Questions

- The desktop shell has no browser extension, so NIP-07 is possibly not available there. The Desktop Shell phase decides if the desktop shell uses NIP-46 only.

## Invariants

- No private key of a site is in a log, an API response or the collection database.
- Studio publishes no manifest that a key other than the key of the host signed.
