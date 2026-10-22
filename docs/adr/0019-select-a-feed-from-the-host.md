# ADR 0019: Select A Feed From The Files Of A Host

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0005](0005-collection-and-first-run-discovery.md) did not select a scan of a connected host for v1.
The first-run wizard then asked for the path of a feed file in the folder of the host.
On 2026-10-22, the operator examined the wizard. The operator asked that Studio lists the feed files that it finds on the host, so that the user selects a feed and does not type a path.
[ADR 0006](0006-connectors-and-the-v1-set.md) and [ADR 0015](0015-connector-interface-and-write-pipeline.md) give no list function to a connector.

## Decision

1. This decision replaces the ADR 0005 alternative "Scan a connected host for feed files. Not selected for v1."
2. Each connector has a fourth function: it lists the paths of the files on the host, relative to the connector root.
3. A list is limited: a maximum folder depth of 4, a maximum of 1000 files, and no hidden folders.
4. The core reads each `.xml` file of 5 MB or less in the list. Each file that is an RSS feed is a candidate, with its title, medium and `podcast:guid`.
5. The UI shows the candidates. The user selects one. Each candidate that is already in the collection shows this, and the user cannot add it again.
6. The UI also lets the user type a path, for a file that is not in the list.
7. A list does not change a file on the host.

## Consequences

- Each new connector must implement the list function.
- A large host gives a partial list. The UI tells the user when the limit stopped the list.

## Invariants

- A list reads files only. It does not write.
