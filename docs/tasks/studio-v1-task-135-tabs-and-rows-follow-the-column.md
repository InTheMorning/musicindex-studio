# Studio v1 Task 135: The Tabs And The Field Rows Follow The Editor Column

Decision: [ADR 0197](../adr/0197-the-tabs-and-the-field-rows-follow-the-editor-column.md).

Status: Open.

## Goal

At each width of the editor column, no tab label breaks inside a word, and no field value is cut off. The fit walk of musicindex ADR 0065 gives no finding at 390 px and at 761 px.

## Code Locations

These come from a search on 2026-11-13. Confirm each one before a change.

- `crates/api/ui/app.js:2129` `PageTabs`: one component for the feed tabs and the Settings tabs. It shows `<select class="button page-tab-menu">` when `useNarrow()` is true, else `<nav class="page-tabs segmented">`.
- `crates/api/ui/app.js:1829` `useNarrow()`: `useMedia('(max-width: 760px)')`. Other users of `useNarrow()` are at about lines 3141 and 3640. They stay as they are.
- `crates/api/ui/styles.css:1458` `.segmented` and `.segment`. Line 1479 has `overflow-wrap: anywhere`.
- `crates/api/ui/styles.css:1492` `.page-tabs` (`margin: 0 32px 8px; max-width: 816px`) and `.page-tab-menu` (`width: calc(100% - 32px); margin: 0 16px 8px`).
- `crates/api/ui/styles.css:1336` `@media (min-width: 761px)`: the two columns of `.group > .field` and `.group > .disclosure > .form > .field` (`170px 1fr`).
- `crates/api/ui/styles.css:1510` `@media (max-width: 760px)`: `.field.setting-row`.
- `crates/api/ui/styles.css:1973` `.host-op-row`: the rows of the tab **Check**.
- `crates/api/ui/shell.js`: `pageTabs` and the segment keys. `crates/api/ui-tests/shell.test.mjs` tests them.
- `crates/api/ui/pickers.js:176` `useListPlace`: the only `ResizeObserver` of the UI.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page tabs | extend | crates/api/ui/app.js | `PageTabs` gets the measured switch of ADR 0197 items 1 and 2 for the feed tabs and the Settings tabs. |
| Tab model | extend | crates/api/ui/shell.js | `pageTabs` is here. The fit rule of item 1 goes adjacent to it as a pure function with a Node test. |
| Width hook | new | crates/api/ui/app.js | Searched `crates/api/ui/*.js` and `~/build/musicindex-lib`. `useListPlace` in `pickers.js` places a list and gives no width. `useMedia` reads the window, not an element. |
| Field row layout | extend | crates/api/ui/styles.css | The two-column rule at line 1336 changes from a window query to a group width of 560 px. |
| Fit walk | reuse | tools/drive-ui.mjs | The fit step of musicindex ADR 0065 is the mechanical check. |

## Rules For The Delegate

- Read ADR 0197, ADR 0157 and musicindex ADR 0051 R-SEG-1 (`../musicindex/docs/adr/0051-one-segmented-control-and-one-selected-look.md`) first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR, a task file, `tools/fit-check.mjs` or `tools/drive-ui.mjs`.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Part 135a: The Tabs

1. In `shell.js`, add a pure function, for example `tabsFit({ available, labelWidths, gap, padding })`. It gives true when each segment can have the width of the longest label plus 4 px at each side (ADR 0197 item 1, R-SEG-1 items 4 and 5). Include the gap between segments and the padding of the tray. The width of a label includes the pencil mark of a Settings tab that is not saved.
2. In `PageTabs`, measure the width of the column that holds the tabs, and the width of each label on one line. Measure again when the column width or a label changes. One way: a wrapper that a `ResizeObserver` watches, and a hidden copy of the labels with `white-space: nowrap` and `visibility: hidden`. Then show the segmented control when `tabsFit` gives true, else the select menu. At 760 px or less, the select menu shows as before, because the column is narrow.
3. Remove `overflow-wrap: anywhere` from `.segment`. Give `.segment` `white-space: nowrap` (ADR 0197 item 3).
4. The closed `.page-tab-menu` has `width: auto` and `max-width` of its column, so it is as wide as its longest option with its padding and arrow (item 4). Its left edge is on the left edge of the page content, the same as the left edge of `.page-tabs`. Do not style the open list (item 5).
5. Node test in `crates/api/ui-tests/shell.test.mjs`: `tabsFit` gives true at the exact width, false at 1 px less, and uses the longest label for each segment.

## Part 135b: The Field Rows And The Check Rows

1. Give `.group` `container-type: inline-size`. Replace the window query at line 1336 with `@container (min-width: 560px)` for the same selectors (ADR 0197 item 6). A group narrower than 560 px puts the label above the value.
2. Do the same for `.field.setting-row` at line 1510, so that the rule follows the group, not the window.
3. In a group narrower than 560 px, the button of a `.host-op-row` goes below its text (item 7). The kind word and the text stay on the first line.
4. If `container-type` changes the width of a group that has no set width, stop and report.

## Acceptance Criteria

Mechanical:

1. The Node test of Part 135a passes.
2. The four commit checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `node --test crates/api/ui-tests/*.test.mjs`.
3. The fit walk gives no finding at 390 px and at 761 px for these walks, with `.studio-dev/walk/run.sh`: `generated/light-1280.json` with `setup-131.py`, `steps-131-final-light.json`, `steps-175-light.json` with `setup-175.py`, and a walk of Settings. The main session runs them.

Visual, by the main session, with screenshots:

- A feed page at 761, 900, 1000 and 1280 px with the left panel, and at 390 px, in the light and the dark theme. The tab **Check** and Settings at 761 px.
- The closed menu at 761 px: its width and its left edge.
- WebKitGTK of the desktop app applies `@container`. Check this with the WebKit script drive, because Chromium screenshots do not prove it.

Human, by the operator, in the desktop app:

- Make the window narrower, from wide to about 760 px. The tabs change to the menu before a label breaks. The fields put the label above the value.
- Open the menu. The native list opens, and each tab is in it.

## Stop And Ask When

- A label width depends on a font that loads after the first measure, and the switch shows the wrong form.
- `container-type` changes a layout outside the groups of the editor.
- A different row than those of item 7 needs a new rule to pass the fit walk.

## Report

- The changed functions and rules, with file and line.
- The Node test names.
- The output of the commit checks.
