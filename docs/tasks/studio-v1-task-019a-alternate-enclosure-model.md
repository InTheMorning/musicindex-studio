# Studio v1 Task 019a: The Alternative Enclosure In The Feed Model

Parent: [task 019](studio-v1-task-019-lossless-import.md). Decision: [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md) items 6 and 7.
Sequence: it depends on nothing. It can go at the same time as task 019b.

## Goal

Read and write `podcast:alternateEnclosure` in the typed model of an item. A pending file that a `podcast:source` refers to is in use.

## Files

Inspect: `crates/core/src/feed.rs` (`Item`), `crates/core/src/feed/value.rs` (`ValueBlock`, a model of an element with child lists), `crates/core/src/feed/elements.rs` (`element_type!`), `crates/core/src/feed/sync.rs` (the insertion point and `hosted_insertion_point`), `crates/core/src/feed/links.rs`, `crates/core/src/pipeline.rs` (`referenced_files`), `crates/core/tests/corpus/jimmyv4v-music-jimmy-v-music.xml`, the corpus tests and the pipeline tests.

Change: `feed.rs`, a new `crates/core/src/feed/enclosures.rs` or `value.rs` only if the pattern requires it, `elements.rs`, `sync.rs`, `links.rs`, `pipeline.rs`, and the tests.

Do not change: `crates/api`, the UI, `media.rs`, `collection.rs`.

## Architecture

These choices are fixed. Do not change them.

- `pub struct AlternateEnclosure { pub media_type: Option<String>, pub length: Option<String>, pub bitrate: Option<String>, pub title: Option<String>, pub rel: Option<String>, pub default: Option<String>, pub sources: Vec<Entry<Source>> }`. It is `Modeled` for `(Ns::Podcast, "alternateEnclosure")`. The attributes are `type`, `length`, `bitrate`, `title`, `rel` and `default`.
- `Source` is an `element_type!` for `(Ns::Podcast, "source")` with `uri => "uri"` and `content_type => "contentType"`.
- `Item` gets `pub alternate_enclosures: Vec<Entry<AlternateEnclosure>>` with `#[serde(default)]`, so that a model with no such field also reads.
- The read and the write keep each attribute and each child that the model does not have, such as `height`, `timeOffset` and `podcast:integrity`. A read and a write with no edit give the same bytes.
- A new `podcast:alternateEnclosure` in an item with none uses the hosted position of ADR 0031. When that gives no point, it goes directly after the `enclosure` of the item. A second new one goes after the first.
- `links.rs` checks the `uri` of each source, with the field path `items/{i}/alternate_enclosures/{j}/sources/{k}/uri`.
- `referenced_files` also counts the `uri` of each `podcast:source` of each item.

## Steps

1. Add `Source` and `AlternateEnclosure`, and the field of `Item`.
2. Add the position rule.
3. Change `links.rs` and `referenced_files`.
4. Add the tests.

## Constraints

- Keep each existing test.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The corpus tests pass with no change of a corpus file. A test reads the two alternative enclosures of the Jimmy V feed, with their type, length, bitrate and source URI.
- A test adds two alternative enclosures to a new item that has `title`, `guid`, `enclosure` and `podcast:episode`. The written item has the sequence `title`, `guid`, `enclosure`, the two alternative enclosures, `podcast:episode`, each with its own line and indent.
- A test changes the `length` of a hosted alternative enclosure. The written file keeps `height` and `timeOffset`.
- A pipeline test: a pending file that only a `podcast:source` refers to is in the diff and goes to the host.
- A source `uri` that is not a web address gives its field path in the address check.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the position rule needs a change of ADR 0031 behavior for a different element. Stop and report when a corpus file changes after a read and a write.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-019a-alternate-enclosure-model.md
- docs/adr/0033-a-lossless-import-gives-an-mp3-enclosure.md
- docs/adr/0031-a-new-element-takes-its-hosted-position.md
- The files that the Files section names.

Goal:
- Read and write `podcast:alternateEnclosure` in the item model. A pending file that a `podcast:source` refers to is in use.

Constraints:
- Follow the Architecture section of the task file fully.
- A read and a write with no edit give the same bytes.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api, the UI, media.rs, collection.rs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
