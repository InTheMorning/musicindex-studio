# Studio v1 Task 055b: The Date And Time Field

Parent: [task 055](studio-v1-task-055-date-time-fields.md). Decision: [ADR 0093](../adr/0093-dates-and-times-use-a-picker-in-local-time.md) items 1 to 5, 8 and 9.

Status: Not started. It needs task 055a.

## Goal

Each date and time field of the UI is a `DateTimeField` with the native picker, the zone hint, "Now" and, for an optional field, "Clear". The track field "Publication date" edits `pub_date_time`. The live times keep the local offset.

## Files

Inspect: `crates/api/ui/app.js` (`Field`, `SecretInput` as an example of a shared control), `crates/api/ui/editor.js` (the row `pub_date` near line 2516, `TimeField` near line 2579, `AddLiveSheet` near line 2585, the new items near lines 385 and 1131), `crates/api/ui/live.js` (`startToIso`, `isoToLocalInput`), `crates/api/ui/styles.css`, `crates/api/src/ui.rs` (the list of UI files), `crates/api/ui-tests/live.test.mjs`, `crates/api/ui-tests/secrets.test.mjs` (an example of a guard test).

Change: `crates/api/ui/datetime.js` (new), `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/live.js`, `crates/api/ui/styles.css`, `crates/api/src/ui.rs`, `crates/api/ui-tests/datetime.test.mjs` (new), `crates/api/ui-tests/live.test.mjs`.

Do not change: the core, the API routes, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `crates/api/ui/datetime.js` is a model module with no Preact and no browser state (ADR 0042). It has:
  - `localInputToIso(value)`: the value of a `datetime-local` field, such as `2026-10-30T20:00`, gives RFC 3339 text with seconds `:00`. The offset is the local offset of that date, for example `2026-10-30T20:00:00+01:00`. The offset `0` gives `+00:00`. An empty or incorrect value gives null. It replaces `startToIso`.
  - `isoToLocalInput(text)`: moved from `live.js` with no change of behavior.
  - `offsetText(minutes)`: `60` gives `UTC+01:00`, `-330` gives `UTC-05:30` and `0` gives `UTC+00:00`.
  - `zoneHint(iso, zone, now)`: `zone` is the IANA zone name that the caller gives. It gives `"<zone>, <offsetText>"`. The offset is the offset of `iso`, or the offset of the `Date` `now` when `iso` is null.
  - `nowInput(now)`: the value of a `datetime-local` field for the `Date` `now`, with the precision of one minute.
  - `unreadDateHint(text)`: `The feed has the date “<text>”. Studio cannot read it. Select a time to replace it.`
- `live.js` imports `localInputToIso` and `isoToLocalInput` from `datetime.js`. It does not export `startToIso`.
- `DateTimeField({ id, label, value, onChange, required = false, sourceText = null })` in `app.js`:
  - It renders a `Field` with `type="datetime-local"` and the value `isoToLocalInput(value)`. Each input calls `onChange(localInputToIso(e.target.value))`.
  - The hint is `unreadDateHint(sourceText)` when `value` is null and `sourceText` is not empty. Else the hint is `zoneHint(value, Intl.DateTimeFormat().resolvedOptions().timeZone, new Date())`.
  - Its children are the button "Now" (`class="button"`), which calls `onChange(localInputToIso(nowInput(new Date())))`, and, when `required` is false, the button "Clear" (`class="button"`), which calls `onChange(null)`. "Clear" is disabled when `value` is null.
  - It is the only control in `crates/api/ui/` with `type="datetime-local"`.
- The row `pub_date` of the track page renders `DateTimeField` with the label "Publication date", `value=${item.pub_date_time}`, `sourceText=${item.pub_date}` and `onChange=${change('pub_date_time')}`. The hint "Some apps sort the tracks by this date." stays as a note below the field. The UI does not change `item.pub_date`.
- The new items near lines 385 and 1131 get `pub_date_time: null`.
- `TimeField` in `editor.js` is removed. The live item page uses `DateTimeField` for "Start time" (required) and "End time" (optional).
- `AddLiveSheet` keeps its start as RFC 3339 text and uses `DateTimeField` with `required`. The add button is ready when the title is not blank and the start is not null.
- `ui.rs` serves `datetime.js` as it serves `op3.js`.
- The two buttons have the target size of the profile (32 px with a mouse, 44 px with touch). With no room, they go below the input. They do not go past the edge of the group.

## Steps

1. Write `datetime.js` and `datetime.test.mjs`. Move the tests of `startToIso` and `isoToLocalInput` from `live.test.mjs`, and change them to the new names.
2. Add `datetime.js` to `ui.rs`.
3. Write `DateTimeField` in `app.js`, and add the styles.
4. Change the row `pub_date`, the new items, the live item page and `AddLiveSheet`.
5. Add the guard test.
6. Check the syntax of each changed UI file with a `.mjs` copy (`node --check` misses template errors in `ui/*.js`).
7. Run the four commit checks.

## Constraints

- No feed rule in the UI (ADR 0065). The UI makes no RFC 2822 text and does not change `pub_date`.
- No picker library. The UI loads nothing from other servers (ADR 0016).
- Each comment, each UI text and each test message is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- `datetime.test.mjs` sets `process.env.TZ = 'Europe/Berlin'` before it imports `datetime.js`.
- A Node test: `localInputToIso('2026-03-28T12:00')` gives `2026-03-28T12:00:00+01:00`, and `localInputToIso('2026-03-30T12:00')` gives `2026-03-30T12:00:00+02:00`.
- A Node test: `localInputToIso('')` and `localInputToIso('not a time')` give null.
- A Node test: `isoToLocalInput('2026-10-18T00:00:00.000Z')` gives `2026-10-18T02:00`, and `isoToLocalInput(localInputToIso(v)) === v` for `2026-10-30T20:00`.
- A Node test: `offsetText` gives the three values of the Architecture section.
- A Node test: `zoneHint('2026-07-01T12:00:00+02:00', 'Europe/Berlin', now)` gives `Europe/Berlin, UTC+02:00`. `zoneHint(null, 'Europe/Berlin', new Date('2026-01-15T12:00:00Z'))` gives `Europe/Berlin, UTC+01:00`.
- A Node test: `nowInput(new Date('2026-10-30T19:00:45Z'))` gives `2026-10-30T20:00`.
- A Node guard test: no module of `crates/api/ui/` other than `app.js` has `type="datetime-local"`, and `app.js` has it one time only. The message: `ADR 0093 item 1: <file> makes a date and time field of its own. Use DateTimeField of app.js.`
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the JSON model of an item does not have `pub_date_time`. Task 055a gives it.
- Stop and report when a page needs a date with no time. ADR 0093 gives no date field.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0093-dates-and-times-use-a-picker-in-local-time.md
- docs/tasks/studio-v1-task-055-date-time-fields.md
- docs/tasks/studio-v1-task-055b-date-time-field.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Each date and time field is a `DateTimeField` with the native picker, the zone hint and "Now".
- The track field "Publication date" edits `pub_date_time`.
- The live times keep the local offset.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- No picker library.

Do not touch:
- The core, the API routes, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
