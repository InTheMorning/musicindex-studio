# Studio v1 Task 052a: The Plain Address

Parent: [task 052](studio-v1-task-052-op3-switch.md). Decision: [ADR 0076](../adr/0076-an-op3-switch-on-feed-settings.md) items 1 to 4.

Status: Open.

## Goal

Each comparison of a feed address with a file address, and each read of an enclosure by Studio, uses the address with no OP3 prefix. A file move keeps the prefix.

## Files

Inspect: `crates/core/src/media.rs`, `crates/core/src/pipeline.rs` (`feed_addresses` and each caller, `host_operations`, `library_file_in_use`, `feed_refers_in_library`, the confirmation near line 1626), `crates/core/src/remote.rs` (near lines 58 and 141), `crates/core/src/inventory.rs` (`copies`), `crates/core/src/download.rs`, `crates/core/src/folder_scan.rs`, `crates/api/src/media.rs` (`audio`, the tickets), `crates/api/src/routes.rs` (`media_facts`, the size step near line 3800), `crates/core/tests/corpus/` (the OP3 addresses of `jimmyv4v-music-jimmy-v-music.xml`, `musicsideproject-music-b-sides-the-sch3m3s.xml` and `phafe-music-the-singles.xml`).

Change: `crates/core/src/media.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/remote.rs`, `crates/core/src/inventory.rs`, `crates/core/src/download.rs`, `crates/api/src/media.rs`, `crates/api/src/routes.rs`, and their tests. Change another file only when it compares or reads an enclosure address, and then say so.

Do not change: the UI, the corpus files, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `media::plain_address(url: &str) -> Cow<'_, str>`:
  - A text that does not start with `https://op3.dev/e` gives the text.
  - Else remove `https://op3.dev/e`. Then remove a `,pg=…` part, which ends before the next `/`. Then remove that `/`.
  - When the remaining text starts with `http://` or `https://`, give it. Else add `https://` before it.
- `pipeline::feed_addresses` gives `Vec<Cow<'_, str>>` of plain addresses. Change each caller.
- Each other comparison of an enclosure `url` or a source `uri` with a file address uses `plain_address` on the feed side.
- Each read of an enclosure by Studio uses the plain address: `/media/facts`, `/media/audio` and its tickets, and the downloads of ADR 0038.
- When the core writes a new file address into a feed with a prefixed enclosure, it keeps the prefix and replaces the plain part. Give this one function, `media::with_plain_address(url, new_plain) -> String`.

## Steps

1. Write `plain_address` and `with_plain_address`, with tests.
2. Change `feed_addresses` and its callers.
3. Find each other comparison and each read with `grep -n "enclosure\|\.uri\b\|feed_addresses" crates/core/src crates/api/src -r`. Change each one. List each place in your report.
4. Run the four commit checks.

## Constraints

- No change of a feed when the user did not change it. The corpus tests continue to pass.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

## Acceptance Criteria

Mechanical:

- A core test of `plain_address`: each OP3 form of the corpus, `https://op3.dev/e/http://example.com/a.mp3`, a `pg=` form, and an address with no prefix.
- A core test of `with_plain_address`: the prefix and its `pg=` part stay.
- A core test: `library_file_in_use` finds a file that a prefixed enclosure refers to.
- A core test: the deletion check of ADR 0035 item 17 blocks the deletion of a file that a prefixed enclosure refers to.
- A core test: the confirmation finds a prefixed enclosure by its plain address.
- An API test: `/media/facts` and `/media/audio` read the plain address from a local test server.
- The four commit checks pass.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a comparison needs the prefixed address.
- Stop and report when a corpus feed changes after a read and a write with no edit.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0076-an-op3-switch-on-feed-settings.md
- docs/tasks/studio-v1-task-052-op3-switch.md
- docs/tasks/studio-v1-task-052a-plain-address.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Each comparison and each read of an enclosure uses the address with no OP3 prefix. A file move keeps the prefix.

Constraints:
- Follow the Architecture section of the task file fully.
- No UI change.

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
