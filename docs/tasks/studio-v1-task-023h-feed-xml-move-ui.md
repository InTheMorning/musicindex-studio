# Studio v1 Task 023h: The Feed XML Move UI

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 16 to 21, [ADR 0037](../adr/0037-button-roles.md).
Sequence: after [task 023f](studio-v1-task-023f-folder-scan-ui.md) and [task 023g](studio-v1-task-023g-feed-xml-move.md). It is the last packet of task 023.

## Goal

The user selects the host of the feed XML on the Files page. The review sheet shows the move, the method at the address from before, and the announcement that Studio does not do yet.

## Files

Inspect: `crates/api/ui/editor.js` (`FilesPage`, `ReviewSheet`, `OtherFileRow`, the blocked deletions), `crates/api/ui/app.js`, `crates/api/ui/styles.css`, the routes of task 023g.

Change: `editor.js`, `app.js`, `styles.css`.

Do not change: `crates/core`, `crates/api/src`, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### The Files Page

- The "Hosts" section gets a second field below "Default host of the release files": "Host of the feed XML", a select of each host. Its value is the host of the pending move, or the host of the feed. A change calls `PUT /feeds/{id}/feed-host`, then loads the draft again, as after a host operation.
- While a move is pending, the field has the hint "The feed moves to hostB when you write. The address from before gets a pointer file." With `redirect`, the hint says "…gets a redirect." instead.
- An other file with `pointer` true shows "Pointer to the new address", and no "Delete" button.

### The Review Sheet

- A feed move shows as the first row of the operations: "Move the feed" with the new address, and the second line "The address from before gets a pointer file with the new address." or "…gets a redirect to the new address."
- Below it, a hint: "Studio does not tell the musicindex API and Podcast Index the new address yet."
- The row has "Undo" (normal). It calls `PUT /feeds/{id}/feed-host` with the host of the feed.
- A blocked deletion with `pointer` true shows "Studio keeps this file. It tells the apps the new address of the feed."

## Steps

1. Add the field and the hints of the Files page.
2. Add the rows of the review sheet.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- Each button has the role of ADR 0037.
- The styles agree with `each_background_uses_a_token`.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Run `drive-ui.mjs` with `bun` when `node` is not on the PATH. Examine each screenshot in the dark and the light theme.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` with a release on host A and a second local host B. It selects host B as the host of the feed XML, opens the review sheet, writes, and opens the Files page again. The page shows each mark of the Architecture section. After the write, the feed address is on host B, and the pointer file on host A has no "Delete" button. `$small()` gives an empty list, and the console shows no error.

Visual:

- The visual check of task 023 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the UI needs a field that the routes of task 023g do not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023h-feed-xml-move-ui.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The user selects the host of the feed XML on the Files page. The review sheet shows the move, the method at the address from before, and the announcement that Studio does not do yet.

Constraints:
- Follow the Architecture section of the task file fully.
- Each control is 44 px high or more. Each button has the role of ADR 0037.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, crates/api/src, draft.js.
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
