# Studio v1 Task 203: Send The Podping Again Only When It Helps

Decision: [ADR 0229](../adr/0229-the-media-host-after-a-follow-a-podping-again-and-empty-folders.md) items 4 to 7.

Status: Open.

## Problem

Finding 2 of the [rc.29 human check](../reviews/2026-11-20-rc29-human-check.md). The waiting notice after a write shows **Send the Podping Again** on a sent line. The button has no use before Music Index answers, and it adds a row, so the notice changes its height.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Podping lines | extend | crates/api/ui/outsidecheck.js | `podpingLines` near line 101 gives `action` to each sent line. `moveNotice` near line 124 and `writeNotice` near line 151 call it. `WRITE_NOTICE_AGE` near line 137. |
| Line render | extend | crates/api/ui/app.js | `PodpingLines` near line 5081. The notice of a write near line 5068 and of a move near line 5064. |
| Notice style | extend | crates/api/ui/styles.css | The class `podping-line`. |
| UI tests | extend | crates/api/ui-tests | The tests of `writeNotice` and `moveNotice`. Search for `Send the Podping Again`. |

## Changes

1. `podpingLines` takes an option, for example `{ again }`. A sent line gets `action: 'Send the Podping Again'` only when `again` is true (item 4). A failed line always gets it. A not sent line keeps **Send the Podping**.
2. `writeNotice` passes `again: true` only in the state `no_reply` (item 5).
3. `moveNotice` takes `nowSeconds`. It passes `again: true` for a sent line when `nowSeconds - finished_at > WRITE_NOTICE_AGE` (item 6). Pass `nowSeconds` from the caller in app.js. Make sure that the notice renders again after the wait. Use the timer of `nextNoticeLoad` or a new timeout, and give the reason in the Result section.
4. The button is in the podping line, at its right, on the same row. The address shrinks with an ellipsis to give the button its space. The line does not wrap to a second row at 390 px or at 1280 px (item 7).

## Checks

Mechanical:

- UI test: the waiting notice has no line action for a sent line. The no reply notice has **Send the Podping Again**. A failed line has it in each state.
- UI test: a sent line of the move notice has no action before the wait and has the action after it.
- `bash tools/gate.sh` passes.

Visual (the main session):

- Walk screenshots of the waiting and the no reply notice at 1280 px and 390 px. The notice keeps one height, and the line has one row.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result
