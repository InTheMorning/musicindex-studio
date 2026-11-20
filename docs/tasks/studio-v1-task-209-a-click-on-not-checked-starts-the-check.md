# Studio v1 Task 209: A Click On "Not Checked" Starts The Check

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 17 to 19.

Status: Done. One point for the narrow layout is open. Refer to the Result.

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

## Result

Changed files:

- `crates/api/ui/outsidecheck.js`: `STATE_MARKS` has a "Checking" mark. `stateMark` takes a third argument, a Set of running feed IDs. `feedsToCheck` gives the feeds that are not checked.
- `crates/api/ui/app.js`: `useOutsideChecks` gives `{ results, running, check }`. `OutsideContext` carries `running` and `check`. `StateMark` shows the existing `player-wait` spinner for the mark "Checking". No style change.
- `crates/api/ui-tests/outsidecheck.test.mjs`: two new tests.

Click handler: `useOpenAndCheck` in `crates/api/ui/app.js` near line 2005. It opens the row. It starts `check` only when the mark kind is `unchecked`. `DestinationRow` (line 3325), `LevelRow` (line 3362) and the head of a level (line 3385) use it. The outline row of an open feed does not use it, because a test pins its source.

Rules of the mark: the order is outside, pending, checking, unchecked. A feed that a check reads now is not read again. The check at load does not show "Checking".

Mechanical checks, run by the delegate:

- `tools/node --test crates/api/ui-tests/*.test.mjs`: 922 pass, 0 fail (920 before).
- Syntax check of `app.js` and `outsidecheck.js` as `.mjs` copies: passed.
- `bash tools/gate.sh`: "gate: each check passed".

Visual check (2026-11-20), drive-ui on test data with unreadable folder hosts. The page delays each check request by 4 seconds, so that the screenshot shows the spinner:

- At 1280 px, the row "Not linked" has the mark "Not checked". After the click, the mark is the spinner with the name "Checking". After the check, the mark is "Not checked" again, because the host stays unreadable.
- `$edges()` gives no finding. The spinner fits in the place of the mark.

Open: at 390 px, a tap on a row opens the page of the row, and the panel list goes off the screen. Thus the spinner does not show there. The check runs, and the row shows its new mark when the user goes back.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
