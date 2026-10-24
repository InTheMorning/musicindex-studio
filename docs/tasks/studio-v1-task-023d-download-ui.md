# Studio v1 Task 023d: The Download UI

Parent: [task 023](studio-v1-task-023-release-library-7b.md). Decisions: [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 6, 7, 11 and 12, [ADR 0037](../adr/0037-button-roles.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: after [task 023c](studio-v1-task-023c-download-queue.md).

## Goal

The Files page offers the downloads, shows their progress, and asks before a copy or a move of a remote file. The review sheet marks each operation that waits for a download.

## Files

Inspect: `crates/api/ui/editor.js` (`FilesPage`, `FileEntry`, `CopyRow`, `ReviewSheet`, the operation rows), `crates/api/ui/app.js`, `crates/api/ui/styles.css`, the routes of task 023c, `crates/api/src/tests.rs` (the UI tests).

Change: `editor.js`, `app.js`, `styles.css`.

Do not change: `crates/core`, `crates/api/src`, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### The Files Page

- When `remote_count` is more than 0, the page shows a notice above "Library files": "12 files are not in the library (340 MB)." with "Download all" (action). While the downloads run, the notice shows "Downloading 3 of 12…" and "Stop all" (normal).
- The summary line of a remote file shows "Not in the library", "Queued", "Downloading 45%" or "Download failed". The open entry has "Download" (action), or the progress and "Stop" (normal), or the error message and "Try again" (action).
- After a download, the entry shows the copies as for a local file. When `length_in_feed` is not the size of the file, the entry shows "The feed gives 48.1 MB. The file is 47.9 MB." The page does not change the feed for it.
- A remote file gets the "Copy to" and "Move to" buttons of a local file. Before it sends the operation, the page asks with `confirm`: "Studio downloads this file first (48 MB). The copy goes into the draft now, and the write waits for the download." The size comes from the row.
- While a download of the feed is queued or running, the page reads `GET /feeds/{id}/downloads` each second. It stops when no download of the feed is queued or running. When a download completes, the page loads the files route again. The open entry stays open.

### The Review Sheet

- An operation with `waiting` true shows the second line "Waiting for the download. Studio writes it after the download." While an operation waits, "Write to the host" is disabled, with the hint "Wait for each download, then write."
- A write error `waiting_for_download` shows its message.

### Sizes

- A size uses the format of the Files page: `48.1 MB`, from the existing function.

## Steps

1. Add the notice and the download states of the Files page.
2. Add the confirmation of a copy or a move of a remote file.
3. Add the waiting operations of the review sheet.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- Each button has the role of ADR 0037.
- The styles agree with `each_background_uses_a_token`.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Run `drive-ui.mjs` with `bun` when `node` is not on the PATH. Examine each screenshot in the dark and the light theme.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `drive-ui.mjs` with a published feed with two remote files on a test HTTP server, and a second local host. The agent downloads one file with "Download", sees the progress, then copies the second file with its confirmation, and opens the review sheet. The page shows each mark of the Architecture section, and "Write to the host" is disabled until the download completes. `$small()` gives an empty list, and the console shows no error.

Visual:

- The visual check of task 023 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the UI needs a field that the routes of task 023c do not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-023d-download-ui.md
- docs/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The Files page offers the downloads, shows their progress, and asks before a copy or a move of a remote file. The review sheet marks each operation that waits for a download.

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
