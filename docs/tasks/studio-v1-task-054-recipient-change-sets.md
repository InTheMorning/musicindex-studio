# Studio v1 Task 054: Recipient Change Sets

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Recipient Batch Edits, File Hosts And Dates".
Decisions: [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md), [ADR 0027](../adr/0027-the-recipient-picker.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) and [ADR 0088](../adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md).

Status: In progress. Packets 054a and 054b are complete. The plan gives the sequence.

## Goal

The user changes, renames, removes or adds one recipient in many feeds. One review shows each changed feed, and one write writes each feed that the user can write.

## Packets

The packets go in this sequence. Each packet uses the result of the packet before it.

1. [054a: the operations and the change set in the core](studio-v1-task-054a-change-set-core.md).
2. [054b: the routes of a change set](studio-v1-task-054b-change-set-routes.md).
3. [054c: the Recipients page and the review of a change set](studio-v1-task-054c-recipients-page.md).

## Constraints

- No feed rule in the UI (ADR 0065). The core calculates the splits, finds the places and gives the outcomes.
- Each write of a host uses `pipeline::write_draft` with the ID of a diff that the review showed (ADR 0007).
- No test sends a request to a host outside the computer.
- Each comment, each test message and each UI text is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical: the mechanical criteria of ADR 0091 §Verification, the criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The bottom of the left panel shows Recipients, File hosts, Settings, then Lock Studio.
- The Recipients page shows one summary line for each recipient.
- The page of a recipient shows each feed with its places and a checkbox.
- Each operation sheet opens from the page of a recipient.
- The review shows each feed with its outcome, and each changed feed opens its diff.
- During a write with one feed that fails, the review shows the status of each feed.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: in a collection with more than ten feeds, change the address of one recipient. Then write the change set, and examine two feeds on their file hosts.
