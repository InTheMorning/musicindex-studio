# Task 031 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 031](../tasks/studio-v1-task-031-announce.md)
- Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) and [ADR 0007](../adr/0007-draft-diff-write-and-announce.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 031a: no test uses `DEFAULT_HUB`. `TestHub` stops when its value is dropped.
- 031a: the rules and the constants are in `announce.rs`, and `collection.rs` has only storage steps.
- 031b: `diff_at` and `write_draft_signed` get the plan from the same function with the same values.
- 031b: the queue call is the last step of a write with no error. No error path reaches it.
- 031c: the worker holds no workspace lock while it sends a request.
- 031c: each API test sets the hub to a `TestHub`, and no test sleeps for more than 5 seconds.
- 031d: the text logic is in `announce.js`, not in the Preact components.

## Invariants

- No announcement occurs without a diff that the user has seen ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md)).
- An announcement never changes or stops a write ([ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- The core has no async runtime and no HTTP server code outside `testing` ([ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [031a](../tasks/studio-v1-task-031a-queue-and-sender.md) | Open | |
| [031b](../tasks/studio-v1-task-031b-diff-and-write.md) | Open | |
| [031c](../tasks/studio-v1-task-031c-worker-and-routes.md) | Open | |
| [031d](../tasks/studio-v1-task-031d-announce-ui.md) | Open | |
