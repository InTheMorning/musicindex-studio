# Studio v1 Task 042b: The Listing Routes

Parent: [task 042](studio-v1-task-042-core-feed-rules.md). Decisions: [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) items 5, 7, 8 and 9, [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md) items 2 to 6, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the second packet of task 042. It depends on 042a.

Status: Complete - 2026-10-27.

## Goal

The switch "Hide from apps" and the check boxes of "Advanced" call core routes. The core applies the listing rule, and the draft response tells the UI whether the feed is hidden.

## Files

Inspect: `crates/core/src/feed.rs` (`Feed::hidden`, the fields `itunes_block` and `blocks`), `crates/core/src/feed/elements.rs` (`Block`), `crates/core/tests/edits.rs` (the block tests of task 040), `crates/api/src/lib.rs` (the routes near `/feeds/{id}/live-items/{guid}/status`), `crates/api/src/routes.rs` (`set_live_status`, `draft_json`), `crates/api/src/tests.rs` (the live status tests), `crates/api/ui/listing.js`, `crates/api/ui/draft.js` (`useDraft`, `show`, `saveOnce`), `crates/api/ui/editor.js` (`ListingField`, `ChannelFields`, the notice in the review sheet near `isHidden(draft.model)`, the live status call `changeStatus`), `crates/api/ui-tests/listing.test.mjs`.

Change: `crates/core/src/feed.rs`, `crates/core/tests/edits.rs`, `crates/api/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/tests.rs`, `crates/api/ui/listing.js`, `crates/api/ui/draft.js`, `crates/api/ui/editor.js`, `crates/api/ui-tests/listing.test.mjs`.

Do not change:

- `Feed::hidden`, and the feed list route with its `hidden`.
- `shell.js` and `app.js`.
- The cover, the alternative enclosures and the item block tags (ADR 0063 item 7).

## Architecture

These choices are fixed. Do not change them.

### The Rule In The Core

- Add `Feed::set_hidden(&mut self, hidden: bool)` in `feed.rs`, after `Feed::hidden`. It applies ADR 0063 items 3 and 4:
  - With `true`: `itunes_block` gets `Yes`. Each block with no `id` gets `yes`. When the feed has no block with no `id`, a new entry with `yes` and no `id` goes last in `blocks`. Then each block with an `id` keeps its position in the document (ADR 0012).
  - With `false`: an `itunes_block` that the feed has gets `No`. Each block with no `id` gets `no`. The function adds no tag and removes no tag.
  - A block with an `id` does not change.
- Add `Feed::set_platform_hidden(&mut self, platform: &str, hidden: bool) -> bool`. Each block with that `id` gets `yes` or `no`. It gives `false`, and changes nothing, when the feed has no block with that `id`.
- Give each function a doc comment that names its ADR items.

### The Routes

- `POST /feeds/{id}/listing` with the body `{"hidden": bool}`. The route calls `ensure_draft`, reads the draft with `pipeline::draft_model`, calls `set_hidden` and saves with `pipeline::replace_model`. It gives `draft_json`. It follows `set_live_status`.
- `POST /feeds/{id}/listing/{platform}` with the body `{"hidden": bool}`. It calls `set_platform_hidden`. When the function gives `false`, the route gives `404` with the code `not_found` and the message "the draft has no block for this platform".
- Both routes change the draft only. They never write to a host.
- `draft_json` gives `"hidden": model.hidden()`. Each route that gives `draft_json` then gives `hidden`.

### The UI

- `listing.js`: remove `isHidden`, `withHidden` and `withPlatformBlock`. Keep `platformBlocks`. Change the header comment: the core applies the rule, and the module gives the rows of "Advanced".
- `draft.js` `useDraft`: keep `hidden` as state. `show` and `saveOnce` set it from the response. The hook gives `hidden`.
- `editor.js` `ListingField`: the switch shows `draft.hidden`. A change of the switch calls `draft.flush()` first, then `POST /feeds/{id}/listing`, then `draft.show(response)`, as `changeStatus` does. A check box of "Advanced" does the same with the route for its platform and `row.id`.
- `editor.js`: give `ListingField` the feed ID and the draft. Each page that shows it passes them.
- `editor.js`: the notice of the review sheet uses `draft.hidden`.
- An error of a route shows as the other errors of `useAction` show.

## Steps

1. Add `set_hidden` and `set_platform_hidden` with their core tests.
2. Add the two routes and `hidden` in `draft_json`, with their API tests.
3. Change `listing.js` and its Node tests.
4. Change `draft.js` and `editor.js`.
5. Run the four checks.

## Constraints

- No feed rule in the UI (ADR 0065). The UI shows the value of the core and calls the core routes.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- The routes need the access token, as each `/api/` route does (ADR 0017, ADR 0055).

## Acceptance Criteria

Mechanical:

- A core test: `set_hidden(true)` on a feed with no block tag gives `itunes_block` `Yes` and one block `yes` with no `id`.
- A core test: `set_hidden(true)` on a feed with a block for one platform keeps the bytes of the document from the start to the end of that block.
- A core test: `set_hidden(false)` on a hidden feed with a block for one platform gives `No` and `no`. The block for the platform does not change. No tag is added.
- A core test: `set_hidden(false)` on a feed with no block tag adds no tag.
- A core test: `set_platform_hidden` changes only the blocks with that `id`, and gives `false` for an unknown `id`.
- An API test: `POST /feeds/{id}/listing` with `true` gives `"hidden": true`, and the draft XML has both tags with `Yes` and `yes`.
- An API test: `POST /feeds/{id}/listing` with `false` gives `"hidden": false`, and the draft XML has `No` and `no`.
- An API test: `POST /feeds/{id}/listing/{platform}` changes only the block of that platform. An unknown platform gives `404`.
- An API test: `GET /feeds/{id}/draft` of a hidden draft gives `"hidden": true`.
- A Node test: `platformBlocks` gives the rows as before.
- A Node test: `listing.js` does not export `isHidden`, `withHidden` or `withPlatformBlock`.
- The core block tests of task 040 pass.
- The four commit checks pass.

Visual: the switch of task 042, for the operator at the end of the task.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a block test of task 040 must change.
- Stop and report when a page that shows `ListingField` has no draft to pass.
- Stop and report when the server shell needs a change for the new routes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-042b-listing-routes.md
- docs/adr/0065-the-core-applies-each-feed-rule.md
- docs/adr/0063-one-switch-hides-a-feed-from-apps.md
- The files that the Files section names.

Goal:
- Add `Feed::set_hidden`, `Feed::set_platform_hidden` and the two listing routes.
- Give `hidden` in the draft response.
- Remove `isHidden`, `withHidden` and `withPlatformBlock` from the UI, and call the routes.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- The routes follow `set_live_status`. They change the draft only.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `Feed::hidden`, the feed list route, `shell.js`, `app.js`, the cover, the alternative enclosures and the item block tags.
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
