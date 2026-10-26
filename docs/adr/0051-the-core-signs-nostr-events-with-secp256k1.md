# ADR 0051: The Core Signs Nostr Events With secp256k1

- Status: Accepted
- Date: 2026-10-25
- Owner: operator

## Context

[ADR 0044](0044-the-nsite-signer.md) item 2 tells the core to sign an nsite event with the `nostr` crate, with its default features only. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) item 1 gives the crates of nsite and Blossom: `nostr`, `tungstenite` and `ureq`.

On 2026-10-25, packet 030a stopped on an escalation condition. In `nostr` 0.45, these functions need the `os-rng` feature:

- `Keys::sign_schnorr`.
- The `SignEvent` trait for `Keys`.
- `EventBuilder` functions that sign.

The `os-rng` feature enables the `rand` crate. With the default features, `Keys::sign_schnorr_with_aux_rand` signs. It needs a `Secp256k1` context, and `nostr` does not export the `secp256k1` crate or its own context.

The planning model made a probe on 2026-10-25. The probe made an `UnsignedEvent` with a fixed time and a `d` tag. It signed the event with `sign_schnorr_with_aux_rand` and a context of `secp256k1` 0.30. `Event::verify` accepted the result.

On 2026-10-25 the operator chose to add the `secp256k1` crate.

## Decision

1. The core depends on `secp256k1` 0.30, with no default features and the `std` feature. This is the version and the feature set that `nostr` 0.45 uses, so the compilation gets no new code. This adds `secp256k1` to the crates of nsite and Blossom in ADR 0045 item 1.
2. The core signs an event in one function, `nostr_events::sign_with_keys`:
   - It makes a context with `Secp256k1::signing_only()`.
   - It gets 32 bytes from `getrandom::fill` as the auxiliary random data of BIP-340.
   - It signs the event ID with `Keys::sign_schnorr_with_aux_rand` and adds the signature with `UnsignedEvent::add_signature`.
3. `nostr` keeps its default features only (ADR 0044 item 2). The core does not enable `rand` or `os-rng`.

## Alternatives Considered

- Enable the `os-rng` feature of `nostr`. Not chosen by the operator. It adds a second random source to the signature, and it needs a change of ADR 0044 item 2.
- Write a BIP-340 signature in Studio. Rejected: a signature is cryptographic code, and a tested crate gives it.

## Consequences

- An update of `nostr` to a new `secp256k1` version needs an update of the direct dependency in the same change. If not, the two versions do not agree, and the compilation fails.
- The vault and the signature use one random source: `getrandom`. `tungstenite` uses `rand` 0.10 for the masks of its WebSocket frames. The `rand` feature of `nostr` stays off.
- The license of `secp256k1` and `secp256k1-sys` is CC0-1.0.

## Invariants

- The core has one function that signs a Nostr event with a key.
- `nostr` has its default features only.
