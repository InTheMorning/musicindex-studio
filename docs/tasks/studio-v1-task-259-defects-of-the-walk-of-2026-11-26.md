# Studio v1 Task 259: Defects Of The Walk Of 2026-11-26

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md).
Findings: [walk of 2026-11-26](../reviews/2026-11-26-human-check-walk.md), findings 2, 4, 5, 6, 8 and 9.
Decisions: [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 38 and 39, ADR 0131 items 1 to 10, [ADR 0234](../adr/0234-a-release-move-is-a-guided-flow.md) item 7.

Status: not started.

Operator choices (the main session, defect corrections with no new decision):

- Finding 2: the top area "Drop here: no label and no publisher" shows only when a drop there changes the place of the dragged page. A drop there does "Go independent…" and then "Not published" (ADR 0254 item 39). Thus the area shows only when the dragged page has a label page directly above it, or a publisher page above it.
- Finding 4: the drag handle of a Catalog row goes after the toggle column and directly before the art. The indent moves the toggle, the handle and the row together.
- Finding 5: a click or a tap outside an open menu of `MenuControl` closes the menu. The click does not move the focus to the menu button.
- Finding 6: each row "Find on Music Index" shows the icon `search` before its text, as the row "My music is online already" of the welcome page.
- Finding 8: the retire state reads the written copy of ADR 0234 item 7 (`pipeline::written_copy`), not only the copy in the release folder. A feed made in Studio has no release folder.
- Finding 9: after **Check Again**, the Retire block shows "Checked at {local time}". The UI records the time when the answer arrives. The API does not change for this finding.

## Goal

Correct six defects that the operator found in the human check walk of 2026-11-26.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Retire facts | extend | crates/core/src/feed_retire.rs | `retire_facts` uses `pipeline::written_copy` for the written hidden state (finding 8). |
| Written copy | reuse | crates/core/src/pipeline.rs | `written_copy` already falls back to `collection.written_xml`. |
| Retire API tests | extend | crates/api/src/tests/feed_retire.rs | A new test next to the test near line 291 that checks `draft_only` and `written`. |
| Retire steps model | extend | crates/api/ui/feeddelete.js | `retireSteps` or a new small function gives the text "Checked at {time}" from a time that the caller gives (finding 9). |
| Retire block | extend | crates/api/ui/editor.js | `loadRetire` (near line 2797) keeps the time of the last answer of **Check Again**. The block shows the line. |
| Retire Node tests | extend | crates/api/ui-tests/feeddelete.test.mjs | The text of the check time. |
| Top drop area | extend | crates/api/ui/app.js | `CatalogDropTop` (near line 3695) shows only when the model says that a drop changes the page (finding 2). |
| Page actions model | extend | crates/api/ui/movepages.js | A new exported function, for example `topDropChanges(page, feeds)`, next to `pageActions`. It reads `parent_ids` and `publisher_roles` as `pageActions` does. Do not copy `ancestors`. |
| Page actions Node tests | extend | crates/api/ui-tests/movepages.test.mjs | The new function, near line 255. |
| Catalog row | extend | crates/api/ui/app.js | `DestinationRow` (near line 3742): the handle goes after the toggle (finding 4). |
| Catalog row styles | extend | crates/api/ui/styles.css | `.tree-row` (line 757) and `.drag-handle` (line 764), only when the new sequence needs it. |
| Menu | extend | crates/api/ui/menu.js | `MenuControl` adds a document `pointerdown` listener while the menu is open, and removes it on close (finding 5). |
| Find row of a picker | extend | crates/api/ui/pickers.js | `PickerLayout` (line 224) shows `<${Icon} name="search" />` in the row of `musicindex` (finding 6). `pickers.js` already imports `Icon`. The sheets of `values.js`, `editor.js` (lines 226, 255, 292), `peoplepages.js` (lines 433, 593) and `app.js` (line 1752) use this row. |
| Inline Find row of the Add sheet | extend | crates/api/ui/app.js | The row near line 1744 does not use `PickerLayout`. It gets the same icon. |
| Inline Find row of the editor | extend | crates/api/ui/editor.js | The row near line 816 does not use `PickerLayout`. It gets the same icon. |

Searched for an existing outside-click helper: `crates/api/ui` and `~/build/musicindex-lib` have none (`useFieldList` in `pickers.js` uses blur, not a document listener). The menu change adds the listener in `MenuControl` only.

## Constraints

- No feed rule in the UI (ADR 0065). The model of finding 2 reads only the roles and the parents that the feed list gives, as `pageActions` does.
- The narrow layout (760 px or less) has no drag and no handle. Do not change it.
- The handle keeps `-webkit-user-drag: element` and the `setTimeout` start of the drag (ADR 0254 item 38, WebKitGTK).
- The outside click must not close a menu when the click is on the menu or on its button. Escape keeps its behavior.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A test message names the owner and the fix, for example: `Walk 2026-11-26 finding 8: a written hidden feed with no release folder must give ask "written". Read pipeline::written_copy in retire_facts.`
- Do not commit. Leave the change in the working tree and report it.

## Packets

### 259a: The Retire State Of A Feed Made In Studio (core and API)

- `retire_facts` reads the written hidden state through `written_copy`.
- An API test: a feed made in Studio, with "Hide from apps" on and written, gives `ask` `written`. Before the correction, the test gives `draft_only`. Also cover a draft that a later request makes again with the same bytes.

### 259b: The Catalog, The Menus, The Find Rows And Check Again (UI)

- Findings 2, 4, 5, 6 and 9, as the operator choices give.

## Acceptance Criteria

Mechanical:

1. 259a: the API test of finding 8 passes, and fails on the code before the correction.
2. 259b: a Node test: the function of finding 2 gives false for an independent page with no publisher above it, and true for a page with a label parent or a publisher above it.
3. 259b: a Node test: the text of finding 9 is "Checked at {time}" for a given time, and no line before the first check.
4. `cargo test -p studio-core`, `cargo test -p studio-api`, `node --test crates/api/ui-tests/` and `tools/gate.sh` pass.

Visual (the main session with screenshots and `$edges()`):

- The Catalog at 1280 px with a child row and a top row: each handle is directly before its art, inside the row area.
- A drag of an independent page with no publisher: no top area. A drag of a page with a publisher: the top area shows.
- A menu **⋯** opens, and a click on an empty area closes it.
- The sheet "Make a new artist page": the row "Find on Music Index" has the search icon.
- The Retire block after **Check Again**: the line "Checked at …".

## Stop And Ask When

- A correction needs a change of an ADR rule.
- The test of finding 8 passes before the correction. Then the cause is different. Report the facts.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-259-defects-of-the-walk-of-2026-11-26.md
- docs/reviews/2026-11-26-human-check-walk.md, findings 2, 4, 5, 6, 8 and 9
- The locations of "Base components"

Goal:
- The packet that the main session names (259a or 259b). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- docs/, AGENTS.md, crates/desktop/, crates/desktop-app/. Change crates/core/ and crates/api/src/ only in packet 259a.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- cargo test -p studio-core (packet 259a only)
- cargo test -p studio-api (packet 259a only)
- node --test crates/api/ui-tests/
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
