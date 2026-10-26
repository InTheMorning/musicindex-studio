# Studio v1 Task 033f: The Grants In The UI

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 11, 14 and 17, [ADR 0037](../adr/0037-button-roles.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the sixth packet of task 033. It depends on 033b and 033e.

## Goal

The review sheet has the grant check box for a pending show. The live item page shows the grant and each granted write. The user can remove a grant.

## Files

Inspect: `crates/api/ui/editor.js` (`ReviewSheet`, `AnnounceLines`, the live item page), `crates/api/ui/live.js`, `crates/api/ui-tests/live.test.mjs`, `crates/api/src/routes.rs` (the grant routes of 033e), `tools/drive-ui.mjs`.

Change: `crates/api/ui/editor.js`, `crates/api/ui/live.js`, `crates/api/ui-tests/live.test.mjs`, `crates/api/ui/styles.css` (only when a class is necessary). `crates/api/src/routes.rs` and `crates/api/src/tests.rs` only for `DELETE /api/v1/feeds/{id}/grants/{grant_id}`, which sets `ended_at`.

Do not change: the core, the app routes.

## Architecture

These choices are fixed. Do not change them.

- The review sheet shows, for each `pending` live item of the draft, a check box "Let connected apps start and end this show, and set its live value". It calls `PUT` or `DELETE /feeds/{id}/grants/{guid}` and examines the diff again. The `grants` of the diff give its state.
- When the collection has no connected app, the check box has the hint "No app is connected. Connect an app in Hosts."
- The live item page shows the grant state with `grantSummary` and lists each granted write with `grantWriteText`. "Stop the grant" with the destructive role ends an active grant.
- `live.js` gets:
  - `grantSummary(grant, timeText)`: "Connected apps can start and end this show until <time>.", "The grant ended at <time>." or "No grant."
  - `grantWriteText(write, timeText)`: "<app name> · <change> · <time>", with the change texts "Went live", "Ended the show", "Set the live value" and "Removed the live value".

## Steps

1. Add the `live.js` functions and their Node tests.
2. Add the check box to the review sheet.
3. Add the grant state and the list to the live item page, and "Stop the grant".
4. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- The UI loads nothing from other servers, and it has no build step.
- Each control has the target size of its profile. Each function works from the keyboard.

## Acceptance Criteria

Mechanical:

- The Node tests examine `grantSummary` and `grantWriteText` for each case.
- An API test of `DELETE /feeds/{id}/grants/{grant_id}`: after it, the app route gives 409 `grant_ended`.
- The four commit checks pass.

Visual:

- The planning model serves Studio with a temporary ignored test that adds a connected app and uses its token as the other app. The screenshots show the check box in the review sheet and the live item page with an active grant and two granted writes. They also show the page after "Stop the grant". Each one is in the dark theme, in the light theme and at 390 px.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the review sheet needs a second diff request after a change of a grant.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033f-grants-ui.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- The review sheet has the grant check box for each pending show. The live item page shows the grant, each granted write and "Stop the grant". The text logic is in `live.js` with Node tests.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The core, the app routes.
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
