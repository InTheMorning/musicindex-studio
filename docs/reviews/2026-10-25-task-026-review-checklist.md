# Task 026 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 026](../tasks/studio-v1-task-026-shell.md)
- Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The three commit checks and the Node tests pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Invariants

- A model module imports no Preact and reads no data from the browser (ADR 0042).
- No page is a placeholder (ADR 0039 item 4).
- The left panel is one panel at any time. Hosts and Lock Studio show at its bottom (ADR 0039 items 1 and 5).
- The draft status, "Versions" and "Review" stay in the editor bar ([ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 1 to 4).
- Each labeled button has its role ([ADR 0037](../adr/0037-button-roles.md)). All functions work from the keyboard.
- The UI loads nothing from other servers, and it has no build step ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [026a](../tasks/studio-v1-task-026a-feed-list-facts.md) | Pass | The planning model accepted one choice of the coding agent: one API test examines each draft state in sequence. A written feed with no draft gives `none`, and the Releases page shows "No changes" for it, as for `unchanged`. Open item: the feed list now reads the draft or the hosted feed of each feed. The time for a large collection is not measured. |
| [026b](../tasks/studio-v1-task-026b-shell-models.md) | Pass | The planning model accepted these choices of the coding agent. The CI step names the test files with `*.test.mjs`, and the documents now give the same command. "A to Z" puts the group "#" after Z before it compares the titles. A host that the connector list does not have shows "Unknown host". The sort compares with the locale `en`. The first CI run must confirm the Node.js step, because this computer has no Node.js. |
| [026c](../tasks/studio-v1-task-026c-shell-ui.md) | Pass after two corrections | The planning model examined the screenshots of Releases in rows, A to Z and tiles, and of the outline. Other screenshots show Hosts at two heights, 390 px with touch, and the light theme. (1) After Back, the saved scroll position did not show the marked row of a feed at the end of the list. The row now moves into view by the smallest scroll. (2) At 390 px, the drawer covered the bar. It now starts below the bar. The planning model ran the two cases again with `tools/node`. The planning model accepted three choices of the coding agent. The rows use `releases-*` class names. The letter bar stays below the bar. Back at index 0 on a feed puts Releases before it. Open items: no screenshot shows the track rows of the outline. A change of the pointer type after the load does not change the bar. Bun was not on the computer during the check, so the agent ran the model tests with Deno. |
| [026d](../tasks/studio-v1-task-026d-value-routes.md) | Pass | The planning model examined the Value routes page in the dark theme and in the light theme. It has the groups "Recipients" and "Fees", and the tooltip with the raw share on keyboard focus. The planning model accepted two choices of the coding agent. The share bar in the "Album details" row gets no focus, because it is in a button. The tooltip keeps the percentage format of the app, "47.5 %". Open items: no screenshot shows the section on the track page. No screenshot shows a tooltip at the top edge of a row. |
| [026e](../tasks/studio-v1-task-026e-narrow-stack.md) | Pass after two corrections | The planning model examined the root page, the outline page and the album page at 375 px with touch and with a mouse. It examined the dark theme and the light theme. (1) The back control replaced the history entry and did not go back. The browser Back button then did not agree with the stack (musicindex ADR 0030, Consequences). A new start now makes one history entry for each page of the stack, and the back control goes back in the history. Each history entry records if its earlier entries agree with the stack. After a change of width, the back control moves the stack in place until the next load. The planning model ran the sequence with the two Back controls, Forward and a reload. (2) At 375 px, the notice of the album page went to the right edge. The column of the head took the width of the long address. The column now stays in the page. The planning model accepted three choices of the coding agent. The root page has the brand and the theme button. The root page has no Forward control. A feed from the Hosts page opens its page, not its outline page. |
| Operator check of 026e in Firefox | Pass after three changes | The operator examined the narrow stack in Firefox at 375 px with touch simulation. Steps 1 and 2 pass. Back on Releases opens the root page. The outline page, the album page and the two Back selections give the stack of ADR 0039 item 11a. The operator asked for three changes. (1) The Touch back control is a circular icon button with no word (musicindex ADR 0031). (2) The chevron was a text character and too small. Each back and forward control now has a chevron icon with the line height of its text. (3) The root page and the outline page have the color of the left panel. |
