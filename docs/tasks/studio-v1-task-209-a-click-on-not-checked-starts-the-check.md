# Studio v1 Task 209: A Click On "Not Checked" Starts The Check

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 17 to 19.

Status: Open. Start after task 208, because it changes `crates/api/ui/app.js`.

## Problem

Finding 5 of the [check after rc.29](../reviews/2026-11-20-rc29-human-check.md). A click on a row of the left panel with the mark "Not checked" opens the row and does not check its feeds.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| State marks | extend | crates/api/ui/outsidecheck.js | `STATE_MARKS` near line 250 and `stateMark` near line 266. Add a "Checking" state with a spinner. |
| Outside checks | extend | crates/api/ui/app.js | `useOutsideChecks` near line 4938 reads `/feeds/{id}/outside`. Give it a function that checks a list of feed IDs. |
| Left panel rows | extend | crates/api/ui/app.js | The panel rows near lines 3321 to 3340 and 3398. Find the click handler and report its line. |
| UI tests | extend | crates/api/ui-tests/outsidecheck.test.mjs | The tests of `stateMark`. |

## Changes

1. Item 17: a click on a row with the mark "Not checked" opens the row as before. It also checks each feed of the row that is not checked (`isUnchecked`).
2. Item 18: while the check runs, the mark is a spinner with the text "Checking". After the check, the row shows the mark of `stateMark`, or no mark.
3. Item 19: a row with a different mark, or no mark, starts no check on a click.

## Checks

Mechanical:

- Node test: `stateMark` gives the "Checking" mark while a feed of the set has a running check.
- Node test: a function that selects the feeds to check gives only the feeds that are not checked.
- `bash tools/gate.sh` passes.

Visual:

- drive-ui screenshots at 1280 px and 390 px: a row with "Not checked", the row during the check, and the row after the check. Run `$edges()` on each.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
