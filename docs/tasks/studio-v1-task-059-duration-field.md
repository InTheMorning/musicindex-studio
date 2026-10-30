# Studio v1 Task 059: The Duration Field

Decision: [ADR 0094](../adr/0094-a-duration-is-edited-as-minutes-and-seconds.md). Also [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md).

Status: Not started. Packet 059b starts after the commit of 059a. Task 058 also changes `editor.js`, so 059b and 058 do not occur at the same time.

## Goal

The user reads and types the duration of a track as minutes and seconds. The core writes `itunes:duration`.

## Packet 059a: The Duration In The Core

### Files

Inspect: `crates/core/src/feed.rs` (`Item`, `Item::read_fields`, the field `pub_date_time`), `crates/core/src/datetime.rs` (an example of a small parser with tests), `crates/core/src/pipeline.rs` (`replace_model`, `apply_pub_date_time`), `crates/core/src/media.rs` (`format_duration`), `crates/core/tests/corpus/`.

Change: `crates/core/src/duration.rs` (new), `crates/core/src/lib.rs`, `crates/core/src/feed.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/media.rs` (only to use the new format function), and their tests.

Do not change: the UI, the API routes, the corpus files, the ADRs, the `Cargo.toml` files.

### Architecture

These choices are fixed. Do not change them.

1. `crates/core/src/duration.rs` has:
   - `pub fn parse_duration(text: &str) -> Option<u64>`: the forms of ADR 0094 item 1. Space at the ends is ignored. Minutes and seconds after the first part are from 0 to 59. A fraction of the seconds rounds down. Other text, an empty text and a negative value give `None`.
   - `pub fn format_duration(seconds: u64) -> String`: `HH:MM:SS` with two digits in each part, for example `00:04:21`. More than 99 hours gives more digits in the hours.
2. `media.rs` uses `format_duration` of `duration.rs`. Its output does not change.
3. `Item` gets `pub duration_seconds: Option<u64>` with `#[serde(default)]` and a doc comment as the one of `pub_date_time`. `Item::read_fields` sets it with `parse_duration`. `write_hosted` ignores it.
4. `replace_model` calls `apply_duration_seconds(&current, &mut sent)` after `apply_pub_date_time`. It applies the four cases of ADR 0094 item 2 to each sent item and to the item of each sent live item. A new item with `duration_seconds` null keeps no `itunes:duration`.

### Acceptance Criteria

Mechanical:

