# Studio v1 Task 023f: The Folder Scan UI

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 13 to 15, [ADR 0037](../adr/0037-button-roles.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: after [task 023d](studio-v1-task-023d-download-ui.md) and [task 023e](studio-v1-task-023e-folder-scan.md).

## Goal

The user sees each change in the library folder, at the start and on request, and selects an action for each one.

## Files

Inspect: `crates/api/ui/app.js` (`Shell`, `SidebarContent`, `LibraryPanel`, the feed tree), `crates/api/ui/editor.js` (`FilesPage`), `crates/api/ui/styles.css`, the routes of task 023e.

Change: `app.js`, `editor.js`, `styles.css`.

Do not change: `crates/core`, `crates/api/src`, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### At The Start

- After the shell loads the feeds, and when a library is set, it calls `POST /library/scan` one time. When the scan gives differences, a notice above the main pane says "Studio found 3 changes in the library folder." with "Show" (normal). "Show" opens the Files page of the first feed with a difference. The notice has a close button (icon button).
- The feed tree shows a small mark adjacent to each feed with a difference, with the label "Changes in the library folder" for a screen reader.

### The Files Page

- A section "Changes in the library folder" is at the top of the Files page when the feed has differences. Each difference is one row:
  - New: "New file: `bonus.mp3` (4.1 MB)", with "Add to the library" (action).
  - Missing: "Missing: `01 Song.mp3`", with "Download again" (action) when the file has a copy or an address, and "Forget the file" (destructive).
  - Changed: "Changed: `01 Song.mp3` (4.1 MB, was 4.0 MB)", with "Use the new file" (action) and "Restore" (normal). "Restore" is disabled with the text "No copy on a host" when the file has no copy.
- "Scan the library folder" (action) is adjacent to "Scan the hosts". It runs the scan again.
- After each action, the page runs the scan again and loads the files route again. After "Use the new file", the shell loads the draft again, as after a host operation. The toolbar then shows the change.
- An error such as `no_change` or `no_copy` shows its message in the row.

## Steps

1. Add the scan at the start, the notice and the mark in the tree.
2. Add the section of the Files page, and its actions.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- Each button has the role of ADR 0037.
- The styles agree with `each_background_uses_a_token`.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Run `drive-ui.mjs` with `bun` when `node` is not on the PATH. Examine each screenshot in the dark and the light theme.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` with a release folder that has one new, one missing and one changed file. The notice and the tree mark show at the start. The agent selects "Add to the library", "Forget the file" and "Use the new file". After each one, the row goes away. After "Use the new file", the toolbar shows "Saved, not written", and the review sheet has an upload and a deletion. `$small()` gives an empty list, and the console shows no error.

Visual:

- The visual check of task 023 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the UI needs a field that the routes of task 023e do not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023f-folder-scan-ui.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The user sees each change in the library folder, at the start and on request, and selects an action for each one.

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
