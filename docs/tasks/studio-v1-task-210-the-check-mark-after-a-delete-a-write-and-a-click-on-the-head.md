# Studio v1 Task 210: The Check Mark After A Delete, A Write And A Click On The Head

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 17 and 18. This task corrects defects of task 209. It makes no new decision.

Status: Done. The human check is open.

## Problem

Findings 1, 5 and 6 of the [desktop check of tasks 205 to 209](../reviews/2026-11-20-rc29-human-check.md#desktop-check-of-tasks-205-to-209).

- Finding 1: a feed with a done delete always has the mark "Not checked". `isUnchecked` in `crates/api/ui/outsidecheck.js` does not read `host_deleted`. The feed has no file, so a check cannot read it.
- Finding 5: on a release page, the head box of the left panel (`Outline` in `crates/api/ui/app.js` near line 3455) starts no check on a click.
- Finding 6: a write does not start a new check of the written feed. `useOutsideChecks` in `crates/api/ui/app.js` runs again only when the list of feed IDs changes. Thus the result from before the write stays.

Operator choice (2026-11-20): these are defect corrections of task 209. No ADR change.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| State marks | extend | crates/api/ui/outsidecheck.js | `isUnchecked` near line 258, `feedsToCheck` and `stateMark` after it. |
| Click handler | reuse | crates/api/ui/app.js | `useOpenAndCheck` near line 2005. |
| Outline head | extend | crates/api/ui/app.js | `Outline` near line 3430. The click near line 3457. |
| Outside checks | extend | crates/api/ui/app.js | `useOutsideChecks` near line 5010. It gives `check(list)`. |
| Write of a draft | extend | crates/api/ui/editor.js | `ReviewSheet` near line 2054. After `onWritten(feed)` near line 2082 it sends `ANNOUNCE_RELOAD`. |
| UI tests | extend | crates/api/ui-tests/outsidecheck.test.mjs | The tests of `stateMark` and `feedsToCheck`. The source test of `Outline` near line 279 stays. |

## Changes

1. Finding 1: a feed with `host_deleted` true is not "not checked". `stateMark` gives no mark "Not checked" for it, and `feedsToCheck` does not give it. The check at load does not read it.
2. Finding 5: the head box of `Outline` uses `useOpenAndCheck` with the set of the open feed and `feedMark`. The click keeps its current open rule (`if (!feed.first)`). The check starts also when the page does not change. Keep the two lines that the source test near line 279 pins.
3. Finding 6: after a write, `ReviewSheet` sends a window event with the ID of the written feed. `useOutsideChecks` listens for it and calls `check` with that feed. Thus the row shows "Checking", then the new result.

## Checks

Mechanical:

- Node test: `stateMark` gives null for a set that has only a feed with `host_deleted` true and no result.
- Node test: `feedsToCheck` leaves out a feed with `host_deleted` true.
- Node test (source): `Outline` uses `useOpenAndCheck`.
- Node test (source): `ReviewSheet` sends the write event, and `useOutsideChecks` listens for it.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual (main session):

- drive-ui at 1280 px on a release page with the mark "Not checked": a click on the head box shows "Checking", then the new mark.

Human:

- Step 6 of the desktop check again: a click on the head box of a release with the dotted circle.

## Result

Changes:

- `crates/api/ui/outsidecheck.js` line 261: `isUnchecked` returns false for a feed with `host_deleted` true. `stateMark` and `feedsToCheck` use it.
- `crates/api/ui/app.js` line 3437: `Outline` calls `useOpenAndCheck` for the open feed. The head box uses it. The open rule `if (!feed.first)` stays. The two lines that the source test pins stay.
- `crates/api/ui/app.js` near line 5022: the check at load leaves out a feed with `host_deleted` true. The main session added this line.
- `crates/api/ui/editor.js` line 1934: the exported constant `OUTSIDE_RECHECK`. Line 2088: `ReviewSheet` sends it with the ID of the written feed after a write.
- `crates/api/ui/app.js` lines 5049 to 5053: `useOutsideChecks` listens for `OUTSIDE_RECHECK`, calls `check` and removes the listener in the effect cleanup.
- `crates/api/ui-tests/outsidecheck.test.mjs`: two new tests at the end of the file. One covers `host_deleted`. One checks the source for the head box, the send and the listener.

Checks:

- `tools/node --test crates/api/ui-tests/*.test.mjs`: 924 pass, 0 fail (922 before).
- `tools/node --check` on a copy of each changed ui file: passes.
- `bash tools/gate.sh`: "gate: each check passed".
- Visual (2026-11-20, main session): drive-ui at 1280 px on test data with unreadable folder hosts and a 4-second delay of each check request. A click on the head box of "Test Album" shows the spinner "Checking", then "Not checked" again. `$edges()` gives no finding.
- Human: step 6 of the desktop check again is open.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit and do not stage. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
