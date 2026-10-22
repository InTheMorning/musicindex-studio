# Studio v1 Task 006: Collection, Local Connector And Write Pipeline

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 3.
Decisions: [ADR 0005](../adr/0005-collection-and-first-run-discovery.md), [ADR 0006](../adr/0006-connectors-and-the-v1-set.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0014](../adr/0014-collection-in-sqlite.md) and [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md).

## Goal

Keep a collection of feeds in SQLite. Read and write a feed through the local folder connector. Write only through the draft, diff and snapshot pipeline.

## Files Likely To Change

- `crates/core/Cargo.toml`
- `crates/core/src/collection.rs`, `crates/core/src/connector.rs`, `crates/core/src/pipeline.rs` and their submodules

## Acceptance Criteria

Mechanical:

- A new database gets the current schema. A database at the current version opens with no change.
- The collection adds and lists connectors and feeds, and keeps the item GUIDs of each feed.
- The local folder connector reads a file with its version marker, and writes a file only with the expected marker.
- A write with a version marker that is not current gives a conflict and does not change the file.
- A write with a diff ID that is not the current diff ID gives an error and does not change the file.
- A write stops with a conflict when the live file changed after the draft started.
- A write keeps a snapshot of the file that it replaces. A rollback makes a draft from the snapshot.
- An update of a snapshot row gives an error.
- The database file contains no secret of a connector.
- A path that goes out of the connector folder gives an error.

Visual: none.

## Test Commands

```sh
cargo test -p studio-core
```
