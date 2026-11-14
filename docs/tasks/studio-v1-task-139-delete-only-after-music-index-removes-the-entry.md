# Studio v1 Task 139: Studio Deletes A Feed File Only After Music Index Removes The Entry

Decision: [ADR 0200](../adr/0200-studio-deletes-a-feed-file-only-after-music-index-removes-the-entry.md).

Status: Open.

## Goal

1. The sheet "Delete from File Host" shows **Delete the File Only** and the tag "Recommended" only when the retire state has `removal_done: true` (ADR 0200 items 2 to 4).
2. Step 3 of the Retire block has no button before `removal_done` (item 5).
3. The route `POST /feeds/{id}/delete/draft` refuses with `409` and the code `index_holds_feed` when the read of Music Index does not give `removed` (items 6 and 7).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/feeddelete.js:4` `DELETE_CHOICES`, and `:24` `retireSteps` with the comment of ADR 0131 item 9 and `deleteButton`.
- `crates/api/ui/editor.js`, `FeedDelete` (about line 2348): the sheet `delete-host-title` maps `DELETE_CHOICES`. `firstOpen` opens the sheet. `loadRetire` reads `/feeds/${feed.id}/retire/state`.
- `crates/api/src/feed_retire.rs`: `retire_state` reads the index with `blocking_unlocked` after `blocking`. `delete_draft` calls `feed_delete::start_delete_draft` under the lock.
- `crates/core/src/feed_retire.rs:130` to `:157`: `read_removal`, `IndexRemoval` and `retire_state` with `removal_done`.
- The API maps a `PipelineErrorKind` to a status and a code. Find the map (search `read_only` in `crates/api/src`).
- Tests: `crates/api/ui-tests/feeddelete.test.mjs` (the tests of `DELETE_CHOICES` and "Delete Anyway"), `crates/api/src/tests/feed_retire.rs` (each call of `delete/draft`, and `app_with` with `CountingIndex`).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Delete choices | extend | crates/api/ui/feeddelete.js | A pure function gives the rows of the sheet from the retire state. |
| Retire steps | extend | crates/api/ui/feeddelete.js | Step 3 has no button before `removal_done`. |
| Retire state route | reuse | crates/api/src/feed_retire.rs | The sheet reads the same route, and the delete route uses the same read pattern. |
| Index removal read | reuse | crates/core/src/feed_retire.rs | `read_removal` gives the answer for the guard. |
| Delete draft route | extend | crates/api/src/feed_retire.rs | The route reads Music Index and refuses a held feed. |
| Delete sheet | extend | crates/api/ui/editor.js | `FeedDelete` reads the state when the sheet opens. |

## Rules For The Delegate

- Read ADR 0200, ADR 0131 and ADR 0198 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not add a route. Do not change the schema of the collection.
- If a step needs an architecture decision that ADR 0200 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `crates/core/src/feed_retire.rs`: add `pub fn ensure_index_removed(removal: &IndexRemoval) -> Result<(), PipelineError>`. It gives `Ok` only for `IndexRemoval::Removed`. Each other answer gives an error with a kind that the API maps to `409` and the code `index_holds_feed`. The message starts with "ADR 0200 item 6". If the map needs a new `PipelineErrorKind`, add one variant.
2. `crates/api/src/feed_retire.rs` `delete_draft`: first, under the lock, get the GUID of the feed the same way as `retire_state` (`retire_facts(...).guid`) and the index. Then read `read_removal` with `blocking_unlocked`. Then call `ensure_index_removed`. Then start the draft under the lock, as now. Update the module comment: two routes send a request to the index (ADR 0200 item 7).
3. `crates/api/ui/feeddelete.js`: replace the use of `DELETE_CHOICES` in the sheet with `export function deleteChoices(state)`. With `state?.removal_done === true` it gives the three rows of today, with `recommended: true` on **Retire It**. Otherwise it gives the two rows **Move It to a Different File Host** and **Retire It**, with no `recommended`. Keep `DELETE_CHOICES` only if a different module still uses it; otherwise remove it.
4. `crates/api/ui/feeddelete.js` `retireSteps`: step 3 has `button: null` before `removal_done`. Remove "Delete Anyway" and its comment. The texts do not change.
5. `crates/api/ui/editor.js` `FeedDelete`: when `firstOpen` becomes true, read `POST /feeds/${feed.id}/retire/state` one time into a state for the sheet. Use the answer of the last request only, as `loadRetire` does. The sheet maps `deleteChoices(answer)`. A failed read keeps the two rows and shows no error in the sheet.
6. Tests that call `delete/draft` need an index that gives `404`. Change their setup, not the guard.

## Acceptance Criteria

Mechanical:

1. A Node test of `deleteChoices`: `removal_done: true` gives three rows, with the tag on **Retire It** and the last row destructive. `null`, `removal_done: false`, and each answer of ADR 0200 item 3 give two rows and no tag. Each failure message starts with "ADR 0200" and gives the fix.
2. A Node test of `retireSteps`: step 3 has no button for `removal_done: false`, also with `held_with_other_guid`. It has **Delete the File** for `removal_done: true`. Delete the old tests of "Delete Anyway".
3. A Node scan test of `editor.js`: the sheet maps `deleteChoices`, and the sheet reads `retire/state` when it opens.
4. An API test: `POST /feeds/{id}/delete/draft` gives `409` with `index_holds_feed` when the index gives a record (`200`) and when the read fails. It gives `200` when the index gives `404`. Prove that the first case fails on the code before the change.
5. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The sheet of a feed that Music Index holds, and of a feed that it does not hold, at 1280 px light and 390 px dark.
2. The Retire block of a held feed: step 3 has no button.

Human, by the operator: the gate of ADR 0200.

## Report

- The changed lines, with file and line.
- The test names, the proof of criterion 4, and the output of the four checks.
