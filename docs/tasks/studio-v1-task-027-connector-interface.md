# Studio v1 Task 027: The Connector Interface Change

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 9, the first task.
Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) items 1 to 9, [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md) and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) items 6, 15 and 16.

## Goal

This task prepares the connector interface for the remote connectors, before the first one exists. A connector gets a streaming write, two more error types, an optional SHA-256 from the host and a test module of the trait. `from_row` gets the secret of a connector from the open vault. The write confirms each new file with `stat` and its public address, and it does no deletion when a file does not pass.

`LocalFolder` is the only connector after this task. It uses each change first.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-task-027-review-checklist.md) before its commit. The packets go in this sequence:

1. [027a: the trait changes](studio-v1-task-027a-trait-changes.md).
2. [027b: the test module of the trait](studio-v1-task-027b-trait-test-module.md). It depends on 027a.
3. [027c: the confirmation before the deletions](studio-v1-task-027c-confirmation.md). It depends on 027a.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- `LocalFolder` passes the test module of the trait (027b).
- A write with a new file that does not pass the confirmation deletes no file. A second write with a new approval does the deletions after the file passes (027c).
- The four commit checks pass.

Visual:

- The Review sheet shows a confirm operation with its label and its note, in the dark theme and in the light theme (027c). The planning model checks it with `tools/drive-ui.mjs`, and then the operator checks it.

## Out Of Scope

- A remote connector. Each one is a different task of Phase 9.
- The client crate of a remote connector. Each connector task records its crate in an ADR.
- The UI that adds a remote host.
