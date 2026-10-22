# ADR 0014: The Collection Is In SQLite

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

Studio keeps the collection of each user ([ADR 0005](0005-collection-and-first-run-discovery.md)): the feeds, the connectors, the drafts and the snapshots ([ADR 0007](0007-draft-diff-write-and-announce.md)).
The desktop shell and the server shell use the same core ([ADR 0001](0001-one-core-with-desktop-and-server-shells.md)).
Stophammer uses SQLite.

## Decision

1. The collection is in one SQLite database, on the desktop and on the server.
2. The core uses `rusqlite` with the bundled SQLite library, so that the desktop app needs no system SQLite.
3. The database schema has a version number. Each change of the schema is a numbered migration. A migration does not remove data that a user can need.
4. The database contains no secret. A connector row gives the ID of its vault record ([ADR 0003](0003-user-unlocked-secret-vault.md)).

## Alternatives Considered

- One JSON file for each collection. Rejected: the snapshots and drafts make it large, and a change writes the full file.
- PostgreSQL on the server. Rejected: two database engines need two sets of tests.

## Consequences

- The server keeps the collections of all users in one database, with a user ID on each row. Phase 9 adds the user ID.

## Invariants

- The collection database contains no secret.
- A snapshot row does not change after it is made.
