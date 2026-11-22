# Studio v1 Task 228: One Collection Row Opens Switch Collection

Decision: [ADR 0242](../adr/0242-the-collection-row-of-the-footer-opens-switch-collection.md) items 1 to 6.

Status: Done.

## Problem

The footer of the desktop had two rows for one subject. "Collection: {name}" opened Settings, and "Switch Collection…" opened the sheet. With one collection, the footer had no collection row.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Panel footer model | extend | crates/api/ui/shell.js | `leftPanel` gives one row `collection` with the text of ADR 0242 item 1. |
| Panel footer rows | extend | crates/api/ui/app.js | The branch `collection` of `panelRows` opens the sheet. The branch `switch` is removed. |
| Switch Collection sheet | extend | crates/api/ui/app.js | `SwitchCollectionSheet` gets the link "Manage Collections…". |
| Settings tabs | reuse | crates/api/ui/pages.js | The tab `data` holds the section Collections. |

## Changes

1. `shell.js`, `leftPanel`: on the desktop with two or more collections, the row `collection` has the mark "Collection: {name}" and `mark: true`. With one collection, it has "Switch Collection…", the icon `folder-open` and `mark: false`. The row `switch` is removed.
2. `app.js`, `panelRows`: the row `collection` opens the sheet. Only a row with `mark: true` has the class `collection-mark`.
3. `app.js`, `SwitchCollectionSheet`: the footer has the link "Manage Collections…". It closes the sheet and opens Settings, tab Data.
4. `styles.css`: the link is at the left end of the sheet footer.
5. `shell.test.mjs`: one test for the rows of ADR 0241 items 1 and 8 and ADR 0242 items 1, 3 and 6. The test of `collectionlist.test.mjs` that stated "the row opens Settings" is deleted, because ADR 0242 replaces that rule.

## Acceptance Criteria

Mechanical:

- The Node tests of change 5 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots of the release Tauri binary):

- One collection: the footer ends with "Switch Collection…". Two collections: the footer ends with the accent row "Collection: HeyCitizen". Each row opens the sheet.
- The sheet has "Manage Collections…" at the left and **Cancel** at the right. The link opens Settings, tab Data.

## Results

- `tools/gate.sh` passes.
- The screenshots of each visual criterion pass (`.studio-dev/t224/tout228one`, `tout228two` and `tout228c`).
- The main session made the change, because it is small and the main session had the files open.
