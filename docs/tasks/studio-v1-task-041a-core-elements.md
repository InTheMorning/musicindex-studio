# Studio v1 Task 041a: The New Elements In The Core

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 1 to 5, [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md) and [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md).
Sequence: the first packet of task 041. It depends on task 040.

Status: Not started.

## Goal

The core models `itunes:explicit` of the channel and of each item, `itunes:owner`, `itunes:keywords`, `managingEditor`, `webMaster` and `podcast:season`. A read and a write with no edit give the same bytes.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed`, `Feed::read`, `Feed::write_hosted`, `Item`, `Item::read_fields`, `Item::apply_children`), `crates/core/src/feed/elements.rs` (`element_type!`, `Locked`, `RssImage`), `crates/core/src/feed/sync.rs` (`Children::text`, `Children::one`), `crates/core/tests/edits.rs` (the tests of `podcast:block` of task 040), `crates/core/tests/corpus.rs`.

Change: `crates/core/src/feed.rs`, `crates/core/src/feed/elements.rs`, `crates/core/tests/edits.rs`.

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

- The sequence of the calls in `write_hosted` gives the position of a new element with no hosted position. Add the calls at these positions:
  - `managingEditor` and `webMaster` directly after `language`.
  - `itunes:explicit`, `itunes:owner` and `itunes:keywords` directly after `itunes:author`.
- In `Item::apply_children`, add `podcast:season` directly before `podcast:episode`, and `itunes:explicit` directly after `itunes:duration`.
- A new element with a hosted position takes that position (ADR 0031). Do not add a special case for it.

### No Rule In The Core

- The core does not change a value. It does not convert `yes` to `true`, and it does not refuse a text that is not an email address (ADR 0064 items 4 and 5).
- The core adds no element that the model does not give (ADR 0064 item 3).

## Steps

1. Add `Season` and `Owner` with the `read` and `apply` functions.
2. Add the fields to `Feed` and `Item`, with the read calls and the write calls.
3. Add the core tests.
4. Run the four checks.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- No change to `sync.rs`. When `Children` cannot write a new element at the position above, stop and report.

## Acceptance Criteria

Mechanical:

- The corpus tests pass: each feed gives the same bytes after a read and a write with no edit.
- A core test reads each new element from a feed that has all of them. It gives the text and the attributes as the source wrote them.
- A core test changes each new element, one at a time. The diff of the document changes only that element.
- A core test sets each new element in a feed that does not have it, and with no hosted file. The element goes to the position of the Write Sequence section.
- A core test sets `itunes:explicit` in a feed whose hosted file has it in a different position. The new element takes the hosted position.
- A core test writes the text `No` into a channel `itunes:explicit` that holds `no`, and the text `true` into one that holds `false`. Each feed holds the text that the model gives, and no other byte changes.
- A core test changes `itunes:email` of an owner that also has an unknown child. The unknown child stays.
- A core test reads a model JSON with no new field. Each new field is empty.
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
- Stop and report when `Children` cannot put a new element at the position of the Write Sequence section.

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

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No change to `sync.rs` or `xml.rs`.
- The core does not change or refuse a value.
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
