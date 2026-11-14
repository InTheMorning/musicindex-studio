# Studio v1 Task 137: The Review Of A Delete Names The Delete, And A Deleted Feed Shows Only What It Can Do

Decision: [ADR 0198](../adr/0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md).

Status: Open.

## Goal

1. The review of a delete has a red **Delete the Feed File** or **Delete N Files** as its first button. **Cancel the Delete** has the usual style, at the right edge (ADR 0198 items 1 to 4).
2. Step 1 of the Retire block of a deleted feed says "Hide from apps is on in the last write." The core gives the state (item 5).
3. A deleted feed hides **Stop Retiring**, the select menu "Link to an artist or label page" and **Link to This Page** (items 6 to 8).

## Code Locations

These come from a search on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/editor.js:2197` and `:2199`: the review footer, with **Discard the Draft** (`button destructive`) and **Write to the File Host** (`button primary`). The labels `signProgressLabel(progress)` and `'Writing…'` show during the write.
- `crates/core/src/pipeline.rs:3726`: the operation that deletes the feed file has `kind: OperationKind::Delete` and `deletes_feed_file: true`. `crates/api/ui/feeddelete.js:77` `operationNote` reads `op.kind === 'delete' && op.deletes_feed_file`.
- `crates/core/src/feed_retire.rs:100` `retire_facts`: `written` is false after a delete, so `(false, Some(true))` gives `RetireAsk::DraftOnly`. `collection.feed_host_delete(id)?.is_some()` gives the done delete.
- `crates/api/ui/feeddelete.js:25` `retireSteps`: the texts of step 1. Do not change the texts.
- `crates/api/ui/editor.js:2419`: **Check Again** and **Stop Retiring** in `FeedDelete`.
- `crates/api/ui/editor.js:2456` `ArtistsAndLabels`, and about line 2510: the select menu `album-publisher` and **Link to This Page**.
- `crates/core/src/collection.rs:2193`: the delete sets `connector_id` and `path` to NULL, and records `feed_host_deletes`.
- Tests: `crates/core/tests/feed_delete.rs`, `crates/api/src/tests/feed_retire.rs:247`, `crates/api/ui-tests/feeddelete.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Review footer | extend | crates/api/ui/editor.js | The footer gets the labels and the order of a delete review. |
| Delete view data | extend | crates/api/ui/feeddelete.js | A pure function gives the label and the style of the write button from the diff. |
| Retire facts | extend | crates/core/src/feed_retire.rs | `retire_facts` gives `Written` for a done delete. |
| Feed row of the API | extend | crates/core/src/collection.rs | The feed row gets the done delete. Find the struct that the API sends as the feed row. |
| Linked pages section | extend | crates/api/ui/editor.js | `ArtistsAndLabels` hides the link controls of a deleted feed. |

## Rules For The Delegate

- Read ADR 0198, ADR 0131 and ADR 0193 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- If a step needs an architecture decision that ADR 0198 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `crates/api/ui/feeddelete.js`: add a pure function `deleteWrite(diff)`. For a diff with an operation with `deletes_feed_file`, it gives `{ label, destructive: true, cancel: 'Cancel the Delete' }`. The label is `Delete the Feed File` for one operation of kind `delete`, and `Delete ${n} Files` for more than one. For a different diff it gives `null`.
2. `crates/api/ui/editor.js`, the review footer: with a value from `deleteWrite`, the order is the write button (`button destructive`, with the label of change 1), then **Cancel the Delete** (`button`, the action of **Discard the Draft**), at the right edge. The progress labels stay. With `null`, the footer does not change.
3. `crates/core/src/feed_retire.rs` `retire_facts`: when `feed_host_delete` gives a record and the draft hides the feed, `ask` is `RetireAsk::Written`. Keep the other cases.
4. The feed row of the API gets `host_deleted: bool`, true when `feed_host_deletes` has a record for the feed. Use one query for the list of feeds, not one query for each feed.
5. `crates/api/ui/editor.js`: `FeedDelete` hides **Stop Retiring** when `retire.delete === 'done'`. `ArtistsAndLabels` hides the select menu and **Link to This Page** when `feed.host_deleted` is true. Do not show the hint "Studio has no other artist or label page to link to." for such a feed.

## Acceptance Criteria

Mechanical:

1. A Node test of `deleteWrite`: one delete operation with `deletes_feed_file` gives `Delete the Feed File`. Three delete operations give `Delete 3 Files`. A diff with no `deletes_feed_file` gives `null`. Each failure message starts with "ADR 0198" and gives the fix.
2. A Node scan test of `editor.js`: the footer puts the destructive write button before **Cancel the Delete**, and **Write to the File Host** stays in the `button primary` path.
3. A Rust test: `retire_facts` gives `Written` for a feed with a done delete and a hidden draft, and `DraftOnly` for a feed with no delete and a hidden draft that is not written. Prove that the first case fails on the code before the change.
4. An API test: the feed row of a deleted feed has `host_deleted: true`, and a different feed has `false`.
5. A Node scan test: `FeedDelete` hides **Stop Retiring** for `delete === 'done'`, and `ArtistsAndLabels` reads `host_deleted`.
6. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The review of a delete with the audio and image files, at 1280 px light and 390 px dark. **Delete N Files** is red and at the left edge. **Cancel the Delete** is at the right edge. Walk: `setup-131.py` with new steps `steps-137-{light,dark}.json`.
2. The page of the deleted feed (`setup-131-removed.py`). Step 1 says "in the last write". **Stop Retiring** and **Link to This Page** are not on the page.

Human, by the operator: the gate of ADR 0198.

## Report

- The changed lines, with file and line.
- The test names, the proof of criterion 3, and the output of the four checks.
