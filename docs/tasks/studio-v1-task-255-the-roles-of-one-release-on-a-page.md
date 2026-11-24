# Studio v1 Task 255: The Roles Of One Release On A Page

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0253](../adr/0253-the-roles-of-a-feed-and-of-each-link.md) item 21, [ADR 0251](../adr/0251-review-writes-the-two-sides-of-a-changed-link.md), ADR 0065.

Status: Done. Screenshots in Light, Dark and at 390 px passed.

Operator choice, 2026-11-24 (from three drawings): on a publisher page, each release row shows the roles of its link in one line. The row also has a **Change…** button. The button opens a sheet "Roles on {release}" with one check box for each role of the page, and **Cancel** and **Save**. The other two drawings were check boxes in each row, and an item only in the ••• menu of the Catalog.

## Goal

The user adds a role to the link of one release, or removes a role from it, on the page of the publisher feed.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link role route | reuse | crates/api/src/routes.rs | `set_link_roles`, `PUT /api/v1/feeds/{page}/links/{release}` (packet 250d). |
| Role words | reuse | crates/api/ui/shell.js | `relLabel` and `roleWord` give the line and the labels of the check boxes. |
| Releases section | extend | crates/api/ui/editor.js | The release row of the publisher page gets the line and the button. |
| Sheet | reuse | crates/api/ui/editor.js | `useSheet` and the `dialog class="sheet small"` markup of `ParentSheet` in crates/api/ui/app.js. |
| Change of two drafts | reuse | crates/api/ui/editor.js | `changeTwoDrafts` (the `twoDrafts` helper of the publisher page). |
| Role choice model | extend | crates/api/ui/shell.js | A new pure function gives the check boxes of the sheet. |

## Constraints

- The UI has no feed rule (ADR 0065). The core refuses a role that a different page has (`role_taken`). The sheet shows that error with `ErrorNotice`.
- Only a row with a release of the collection (`row.match`) has the button. A row of a feed outside the collection shows no button.
- A read-only page (no `connector_id`) shows the line and no button.
- The sheet has one check box for each role of the page (`feed.publisher_roles`), in the sequence of `ROLES`. A link must keep one role: **Save** is disabled when no box is checked.
- Labels use `roleWord`, so the role `hosting` shows "Publisher".
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- The agent does not commit and does not stage. The main session reads the diff, takes the screenshots, runs the gate and commits.

## Files To Read First

- `crates/api/ui/editor.js`: the publisher page near `const twoDrafts` (near line 4379), the `releases:` section, `changeTwoDrafts`, `useSheet`.
- `crates/api/ui/app.js`: `ParentSheet` (near line 1469).
- `crates/api/ui/shell.js`: `ROLES`, `roleWord`, `relLabel`.
- `crates/api/ui-tests/shell.test.mjs`.

## Steps

1. `shell.js`: `linkRoleChoices(pageRoles, rel)` gives `[{ role, label, checked }]` for each role of the page, in the sequence of `ROLES`. `checked` is true when the `rel` of the link has the role.
2. `editor.js`: in each release row with `row.match`, a line with `relLabel(row.entry.rel)` and a button **Change…** with `aria-label` "Change the roles on {title}".
3. A sheet `LinkRolesSheet` with the title "Roles on {title}", the check boxes and **Cancel** and **Save**. **Save** calls the route with the selected roles through `twoDrafts`, and closes the sheet when the call succeeds. An error shows in the sheet.
4. Node test in `shell.test.mjs` for `linkRoleChoices`: the sequence, the word "Publisher" for `hosting`, and a role of the `rel` that the page does not have.

## Acceptance Criteria

Mechanical:

- The test of step 4 passes.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `tools/gate.sh` pass.
- A `node --check` of a `.mjs` copy of `editor.js` and of `shell.js` passes.

Visual (the main session, with screenshots):

- The release row of a page with two roles shows the line and **Change…** at 1100 px and at 390 px, in Light and Dark.
- The sheet shows the check boxes, and **Save** changes the line of the row.
- `$edges()` and the fit check give no finding.

## Stop And Ask When

- The route refuses a change that the sheet allows, and the error has no message for the user.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-255-the-roles-of-one-release-on-a-page.md
- The files of "Files To Read First"

Goal:
- Steps 1 to 4 of the task file.

Constraints:
- The constraints of the task file.

Do not touch:
- crates/core/, crates/api/src/, docs/, AGENTS.md, crates/desktop/, crates/desktop-app/.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- tools/node --test crates/api/ui-tests/*.test.mjs
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
