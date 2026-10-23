# Studio v1 Task 015: A New Element Takes Its Hosted Position

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decision: [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md).
Depends on: [task 010](studio-v1-task-010-draft-api.md).

## Goal

When the writer adds a direct child of the channel or of an item, put it at its position in the hosted file.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed::write`, the item writer near `Children::new(element, &[], names)`), `crates/core/src/feed/sync.rs` (`Names`, `Children`, `write_single`, `insertion_point`, the list sync that calls `insertion_point`), `crates/core/src/pipeline.rs` (`edit_draft`, `replace_model`), `crates/core/src/collection.rs` (`DraftRow`), and the tests in `crates/core/tests/edits.rs`.

Change: `feed.rs`, `feed/sync.rs`, `pipeline.rs`, and the tests.

Do not change `crates/api`, the UI, the collection schema or the corpus files. In `xml.rs`, add only a helper that reads.

## Architecture

These choices are fixed. Do not change them.

- `Feed::write(&self, doc)` keeps its signature and behavior. A new method `Feed::write_hosted(&self, doc: &mut Document, hosted: Option<&Document>) -> Result<(), FeedError>` does the work. `write` calls it with `None`.
- The type of an element is its namespace and its local name, the same pair that `Element::is(ns, local)` tests.
- The hosted sequence of a parent is the list of the types of its element children, in sequence.
- `Names` carries the write context. It gets one field, `hosted: Option<Hosted>`. `Hosted` holds the hosted sequence of the channel, and a map from each item GUID to the hosted sequence of that item. `write_hosted` fills it from `hosted`. Give the field a doc comment that says why it is in `Names`: `Names` goes to each writer of the tree.
- `Children` gets the hosted sequence of its parent, or none. `Feed::write_hosted` gives the channel sequence. The item writer gives the sequence of the hosted item with the GUID of the draft item, or none. The writers of nested elements, such as `podcast:value`, give none (ADR 0031 item 5).
- `insertion_point` gets the hosted sequence and the type of the new element. When no element of that type remains in the draft parent (`used == 0`), and the hosted sequence has that type:
  1. Take the types before the first occurrence of that type in the hosted sequence.
  2. From the nearest of these types back to the first, find the last draft child of that type.
  3. When one is found, the new element goes after it, with the indentation of that child.
  When no child is found, the current rules apply.
- Extension of 2026-10-23 (ADR 0031 items 1, 2 and 6): `hosted_item` gives the sequence of the last hosted `item` to an item that the hosted file does not have. When no earlier sibling is in the draft, the new element goes before the nearest later sibling. The planning model made this extension, with the test `a_new_item_takes_the_sequence_of_the_last_hosted_item`.
- `edit_draft` and `replace_model` parse `base_xml` of the draft and call `write_hosted` with it. When `base_xml` is empty or does not parse, they call it with `None`.

## Steps

1. Add `Hosted`, the field of `Names`, and `write_hosted`.
2. Give `Children` and `insertion_point` the hosted sequence, and add the rule.
3. Change `edit_draft` and `replace_model`.
4. Add the tests of the acceptance criteria.

## Constraints

- A read and a write with no edit give the same bytes. The corpus tests enforce this ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- Keep each existing test.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A feed has `title`, `itunes:author`, `description`, `link` and `language`, in that sequence, then items. The test writes a model with no description over the feed. Then it writes a model with a new description, with that feed as `hosted`. The result has `description` between `itunes:author` and `link`.
- The same test through the pipeline: `replace_model` with the description `None`, then with a new text. The unified diff of the draft against the base changes only the text of the description.
- The same rule for an item: remove and add again the `itunes:image` of an item, and it goes back to its hosted position.
- A new channel element that the hosted file does not have, such as `podcast:license`, goes to the same position as before this task.
- The corpus tests pass with no change.
- The three commit checks pass.

Visual:

- The operator repeats step 4 of the visual check of Phase 5, and the diff changes only the text of the description and the language.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when:

- the item writer cannot get the GUID of its item before it writes the children.
- `Names` is not passed to a writer that adds a direct child of the channel or of an item.
- a corpus test changes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-015-hosted-position.md
- docs/adr/0031-a-new-element-takes-its-hosted-position.md
- crates/core/src/feed.rs
- crates/core/src/feed/sync.rs
- crates/core/src/pipeline.rs
- crates/core/tests/edits.rs

Goal:
- A direct child of the channel or of an item that the writer adds goes to its position in the hosted file.

Constraints:
- Follow the Architecture section of the task file fully.
- A read and a write with no edit give the same bytes. Keep each existing test.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api, the UI, the collection schema, the corpus files.
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
