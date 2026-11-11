# Studio V1 Task 108: A List Is One Tab Stop

Status: Complete, in four packets, with the key drives and the screenshot visual check on 2026-11-11. The human test with a real keyboard in the AppImage is open. The operator set the sequence of task 107, then this task, on 2026-11-10.

Result of 108a and 108b: `crates/api/ui/listkeys.js` gives the model (`createListModel`, `listKey`) and `onTreeKey`, which builds the model from the rows of a tree at each key.

The left panel (the levels and the outline of a feed) and the folders of the file picker use it.

A drive with key events from Chromium at 1100 px gave these results. One Tab goes into each tree and the next Tab leaves it. Shift+Tab goes back to the same row. Up, Down, Home and End move, and they skip the rows in a closed folder. Right and Left open and close a folder. Left on a child row goes to its parent. Enter opens the page of a row or opens a folder.

A row with no child rows has no `aria-expanded`. The file picker records the open folders, so `aria-expanded` follows each render. A file with no public address stays in the tree with `aria-disabled`.

Result of 108c: `onListKey` in `listkeys.js` gives the keys of a list of expandable rows. The credit and support rows (`EntryRow`), the recipient rows and the rows of "Where this person is" use it. Each `summary` has `aria-expanded` from the open condition of its row, and no role `treeitem`. A `focusin` on a row makes it the current row.

A drive with key events from Chromium at 1100 px gave these results. One Tab goes into each list and the next Tab leaves it. Down, Home, Right, Left and Enter act on the rows. From an open row, Tab goes into its fields, and after the last field Tab goes to "Add a credit".

Result of 108d: `RowList` in `pickers.js` makes a list of rows that open a page or select a value into one Tab stop, with `onPageListKey`. It is on the People page, the page of a person, each `PickFold`, `SearchResults`, the Add feeds page and both views of Releases. The tiles view uses the grid mode of the model, with the column count from the positions of the tiles. A disabled row, and a `PickFillRow` that the list already holds, stays in the list with `aria-disabled`.

The drive gave these results. On Releases, Tab goes into the rows and out again, and Down and End move. In the tiles view, Left and Right stay in a row of tiles, Up and Down keep the column, and Enter opens the release. On the People page and on the Add feeds page, each list is one Tab stop, and a fold header keeps its own Tab stop. In the sheet "Add a Recipient", the 17 payees are one Tab stop, and Enter on a row fills the form.

Owner of the rules: [ADR 0172](../adr/0172-a-list-is-one-tab-stop.md). It applies musicindex ADR 0061 R-LISTKEY-1 and R-LISTKEY-2. [ADR 0171](../adr/0171-a-closed-fold-is-a-row-and-enter-closes-the-field-list.md) gives the types of row.

## Goal

Each list and each tree of ADR 0172 item 2 is one Tab stop. The arrow keys, Home and End move in it. Today each row is its own Tab stop, and no list has arrow keys.

## Packets

- 108a: the key module and its Node tests (ADR 0172 items 5 and 6).
- 108b: the trees: the left panel and the folders of the file picker.
- 108c: the lists with expandable rows: `entry-row`, the recipient rows and the rows of "Where this person is".
- 108d: the lists of rows that open a page or select a value, and the tiles view of Releases (ADR 0172 items 2 and 3).

Do the packets in this sequence. Each packet leaves the tests green.

## Files To Inspect

- `crates/api/ui/app.js`: `LeftPanel` and `Outline` (around line 2900 to 3060), the expand control with `aria-expanded` (around line 3134), `ReleasesPage` (around line 3158), and `onSegmentKey` (around line 2010) as an example of a key handler.
- `crates/api/ui/editor.js`: `EntryRow` (around line 129) and the folders of the file picker (`picker-folder`, around line 929).
- `crates/api/ui/values.js`: the recipient rows (`details class="recipient"`, around line 139).
- `crates/api/ui/peoplepages.js`: `ListRow` (around line 101), `PeoplePage`, and the rows of "Where this person is" (`where-entry`).
- `crates/api/ui/pickers.js`: `PickFold` and `SearchResults`.
- `crates/api/ui/app.js`: the Add feeds page (`AddFeeds`).
- `../musicindex/docs/adr/0061-a-list-is-one-tab-stop.md`. Read it only. Do not change it.

## Constraints

- No feed rule in the UI (ADR 0065).
- One key module for all lists and trees (ADR 0172 item 6). A list does not write its own key handler.
- A tree uses the roles `tree` and `treeitem` with `aria-expanded`. An expandable row is a button with `aria-expanded`, not a tree item (musicindex ADR 0061 R-LISTKEY-2).
- A fold header, a segmented control, a select menu and the field list keep their own keys (ADR 0172 item 4).
- `node --check` misses template errors. Check a `.mjs` copy of each changed UI file.

## Acceptance Criteria

Mechanical, with Node tests of the key module:

- A list model gives the next row for Up and Down, the first row for Home and the last row for End.
- Right opens a closed row, and Left closes an open row.
- On a child row of a tree model, Left gives the parent row.
- Enter gives the primary action of the row: the page of a row with a page, and the open condition of an expandable row.
- The module keeps the current row of a list after the focus leaves it.

Mechanical, with a browser drive at 1100 px:

- For each list and tree of ADR 0172 item 2, one Tab goes into the list, and the next Tab leaves it.
- In an open expandable row, Tab goes into its fields, and after the last field it leaves the list.
- The roles: `tree` and `treeitem` with `aria-expanded` on the left panel and the file picker, and a button with `aria-expanded` on each expandable row.
- The chevron of a row has no Tab stop.

Visual, by the agent:

- The focus ring on the current row of each list and tree, in the dark and the light theme, at 1100 px. Use `$edges()` and `$overflow()`.

Human, by the operator:

- The keys on the left panel and on the Credits page, with a real keyboard, in the AppImage.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- A list that ADR 0172 item 2 does not name has rows with their own Tab stops.
- A row has two primary actions, so Enter cannot choose one.
- WebKitGTK reads the roles of a tree in a different way from Chromium.
