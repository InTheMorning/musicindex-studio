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
| [031a](../tasks/studio-v1-task-031a-queue-and-sender.md) | Pass | A replaced `waiting` row kept the error text of an earlier attempt. The planning model made `restart_announcement` remove it, and added a test. The manual checks pass. No test uses `DEFAULT_HUB`, and `TestHub` stops when its value is dropped. The rules and the constants are in `announce.rs`, and `collection.rs` has only storage steps. `trim_announcements` trims each feed in one SQL step. The result agrees with the packet. The removal of a feed needed no change, because `ON DELETE CASCADE` removes its rows, and a test examines this. |
| [031b](../tasks/studio-v1-task-031b-diff-and-write.md) | Pass | The queue call came after the scans of the hosts, so a failed scan lost the announcements of a written feed. The planning model moved it to directly after step 5, before the scans. The manual checks pass. `diff_at` and `write_draft_signed` get the plan from `plan_announcements` with the same values. No error path before step 5 reaches the queue call. For a feed with no writable host, the address is the public address of its row. |
| [031c](../tasks/studio-v1-task-031c-worker-and-routes.md) | Pass | Each test router starts the worker, and a new workspace had the real hub. Thus a slow test that writes a feed could send a request to the real hub after 30 seconds. The planning model made a test build use port 9 as the hub of a new workspace, as the tests do for the musicindex API. The worker starts in `local_router`, because only the router has the shared workspace. `resume_announcements` does the expiry and wakes the worker. The manual checks pass. The worker holds no lock during a request. Each new API test sets a `TestHub`, and each wait is 5 seconds or less. |
| [031d](../tasks/studio-v1-task-031d-announce-ui.md) | Pass | The planning model made each announcement line a row with the form of a host operation, with the label "Announce". The first form had a heading rule and list bullets. The lines now show only when the sheet can write. The feed move hint "Studio does not tell the musicindex API and Podcast Index the new address yet" was false after ADR 0052, so the planning model removed it and its style. Three comments cited the wrong item of ADR 0052. The `drive-ui.mjs` check against a local `TestHub` passes in the dark theme and in the light theme. It shows the podping line and the line of the setting off. It shows the feed facts with "Failed", "Waiting" and then "Sent", and the Hosts panel with the podping box disabled. At 390 px, the review sheet and the Hosts panel have no small control. The feed page loads the list again after a write through a window event, and groups the rows of one write by their creation time. The human test with a real feed is open. |
