# Studio v1 Task 022b: The Button Roles And The Files Page

Parent: [task 022](studio-v1-task-022-ui-pass-after-7a.md). Decisions: [ADR 0037](../adr/0037-button-roles.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Sequence: after [task 022a](studio-v1-task-022a-api-marks-and-host-address.md). It is the last packet of task 022.

## Goal

Each button shows its role. The Files page and the review sheet show what each file and operation is, and give feedback for each operation.

## Files

Inspect: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/values.js`, `crates/api/ui/styles.css`, `crates/api/src/tests.rs` (`each_background_uses_a_token`, `each_details_element_is_a_named_entry_or_a_section`), the routes of task 022a.

Change: `app.js`, `editor.js`, `values.js`, `styles.css`, `tests.rs` (one new test).

Do not change: `crates/core`, `crates/api/src` except `tests.rs`, `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### Button Roles (ADR 0037)

- `styles.css` gets the tokens `--accent-tint` and `--error-tint`, each with a dark and a light value, and the classes `.button.action` (accent label on `--accent-tint`) and `.button.destructive` (`--error` label on `--error-tint`). Each has a hover and a pressed state.
- Each button with a text label in `app.js`, `editor.js` and `values.js` gets the role of the table of ADR 0037 item 1. No button with a text label keeps `button quiet`.
- On each background level, the neutral fill of `.button` is different from the level. Use a fill token, not a level token.
- A new test in `tests.rs`, `each_quiet_button_is_an_icon_button`: in each UI script, each `class="button quiet` also has `icon-button`. Its failure message names ADR 0037 item 5 and the fix.

### The Files Page

- The entry of a library file shows one button for each other host: "Copy to hostB" (action). After a copy, the button is "Move to hostB" on the row of the copy that the feed uses. There is no list with a second button.
- The copy that the feed uses shows "Used by the feed". A copy or an other file with `pending` shows "Copy pending" or "Deletion pending", and its button becomes "Undo" (normal), which removes the operation. A `pending_copies` entry shows as a row "Copy to hostB pending" with "Undo".
- An other file with `in_use` shows "Used by the feed", and its "Delete" is disabled with the text "Remove it from the feed first".
- "Host of this file" shows "Use the default host of the release" when the file has no host of its own.
- The entry has no empty space between its rows.
- After each operation, the page loads the files route again. After a write and after a discard of the draft, the page loads the files route again.

### The Review Sheet

- Each operation row has a second line:
  - Upload: "The feed uses this file."
  - Copy: "A spare copy. The feed does not use it."
  - Delete: "Studio deletes it after the feed is written."

### The Hosts Screen

- Each local folder host has "Edit" (normal). It opens fields for the name and the public address, with "Save" (primary) and "Cancel" (normal). "Save" calls `PUT /connectors/{id}`.

## Steps

1. Add the tokens, the classes and the test, and give each button its role.
2. Change the Files page and the review sheet.
3. Add the host edit.

## Constraints

- Each control is 44 px high or more, and each control works from the keyboard.
- The styles agree with `each_background_uses_a_token`.
- The binary contains the UI files. After each UI change, stop your own Studio, build it again and start it again before a `drive-ui.mjs` check. Examine each screenshot in the dark and the light theme.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- The three commit checks pass, with the new test.
- The agent runs `drive-ui.mjs` with a release on host A, a library file, and host B. It copies the file to host B with one click, writes, moves it with one click, deletes an other file and undoes the deletion, and edits the address of host B. The page shows each mark of the Architecture section. `$small()` gives an empty list, and the console shows no error.

Visual:

- The visual check of task 022 covers this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when the UI needs a route that task 022a does not give.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-022b-button-roles-and-files-page.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- Each button shows its role. The Files page and the review sheet show what each file and operation is, and give feedback for each operation.

Constraints:
- Follow the Architecture section of the task file fully.
- Each control is 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, crates/api/src except tests.rs, draft.js.
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
