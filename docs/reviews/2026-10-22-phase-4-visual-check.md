# Phase 4 Visual Check

- Date: 2026-10-22
- Reviewed: the visual criteria of [task 008](../tasks/studio-v1-task-008-ui-shell-and-wizard.md), on the local desktop shell
- Checker: operator, in Firefox on Linux, with KeePassXC
- Result: pass. Each visual criterion of task 008 passes after the corrections below.

## Results

| Check | Result | Note |
| --- | --- | --- |
| 2.1 to 2.10, wizard | Pass | After the corrections below |
| 3.1 Font | Pass after correction | The brand was the word "Studio" in Figtree. musicindex.org shows its lockup. Studio now shows the same lockup with a "Studio" label. |
| 3.2 Logo | Pass after correction | Same cause as 3.1 |
| 3.3 to 3.5 | Pass | |
| 3.6 Themes | Pass after correction | The theme toggle used text characters. It now uses the SVG icons of musicindex.org. |
| 3.7 to 3.11 | Pass | |
| 4.1 Control size | Pass | The agent measured each screen in headless Chromium. Only checkboxes and radio buttons are below 44 px, in label rows of 44 px. |
| 4.2 Console | Pass | Three console lines came from browser extensions, not from Studio: a `file:///` error from KeePassXC-Browser, a blocked inline script from `contentScriptOnStart.bundle.js`, and a layout warning from `index.js`. The Studio policy blocked the inline script, as it must. |
| 5.1 Wrong passphrase | Pass | |
| 5.2 No token | Pass | |
| 5.3 Studio stopped | Pass | |
| 6.1 Narrow window | Pass | At 320 px. The empty-state text said "the list at the side", which is not true on a narrow screen. The text is now "Select one in the Feeds list." |

## Defects Found During The Check

These defects were corrected before the check went on:

- The wizard had no way back to a previous step. Commit `01bd820`.
- The advice text on the passphrase step, and errors in boxes that looked like fields. [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md), commit `2cfcbdf`.
- The feed step asked for a typed path. It now lists the feed files of the host. [ADR 0019](../adr/0019-select-a-feed-from-the-host.md), commit `378bc8b`.
- The wizard does not know the situation of the user. The plan records this as an open design item.

## Missing Tests

- The lockup image was referred to but not served, and no test found it. A test now makes sure that each `/ui/` address in the UI files is served.
