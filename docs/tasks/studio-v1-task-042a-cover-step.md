# Studio v1 Task 042a: The Cover Step

Parent: [task 042](studio-v1-task-042-core-feed-rules.md). Decisions: [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) items 4 and 6, [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md) items 3 to 5 and 8, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the first packet of task 042. It depends on task 040.

Status: Not started.

## Goal

The core keeps the form of the cover at the draft save. The UI sends the new address in `image`, and it has no copy of the cover rule.

## Files

Inspect: `crates/core/src/pipeline.rs` (`replace_model`, `keep_identity`, `number_tracks`, `one_cover`), `crates/core/src/feed.rs` (the field `rss_image` and its doc comment), `crates/core/src/feed/elements.rs` (`RssImage`), `crates/core/tests/pipeline.rs` (`a_write_gives_the_rss_image_the_address_of_itunes_image`), `crates/core/tests/edits.rs` (the cover tests of task 038), `crates/api/ui/cover.js`, `crates/api/ui/draft.js` (`saveOnce`, `copyServerFields`), `crates/api/ui/editor.js` (`CoverField`, the import of the album image near `withCover(m, albumImageUrl)`), `crates/api/ui-tests/cover.test.mjs`.

Change: `crates/core/src/pipeline.rs`, `crates/core/src/feed.rs` (the doc comment only), `crates/core/tests/pipeline.rs`, `crates/api/ui/cover.js`, `crates/api/ui/draft.js`, `crates/api/ui/editor.js`, `crates/api/ui-tests/cover.test.mjs`.

Do not change:

- `one_cover` and the write rule of ADR 0060 item 8.
- The cover tests of task 038 in `crates/core/tests/edits.rs`.
- The listing, the alternative enclosures, `shell.js` and `app.js`.

## Architecture

These choices are fixed. Do not change them.

### The Step In The Core

- Add `fn keep_cover_form(current: &Feed, sent: &mut Feed)` to `pipeline.rs`. `replace_model` calls it after `number_tracks`.
- Give it a doc comment that names ADR 0060 items 3 to 5 and ADR 0065 item 6.
- The step does nothing when `sent.image` is the same as `current.image`.
- Else it applies the first case that matches:
  1. `sent.image` is `None` or an empty text: set `sent.image` and `sent.rss_image` to `None`.
  2. `current` has `rss_image` and no `image`: put the address in `rss_image.url` of `sent`, and set `sent.image` to `None`. Keep the other children of `rss_image`.
  3. `current` has `rss_image` and `image`: put the address in `rss_image.url` of `sent`. `sent.image` keeps the address.
  4. Else: `sent.image` keeps the address. The step adds no `rss_image`.
- In cases 2 and 3, start from `sent.rss_image`. When it is `None`, start from `current.rss_image`.
- The step compares the texts as they are. ADR 0061 trims the text at the write.
- Remove the sentence "The core has no cover rule" from the doc comment on `rss_image` in `feed.rs`. Name ADR 0065 item 6 there.

### The UI

- `cover.js`: remove `withCover`. Keep `coverOf`, `coverForm` and `coverConflict`.
- `cover.js`: add `withSavedCover(local, sent, saved)`. It gives `local` with `image` and `rss_image` from `saved` when the cover of `local` is the same as the cover of `sent`. Else it gives `local`. Two covers are the same when `image` is the same and `rss_image.url` is the same, with `null` for a missing tag.
- `draft.js` `saveOnce`: after `copyServerFields`, call `withSavedCover(current.current, sent, saved.model)`.
- `editor.js` `CoverField`: a change gives `{ ...m, image: address }`. A clear gives `{ ...m, image: '' }`, never `null`.
- `editor.js`: the import of the album image gives `{ ...m, image: albumImageUrl }`.
- The note of ADR 0060 item 4 and its action "Use this address" do not change. The action sends the address of the RSS image in `image`.

## Steps

1. Add `keep_cover_form` and its core tests.
2. Change the doc comment in `feed.rs`.
3. Add `withSavedCover` with its Node tests. Remove `withCover` and its Node tests.
4. Change `draft.js` and `editor.js`.
5. Run the four checks.

## Constraints

- No feed rule in the UI (ADR 0065). The UI sends the plain address and shows the value of the core.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate. No new route.
- A read and a write with no edit give the same bytes. The corpus tests must pass with no change.

## Acceptance Criteria

Mechanical:

- A core test in `crates/core/tests/pipeline.rs` for each case, through `replace_model`:
  - A DeMu draft (only `<image>`) with a new `image`: the draft has the new `<url>`, the other children of `<image>` and no `itunes:image`.
  - A draft with only `itunes:image` and a new `image`: the draft has the new `href` and no `<image>`.
  - A draft with both tags and the same address, and a new `image`: both tags have the new address.
  - A draft with both tags and different addresses, and the same `image` as the draft: both addresses stay.
  - The same draft and the RSS address in `image`: both tags have the RSS address.
  - A draft with no cover tag and a new `image`: the draft has `itunes:image` and no `<image>`.
  - A draft with both tags and an empty `image`: the draft has no `<image>` and no `itunes:image`.
  - A DeMu draft with an empty `image`: the draft has no `<image>`.
  - A DeMu draft with `image` set to `None`: the draft does not change.
- A Node test: `withSavedCover` gives the saved cover when the local cover is the same as the sent cover.
- A Node test: `withSavedCover` keeps the local cover when the user changed it during the save.
- A Node test: `cover.js` does not export `withCover`.
- The corpus round-trip tests pass.
- The four commit checks pass.

Visual: the cover field of task 042, for the operator at the end of the task.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a corpus test or a cover test of task 038 fails.
- Stop and report when a different route gives the model that the UI uses for the cover.
- Stop and report when `copyServerFields` must change in a way that this packet does not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-042a-cover-step.md
- docs/adr/0065-the-core-applies-each-feed-rule.md
- docs/adr/0060-the-album-cover-keeps-the-form-of-its-feed.md
- The files that the Files section names.

Goal:
- Add the step `keep_cover_form` to `pipeline::replace_model`.
- Remove `withCover` from the UI. The UI sends the plain address and takes the saved cover.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- A clear sends an empty text in `image`, never `null`.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `one_cover`, the cover tests of task 038, the listing, the alternative enclosures, `shell.js` and `app.js`.
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
