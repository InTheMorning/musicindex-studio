# Studio v1 Task 012: The Value Editor

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0008](../adr/0008-feed-types-and-feed-identity.md) and [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md).
Depends on: [task 011](studio-v1-task-011-album-editor.md).

## Goal

Let the user edit the payment blocks of the channel and of each track: the value recipients and the value time splits.

## Steps

1. Edit the `podcast:value` blocks of the channel: type, method, the `suggested` value and recipients.
2. Edit each recipient: name, type, address, split, `customKey`, `customValue` and `fee`.
3. Add, remove and move a recipient.
4. Edit the value blocks of a track. Show that a track block replaces the channel blocks for that track.
5. Edit the value time splits of a track block: start time, duration, remote start time, remote percentage, the remote item and the recipients.
6. Add, remove and move a value time split.

## Constraints

- The constraints of [task 011](studio-v1-task-011-album-editor.md) apply.
- A split is a number of shares, not a percentage. The editor shows the percentage of each recipient as information. It does not change the shares.

## Acceptance Criteria

Mechanical:

- A sent value block with one changed recipient changes only that recipient element.
- A value time split goes to JSON and back with no change, with its remote item and its recipients.
- The model of each corpus feed with `podcast:valueTimeSplit` goes to JSON and back with no change.

Visual:

- The operator edits each recipient field of the channel block, and the diff shows each change.
- The operator adds a track block, and the diff shows it in that track only.
- The operator adds, edits and removes a value time split, and the diff shows each change.
- The percentage of each recipient agrees with its shares.
