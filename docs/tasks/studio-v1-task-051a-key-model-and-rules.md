# Studio v1 Task 051a: The Keys In The Feed Model And The Key Rules

Parent: [task 051](studio-v1-task-051-nostr-keys.md). Decision: [ADR 0074](../adr/0074-nostr-keys-for-artists-labels-and-contributors.md) items 1 to 6.

Status: Complete - 2026-10-28. The four commit checks pass.

## Goal

`Person` has the field `npub`. The core gives each changed key the `npub` form, lists each key that does not decode, and refuses a new bad key at the write.

## Files

Inspect: `crates/core/src/feed/elements.rs` (`Person`, `Txt`, the macro `element_type!`), `crates/core/src/feed.rs` (`txts`, `persons`, `bad_addresses`), `crates/core/src/feed/links.rs` (the `Checker` pattern), `crates/core/src/nostr_events.rs` (`parse_public_key`), `crates/core/src/pipeline.rs` (`replace_model`, `keep_explicit_form`, the `BadAddress` check of the write near line 2404, `hosted_document`), `crates/api/src/routes.rs` (`draft_json`, the save route), `crates/core/tests/corpus/phafe-music-the-singles.xml`.

Change: `crates/core/src/feed/elements.rs`, `crates/core/src/feed.rs`, a new file `crates/core/src/feed/keys.rs`, `crates/core/src/nostr_events.rs`, `crates/core/src/pipeline.rs`, `crates/api/src/routes.rs`, and their tests.

Do not change: the UI, the corpus files, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `Person` gets `npub => "npub"`. The corpus tests show that a read and a write with no edit give the same bytes.
- `nostr_events::npub_text(text: &str) -> Option<String>`: trim the text, and remove a `nostr:` prefix. Then accept `npub1…`, `nprofile1…` (use its public key) or 64 hexadecimal characters. Give the `npub` text. Give `None` for other text and for an empty text.
- `pipeline.rs` gets the step `keep_key_form(current, sent)` in `replace_model`, after `keep_explicit_form`. It examines:
  - `npub` of each channel person, and of each person of each item and each live item.
  - `value` of each channel `txts` entry with `purpose` equal to `npub`.
  
  A value that equals the value of the same entry in `current` (by entry key) does not change. A new entry has no current value. For each other value, `npub_text` gives the new value. When it gives `None`, the value stays as the user typed it.
- `feed/keys.rs` gives `Feed::bad_keys() -> Vec<BadKey>`, with `BadKey { field, value }` as `BadAddress` has. A key is bad when it is not empty and `npub_text(value)` is not `Some(value)`. The field paths are `persons/{i}/npub`, `items/{j}/persons/{i}/npub`, `live_items/{j}/persons/{i}/npub` and `txts/{i}/value`. Use the paths of the JSON model.
- `draft_json` and the save response give `bad_keys`, adjacent to `bad_addresses`.
- The write gets `PipelineErrorKind::BadKey`. It refuses the write when a bad key has a text that no key field of the hosted feed has. A feed with no hosted file has no such text. The message names each field, as the `BadAddress` message does.

## Steps

1. Add `npub` to `Person`, and run the corpus tests.
2. Write `npub_text` and its tests.
3. Write `keep_key_form` and its tests.
4. Write `bad_keys`, the API fields and the write check, with their tests.
5. Run the four commit checks.

## Constraints

- No HTTP request in this packet.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- The corpus tests pass with no change of a corpus file.
- A core test: `npub_text` gives the same `npub` for an `npub`, an `nprofile` of the same key, the hexadecimal key, and `nostr:npub1…`. It gives `None` for `npub1xyz`, for an `nsec` and for an empty text.
- A core test: a sent hexadecimal key of a new person becomes the `npub`. A sent `nprofile` in the feed key becomes the `npub`. A key that the user did not change stays as it is, also when it is bad.
- A core test: an edit of one person key changes only that attribute in the written XML.
- A core test: `bad_keys` lists a bad person key, a bad item person key and a bad feed key, with their paths.
- A core test: the write refuses a new bad key with `BadKey`. It does not refuse a bad key that the hosted feed has.
- An API test: the draft response and the save response give `bad_keys`.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a corpus feed changes after a read and a write with no edit.
- Stop and report when `replace_model` cannot find the current entry of a sent person by its key.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0074-nostr-keys-for-artists-labels-and-contributors.md
- docs/tasks/studio-v1-task-051-nostr-keys.md
- docs/tasks/studio-v1-task-051a-key-model-and-rules.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Person has npub. The core gives each changed key the npub form, lists bad keys, and refuses a new bad key at the write.

Constraints:
- Follow the Architecture section of the task file fully.
- No HTTP request. No UI change.

Do not touch:
- The UI, the corpus files, the ADRs.
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
