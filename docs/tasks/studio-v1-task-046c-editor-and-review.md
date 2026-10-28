# Studio v1 Task 046c: The Editor And Review For A Read-Only Feed

Parent: [task 046](studio-v1-task-046-read-only-editor.md). Decisions: [ADR 0067](../adr/0067-a-read-only-feed-opens-in-the-editor.md) items 1, 2, 5, 6, 7 and 9, [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).
Sequence: the third packet of task 046. It depends on 046b.

Status: Ready.

## Goal

A read-only album, publisher feed or playlist opens in the editor, with "read-only" in the eyebrow. Review shows the diff and the two host choices. After a host choice, Review shows the warning of ADR 0067 item 9 and the write button.

## Files

Inspect: `crates/api/ui/shell.js` (`isEditable`, `outlineItems`), `crates/api/ui/app.js` (`FeedDetail`, `ReadOnlyFeed`, `EditableFeed`, the uses of `isEditable`, the route to Hosts), `crates/api/ui/editor.js` (`ReviewSheet`, `DraftToolbar`, `moveNote`, `FeedMoveRow`), `crates/api/ui/draft.js` (the fields that the draft response gives), `crates/api/ui-tests/shell.test.mjs`.

Change: `crates/api/ui/shell.js`, `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/draft.js` (for `has_host` only), `crates/api/ui/styles.css` (only when a new class needs a style), `crates/api/ui-tests/shell.test.mjs`, a new or a changed Node test for the Review model.

Do not change:

- Each Rust file.
- `ReadOnlyFeed` for a feed type that the editor does not open. It stays as now.
- The link controls of a publisher feed. They refuse a read-only release, as now.

## Architecture

These choices are fixed. Do not change them.

### The Editor

- `isEditable(feed)` gives true for an album, a publisher feed and a playlist, also with no connector. Add `isReadOnly(feed)`: true when the feed has no `connector_id`.
- The eyebrow of the compact header of `EditableFeed` shows ` · read-only` after the type label when `isReadOnly(feed)`.
- `outlineItems` gives the same items for a read-only feed as for the same feed type with a connector.

### Review

- Add `export function reviewHost({ hasHost, isReadOnly, feedMove })` to the model module of the Review sheet. When that sheet has no model module, add it to `shell.js`. It gives one of these:
  - `{ kind: 'host' }`: the feed has a connector. Review is as now.
  - `{ kind: 'choose' }`: the feed is read-only and has no host choice.
  - `{ kind: 'move', connectorId }`: the feed is read-only, with a host choice.
- With `choose`, Review shows the diff, then these controls, and no button "Write to the host":
  - The button "Connect the Host of This File", with the role action. It opens Hosts.
  - The field "Move to a host": a select menu of the connectors that have a public address, then the button "Move to This Host" with the role action. The button sends `PUT /feeds/{id}/feed-host` with the selected connector.
- With `move`, Review shows the feed move row, the button "Write to the host" as now, and this warning in a `notice`: "The file at the address before the move stays and does not change. Apps that read that address continue to get the version from before the move."
- The import route refuses a read-only feed with 409 `read_only` and "choose a host for this feed first, in Review" (packet 046b). The import sheet shows that error text, as it shows each other error.
- The snapshot routes refuse a read-only feed with 409 `read_only`. The Versions sheet shows the error text, as it shows each other error.
- "Undo" of the move row of a read-only feed sends `PUT /feeds/{id}/feed-host` with `{ "connector_id": null }`.
- `moveNote` gives a text for the method `none`: "Studio writes nothing at the address before the move."

## Steps

1. Change `isEditable`, add `isReadOnly`, and change the Node tests.
2. Add `reviewHost` and its Node tests.
3. Change `EditableFeed`, `ReviewSheet`, `FeedMoveRow` and `moveNote`.
4. Run the four checks.
5. Make the screenshots of the visual check with `tools/drive-ui.mjs`, and run `$edges()`.

## Constraints

- No feed rule in the UI (ADR 0065). The core decides the path, the self link and the method of the move. The UI shows the values of the diff.
- One setting with a few values is a select menu, not a radio group.
- A labeled button has a role (ADR 0037). Use title case for the button labels, as the other buttons do.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- All functions work from the keyboard.

## Acceptance Criteria

Mechanical:

- A Node test: `isEditable` is true for a read-only album, a read-only publisher feed and a read-only playlist, and false for a read-only podcast feed.
- A Node test: `isReadOnly` is true only for a feed with no `connector_id`.
- A Node test: the outline of a read-only album gives the same item IDs as the outline of the same album with a connector.
- A Node test: `reviewHost` gives `host`, `choose` and `move` for the three cases of this packet.
- A Node test: `moveNote('none')` gives the text of this packet.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`, in the dark theme, the light theme and at 390 px:

- A read-only album opens in the editor with its cover, its tracks and "read-only" in the eyebrow.
- Review of a read-only draft with an edit shows the diff, "Connect the Host of This File", the select menu and "Move to This Host", and no button "Write to the host".
- After "Move to This Host", Review shows the move row, the warning and "Write to the host". "Undo" gives the two choices again.
- `$edges()` finds no content at the edge of a group.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a different use of `isEditable` means "Studio can write this feed".
- Stop and report when the draft response does not give `has_host`.
- Stop and report when Review has no place for the two choices at 390 px.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-046c-editor-and-review.md
- docs/adr/0067-a-read-only-feed-opens-in-the-editor.md
- The files that the Files section names.

Goal:
- A read-only album, publisher feed or playlist opens in the editor, with "read-only" in the eyebrow.
- Review gives the two host choices, then the warning and the write button after a choice.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- Each Rust file, `ReadOnlyFeed` for the other feed types, and the link controls of a publisher feed.
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
