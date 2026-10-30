# Studio v1 Task 055a: The Times In The Core

Parent: [task 055](studio-v1-task-055-date-time-fields.md). Decision: [ADR 0093](../adr/0093-dates-and-times-use-a-picker-in-local-time.md) items 6, 7, 10 and 11.

Status: Not started.

## Goal

The core reads and writes RFC 2822 and RFC 3339 times. The JSON model of each item gives `pub_date_time`. At the draft save, the core writes `pubDate` from a changed `pub_date_time`. A live item with a start or an end that the core cannot read has a problem.

## Files

Inspect: `crates/core/src/grant.rs` (`parse_time`, `digits`, `days_in_month`, `zone_offset`, `days_from_civil` and their tests), `crates/core/src/feed.rs` (`Item`, `LiveItem`, `Feed::read`, the field `op3`), `crates/core/src/pipeline.rs` (`replace_model`, `apply_op3`, the live problem check near line 2634), `crates/core/src/live.rs` (`problems`), `crates/core/src/lib.rs`, `crates/core/tests/corpus/`, `crates/api/src/routes.rs` (the draft model routes).

Change: `crates/core/src/datetime.rs` (new), `crates/core/src/lib.rs`, `crates/core/src/grant.rs`, `crates/core/src/feed.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/live.rs`, and their tests. Change an API test only when the JSON model of an item is in its expected text, and then say so.

Do not change: the UI, the corpus files, the ADRs, `Cargo.toml` files.

## Architecture

These choices are fixed. Do not change them.

- `crates/core/src/datetime.rs` has:
  - `pub struct Moment { pub seconds: i64, pub offset_minutes: i32 }`. `seconds` is the time since the Unix epoch in UTC. `offset_minutes` is the offset of the text that the core read or will write.
  - `pub fn parse_time(text: &str) -> Option<i64>`: moved from `grant.rs` with no change of behavior. Its helper functions move with it. `grant.rs` imports it, and the grant tests continue to pass.
  - `pub fn parse_rfc3339(text: &str) -> Option<Moment>`: the same forms as `parse_time`, and it keeps the offset. `Z` gives the offset 0.
  - `pub fn parse_rfc2822(text: &str) -> Option<Moment>` reads these parts in sequence:
    - an optional weekday and comma
    - a day with one or two digits
    - an English month name of three letters
    - a year with four digits
    - `HH:MM` with optional `:SS`
    - a zone.

    The zone is `+HHMM`, `-HHMM`, `GMT`, `UT`, `UTC`, `Z`, or a US zone of RFC 5322 section 4.3 (`EST`, `EDT`, `CST`, `CDT`, `MST`, `MDT`, `PST`, `PDT`). Space at the ends is ignored. A weekday that does not agree with the date is accepted, because some feeds have this error. All other text gives `None`.
  - `pub fn format_rfc2822(moment: Moment) -> String`: for example `Fri, 30 Oct 2026 20:00:00 +0100`. The day has two digits. The zone is always `+HHMM` or `-HHMM`.
  - `pub fn format_rfc3339(moment: Moment) -> String`: for example `2026-10-30T20:00:00+01:00`. The offset 0 gives `+00:00`, not `Z`.
- `Item` gets `pub pub_date_time: Option<String>` with `#[serde(default)]` and a doc comment as the one of `Feed::op3`. `Feed::read` sets it for each item and for the item of each live item: `format_rfc3339` of `parse_rfc2822(pub_date)`, or `None`. `write_hosted` ignores it.
- `replace_model` calls `apply_pub_date_time(&current, &mut sent)` after `apply_op3`. For each sent item and the item of each sent live item:
  - Find the current item by its key. A sent item with no key is new.
  - `expected` is the `pub_date_time` that the core computes from the current `pub_date`. A new item has `expected = None`.
  - `sent.pub_date_time == expected`: set `pub_date` to the current `pub_date`. A new item keeps no `pub_date`.
  - Else, `sent.pub_date_time` is `Some(text)`: `parse_rfc3339(text)` gives a moment, and `pub_date = Some(format_rfc2822(moment))`. Text that `parse_rfc3339` cannot read gives `PipelineErrorKind::InvalidModel` with the message `pub_date_time is not an RFC 3339 time with a zone`.
  - Else, `sent.pub_date_time` is `None`: `pub_date = None`.
  - The step ignores the sent `pub_date`. Only the core sets it.
