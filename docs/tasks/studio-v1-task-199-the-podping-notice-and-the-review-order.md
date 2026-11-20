# Studio v1 Task 199: The Podping Notice And The Review Order

Decision: [ADR 0228](../adr/0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 8 to 10 and 14 to 16.

Status: Done.

## Problem

Findings 4 and 11 of the [rc.28 human check](../reviews/2026-11-20-rc28-human-check.md). The confirmation of a write shows **Send the Podping Again**, and the notice becomes smaller. In a long Review sheet, the XML changes are below the file operations, and a write error shows at the top.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Write notice model | extend | crates/api/ui/outsidecheck.js | `writeNotice` near line 149 and `podpingLines` near line 101. |
| Notice render | extend | crates/api/ui/app.js | `OutsideNotices` near line 4952. The notice markup is near line 5052. |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` near line 1967. The write error is near line 2209, `HostOperations` near line 2229, `DiffView` near line 2257 and the footer near line 2273. |
| Fold | reuse | crates/api/ui | Use the fold element that the sheet uses for "Details" now. Search for `<details` in editor.js. |

## Changes

1. Each state of the write notice has **Dismiss**. The notice stays until **Dismiss** or until the user leaves the page (item 8).
2. The confirmed state has no podping line and no **Send the Podping Again** (item 9). Find why the operator saw that button after the confirmation, and report the cause.
3. The confirmed notice keeps the height of the notice before it (item 10). Use a minimum height from the measured height, not a fixed number of pixels.
4. In `ReviewSheet`, `DiffView` comes before `HostOperations` (item 14).
5. `HostOperations` is in a closed fold with the head "File operations ({n})" (item 15).
6. The write error shows in the footer, directly above `AnnounceFooterChoice` (item 16). The other notices keep their positions.

## Checks

Mechanical:

- UI test: `writeNotice` gives a dismiss action in each state, and the confirmed state has no lines.
- UI test: the sheet source has `DiffView` before the fold of `HostOperations`, and the write error in the footer.
- `bash tools/gate.sh` passes.

Visual (the main session):

- Walk screenshots of the Review sheet at 1280 px and 390 px with file operations and an error (`.studio-dev/walk/mine-183-1280.json` and `mine-183-390.json` give an error).

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

- `writeNotice` gives `dismiss` in the waiting, no reply and confirmed states. **Dismiss** keeps the notice hidden for that write. A new write or a different feed shows it again.
- The confirmed state has no podping line. The code gave no line for it before this task too (`writeNotice` in `outsidecheck.js`, since ed220d8), and a test now asserts it.
- Cause of the button after the confirmation: not found in the code. Only `podpingLines` gives **Send the Podping Again**, and the confirmed state passes no lines. Inference: the operator saw the button on the waiting notice, and the smaller row was the confirmation that replaced it.
- The confirmed notice gets a minimum height from the measured height of the notice before it.
- `ReviewSheet` shows `DiffView` before `HostOperations`. `HostOperations` is in the closed fold "File operations ({n})". The write error is in `.footer-error`, directly above `AnnounceFooterChoice`.
- Walks with the main session:
  - `t199-1280.json` and `t199-390.json` (setup `setup-155b.py`): the write error is in the footer at both widths. With **Details** open at 390 px, the footer starts at 378 px, and the body keeps about 240 px.
  - `t199b-1280.json` and `t199b-390.json` (setup `setup-142-move.py`): the XML changes come before the closed fold "File operations (1)". The open fold shows the move row.
  - `t199c-1280.json` (setup `setup-155b.py`): the waiting notice and the confirmation both have a height of 105 px. **Dismiss** removes the notice.
  - `$edges()`, `$overflow()` and the fit checks gave no finding.
- Open: a long error text in the footer leaves less space for the body on a phone. No walk showed a body with no space.
