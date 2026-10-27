# Studio v1 Task 041: The Fields That MSP 2.0 Edits

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md), [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) and [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md).

Status: In progress. Packet 041a is complete. The operator accepted ADR 0064 on 2026-10-27. [Task 042](studio-v1-task-042-core-feed-rules.md) of [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) is complete.

## Goal

An MSP 2.0 user can change each field that MSP 2.0 edits, other than `itunes:category`. Studio writes no field that the user did not set, and a switch keeps the form of the value in the feed.

## Packets

The packets go in this sequence:

1. [041a: the new elements in the core](studio-v1-task-041a-core-elements.md). It depends on task 042.
2. [041b: the channel fields](studio-v1-task-041b-channel-fields.md). It depends on 041a.
3. [041c: the lyrics kind of the picker](studio-v1-task-041c-lyrics-kind.md). It depends on 041b.
4. [041d: the track fields](studio-v1-task-041d-track-fields.md). It depends on 041c, because the lyrics field uses the new kind.
5. [041e: the artwork list](studio-v1-task-041e-artwork-list.md). It depends on 041d, because 041d and 041e both change the track page.

The packets run one at a time, because 041b to 041e all change `crates/api/ui/editor.js`.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0064. The [review checklist](../reviews/2026-10-27-task-041-review-checklist.md) records them.

- The form rule of the explicit value (ADR 0064 item 4) is a step of `pipeline::replace_model`, as `keep_identity`, `number_tracks` and `keep_cover_form` are. The UI sends `true` or `false`. Packet 041a adds the step and its core tests.
- The lyrics kind of ADR 0064 item 9 changes the core, the upload and the picker. It is packet 041c, before the track fields.
- A new element with no hosted position goes before the first element of a type that must come after it (ADR 0031 item 3). The sequence of the write calls does not give a position.
- The near ratio uses the list and the tolerance of packet 041e. The behavior agrees with MSP 2.0. No code comes from MSP 2.0.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the operator. Each one in the dark theme, in the light theme and at 390 px:

- The album page with "Explicit" after "Hide from apps", and the page "Feed settings" with the owner, the keywords, the lock and the two editor fields.
- The track page with "Explicit", the credits selection with "Credits for this track", two lyrics entries and the season in "Identity".
- The artwork list of the album, after the user selects an image. The dimensions, the type and the ratio show.
