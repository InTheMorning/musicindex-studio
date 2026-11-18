# ADR 0214: Each Sheet Fits The Window, And The Walk Opens It

- Status: Accepted
- Date: 2026-11-15
- Revised: 2026-11-17, section "Consequences": the walk data of task 166.
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-15 by the operator, through musicindex ADR 0077 (musicindex `ff65fb9`, revised in `d3db352`). The operator selected that a bottom sheet in a narrow window spans the window, with the window margin as its inner padding.
- Technical choices: the planning model selected items 4 and 6 to 10, and the shape of the list in item 5.
- Bar: crosses repositories. Studio adopts musicindex ADR 0077, which binds musicindex.org, Studio and HitIt. Without this record, an agent can add a sheet and not add it to the list of item 5.
- Uses: musicindex ADR 0077 R-SHEET-1 and R-SHEET-2. musicindex ADR 0068 R-SPACE-1 item 2 for the window margin. musicindex ADR 0065 R-FIT-2 and R-FIT-3 for the fit check. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 16 for the bottom sheet at 760 px or less.

## Context

On 2026-11-15 the operator saw the sheet "Edit a Wallet" of People wider than the window, at a narrow width, in an earlier build. The operator cannot see the fault in the latest build. The cause was not found. At 390 px, the Chromium walk and a WebKitGTK walk gave a sheet of the window width. The WebKitGTK walk used the Medium scale and the Large scale.

The fit walk of musicindex ADR 0065 examined the pages only. No walk opened a sheet, so no check could find a sheet that does not fit. musicindex ADR 0077 makes each app open each sheet in its check.

## Decision

### The Width Of A Sheet

1. At a window width of 760 px or less, a sheet is a bottom sheet (ADR 0029 item 16). It spans the window width. Its header, its body and its footer keep each control, picture and text a minimum of 16 px from each side of the window.
2. At a window width of more than 760 px, a sheet has the smaller of two widths: its design width, and the window width less 20 px on each side. The design width is 760 px, and 520 px for a small sheet.
3. Each child of a sheet and of its body can become narrower than its content (`min-width: 0`). The body of a sheet scrolls only vertically.
4. A menu of `menu.js` is not wider than the window width less 16 px on each side. A menu with its button at the right end of its row opens to the left.

### The List And The Walk

5. `tools/sheets.json` lists each sheet, dialog, popover and menu of Studio. Each entry has a name, the source component and the drive-ui steps that open it. Each new sheet goes into the list in the commit that adds the sheet.
6. An entry that needs a condition that the walk data does not have names that condition in `data`. The step `{"sheets": "tools/sheets.json", "data": NAME}` of `tools/drive-ui.mjs` opens only the entries of that condition. The default condition is `walk`: the walk data with the person of the ADR 0076 walk.
7. For each width of R-FIT-2 (390 px and 761 px), the step loads the start address again and opens the entry at the start size of the walk. Then it sets that width.
8. The step examines the panel that the entry names in `panel`. The default is an open dialog, and a menu entry names `.sort-menu`. With no such panel open, the entry fails.
9. The step examines the panel with the width check of R-SHEET-2 item 3 and with the fit helpers of R-FIT-5. The overflow check and the overlap check examine only the controls in the sheet. A dialog in the sheet is a sheet of its own. A box that hides its overflow and cuts content is a fault, but a text with an ellipsis is not.
10. The step exits with status 1 and gives the message of R-SHEET-2 item 5.

## Alternatives Considered

### A side gap for a narrow sheet

A narrow sheet keeps 16 px from each side of the window. The operator selected the iOS layout: a bottom sheet spans the window, and its content keeps the margin.

### A list in the code of the walk

The walk keeps its entries in `tools/drive-ui.mjs`. A JSON file of its own keeps the list easy to read, and other walks can use it.

## Consequences

- At 390 px the close button of each sheet is 16 px from the right side, not 12 px.
- The People sort menu opens to the left.
- The folder path of a file host has the break attribute of R-FIT-3 item 3. The full key in the head of a payee page also has it.
- Each condition other than `walk` has a setup file and a steps file in `.studio-dev/walk`, and `.studio-dev/walk/sheets-all.sh` runs each condition ([task 166](../tasks/studio-v1-task-166-walk-data-for-each-sheet.md)). The condition `collections` needs a start with `CollectionsPlace`, which `run.sh` does not give. Its 6 entries do not open, and walk data for them is open work.
- The folder name and the selected folder of the Release Folder sheet have the break attribute of R-FIT-3 item 3.
- An entry opens at the start size. A sheet that opens only from a control of the narrow layout needs an entry with a narrow start size.

## How This Decision Is Checked

Mechanical:

- `.studio-dev/walk/run.sh` with the step `{"sheets": "tools/sheets.json", "data": "walk"}` opens each `walk` entry at 390 px and at 761 px. It exits with status 1 on these faults:
  - A sheet that is wider than the window allows.
  - Content in the inner padding of a bottom sheet.
  - Each fault of R-FIT-3 in the sheet.
- Negative test: the CSS gets the width `min(760px, 100vw - 32px)` from before and 12 px of right padding in the header. The walk then exits with status 1 and gives the two messages of R-SHEET-2 item 5.

The walk cannot prove that the list of item 5 is complete. The last review of a diff that adds a sheet examines the list.

Visual: make the window 390 px wide. Open People, the person, and Actions → Edit on a wallet row. The sheet spans the window from side to side, and its content has a minimum of 16 px on each side. Each field is in view. The sheet does not scroll horizontally.

The operator passed this visual check on 2026-11-15, on commit `cbca4ec`.
