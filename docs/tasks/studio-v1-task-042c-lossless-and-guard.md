# Studio v1 Task 042c: The Lossless Enclosures And The Guard

Parent: [task 042](studio-v1-task-042-core-feed-rules.md). Decisions: [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) items 10 to 12, [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) item 6, [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md) items 4 and 5, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the third packet of task 042. It depends on 042b.

Status: Not started.

## Goal

The core makes the two alternative enclosures of a lossless file. A Node test fails when one of the feed rules that task 042 removed comes back into the UI.

## Files

Inspect: `crates/core/src/feed/enclosures.rs` (`AlternateEnclosure`), `crates/core/src/feed/elements.rs` (`Source`), `crates/api/src/routes.rs` (`import_file`, `import_upload` and its facts JSON), `crates/api/src/uploads.rs` (the upload in parts calls `import_file`), `crates/api/src/tests.rs` (the import tests of a WAV file and of a FLAC file), `crates/api/ui/editor.js` (`alternateEnclosures`, `withNewAudio`, `ImportSheet`), `crates/api/ui-tests/`.

Change: `crates/core/src/feed/enclosures.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/api/ui/editor.js`, and the new file `crates/api/ui-tests/feed-rules.test.mjs`.

Do not change:

- `uploads.rs` and the WAV and FLAC conversion.
- The facts of the lossless file in `lossless`.
- The picker routes, the cover and the listing.

## Architecture

These choices are fixed. Do not change them.

### The Enclosures In The Core

- Add `pub fn lossless_pair(mp3_url: &str, mp3_size: u64, flac_url: &str, flac_size: u64) -> Vec<Entry<AlternateEnclosure>>` to `enclosures.rs`. Use the types of the sizes that the import facts use.
- It gives two new entries, as ADR 0033 item 6 gives:
  1. `media_type` `audio/mpeg`, `length` the MP3 size, `title` `Standard (MP3)`, `default` `true`, and one source with the MP3 address.
  2. `media_type` `audio/flac`, `length` the FLAC size, `title` `Maximum fidelity (lossless)`, no `default`, and one source with the FLAC address.
- `bitrate`, `rel` and the `content_type` of each source are `None`.
- Give the function a doc comment that names ADR 0033 item 6.

### The Upload Facts

- The facts of `import_upload` give `"alternate_enclosures"`. For a lossless file, the list holds the two entries of `lossless_pair`. Else it is an empty list.
- The facts keep `lossless` with its fields as now.

### The UI

- `editor.js`: remove `alternateEnclosures`.
- `withNewAudio` uses `withCids(facts.alternate_enclosures || [])`. The facts of a selected file have no such list, so the track gets no audio alternative enclosure, as now.
- The import sheet gives `alternate_enclosures: facts.alternate_enclosures || []` to each new track.

### The Guard

- Add `crates/api/ui-tests/feed-rules.test.mjs` (ADR 0065 item 12). It has one test for each removed function:
  - `cover.js` exports no `withCover`.
  - `listing.js` exports no `isHidden`, `withHidden` or `withPlatformBlock`.
  - The text of `editor.js` defines no function `alternateEnclosures`.
- Each failure message names ADR 0065, the item and the fix. For example: "ADR 0065 item 6: withCover is a feed rule. The core applies it at the draft save. Send the address in image."
- Remove the tests of 042a and 042b that assert the same removals. This test file replaces them.

## Steps

1. Add `lossless_pair` with a core unit test.
2. Add `alternate_enclosures` to the upload facts, with API tests.
3. Change `editor.js`.
4. Add the guard test, and remove the duplicate tests.
5. Run the four checks.

## Constraints

- No feed rule in the UI (ADR 0065).
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.

## Acceptance Criteria

Mechanical:

- A core test: `lossless_pair` gives the two entries with the media types, the lengths, the titles, the `default` and the sources of ADR 0033 item 6.
- An API test: the facts of a WAV import give two alternative enclosures. The first has the MP3 address and `default` `true`. The second has the FLAC address.
- An API test: the facts of an MP3 import give an empty `alternate_enclosures`.
- The Node test file `feed-rules.test.mjs` passes. With `withCover` added back to `cover.js`, it fails with a message that names ADR 0065.
- The four commit checks pass.

Visual: the WAV import of task 042, for the operator at the end of the task.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `uploads.rs` needs a change.
- Stop and report when a client other than the editor uses `lossless` to make the alternative enclosures.
- Stop and report when a test of the type dot, the words or the shell fails.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-042c-lossless-and-guard.md
- docs/adr/0065-the-core-applies-each-feed-rule.md
- docs/adr/0033-a-lossless-import-gives-an-mp3-enclosure.md
- The files that the Files section names.

Goal:
- Add `lossless_pair` to the core, and give `alternate_enclosures` in the upload facts.
- Remove `alternateEnclosures` from the UI.
- Add the guard test `feed-rules.test.mjs`.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- Each failure message of the guard names ADR 0065, the item and the fix.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `uploads.rs`, the WAV and FLAC conversion, the picker routes, the cover and the listing.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
