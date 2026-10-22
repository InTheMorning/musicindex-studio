# ADR 0004: Server Sign-In With A Nostr Signer Or Email And Passphrase

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

The server shell has many users. The sign-in also opens the user's vault ([ADR 0003](0003-user-unlocked-secret-vault.md)).
Many V4V artists use a Nostr signer. Other artists use only email.
MSP 2.0 accepts NIP-98 without a URL binding, a method binding or a nonce. That is a known defect.

## Decision

1. The server shell accepts two sign-in methods:
   - A Nostr signer through NIP-07 or NIP-46.
   - An email address and a passphrase.
2. A Nostr sign-in uses a single-use challenge from the server. The signed event is bound to the URL and the method of the request.
3. The desktop shell has no sign-in. The vault passphrase protects it.

## Alternatives Considered

- An email magic link. Rejected: it cannot open a vault without a second secret.
- A passkey. Not selected for v1. A passkey does not give a stable vault key on each device of the user.

## Consequences

- A user with two sign-in methods needs a vault key that each method can open. The vault design must allow this.

## Invariants

- A session can write only to the feeds of its own user.
- The server rejects a sign-in event that it accepted before.
