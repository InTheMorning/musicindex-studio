# ADR 0094: A Duration Is Edited As Minutes And Seconds

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Uses: [ADR 0012](0012-lossless-tree-with-a-typed-model.md) (the lossless tree), [ADR 0042](0042-ui-models-with-node-tests.md) (the UI models), [ADR 0062](0062-a-file-picker-for-each-file-field.md) (the file picker and the audio player), [ADR 0065](0065-the-core-applies-each-feed-rule.md) (each feed rule is in the core) and [ADR 0093](0093-dates-and-times-use-a-picker-in-local-time.md) (the same method for `pubDate`).

## Context

- On 2026-10-30 the operator showed the track field "Duration" with the value `261`. The user must calculate that `261` is 4 minutes and 21 seconds.
- The field is a text field with the hint "Seconds, or HH:MM:SS." The core writes `itunes:duration` with the text that the UI sends, with no check.
- The UI makes duration text itself. `durationFromSeconds` in `crates/api/ui/audio.js` gives `HH:MM:SS` from the audio player. This is a feed rule in the UI, against ADR 0065.
- The core gives the duration of an imported audio file as `HH:MM:SS` (`format_duration` in `crates/core/src/media.rs`).
- The corpus has 549 `itunes:duration` values: 488 in the form `MM:SS`, 43 in seconds and 18 in the form `HH:MM:SS`.
- The operator selected this decision from a questionnaire on 2026-10-30. The operator examined a display of minutes next to the value, and a read-only field, and did not select them.

## Decision

1. The JSON model of each item gets `duration_seconds`: the number of seconds that the core reads from `itunes:duration`, or null. The core reads seconds, seconds with a fraction, `M:SS`, `MM:SS`, `H:MM:SS` and `HH:MM:SS`. A fraction rounds down. Other text gives null.
2. At the draft save, the core compares the sent `duration_seconds` of each item with the value that it reads from the current `itunes:duration`:
   - The same value: the core keeps the current `itunes:duration`, byte for byte.
   - A different number: the core writes `itunes:duration` as `HH:MM:SS`, the form that `media.rs` already writes.
   - Null: the core removes `itunes:duration`.
   - The core ignores the sent `duration`. Only the core sets it.
3. The track field "Duration" shows `duration_seconds` as `M:SS`, or as `H:MM:SS` from one hour. The user types `4:21`, `1:02:03` or `261`. A model function in `crates/api/ui/duration.js` reads the text and gives the number of seconds. This is an edit aid, because the core makes the feed text (ADR 0065).
4. Text that the function cannot read shows the error "Type the duration as minutes and seconds, for example 4:21." The model does not change until the text is correct.
5. When the core cannot read the `itunes:duration` of an item, the field is empty. Its hint gives the source text, as ADR 0093 item 8 does for a date.
6. The audio player, the file picker and the import set `duration_seconds`, not `duration`. `durationFromSeconds` of `audio.js` goes.

## Alternatives Considered

- A display of minutes next to the value, with no change of the field. Not selected by the operator: the user then types seconds or `HH:MM:SS`.
- A read-only field from the audio file. Not selected by the operator: some hosts give no duration, and the user cannot correct an incorrect value.
- The core writes seconds, as Apple recommends. Not chosen: Studio already writes `HH:MM:SS` from an imported file, and the apps read both forms. One form for each write keeps the feeds of a user the same.
- The UI makes the `HH:MM:SS` text. Not chosen: the form of `itunes:duration` is a feed rule (ADR 0065).

## Consequences

- The user reads and types a duration as minutes and seconds.
- An `itunes:duration` that the user does not change keeps its bytes.
- The UI has one less feed rule.

## Invariants

- A core test reads each `itunes:duration` of the corpus.
- A core test examines the four cases of item 2.
- The corpus tests make sure that a read and a write with no edit give the same bytes.
- `crates/api/ui-tests/duration.test.mjs` tests the read and the display of item 3, and the error of item 4.

## Verification

Mechanical: the invariants above, and the four commit checks.

Visual: the track page with a duration of less than one hour, with a duration of more than one hour, and with an unreadable duration. In the two themes, and at 390 px.
