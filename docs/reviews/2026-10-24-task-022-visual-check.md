# Task 022 Visual Check

- Date: 2026-10-24
- Reviewed: the visual criterion of [task 022](../tasks/studio-v1-task-022-ui-pass-after-7a.md). The operator examines the Files page, the review sheet, the Hosts screen and one sheet of the editor. Each screen is in the dark theme and in the light theme.
- Decision: [ADR 0037](../adr/0037-button-roles.md)
- Data: the data of the [milestone 7a visual check](2026-10-23-phase-7a-visual-check.md) in `.studio-dev/phase-7a/`
- Checker: operator
- Result: pass after three corrections

## Results

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | Dark theme. Open the Files page and the entry of a library file. | Pass |
| 2 | Click "Copy to folder". | Pass |
| 3 | Open the review sheet. | Pass |
| 4 | Click "Undo" in the review sheet, and close it. | Pass after one correction |
| 5 | Delete a file that no feed uses, then click "Undo". | Pass |
| 6 | Open the Hosts screen, and edit `hostB`. | Pass |
| 7 | Open the import sheet with no file. | Pass |
| 8 | Light theme. Open the Files page, and delete a file that no feed uses. | Pass after one correction |
| 9 | Examine the pending row in the light theme. | Pass |
| 10 | Open the review sheet in the light theme. | Pass |
| 11 | Open the Hosts screen in the light theme, and edit `hostB`. | Pass after one correction |
| 12 | Undo the deletion, and examine the host cards. | Pass |

## Defects Found During The Check

- Step 4: after "Undo" in the review sheet, the Files page continued to show "Copy to folder pending". The review sheet did not tell the Files page that an operation changed. The toolbar also kept a sign of a pending operation in the browser only, and "Undo" did not remove it. The toolbar and "Discard the draft" then showed a change that the draft did not have.

  The correction: the shell loads the draft again after each change of the host operations, because `draft.changed` from the server counts each operation. An "Undo" in the review sheet also loads the Files page again. A load of the same feed keeps the open entry open.
- Step 8: in the light theme, a neutral button had a white fill with a border, and a group and the toolbar also have a white fill. This did not obey ADR 0037 item 4. A neutral button now has the `--fill` token in the two themes, with no border.
- Step 11: the feed tree and "Hosts" were in one scroll area, so a long tree moved "Hosts" out of view. Now only the feed tree and "Add a feed" scroll. The brand and the footer of the sidebar stay in view. "＋ Add a host" had the normal role, but it opens a form to add something, as "Import files" does. It now has the action role.

## Open Items

- The entry of a library file shows "Copy to" below a divider, so it looks like a copy row of its own.
- On a row of "Other files on the hosts", the note "Remove it from the feed first" shortens the file name.
- With no file selected, the add button of the import sheet says "Add 0 tracks".
- The Hosts screen lists for each host only the feeds whose XML file is on that host. A host that holds only an audio file of a feed shows "None yet".
- A neutral button uses a translucent fill, so it has a different grey on the page and on a group. The operator asked about it. The planning model kept it, because a translucent fill is different from each surface.
