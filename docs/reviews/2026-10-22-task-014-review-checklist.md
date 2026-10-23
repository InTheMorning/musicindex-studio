# Task 014 Review Checklist

Reviewed work: the diff of each packet of [task 014](../tasks/studio-v1-task-014-editor-layout.md), against [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md).
The planning model uses this list after each packet, before the commit of that packet.

## Each Packet

- The diff changes only the files that the packet names.
- The diff makes no change to `crates/core` and no change to the API routes.
- The three commit checks pass.
- The `drive-ui.mjs` check of the packet passes, and the planning model examines its screenshots.
- `$small()` gives an empty list. The console shows no error.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.

## Invariants

- No write occurs without a current diff that the user saw ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- The write button does not show while an address is not correct ([ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md)).
- Each `details` element has a `name` or the class `disclosure` ([ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md)).
- The UI loads nothing from other servers ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- The editor uses three background levels, and no border identifies a level (ADR 0029).
- The theme toggle has no change.

## Packet Results

| Packet | Result | Required corrections |
| --- | --- | --- |
| [014a](../tasks/studio-v1-task-014a-background-levels.md) | Pass after one correction | The fields had no `border` rule, so the browser showed its default inset border. The planning model added `border: 0`. An incorrect field keeps its red border. |
| [014b](../tasks/studio-v1-task-014b-toolbar-and-sheets.md) | Pass after two corrections | A load or a save for a feed could end after the user selected a different feed, and show its result under that feed. The planning model added a guard for the current feed to `useDraft`. `$small()` of `tools/drive-ui.mjs` counted the controls in a closed `details` element. The planning model changed it to use `checkVisibility()`. |
| [014c](../tasks/studio-v1-task-014c-tree-and-pages.md) | Pass after one return | The first version had five layout defects: the header was not in the page column, the sections had no gap, the recipients and entries were cards in a group, the add buttons touched the group edge, and the pages had no name. The coding agent corrected them. The toolbar now shows a breadcrumb. |
| [014d](../tasks/studio-v1-task-014d-phone-width.md) | Pass after one correction | The packet gave a bar only to the feed, Hosts and Add feeds screens. At a narrow width, the start screen, the load screen and the error screen then had no menu button, and the user could not open the drawer. The coding agent reported it. The planning model gave these screens a bar. |
