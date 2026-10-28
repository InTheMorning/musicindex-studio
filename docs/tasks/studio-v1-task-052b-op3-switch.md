# Studio v1 Task 052b: The Switch

Parent: [task 052](studio-v1-task-052-op3-switch.md). Decision: [ADR 0076](../adr/0076-an-op3-switch-on-feed-settings.md) items 5 to 12.

Status: Open.

## Goal

The page "Feed settings" of an album has the switch "OP3 statistics". The core gives its start value, and adds or removes the prefix of each track enclosure at the draft save.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed`, `Feed::read`, `adopt`, `write_hosted`), `crates/core/src/pipeline.rs` (`replace_model`, `keep_explicit_form`), `crates/core/src/media.rs` (`plain_address` of task 052a), `crates/api/src/routes.rs` (`draft_json`), `crates/api/ui/editor.js` (`SettingsPage`, the switch of "Lock against import"), `crates/api/ui/explicit.js` and `crates/api/ui-tests/explicit.test.mjs` (a switch model example), `crates/api/ui/pages.js` (`settings`), `crates/api/ui/draft.js`, `docs/user-guide/edit-album.md`.

Change: `crates/core/src/feed.rs`, `crates/core/src/pipeline.rs`, `crates/core/src/media.rs`, `crates/api/src/routes.rs`, a new model module `crates/api/ui/op3.js`, a new test `crates/api/ui-tests/op3.test.mjs`, `crates/api/ui/editor.js`, `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/draft.js`, `crates/api/src/ui.rs` (the list of served files), `docs/user-guide/edit-album.md`.

Do not change: the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `media::op3_prefix(url, guid) -> String`: `https://op3.dev/e,pg=GUID/` and the address with no `https://`. An `http://` address keeps its scheme. A prefixed address does not change.
- `Feed` gets `op3: bool`, with `#[serde(default)]`. `Feed::read` sets it to true when the feed has one or more items, and the enclosure `url` of each item starts with `https://op3.dev/e`. `write_hosted` ignores it.
- `pipeline.rs` gets the step `apply_op3(current, sent)` in `replace_model`, after `keep_explicit_form`. Use the `podcast:guid` of `current`:
  - `sent.op3` true: give `op3_prefix` to each item enclosure with no prefix.
  - `sent.op3` false and `current.op3` true: give `plain_address` to each item enclosure.
  - `sent.op3` false and `current.op3` false: no change.
- `draft_json` and the save response give `op3_tracks: { prefixed, tracks }`, the counts of the items of the draft.
- `op3.js` has no Preact and no browser state (ADR 0042). It exports `op3Hint({ prefixed, tracks })`: `null` when `prefixed` is 0 or equals `tracks`. Else "N of M tracks use OP3". It exports `op3Page(guid)`: `https://op3.dev/show/GUID`.
- `pages.js`: the page `settings` gets the section `op3`, tier `setup`, after `locked`. Only an album shows it. Do not show it on a publisher feed or a playlist.
- `SettingsPage` renders `op3` as the switch of "Lock against import": the label "OP3 statistics" and the hint "OP3 counts the downloads of each track. Studio adds the OP3 address to each audio address." The hint of `op3Hint` follows. When the switch is on, the section shows the link "OP3 statistics" to `op3Page(model.podcast_guid)`, with `target="_blank"` and `rel="noopener"`.

## Steps

1. Write `op3_prefix`, `Feed.op3` and `apply_op3`, with core tests.
2. Add `op3_tracks` to the API, with an API test.
3. Write `op3.js` and `op3.test.mjs`. Change `pages.js`, `pages.test.mjs`, `draft.js` and `editor.js`.
4. Add the switch to `edit-album.md`.
5. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the changed guide lines.
6. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). The UI sends only `op3`. Name each new function that changes the model in your report.
- Each labeled button has its role of ADR 0037.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- A core test: `op3_prefix` of an `https` address, of an `http` address and of a prefixed address.
- A core test: `Feed::read` gives `op3` true for an album with a prefix on each track. It gives false for a mixed set and for an album with no track.
- A core test: a sent `true` adds the prefix with the feed GUID to each track. A sent `false` over `true` removes each prefix. A sent `false` over a mixed set changes no address.
- A core test: a new track that the user adds while `op3` is true gets the prefix.
- A core test: the switch does not change an alternate enclosure, an image or a live item.
- `op3.test.mjs` tests `op3Hint` and `op3Page`. `pages.test.mjs` gives `op3` after `locked`.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when the JSON model cannot hold a value that is not an element.
- Stop and report when the Feed settings page has no way to know that the feed is an album.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0076-an-op3-switch-on-feed-settings.md
- docs/tasks/studio-v1-task-052-op3-switch.md
- docs/tasks/studio-v1-task-052b-op3-switch.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The Feed settings page of an album has the switch "OP3 statistics". The core gives its start value and adds or removes the prefix at the draft save.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- The ADRs.
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
