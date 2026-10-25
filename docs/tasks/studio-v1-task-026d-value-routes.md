# Studio v1 Task 026d: The Value Routes Page

Parent: [task 026](studio-v1-task-026-shell.md). Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 6 and 12, [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) item 9.
Shared rules: musicindex ADR 0023 R-WORD-2, ADR 0020 R-VALUE-1.
Sequence: the last packet of task 026. It depends on 026c.

## Goal

The payment page and each payment section have the name "Value routes", with the groups "Recipients" and "Fees". The page is at most 560 px wide, and a share shows its tooltip at once.

## Files

Inspect: `crates/api/ui/values.js` (`AlbumPayments`, `TrackPayments`, `ShareStack`, `shareParts`), `crates/api/ui/editor.js` (`AlbumPage`, `RecipientsPage`), `crates/api/ui/app.js` (`PAGE_NAMES`, `Crumb`), `crates/api/ui/styles.css` (`.share-stack`, `[data-tip]`), `crates/api/src/tests.rs` (the UI tests).

Change: `values.js`, `editor.js`, `app.js`, `styles.css`.

Do not change: `shell.js`, other than the label of the page when a test needs it. Do not change `crates/core/`, the routes and `draft.js`.

## Architecture

These choices are fixed. Do not change them.

- The page `recipients` keeps its view name. Its title in the bar, its row in "Album details" and its section on the track page say "Value routes" (R-WORD-2 items 1 and 3).
- The page and the section of a block show two groups, "Recipients" and "Fees". A recipient with `fee` true is in "Fees". The other recipients are in "Recipients". A group with no entry shows "No fees." or "No recipients." (R-WORD-2 item 2).
- The add of a recipient stays in the group "Recipients". A fee is still a field of each recipient, so a change of `fee` moves the entry to the other group at the next render. The model keeps the sequence of the recipients.
- The page content is at most 560 px wide (ADR 0039 item 12).
- Each segment of the share bar has a tooltip with the name, the percentage and the raw share, for example "Alice 50.0% (share 50)". The tooltip shows at once on hover and on focus, with no delay. A segment can get the focus from the keyboard (R-VALUE-1 item 3).

## Steps

1. Change the names.
2. Divide each block into "Recipients" and "Fees".
3. Limit the width, and change the tooltip.
4. Run the `drive-ui.mjs` check below, and give the screenshots to the planning model.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step and no new UI file ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- All functions work from the keyboard.

## Acceptance Criteria

Mechanical:

- No UI file has the text "Recipients" as the title of a page or of a section, other than the group name.
- The Node tests pass with no change.
- The three commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`:

- The outline item, the bar title and the "Album details" row say "Value routes".
- A block with two recipients and one fee shows the two groups. The page is at most 560 px wide at a window of 1100 px.
- The tooltip of a segment shows the name, the percentage and the raw share at once, with the mouse and with the keyboard.
- The page in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

When Node.js is not on the PATH, run `tools/node` in place of `node`.

## Escalation

Stop and report when the division into groups needs a change of the draft model.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-026d-value-routes.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- The files that the Files section names.

Goal:
- The payment page and each payment section have the name "Value routes", with the groups "Recipients" and "Fees". The page is at most 560 px wide, and a share shows its tooltip at once.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step and no new UI file. All functions work from the keyboard.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- shell.js (only the page label when a test needs it), crates/core/, the routes, draft.js.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. Give the screenshots of the visual criteria to the planning model.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs (or tools/node in place of node when Node.js is not on the PATH)

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
