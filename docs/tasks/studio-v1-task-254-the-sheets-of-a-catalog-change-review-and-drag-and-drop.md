# Studio v1 Task 254: The Sheets Of A Catalog Change, Review, And Drag And Drop

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 18 to 28 and 34 to 39. [ADR 0257](../adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 14 and 15.
Prototype: `.studio-dev/design-brief/prototype/structure.html`. Git ignores it.

Status: In progress. Packets 254a, 254b and 254c are done.

Operator choices (the main session, in the approved ADR 0254):

- Packet 254a: the place text of the core uses the roles of each link on the shared releases (ADR 0253). A kept link has no shared release, so the roles of the feed give it.
- Packet 254a: a page action keeps the link of the release to the page with its first level role. The other removed roles of that link go (ADR 0254 item 26). Thus a label page that is also a publisher can change its publisher, and "Leave all" keeps its role `label`.
- Packet 254a: the message of `role_taken` is the step of the error notice, as for `bad_address`.
- Packet 254b: the Add sheet shows no row for a page below the sheet page, also when a different page is between the two pages (ADR 0254 item 19 gives only a page directly below). The core refuses each of these pages, and the place of such a page does not change.
- Packet 254c: the API gives "Now" and "After" with the core function `catalog_place`. The main session made that function public and extended it for a release and for a publisher page. A release gets its place from the roles of its own links. A publisher page gets "Published by" or "Not published", as a label page does.
- Packet 254c: the title of Review for the change set of the Catalog is "Changes in the Catalog".
- Packet 254c: in Review, a page that is only a publisher shows its name with no "Now" and no "After". The core function `has_catalog_place` gives this rule (ADR 0254, operator choice of 2026-11-25).

## Goal

Each change of a page asks one question in music words. The Add sheet signs or publishes pages. After a change, a notice gives **Undo** and **Review**. The top of the panel counts the changes. Review shows "What changes". On the desktop, a tree row of an artist or a label moves with drag and drop.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page move sheet | extend | crates/api/ui/editor.js | `MoveSheet` and `GatherPagesSheet` become the sheets of ADR 0254 items 18 and 24. |
| Page move models | extend | crates/api/ui/movepages.js | `pageMoveMenus`, `pageMoveLines` and `gatherPagesModel`. |
| Refusal of an open change set | reuse | crates/api/ui/editor.js | `MoveRefusal` gives the text of ADR 0243 item 5. |
| Music Index search | reuse | crates/api/ui/app.js | `AddFindSheet` gives the search of ADR 0233. |
| Publisher page form | reuse | crates/api/ui/app.js | `MakeFeedForm`. |
| Undo notice | reuse | crates/api/ui/undonotice.js | The notice with **Undo** and **Review**. |
| Undo history | reuse | crates/api/ui/undohistory.js | `recordOperation` with an `undo` that calls the route of packet 251c. |
| Change set page | extend | crates/api/ui/app.js | `ChangeSetPage` gets the group "What changes". |
| Tree rows | extend | crates/api/ui/app.js | The rows of task 253 get the menu **⋯** and the drag. |
| Node tests | extend | crates/api/ui-tests/movepages.test.mjs | The sheet models. |
| Sheet sizes | extend | tools/sheets.json | The sizes of the new sheets. |

## Constraints

- Each rule is in the core (ADR 0065). The start selection, the result "After:" and "Changes {N} releases." come from the preview route of packet 251c.
- The sheet "Also on another label…" does not exist (ADR 0257 item 14).
- No change writes a file. Review does not open by itself (ADR 0254 items 34 and 35).
- The phone has no drag and drop. The menu **⋯** gives each change (ADR 0254 items 10 and 39).
- Each button has an accessible name. A sheet traps the focus and closes with Escape, as the other sheets do.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A UI module passes `node --check` on a `.mjs` copy.
- The agent does not commit and does not stage. The main session reads the diff, takes the screenshots, runs the gate and commits.

## Files To Read First

- ADR 0254 items 18 to 28 and 34 to 39, ADR 0257 items 14 and 15, ADR 0243 items 2, 5 and 6, ADR 0227.
- The prototype `.studio-dev/design-brief/prototype/structure.html`.
- The routes of packet 251c in `crates/api/src/routes.rs`.
- `crates/api/ui/editor.js`: `MoveSheet`, `GatherPagesSheet`, `MoveRefusal`.
- `crates/api/ui/movepages.js`, `crates/api/ui/undonotice.js`, `crates/api/ui/undohistory.js`.
- `crates/api/ui/app.js`: `ChangeSetPage`, `AddFindSheet`, `MakeFeedForm`, the tree rows of task 253.

## Packets

### 254a: The Sheets Of A Page Action (UI)

1. The menu **⋯** of a tree row and of a page head gives the actions of ADR 0254 item 23 for its rank, with no "Also on another label…", then "Open".
2. Each action opens a sheet with its one question and its button (item 24). The sheet shows "Now", the search field and the list of pages with "now", the releases with check boxes, the result and the hints (item 25). The start selection comes from the preview route (item 28).
3. **Change Label**, **Go Independent** and **Change Publisher** call the route of packet 251c. A refusal for a different open change set shows `MoveRefusal`.
4. Node tests in `movepages.test.mjs`: the actions of each rank, the question and the button of each sheet, the list titles, the hints, and no "Also on another label…". The tests of the old page move sheet go.

### 254b: The Add Sheet (UI)

1. The add rows **＋ Sign an artist**, **＋ Publish an artist** and **＋ Publish a label** open the Add sheet with its title and the field "Search" (item 18).
2. The three groups of item 19: "In your collection" with check boxes, "On Music Index" (reuse the search of `AddFindSheet`), and "New" with **＋ Make a new {artist or label} page** (reuse `MakeFeedForm`).
3. The texts of item 20, with "Change label…" or "Change publisher…" as the action of the hint (ADR 0257 item 14).
4. The button **Sign** or **Publish** calls `gather_pages` (packet 251c). With no selected row, the button is not available (item 21).
5. Node tests: the rows of the first group, the start with no selection, the texts of item 20 and the button.

### 254c: The Notice, The Count, Review And Undo (UI)

1. After each change, the sheet closes and the notice "{name} is now {place}. Not written yet." shows with **Undo**, **Review** and a close button (item 35). Reuse `undonotice.js`.
2. While the change set of the Catalog is open, the top of the panel says "{N} changes not written" with **Review** (item 36).
3. Review of the change set of the Catalog has the group "What changes" first, with "Now: {place}" and "After: {place}" for each changed page and release. Then "Feeds that Studio writes", **Discard** and **Write**. When the change set does not give "Now" and "After", add them in the API with a test in `crates/api/src/tests.rs`. Do not add the rule in the UI.
4. **Undo** calls the Undo route of packet 251c. The page records it with `recordOperation`, so Ctrl+Z also gives it. The Catalog has no Redo. `undohistory.js` gets an operation with no `redo`: its `undo` does not put it on the Redo stack, and `canRedo` does not change. Extend `makeHistory`. Do not copy it.
5. Node tests: the notice text, the count text, and the group "What changes". In `undohistory.test.mjs`: an operation with no `redo` gives no Redo, and an edit before it keeps its Redo rules.

### 254d: Drag And Drop On The Desktop (UI)

1. Wider than 760 px, a tree row of an artist or a label can move. A row of a higher rank takes the drop and shows the drop color. The top of the tree shows "Drop here: no label and no publisher" during a drag (item 38).
2. A drop does "Change label…" or "Change publisher…" with the dragged page and the row of the drop (item 39). When the start selection holds each release of the page, the change occurs at once with the notice. When it does not, the sheet of packet 254a opens with the dropped row selected.
3. A drop on the top area calls the action `LeaveAll` of packet 251a: "Go independent…" and "Not published" as one change.
4. Node tests: the model of a drop gives the change at once or the sheet. The rows that take the drop follow the rank.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes. `node --test crates/api/ui-tests/` and `tools/gate.sh` pass.
- `rg 'Also on another label' crates/api/ui` gives no line.

Visual (the main session with screenshots and `$edges()`, then the operator):

- The sheet "Change label…" at 1280 px and at 390 px, in Light and Dark.
- The Add sheet with the three groups.
- Two changes, the count at the top of the panel, Review with "What changes", then Undo.
- A drag of an artist onto a label, with the drop color and the notice.

## Stop And Ask When

- A sheet needs a value that the preview route does not give.
- The drag needs a change in `crates/desktop-app/`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-254-the-sheets-of-a-catalog-change-review-and-drag-and-drop.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (254a, 254b, 254c or 254d). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- crates/core/, docs/, AGENTS.md, crates/desktop/, crates/desktop-app/. Change crates/api/src/ only in packet 254c, item 3.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- node --test crates/api/ui-tests/
- cargo test -p studio-api (packet 254c only)
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
