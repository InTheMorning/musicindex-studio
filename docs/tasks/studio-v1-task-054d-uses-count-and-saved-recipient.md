# Studio v1 Task 054d: One Count Of The Uses, And Discard Of The Saved Recipient

Parent: [task 054](studio-v1-task-054-recipient-change-sets.md). Decision: [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 2, 12 and 20.

Status: Complete. `GET /recipients` took 0.09 s for a collection with 58 feeds. A recipient that is not saved and that no feed model uses shows "0 feeds".

## Goal

The Recipients page and the page of a recipient give the same number of feeds. Discard of a change set puts back the saved recipient from before the change.

## Context

- The visual check of 054c on 2026-10-30 found the defect. The Recipients page said "6 feeds" for a recipient, and the page of that recipient listed 7 feeds. The seventh feed had only a draft.
- `GET /recipients` (`list_recipients`) counts the feeds from `Collection::known_recipients`, which reads `feed_recipients`. That table has the recorded state of each hosted feed.
- `POST /recipients/uses` (`recipient_uses`) reads the model of each feed with `files_feed`: the draft, or else the library copy.
- The route `make_change_set` of 054b calls `Collection::change_saved_recipient` for Change address and Rename. `discard_change_set` does not put back the saved recipient.

## Files

Inspect: `crates/api/src/routes.rs` (`list_recipients`, `recipient_uses`, `files_feed`, `make_change_set`, `discard_change_set`), `crates/core/src/collection.rs` (`known_recipients`, `KnownRecipient`, `saved_recipients`, `change_saved_recipient`, `MIGRATIONS`), `crates/core/src/change_set.rs`, `crates/api/src/tests.rs`, `crates/core/tests/change_set.rs`.

Change: `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/core/src/collection.rs`, `crates/core/src/change_set.rs`, `crates/core/tests/change_set.rs`.

Do not change: the UI, the ADRs, an existing migration, the recipient picker of the editor.

## Architecture

These choices are fixed. Do not change them.

1. One function in `routes.rs` finds the uses of the recipients in the model of each feed (`files_feed`). `recipient_uses` and `list_recipients` both use it. A feed with no model (no draft and no library copy) is not a use in both routes.
2. `list_recipients` keeps each entry of `known_recipients`, with its name and its type. Only the field `feeds` changes: it is the number of feeds that the function of item 1 gives for the key of the entry. Some recipients are only in a draft, so `known_recipients` does not have them. The route adds each of them, with the name and the type of the model.
3. The core makes the change of the saved recipient. `make_change_set` of `change_set.rs` changes the saved recipient of the old key for Change address and Rename, and keeps the saved recipient from before the change. The route `make_change_set` does not call `change_saved_recipient` again.
4. A new migration at the end of `MIGRATIONS` adds the column `saved_before TEXT` to `change_sets`. It holds the saved recipient from before the change as JSON, or NULL when no saved recipient changed.
5. `discard_change_set` puts back the saved recipient of `saved_before` before it deletes the change set.

## Steps

1. Write the function of item 1, and use it in the two routes.
2. Move the change of the saved recipient into the core, with the migration.
3. Put back the saved recipient in `discard_change_set`.
4. Run the four commit checks.

## Constraints

- No write to a host.
- No change of the JSON form of a route response, except the value of `feeds` in `GET /recipients`.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- An API test: a recipient in two written feeds and in one feed that has only a draft gives `feeds: 3` in `GET /recipients`, and three feeds in `recipients/uses`.
- An API test: a recipient that only a draft has is in `GET /recipients`.
- A core test: Rename changes the saved recipient, and Discard puts back its name.
- A core test: Change address changes the saved recipient, and Discard puts back its address, custom key, custom value and type.
- A core test: Discard of a change set with no saved recipient does not change the saved recipients.
- The tests of 054a and 054b pass with no change of their expected values, except the test of item 3 that moves to the core.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the function of item 1 makes `GET /recipients` slower than 1 second with the corpus feeds in a test collection.
- Stop and report when the recipient picker of the editor needs a different `feeds` value.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md
- docs/tasks/studio-v1-task-054-recipient-change-sets.md
- docs/tasks/studio-v1-task-054d-uses-count-and-saved-recipient.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The two recipient routes count the uses from the same model of each feed.
- The core changes the saved recipient with the change set, and Discard puts it back.

Constraints:
- Follow the Architecture section of the task file fully.
- No UI change. No change of an existing migration.

Do not touch:
- The UI, the ADRs.
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
