# Studio v1 Task 026a: The Feed List Facts

Parent: [task 026](studio-v1-task-026-shell.md). Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 2 and 2a.
Sequence: the first packet of task 026.

## Goal

`GET /api/v1/feeds` gives for each feed the time that it joined the collection, the state of its draft and its cover image. The Releases page needs these for the sort "Newest" and for the cover and the draft state of each row.

## Files

Inspect: `crates/core/src/collection.rs` (`FeedRow`, `FEED_SELECT`, `feed_row`, the `feeds` table), `crates/core/src/pipeline.rs` (`draft_changed`), `crates/api/src/routes.rs` (`list_feeds`), `crates/api/src/tests.rs`.

Change: `collection.rs`, `routes.rs`, `tests.rs`, and the core tests when a test builds a `FeedRow`.

Do not change: the UI, `pipeline.rs`, the connector code.

## Architecture

These choices are fixed. Do not change them.

- `FeedRow` gets `added_at: i64`, from the column `created_at` of `feeds`. The JSON name is `added_at`. No migration is necessary.
- `list_feeds` gives each row with one more field, `draft_state`:

  | Value | Meaning |
  | --- | --- |
  | `"none"` | The feed has no draft. |
  | `"new"` | The feed has a draft and no hosted file (`last_version` is null). |
  | `"changed"` | `pipeline::draft_changed` gives true. |
  | `"unchanged"` | `pipeline::draft_changed` gives false. |

- `list_feeds` also gives `image`: the `image` of the draft model, or of the hosted feed in the release folder when the feed has no draft, as `files_feed` reads them. It is null when the two give no image.
- `list_feeds` puts `draft_state` and `image` into the JSON of each row. `FeedRow` does not have it. When `draft_changed` gives an error for one feed, that feed gets `"changed"`, and the list gives no error.
- The sequence of the list does not change.

## Steps

1. Add `added_at` to `FeedRow`, with a test.
2. Add `draft_state` and `image` to `list_feeds`, with an API test.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate, and no new migration.

## Acceptance Criteria

Mechanical:

- A feed row has `added_at`. A feed that the collection gets after a different feed has a greater or equal value.
- `GET /feeds` gives `draft_state` `"new"` for a new release, and `"unchanged"` after its write. It gives `"changed"` after an edit of its draft, and `"none"` for a read-only feed with no draft.
- `GET /feeds` gives the `image` of a draft after an edit of the cover address.
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

Stop and report when `draft_changed` needs a change.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-026a-feed-list-facts.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- The files that the Files section names.

Goal:
- GET /api/v1/feeds gives for each feed the time that it joined the collection, the state of its draft and its cover image.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate, and no new migration.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The UI, crates/core/src/pipeline.rs, the connector code.
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
