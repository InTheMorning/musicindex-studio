# Studio v1 Task 009: More Discovery Methods

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 4.
Decision: [ADR 0005](../adr/0005-collection-and-first-run-discovery.md).

## Goal

Add the three discovery methods of ADR 0005 that task 008 did not add:

1. The user pastes one or more feed URLs.
2. The user searches musicindex (Stophammer) by artist or feed name.
3. The user imports an MSP 2.0 backup.

## Constraints

- The API reads public feeds and the musicindex API over HTTPS. The UI makes no request to other servers (ADR 0016).
- A feed that the user found by URL or search is read-only until the user gives it a connector.
- An MSP 2.0 edit token goes into the vault, not into the collection (ADR 0003).
- An HTTP client is a new dependency of `studio-core`. Select one that needs no system TLS library.

## Acceptance Criteria

Mechanical:

- A pasted URL adds a read-only feed with its title and GUID.
- A musicindex search gives feeds with their titles, GUIDs and URLs.
- An MSP 2.0 backup import adds each feed, and puts each edit token in the vault.
- No API response contains an MSP 2.0 edit token.

Visual:

- The wizard offers the four methods.
