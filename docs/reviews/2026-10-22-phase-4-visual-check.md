# Phase 4 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 008](../tasks/studio-v1-task-008-ui-shell-and-wizard.md), on the local desktop shell
- Checker: operator, in Firefox on Linux, with KeePassXC
- Result: open. Sections 4 to 6 are not done.

## Results

| Check | Result | Note |
| --- | --- | --- |
| 2.1 to 2.10, wizard | Pass | After the corrections below |
| 3.1 Font | Fail, corrected | The brand was the word "Studio" in Figtree. musicindex.org shows its lockup. Studio now shows the same lockup with a "Studio" label. |
| 3.2 Logo | Fail, corrected | Same cause as 3.1 |
| 3.3 to 3.5 | Pass | |
| 3.6 Themes | Fail, corrected | The theme toggle used text characters. It now uses the SVG icons of musicindex.org. |
| 3.7 to 3.11 | Pass | |
| 4.1 to 6.1 | Not done | |

The corrected checks 3.1, 3.2 and 3.6 need a second check by the operator.

## Defects Found During The Check

These defects were corrected before the check went on:

- The wizard had no way back to a previous step. Commit `01bd820`.
- The advice text on the passphrase step, and errors in boxes that looked like fields. [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md), commit `2cfcbdf`.
- The feed step asked for a typed path. It now lists the feed files of the host. [ADR 0019](../adr/0019-select-a-feed-from-the-host.md), commit `378bc8b`.
- The wizard does not know the situation of the user. The plan records this as an open design item.

## Missing Tests

- The lockup image was referred to but not served, and no test found it. A test now makes sure that each `/ui/` address in the UI files is served.
