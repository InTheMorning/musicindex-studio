# Studio v1 Task 012: The Value Editor

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0008](../adr/0008-feed-types-and-feed-identity.md) and [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md).
Depends on: [task 011](studio-v1-task-011-album-editor.md).

## Goal

Let the user edit the payment recipients of the album and of each track. Keep each value time split with no change.

## Steps

1. Show the recipients of the first `podcast:value` block of the album. Do not show the method, the payment type or the `suggested` value of the block. Keep their values. A new block gets the method of the album block, or keysend.
2. Edit each recipient: name, type (in a list), address, split, `customKey`, `customValue` and `fee`. A recipient of type `lnaddress` does not show `customKey` and `customValue`. The hint text of the address follows the type.
3. Add, remove and move a recipient.
4. Give a track its own recipients from a copy of the album recipients, and remove them again. Show that the recipients of a track replace the album recipients for that track.
5. Do not offer a second payment block. Keep each other block with no change, and tell the user that it exists.
6. Do not edit a value time split. Show how many a track has, and keep them with no change.
7. Show the album recipients and the recipients of each track as collapsed groups with a summary line ([ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md)). The summary line has a share bar with a legend. A recipient under 5 % folds into "Other". In a track, put "Use the album recipients" first. Show each recipient as one summary line that opens. A recipient has the tertiary background.

## Constraints

- The constraints of [task 011](studio-v1-task-011-album-editor.md) apply.
- A split is a number of shares, not a percentage. The editor shows the percentage of each recipient as information. It does not change the shares.
- On 2026-10-22 the operator decided that the editor does not offer value time splits. A small number of users need them, and v4vmm is to send them to Studio directly.

## Acceptance Criteria

Mechanical:

- A sent value block with one changed recipient changes only that recipient element.
- A new element gets the attribute sequence of the first source element with the same name.
- A value time split goes to JSON and back with no change, with its remote item and its recipients.
- The model of each corpus feed with `podcast:valueTimeSplit` goes to JSON and back with no change.

Visual:

- The operator edits each recipient field of the channel block, and the diff shows each change.
- The operator gives a track its own recipients, and the diff shows them in that track only.
- A track with value time splits shows their number, and a write keeps them.
- The percentage of each recipient agrees with its shares.
- One recipient of a list is open at a time. The people and the support links of the album show as summary lines, and the credits start collapsed.
