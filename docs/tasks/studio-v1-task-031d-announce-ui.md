# Studio v1 Task 031d: The Announcements In The UI

Parent: [task 031](studio-v1-task-031-announce.md). Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) items 5, 6, 11 and 20, [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0037](../adr/0037-button-roles.md) and [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: the fourth packet of task 031. It depends on 031c.

## Goal

The review sheet shows each planned announcement, or the reason for none. The feed page shows the last announcement. The Hosts page has the two settings.

## Files

Inspect: `crates/api/ui/editor.js` (`ReviewSheet`, the operations list near `FeedMoveRow`, `FeedFacts`), `crates/api/ui/app.js` (`Hosts`, `LibraryPanel`, `Field`, `useAction`), `crates/api/ui/signing.js` (the form of a model module), `crates/api/ui-tests/signing.test.mjs`, `crates/api/ui/styles.css`, `tools/drive-ui.mjs`.

Change: `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css` (only if a class is necessary), and the new files `crates/api/ui/announce.js` and `crates/api/ui-tests/announce.test.mjs`.

Do not change: the Rust crates, the other model modules, the shell layout.

## Architecture

These choices are fixed. Do not change them.

### The Model Module

`announce.js` has no Preact and no browser state ([ADR 0042](../adr/0042-ui-models-with-node-tests.md)). It exports:

- `announceLines(diff)` gives a list of text lines for the review sheet:
  - For each entry of `diff.announcements`: "Tell Podcast Index and send a podping: <url>" when `podping` is true, and "Tell Podcast Index: <url>" when it is false.
  - With `announce_skip` `"off"`: one line, "No announcement: the announcements are off in Hosts."
  - With `announce_skip` `"no_address"`: one line, "No announcement: this feed has no public address."
- `announceStatus(entries, timeText)` gives the text of the last announcement. `entries` is the list of `GET /feeds/{id}/announcements`, the newest first. `timeText` turns seconds after the Unix epoch into a local time text.
  - No entry: "No announcement yet."
  - `waiting` or `sending` with `attempts` 0: "Waiting. Studio sends it at <next_at>."
  - `waiting` with `attempts` 1 or more: "Waiting. Studio tries again at <next_at>. Last error: <last_error>"
  - `sent`: "Sent at <finished_at>."
  - `failed`: "Failed at <finished_at>: <last_error>"
  - With two addresses of the same write, the text gives each one, the newest first, each on its own line, with its address.
- `isWaiting(entries)` is true when the newest entry is `waiting` or `sending`.

### The Review Sheet

- Below the operations and the feed move, the sheet shows an "Announcements" heading with the lines of `announceLines(diff)`, in a list. It has no button.
- The lines show each time that the diff has content, also when the feed text did not change.
- After a write, the sheet loads the announcements of the feed again for the feed page.

### The Feed Page

- `FeedFacts` gets a fact "Announcement", with the text of `announceStatus`. The summary line of the disclosure becomes "GUID, file, version and announcement".
- The feed page loads `GET /feeds/{id}/announcements` when it opens and after each write. While `isWaiting` is true, it loads the list again each 5 seconds.

### The Hosts Page

- An "Announcements" panel follows the Library panel, in the same form.
- It has two check boxes: "Tell Podcast Index after each write" and "Send a podping with it". The second one is disabled while the first one is off.
- A "Save" button with the primary role saves the two values with `PUT /settings/announcements` ([ADR 0037](../adr/0037-button-roles.md)). It is disabled while the values are the saved values.
- A hint below the check boxes: "Studio sends the announcement 30 seconds after a write."

## Steps

1. Add `announce.js` and its Node tests.
2. Add the lines to the review sheet.
3. Add the fact and the loads to the feed page.
4. Add the Hosts panel.
5. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the comments and the texts in the files are.
- The UI loads nothing from other servers, and it has no build step ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- Each control has the target size of its profile. Each function works from the keyboard.
- An error uses the `--error` token ([ADR 0018](../adr/0018-passphrase-strength-and-error-color.md)).

## Acceptance Criteria

Mechanical:

- The Node tests give the text of each case of `announceLines` and of `announceStatus` above, and `isWaiting` for each state.
- The four commit checks pass.

Visual:

- The planning model serves Studio against a local `TestHub`, with a temporary ignored test in `crates/api`, and runs `tools/drive-ui.mjs`. It removes the temporary test before the commit. The screenshots show these items in the dark theme and in the light theme:
  - The review sheet with the line "Tell Podcast Index and send a podping".
  - The review sheet with the line of the setting off.
  - The feed facts with "Waiting", then "Sent", and a failure with `TestHub::fail_next`.
  - The Hosts panel, with the podping check box disabled while the first one is off.
  - At 390 px, the Hosts panel and the review sheet with no small control.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the feed page cannot load the announcements without a change to the shell layout.
- Stop and report when a text in the Architecture section does not fit the summary line rules of ADR 0028.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-031d-announce-ui.md
- docs/adr/0052-announcements-through-the-podcast-index-hub.md
- The files that the Files section names.

Goal:
- The review sheet shows the planned announcements or the reason for none. The feed facts show the last announcement. The Hosts page has the two announcement settings. The text logic is in `crates/api/ui/announce.js` with Node tests.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.
- The UI loads nothing from other servers, and it has no build step.

Do not touch:
- The Rust crates, the other model modules, the shell layout.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

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
