# Studio v1 Task 028: The S3 And R2 Connector

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 9, the second task.
Decisions: [ADR 0046](../adr/0046-the-s3-and-r2-connector.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md).

## Goal

A user can add an S3 or R2 bucket as a host. Studio writes feeds and their files to it, with a condition on each write. Before Studio adds the host, it tests that the provider refuses a conditional write that is not correct.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-task-028-review-checklist.md) before its commit. The packets go in this sequence:

1. [028a: the fake S3 server](studio-v1-task-028a-fake-s3-server.md).
2. [028b: the S3 connector](studio-v1-task-028b-s3-connector.md). It depends on 028a.
3. [028c: the provider test and the host route](studio-v1-task-028c-provider-test-and-route.md). It depends on 028b.
4. [028d: the S3 host form](studio-v1-task-028d-s3-host-form.md). It depends on 028c.

## Acceptance Criteria

Mechanical:

- The criteria of each packet.
- The S3 connector passes the test module of the trait against the fake S3 server (028b).
- The API adds an S3 host only after the provider test passes, and the access keys are only in the vault (028c).
- The four commit checks pass.

Visual:

- The Hosts page shows the S3 form, its errors and an added S3 host, in the dark theme and in the light theme (028d). The planning model checks it with `tools/drive-ui.mjs`.

Human test, open until the operator can use a real bucket:

- The operator adds an R2 bucket with a custom domain. Studio writes a new release to it, and a podcast app plays a track from the public address.

## Out Of Scope

- A per-object redirect ([ADR 0046](../adr/0046-the-s3-and-r2-connector.md), Alternatives Considered).
- A SHA-256 in the object metadata.
