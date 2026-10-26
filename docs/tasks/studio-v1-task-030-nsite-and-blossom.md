# Studio v1 Task 030: The nsite And Blossom Connectors

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 9, the fourth task.
Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md), [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md), [ADR 0044](../adr/0044-the-nsite-signer.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md).

## Goal

A user can add an nsite site for a feed and its media, and a Blossom host for media only. Each host signs with a key in the vault or with the NIP-07 signer of the user. A write publishes one manifest for each write step.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-25-task-030-review-checklist.md) before its commit. The packets go in this sequence:

1. [030a: the test relay and the test Blossom server](studio-v1-task-030a-test-relay-and-blossom.md).
2. [030b: the Nostr events and the relay client](studio-v1-task-030b-nostr-events-and-relay-client.md). It depends on 030a.
3. [030c: the trait functions, the host environment and `finish` in the pipeline](studio-v1-task-030c-trait-and-finish.md). It depends on 030b.
4. [030d: the Blossom connector](studio-v1-task-030d-blossom-connector.md). It depends on 030c.
5. [030e: the nsite connector](studio-v1-task-030e-nsite-connector.md). It depends on 030d.
6. [030f: the planned events in the diff, and the signed events in the write](studio-v1-task-030f-signing-plan.md). It depends on 030e.
7. [030g: the nsite and Blossom host routes](studio-v1-task-030g-host-routes.md). It depends on 030f.
8. [030h: the host forms and the signer step](studio-v1-task-030h-host-forms-and-signer-step.md). It depends on 030g.

## Acceptance Criteria

Mechanical:

- The Blossom connector and the nsite connector pass the test module of the trait against the test servers.
- A write with uploads, a feed XML and deletions publishes two manifests.
- A write with "My signer" and a signed event that is not the planned event changes nothing on a host.
- Studio does not delete a blob that a manifest or a kept record of the host refers to.
- The core enables no `nostr` feature other than the default features.
- The four commit checks pass, in CI too.

Visual:

- The Hosts page shows the nsite form and the Blossom form, with the relays and the servers of the key. The Review sheet shows the signer step. The planning model checks both with `tools/drive-ui.mjs`, in the dark theme and in the light theme.

Human tests, open until the operator can use a test key, relays and a Blossom server:

- The operator writes a release to an nsite site with "My signer" in a browser extension.
- A podcast app plays a track from the gateway address.
