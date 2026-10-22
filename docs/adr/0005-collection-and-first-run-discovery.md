# ADR 0005: The Collection And First-Run Discovery

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

Studio keeps the full collection of feeds for each user ([ADR 0001](0001-one-core-with-desktop-and-server-shells.md)).
A new user has feeds on one or more hosts.
Stophammer has no route that finds feeds by owner, npub or feed URL.

## Decision

1. The collection is a list of feeds. Each entry records the feed GUID, the public URL, the connector that writes it and the last known state.
2. [ADR 0020](0020-first-run-asks-one-question.md) replaces items 2 and 3. A first-run wizard adds feeds to the collection. It has three methods:
   - The user pastes one or more feed URLs.
   - The user searches musicindex (Stophammer) by artist or feed name and selects results.
   - The user imports an MSP 2.0 backup.
3. After a feed is found, the wizard asks for a connector that can write it. A feed without a connector stays in the collection as read-only.
4. Write access is the proof of control. If the connector can write the file, Studio accepts that the user controls the feed. Studio adds no proof token to the feed.

## Alternatives Considered

- Scan a connected host for feed files. Not selected for v1. [ADR 0019](0019-select-a-feed-from-the-host.md) replaces this: the wizard lists the feed files of a host.
- Require a `podcast:txt` proof token. Rejected: write access gives the same proof with no change to the feed.

## Consequences

- The collection keeps no secret. The secrets are in the vault ([ADR 0003](0003-user-unlocked-secret-vault.md)).
- The musicindex search method needs a network connection. The other two methods do not.

## Invariants

- An import does not change the feed GUID or an item GUID.
