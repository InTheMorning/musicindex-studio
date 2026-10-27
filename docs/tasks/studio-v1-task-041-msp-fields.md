# Studio v1 Task 041: The Fields That MSP 2.0 Edits

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md), [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) and [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md).

Status: Not started. ADR 0064 is proposed. The packets start after the operator accepts it.

## Goal

An MSP 2.0 user can change each field that MSP 2.0 edits, other than `itunes:category`. Studio writes no field that the user did not set, and a switch keeps the form of the value in the feed.

## Packets

The packets go in this sequence:

1. [041a: the new elements in the core](studio-v1-task-041a-core-elements.md). It depends on task 040.
2. [041b: the channel fields](studio-v1-task-041b-channel-fields.md). It depends on 041a.
3. [041c: the track fields](studio-v1-task-041c-track-fields.md). It depends on 041b, because it uses the model module of the explicit switch.
4. [041d: the artwork list](studio-v1-task-041d-artwork-list.md). It depends on 041c, because 041c and 041d both change the track page.

The packets run one at a time, because 041b, 041c and 041d all change `crates/api/ui/editor.js`.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0064. The [review checklist](../reviews/2026-10-27-task-041-review-checklist.md) records them.

- The form rule of the explicit switch (ADR 0064 item 4) is in a UI model module, `crates/api/ui/explicit.js`. The editor sends the model as text, so only the UI knows that the user set the switch. The listing rule of ADR 0063 uses the same structure. The core models the text as the source wrote it. A Node test examines the form rule, and a core test examines that a write keeps the text that the model gives.
- A new channel element with no hosted position goes to the position that packet 041a gives. Each new element is in one place in the sequence of `Feed::write_hosted` or `Item::apply_children`.
- The near ratio uses the list and the tolerance of packet 041d. The behavior agrees with MSP 2.0. No code comes from MSP 2.0.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The album page with "Explicit" after "Hide from apps", and the page "Feed settings" with the owner, the keywords, the lock and the two editor fields.
- The track page with "Explicit", the credits selection with "Credits for this track", two lyrics entries and the season in "Identity".
- The artwork list of the album, after the user selects an image. The dimensions, the type and the ratio show.