- `live::problems` adds a problem for a start that is not blank and that `parse_time` cannot read: field `start`, text `The start time is not a time with a zone.` It adds the same for `end`: field `end`, text `The end time is not a time with a zone.` A blank end is not a problem. Add `end` to the doc comment of the field names.

## Steps

1. Make `datetime.rs` and move `parse_time` and its helper functions. Move their tests. Run `cargo test -p studio-core grant` and `cargo test -p studio-core datetime`.
2. Write `parse_rfc3339`, `parse_rfc2822`, `format_rfc2822` and `format_rfc3339`, with tests.
3. Add `pub_date_time` to `Item` and compute it in `Feed::read`.
4. Write `apply_pub_date_time`, with tests.
5. Add the two live problems, with tests.
6. Run the four commit checks.

## Constraints

- No change of a feed when the user did not change it. The corpus tests continue to pass.
- No new crate.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- A core test: `parse_rfc2822` reads each `pubDate` of the corpus. The test reads the files of `crates/core/tests/corpus/`.
- A core test of `parse_rfc2822`: `Fri, 6 Feb 2026 09:54:10 +0000`, a value with `GMT`, a value with `-0500`, a value with `EST`, a value with no weekday, a value with no seconds. Each gives the correct seconds and offset.
- A core test of `parse_rfc2822`: an empty text, `2026-10-30`, a month name that is not English and a day of 32 each give `None`.
- A core test: `format_rfc2822` and `format_rfc3339` give the examples of the Architecture section, and `parse_rfc3339(format_rfc3339(m)) == Some(m)` for an offset of 0, `+05:30` and `-06:00`.
- A core test: `Feed::read` gives `pub_date_time` `2026-02-06T09:54:10+00:00` for the `pubDate` `Fri, 6 Feb 2026 09:54:10 +0000`, and `None` for the `pubDate` `yesterday`.
- Core tests of `replace_model` for the four conditions of ADR 0093 item 7:
  - The same value keeps the bytes of `pubDate`.
  - A new time writes RFC 2822 text with the sent offset.
  - A null removes `pubDate`.
  - A new track gets `pubDate` from a sent time.
- A core test: an item with the `pubDate` `yesterday` and a sent `pub_date_time` of null keeps `yesterday`.
- A core test: a sent `pub_date_time` of `2026-10-30T20:00` gives `InvalidModel`, and the draft does not change.
- Core tests of `live::problems`: a start of `2026-10-30T20:00` with no zone, and an end of `soon`, each give their problem. The three corpus live items give no new problem.
- The grant tests pass with no change of their expected values.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a corpus `pubDate` does not parse.
- Stop and report when a corpus feed changes after a read and a write with no edit.
- Stop and report when an `/app/v1/` route writes a live item with no `live::problems` check.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0093-dates-and-times-use-a-picker-in-local-time.md
- docs/tasks/studio-v1-task-055-date-time-fields.md
- docs/tasks/studio-v1-task-055a-times-in-the-core.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The core reads and writes RFC 2822 and RFC 3339 times.
- The core gives `pub_date_time` for each item, and writes `pubDate` from a changed `pub_date_time`.
- The core gives a problem for a live time with no zone.

Constraints:
- Follow the Architecture section of the task file fully.
- No UI change. No new crate.

Do not touch:
- The UI, the corpus files, the ADRs, the `Cargo.toml` files.
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
