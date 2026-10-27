# Studio v1 Task 041a: The New Elements In The Core

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 1 to 5, [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md), [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md) and [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md).
Sequence: the first packet of task 041. It depends on task 040.

Status: Complete on 2026-10-27.

## Goal

The core models `itunes:explicit` of the channel and of each item, `itunes:owner`, `itunes:keywords`, `managingEditor`, `webMaster` and `podcast:season`. A read and a write with no edit give the same bytes. At the draft save, the core keeps the form of the explicit value.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed`, `Feed::read`, `Feed::write_hosted`, `Item`, `Item::read_fields`, `Item::apply_children`), `crates/core/src/feed/elements.rs` (`element_type!`, `Locked`, `RssImage`), `crates/core/src/feed/sync.rs` (`Children::text`, `Children::one`), `crates/core/src/pipeline.rs` (`replace_model`, `keep_identity`, `number_tracks`), `crates/core/tests/edits.rs` (the tests of `podcast:block` of task 040), `crates/core/tests/pipeline.rs`, `crates/core/tests/corpus.rs`.

Change: `crates/core/src/feed.rs`, `crates/core/src/feed/elements.rs`, `crates/core/src/pipeline.rs`, `crates/core/tests/edits.rs`, `crates/core/tests/pipeline.rs`.

Do not change: `crates/core/src/feed/sync.rs`, `crates/core/src/xml.rs`, the corpus files, the API, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Fields

- `Feed` gets these fields, each the text as the source wrote it:
  - `explicit: Option<String>` for `itunes:explicit`.
  - `owner: Option<Owner>` for `itunes:owner`.
  - `keywords: Option<String>` for `itunes:keywords`.
  - `managing_editor: Option<String>` for `managingEditor`.
  - `web_master: Option<String>` for `webMaster`.
- `Item` gets `explicit: Option<String>` for `itunes:explicit` and `season: Option<Season>` for `podcast:season`.
- Each new field has `#[serde(default)]`, so that a model with no such field also reads.

### The Types

- `Season` is an `element_type!` with `@text value` and the attribute `name`.
- `Owner` is a struct with `name: Option<String>` and `email: Option<String>`, for the children `itunes:name` and `itunes:email`. It implements `Modeled` as `RssImage` does: `read_text` for each child, and `Children` with no tail in `apply`. A child of `itunes:owner` that `Owner` does not model stays in the tree.
- Export `Owner` and `Season` from `feed.rs` with the other element types.

### The Write Sequence

- `Children` gives the position of a new element. The sequence of the calls does not give it.
- A new element with a hosted position takes that position (ADR 0031 item 1). Do not add a special condition for it.
- A new element with no hosted position goes before the first element of a type that must come after it (ADR 0031 item 3). In the channel, that is the first `item` or `podcast:liveItem`. In an item, the element goes at the end. ADR 0031 does not use a fixed sequence of elements.
- Put the read calls and the write calls in this sequence, for a clear code:
  - `managingEditor` and `webMaster` directly after `language`.
  - `itunes:explicit`, `itunes:owner` and `itunes:keywords` directly after `itunes:author`.
  - In `Item::apply_children`, `podcast:season` directly before `podcast:episode`, and `itunes:explicit` directly after `itunes:duration`.

### The Form Rule At The Draft Save

- `pipeline::replace_model` gets a third step after `number_tracks`. The step is `keep_explicit_form(&current, &mut sent)`. It applies ADR 0064 item 4 to the channel `explicit` and to the `explicit` of each item.
- The UI sends `true` or `false`. The step changes a sent value only when it differs from the draft value:
  - The draft value can be `yes` or `no` in any letter case, after a trim. Then the step writes `yes` for a sent `true` and `no` for a sent `false`. When the first letter of the draft value is a capital letter, the first letter of the result is a capital letter.
  - In each other case, the step keeps the sent value.
- An item of the sent model finds its draft item by its entry key, as `number_tracks` does. A new item has no draft item, so the step keeps its sent value.
- A sent `null` stays `null`. The step adds no element (ADR 0064 item 3).
- The core does not refuse a text that is not an email address (ADR 0064 item 5).

## Steps

1. Add `Season` and `Owner` with the `read` and `apply` functions.
2. Add the fields to `Feed` and `Item`, with the read calls and the write calls.
3. Add `keep_explicit_form` to `replace_model`.
4. Add the core tests.
5. Run the four checks.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No change to `sync.rs`.

## Acceptance Criteria

Mechanical:

- The corpus tests pass: each feed gives the same bytes after a read and a write with no edit.
- A core test reads each new element from a feed that has all of them. It gives the text and the attributes as the source wrote them.
- A core test changes each new element, one at a time. The diff of the document changes only that element.
- A core test sets each new element in a feed that does not have it, and with no hosted file. A new channel element goes directly before the first `item`. A new item element goes at the end of the item (ADR 0031 item 3).
- A core test sets `itunes:explicit` in a feed whose hosted file has it in a different position. The new element takes the hosted position.
- A core test writes the text `No` into a channel `itunes:explicit` that holds `no`, and the text `true` into one that holds `false`. Each feed holds the text that the model gives, and no other byte changes.
- A core test changes `itunes:email` of an owner that also has an unknown child. The unknown child stays.
- A core test reads a model JSON with no new field. Each new field is empty.
- A core test of `replace_model`: a sent `true` over a draft `no` gives `yes`, and a sent `false` over a draft `Yes` gives `No`.
- A core test of `replace_model`: a sent `true` over a draft `false` gives `true`, and a sent `true` over no draft value gives `true`.
- A core test of `replace_model`: the same rules apply to the `explicit` of an item, and a new item keeps its sent value.
- A core test of `replace_model`: a sent `null` over a draft `no` removes the element, and adds none.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a corpus feed does not give the same bytes after the change.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-041a-core-elements.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- The files that the Files section names.

Goal:
- Model `itunes:explicit`, `itunes:owner`, `itunes:keywords`, `managingEditor`, `webMaster` and `podcast:season` in the core.
- Keep the bytes of each feed after a read and a write with no edit.
- Keep the form of the explicit value at the draft save.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No change to `sync.rs` or `xml.rs`.
- The form rule is in `keep_explicit_form` only. The core does not refuse a text that is not an email address.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The corpus files, the API and the UI.
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
