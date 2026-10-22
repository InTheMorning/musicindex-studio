# Studio v1 Task 009: The One-Question Wizard

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 4.
Decisions: [ADR 0005](../adr/0005-collection-and-first-run-discovery.md), [ADR 0019](../adr/0019-select-a-feed-from-the-host.md) and [ADR 0020](../adr/0020-first-run-asks-one-question.md).

## Goal

Replace the first-run wizard with the one-question wizard of ADR 0020.

## Steps

1. Add an HTTP client to `studio-core`. Select one that needs no system TLS library.
2. Read a public feed by its address, with the same reader as a connector.
3. Follow the links of a feed one level (ADR 0020 items 3 to 6). Get the address of a `podcast:remoteItem` from musicindex when it has only a `feedGuid`.
4. Group the feeds by host, and mark the known services as read-only (ADR 0020 items 7 to 10).
5. Add API routes: discover from addresses, search musicindex, and add a selection of feeds.
6. Make the vault when the user first adds a secret (ADR 0020 items 13 to 15).
7. Make the UI: the question, the address field, the musicindex search, the result screen and the MSP 2.0 link.
8. The MSP 2.0 import puts each edit token in the vault, not in the collection (ADR 0003).

## Constraints

- The API reads public feeds and the musicindex API. The UI makes no request to other servers (ADR 0016).
- Tests do not use the network. They use a local test server.

## Acceptance Criteria

Mechanical:

- A publisher feed address gives the publisher feed and each feed in its `podcast:remoteItem` list.
- An album address with `podcast:publisher` gives the album, its publisher feed and the other feeds of that publisher.
- Studio does not follow a link from a feed that it found through a link.
- A discovery stops at 100 feeds and reports this.
- A `remoteItem` with only a `feedGuid` gets its address from a musicindex lookup.
- Feeds on `wavlake.com`, `feeds.fountain.fm` and `serve.podhome.fm` are read-only. Feeds on other hosts offer a connector.
- A feed that is already in the collection is marked and is not added again.
- The wizard makes no vault until the user adds a secret.
- No API response contains an MSP 2.0 edit token.

Visual:

- The first screen asks one question.
- The result screen groups the feeds by host, and all feeds are selected at the start.
