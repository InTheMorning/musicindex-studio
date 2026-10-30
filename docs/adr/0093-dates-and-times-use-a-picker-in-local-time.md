# ADR 0093: Dates And Times Use A Picker In Local Time

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Uses: [ADR 0012](0012-lossless-tree-with-a-typed-model.md) (the lossless tree), [ADR 0016](0016-ui-without-a-build-step.md) (no build step and no other servers), [ADR 0042](0042-ui-models-with-node-tests.md) (the UI models), [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) (live items), [ADR 0065](0065-the-core-applies-each-feed-rule.md) (each feed rule is in the core) and [ADR 0037](0037-button-roles.md) (button roles).

## Context

- On 2026-10-30 the operator asked for an easier method to set dates and times, in the local time zone and with a picker.
- The track field "Publication date" is a text field. The user types RFC 2822 text, such as `Thu, 22 Oct 2026 12:00:00 GMT`, and calculates the GMT time. The core keeps the text as the user typed it, with no check.
- The start time and the end time of a live item use a `datetime-local` field. `startToIso` in `crates/api/ui/live.js` uses `toISOString()`, so the feed gets UTC with `Z`. The feed does not keep the offset of the artist. The field does not show its zone.
- The corpus has 567 `pubDate` values: 537 with `GMT`, 26 with `+0000` and 4 with `-0500`. One value has a day with one digit. Its three live items have starts such as `2021-09-26T07:30:00.000-0600` and `2026-10-18T00:00:00.000Z`. `grant::parse_time` reads each of them.
- The core has no date library. `grant::parse_time` reads ISO 8601 text with a zone through its own calendar functions.

## Decision

1. Each field for a date and a time uses one component, `DateTimeField`, in `crates/api/ui/app.js`. It uses the native `<input type="datetime-local">`, so the operating system gives the picker on a desktop and on a phone. The UI loads no picker library.
2. The field shows local time with the precision of one minute. The local zone is the zone of the browser in both shells. The zone of the server is not used.
3. Below the field, a hint gives the zone and the offset of the selected time, for example "Europe/Berlin, UTC+01:00". With no selected time, the hint gives the offset of the current time.
4. The field has the button "Now", with the role normal. It sets the current local time. When the field is optional, it also has the button "Clear", with the role normal. It removes the value.
5. The UI sends a time as RFC 3339 text with the offset of the selected date, for example `2026-10-30T20:00:00+01:00`. A model function in `crates/api/ui/datetime.js` makes this text. The offset comes from the selected date, not from the current date. Thus a date after a daylight saving change gets its correct offset.
6. The JSON model of an item gets `pub_date_time`. The core computes it at each read from `pubDate`: RFC 3339 text with the offset of the source, or null when the core cannot read the `pubDate`. The core does not write `pub_date_time` as an element. The track field "Publication date" edits `pub_date_time`, and the UI does not edit `pub_date`.
7. At the draft save, the core compares the sent `pub_date_time` of each item with the value that it computes from the current `pubDate`:
   - The same value: `pubDate` does not change.
   - A different time: the core writes `pubDate` as RFC 2822 text with the sent offset, for example `Fri, 30 Oct 2026 20:00:00 +0100`.
   - A sent null over a value: the core removes `pubDate`.
   - A new item: the core writes `pubDate` from a sent time, and writes no `pubDate` from a sent null.
8. When the core cannot read the `pubDate` of an item, the field is empty. Its hint gives the source text: "The feed has the date “…”. Studio cannot read it. Select a time to replace it." The text stays until the user selects a time.
9. The start time and the end time of a live item keep the offset that the UI sends. The UI does not change them to UTC.
10. A live item with a start or an end that `parse_time` cannot read has a problem: "The start time is not a time with a zone." or "The end time is not a time with a zone." The write of the draft stops, as for the other live problems of ADR 0054.
11. The core reads and writes RFC 2822 and RFC 3339 times in one module, `crates/core/src/datetime.rs`. `grant::parse_time` moves to it. The module uses no new crate.

## Alternatives Considered

- A picker library in the UI. Not chosen: ADR 0016 forbids a build step and a load from other servers, and the native picker works on a phone.
- The UI makes the RFC 2822 text. Not chosen: the format of `pubDate` is a feed rule (ADR 0065).
- The core writes `pubDate` in GMT, as most corpus feeds do. Not chosen: the offset keeps the time that the artist selected, and RFC 2822 permits it.
- A zone setting for each user on the Settings page. Not chosen: the zone of the browser is correct for almost all users, and a setting adds one more step.
- A date crate such as `time`, `chrono` or `jiff`. Not chosen: the core has its calendar functions, and the UI sends a fixed offset, so the core needs no zone database.

## Consequences

- The user selects a date and a time, and does not type a format.
- A feed shows the offset of the artist in `pubDate` and in a live item.
- A live item that a connected app sends with no zone has a problem, and its write stops.
- A `pubDate` that the user does not change keeps its bytes.

## Invariants

- `crates/core/src/datetime.rs` tests the read and the write of each time format of item 11, and each corpus `pubDate`.
- `crates/core/src/pipeline.rs` tests the four cases of item 7.
- The corpus tests make sure that a read and a write with no edit give the same bytes.
- `crates/core/src/live.rs` tests the problems of item 10.
- `crates/api/ui-tests/datetime.test.mjs` tests the RFC 3339 text of item 5 and the hint of item 3, with a date near a daylight saving change.
- A Node test finds an `<input type="datetime-local">` in `crates/api/ui/` that is not in `DateTimeField`.