- A core test: `parse_duration` reads each `itunes:duration` of the corpus.
- A core test: `parse_duration` gives 261 for `261`, `4:21`, `04:21`, `0:04:21`, `00:04:21` and `261.9`. It gives 3723 for `1:02:03`. It gives `None` for an empty text, `4:61`, `four minutes`, `-5` and `1:2:3:4`.
- A core test: `format_duration` gives `00:04:21` for 261 and `01:02:03` for 3723.
- Core tests of `replace_model` for the four cases of ADR 0094 item 2. The same value keeps the bytes of `itunes:duration`, also for the form `261`.
- A core test: an item with the duration `soon` and a sent `duration_seconds` of null keeps `soon`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0094-a-duration-is-edited-as-minutes-and-seconds.md
- docs/tasks/studio-v1-task-059-duration-field.md (packet 059a only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 059a names.

Goal:
- The core reads `itunes:duration` into `duration_seconds`, and writes `itunes:duration` as `HH:MM:SS` from a changed `duration_seconds`.

Constraints:
- Follow the Architecture section of packet 059a fully.
- No UI change. No new crate.

Do not touch:
- The UI, the corpus files, the ADRs, the `Cargo.toml` files.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 059a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Packet 059b: The Duration Field In The UI

### Files

Inspect: `crates/api/ui/editor.js` (the field `duration` near the `TextField` with the hint "Seconds, or HH:MM:SS.", the new items, the picker near `facts.duration`, the player near `metadataDone`, the import near `row.facts.duration`), `crates/api/ui/audio.js` (`durationFromSeconds`, `clockText`, `DURATION_NOTICE`), `crates/api/ui/app.js` (`Field`, `DateTimeField` as an example), `crates/api/ui/datetime.js` (an example of a model module), `crates/api/src/ui.rs`, `crates/api/ui-tests/audio.test.mjs`.

Change: `crates/api/ui/duration.js` (new), `crates/api/ui/editor.js`, `crates/api/ui/audio.js`, `crates/api/ui/app.js` (only for a `DurationField`), `crates/api/src/ui.rs`, `crates/api/ui-tests/duration.test.mjs` (new), `crates/api/ui-tests/audio.test.mjs`.

Do not change: the core, the API routes, the ADRs.

### Architecture

These choices are fixed. Do not change them.

1. `crates/api/ui/duration.js` is a model module with no Preact (ADR 0042). It has:
   - `readDuration(text)`: the number of seconds for `4:21`, `1:02:03` or `261`, with the rules of `parse_duration` of packet 059a. Other text gives null. It is an edit aid. The UI makes no `itunes:duration` text.
   - `durationText(seconds)`: `M:SS` below one hour, else `H:MM:SS`. Null gives an empty text.
   - `DURATION_ERROR`: "Type the duration as minutes and seconds, for example 4:21."
   - `unreadDurationHint(text)`: "The feed has the duration “<text>”. Studio cannot read it. Type a duration to replace it."
2. `DurationField({ id, value, onChange, sourceText, notice })` in `app.js` is a text field with the label "Duration".
   - It shows `durationText(value)`.
   - It keeps the typed text while the user types.
   - An empty text calls `onChange(null)`. Text that `readDuration` reads calls `onChange(seconds)`.
   - Other text shows `DURATION_ERROR` as the field error, and does not call `onChange`.
   - When the field loses the focus, it shows `durationText(value)` again.
   - The hint is `notice` when the caller gives it, else `unreadDurationHint(sourceText)` when `value` is null and `sourceText` is not empty, else "Minutes and seconds, for example 4:21."
3. The track row `duration` uses `DurationField` with `value=${item.duration_seconds}`, `sourceText=${item.duration}` and `onChange=${change('duration_seconds')}`. `notice` is `DURATION_NOTICE` when the metadata read did not give a duration, as before this task. The UI does not change `item.duration`.
4. The player gives `metadataDone` the number of seconds, rounded down, and the track gets `duration_seconds`. The picker and the import set `duration_seconds` with `readDuration(facts.duration)`, because the API gives the duration of a file as `HH:MM:SS`. The new items get `duration_seconds: null`.
5. `durationFromSeconds` of `audio.js` goes, with its tests.
6. `ui.rs` serves `duration.js` as it serves `datetime.js`.

### Acceptance Criteria

Mechanical:

- A Node test: `readDuration` gives 261 for `4:21`, `04:21`, `0:04:21` and `261`. It gives 3723 for `1:02:03`. It gives null for `4:61`, `four`, an empty text and `1:2:3:4`.
- A Node test: `durationText` gives `4:21` for 261, `1:02:03` for 3723, `0:05` for 5 and an empty text for null.
- A Node test: no file of `crates/api/ui/` other than `duration.js` has `durationFromSeconds`, and `editor.js` does not set `duration:` to a value other than null in a new item. The message names ADR 0094 item 6.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- A track of less than one hour shows `4:21`. A typed `1:02:03` writes `01:02:03` in the diff of the review sheet.
- The text `4:61` shows the error.
- `$edges()` finds no content at the edge of a group.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0094-a-duration-is-edited-as-minutes-and-seconds.md
- docs/tasks/studio-v1-task-059-duration-field.md (packet 059b only)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 059b names.

Goal:
- The track field "Duration" shows and accepts minutes and seconds, and edits `duration_seconds`.

Constraints:
- Follow the Architecture section of packet 059b fully.
- No feed rule in the UI (ADR 0065). The UI makes no `itunes:duration` text.

Do not touch:
- The core, the API routes, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 059b.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
