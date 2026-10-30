# Studio v1 Task 056a: The Destination Icons

Parent: [task 056](studio-v1-task-056-file-hosts-and-icons.md). Decision: [ADR 0092](../adr/0092-file-hosts-and-the-destination-icons.md) items 5 to 10.

Status: Not started.

## Goal

Each fixed destination shows its Lucide icon. The current destination has the accent color. `lock-open` is in `musicindex-assets`.

## Files

Inspect: `../musicindex-assets/AGENTS.md`, `../musicindex-assets/icons/lucide/` (`library`, `wallet`, `hard-drive`, `settings`, `lock`), `crates/api/ui/shell.js` (`leftPanel`), `crates/api/ui/app.js` (`Chevron`, `LeftPanelFooter`, `RootPage`, the Releases row of the left panel), `crates/api/ui/styles.css` (`.row`, `.row[aria-current="page"]`, `.row .chevron`), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui-tests/secrets.test.mjs` (a test that reads `app.js`).

Change: `crates/api/ui/shell.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css`, a new Node test file `crates/api/ui-tests/destinations.test.mjs`, `crates/api/ui-tests/shell.test.mjs`. In `../musicindex-assets`: `icons/lucide/lock-open.svg`, `manifest.json` and the previews.

Do not change: the core, the API, the ADRs, an icon of `musicindex-assets` other than `lock-open`.

## Architecture

These choices are fixed. Do not change them.

1. First, a commit in `../musicindex-assets` adds `icons/lucide/lock-open.svg` from the npm package `lucide-static` 1.43.0, as for the other icons. Remove the license comment and the class attribute. Run `python3 scripts/check.py write`, `python3 scripts/check.py verify` and `python3 scripts/preview.py` in that commit.
2. Each fixed entry of `leftPanel` gets `icon`: `library` (`releases`), `hard-drive` (`hosts`), `settings`, `lock` and `lock-open` (`unlock`). The map of item 3 also has `wallet`, for the Recipients row of task 054c. A publisher feed entry gets no `icon`.
3. A new component `Icon({ name })` in `app.js` draws the elements of each icon in `<svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">`. One object maps each name to its elements, copied from `musicindex-assets`. A comment names musicindex ADR 0022 and the source file of each icon. `Chevron` stays as it is.
4. The rows of `LeftPanelFooter`, `RootPage` and the Releases row put `Icon` before the label.
5. In `styles.css`, `.row .icon` has `flex: none`, a width and a height of `1lh` (the line height of the row text), and `color: inherit`. `.row[aria-current="page"]` gets `color: var(--accent)`. Check the accent token name in `styles.css` before the change.
6. `destinations.test.mjs` has these tests:
   - Each fixed destination of `leftPanel` gives the icon of item 2.
   - No destination label of `leftPanel` is the same as a role heading of `shell.js`.
   - The source of `LeftPanelFooter` and `RootPage` in `app.js` has no `dot` class.
   Each failure message names ADR 0092 and the fix, for example: "ADR 0092 item 9: a destination row shows no type dot. Use the icon of its leftPanel entry."

## Steps

1. Add `lock-open` to `musicindex-assets`, and commit it in that repository.
2. Write the icons in `shell.js` and `app.js`, and the Node tests.
3. Take the screenshots, and run `$edges()`.
4. Run the four commit checks.

## Constraints

- The icon has `aria-hidden="true"`. The label gives the accessible name.
- The row keeps its target size.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- The three tests of item 6 pass.
- `python3 scripts/check.py verify` passes in `../musicindex-assets`.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- Each icon is centered on the line of its label.
- The icon and the label of the current destination have the accent color.
- "Unlock Studio" on a server with a locked vault shows `lock-open`.
- `$edges()` finds no content at the edge of a group.

## Escalation

- Stop and report when `lucide-static` 1.43.0 has no `lock-open`.
- Stop and report a contrast below 4.5:1 for the accent color on the fill of the current row.
