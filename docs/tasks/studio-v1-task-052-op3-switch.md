# Studio v1 Task 052: An OP3 Switch On Feed Settings

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "MSP 2.0 Parity".
Decisions: [ADR 0076](../adr/0076-an-op3-switch-on-feed-settings.md), [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Complete - 2026-10-28, except the human test. The four commit checks pass. The screenshot visual check passed in the two themes and at 390 px. The human test is open.

## Goal

The core compares and reads each enclosure by its plain address. The page "Feed settings" of an album has the switch "OP3 statistics", and the core adds or removes the prefix of each track.

## Packets

The packets go in this sequence. 052b needs 052a.

1. [052a: the plain address](studio-v1-task-052a-plain-address.md). Core and API.
2. [052b: the switch](studio-v1-task-052b-op3-switch.md). Core, API and UI.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0076.

- The plain address function is `media::plain_address(url) -> Cow<str>` in `crates/core/src/media.rs`. `pipeline::feed_addresses` gives plain addresses, so each comparison through it uses them.
- The JSON model of a feed gets `op3`, a value that the core computes at each read. The core does not write it as an element. `replace_model` gets the step `apply_op3`.
- The draft response gives `op3_tracks: { prefixed, tracks }` for the count hint.

## Acceptance Criteria

Mechanical: the mechanical criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- "Feed settings" of an album shows "OP3 statistics" off. On, it shows the link "OP3 statistics". With a mixed set, it is off with the count hint.
- After the switch is on, the diff shows the prefix on each track enclosure.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: set the switch of a test album to on, and write the album. Play a track in an app, and open the OP3 page.
