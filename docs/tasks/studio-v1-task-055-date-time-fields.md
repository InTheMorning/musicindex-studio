# Studio v1 Task 055: The Date And Time Fields

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0093](../adr/0093-dates-and-times-use-a-picker-in-local-time.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: In progress. Packet 055a is complete. The plan gives the sequence.

## Goal

The user selects each date and each time with a native picker in local time. The field shows the zone. The core writes `pubDate` as RFC 2822 text, and each live item keeps the offset of the artist.

## Packets

The packets go in this sequence. 055b needs 055a.

1. [055a: the times in the core](studio-v1-task-055a-times-in-the-core.md). Core and API.
2. [055b: the date and time field](studio-v1-task-055b-date-time-field.md). UI.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0093.

- The core module is `crates/core/src/datetime.rs`. It has `parse_time` (moved from `grant.rs`), `parse_rfc2822`, `format_rfc2822` and `format_rfc3339`. A time is a value `Moment { seconds: i64, offset_minutes: i32 }`, so that the core keeps the offset of the source.
- `Item` in `crates/core/src/feed.rs` gets `pub_date_time: Option<String>`, as `Feed` has `op3`. `Feed::read` computes it. `write` ignores it.
- `replace_model` gets the step `apply_pub_date_time`, after `apply_op3`.
- The UI model module is `crates/api/ui/datetime.js`. `isoToLocalInput` moves to it from `live.js`. `startToIso` becomes `localInputToIso` in it, and `live.js` imports it.
- The Node tests set `process.env.TZ` to `Europe/Berlin` before the first `Date`, so that each test run gives the same offsets.

## Acceptance Criteria

Mechanical: the mechanical criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The track page shows "Publication date" as a date and time field, with the zone hint, "Now" and "Clear".
- A track with a `pubDate` that Studio cannot read shows the empty field with the hint of ADR 0093 item 8.
- The live item page and the sheet "Add a live show" show the zone hint and "Now".
- After "Now" on a track, the diff shows a `pubDate` with the local offset, for example `+0100`.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: on a phone, set the publication date of a test track with the picker. Write the album, and examine the `pubDate` in the feed. Set the start time of a test live show, and examine the `start` attribute.
