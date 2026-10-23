# Studio v1 Task 014: The Editor Layout

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 5.
Decisions: [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md) and [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md).
Depends on: [task 012](studio-v1-task-012-value-editor.md).

Mockup: [the editor layout](../notes/2026-10-22-editor-layout-mockup.html). Open it in a browser. It has static data.

## Packets

Four packets do this task, one after the other. Each packet has a prompt for a coding model. The planning model examines each packet with the [review checklist](../reviews/2026-10-22-task-014-review-checklist.md) before its commit.

1. [014a: the background levels](studio-v1-task-014a-background-levels.md): steps 1 and 12.
2. [014b: the draft toolbar and the sheets](studio-v1-task-014b-toolbar-and-sheets.md): step 3.
3. [014c: the sidebar tree and the editor pages](studio-v1-task-014c-tree-and-pages.md): steps 2 and 4 to 10.
4. [014d: the phone width](studio-v1-task-014d-phone-width.md): step 11.

## Goal

Put the draft controls in the toolbar, give the sidebar a tree of the editor pages, and use three background levels. Do not change the behavior of the core or of the API.

## Files

Inspect: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/values.js`, `crates/api/ui/styles.css`, `crates/api/src/tests.rs` and the mockup.

Change: the four UI files and `crates/api/src/tests.rs`.

Do not change: `crates/core`, the API routes in `crates/api/src`, and the tokens that `styles.css` copies from `search.html`.

## Steps

1. Add the three level tokens of ADR 0029 item 14 to `styles.css`, after the Studio tokens. Use them for the page, the groups and the open entries. Remove the borders from the panels, the tracks, the entries and the text fields (ADR 0029 items 11 to 13).
2. Show each section of an editor page as a heading on level 1 and one group on level 2. Divide the rows of a group with an inset separator. At more than 760 px, put each label at the left of its field.
3. Move the draft status, "Versions" and "Review" into the toolbar of the feed page (ADR 0029 items 1 to 4). Make the Review sheet and the Versions sheet with the `dialog` element. Keep each notice of the Changes panel: the save error, the different window, the changed hosted file (`conflict`), the diff that is not current, and the incorrect addresses. Show them in the Review sheet. When an address is not correct, the sheet shows no write button.
4. Make the editor of a feed keep its current page: `album`, `credits`, `recipients`, or a track by its client ID. The draft stays one draft for all the pages.
5. Make the sidebar tree (ADR 0029 items 5 and 6). A feed row opens and closes with `aria-expanded`. The current page has `aria-current="page"`. A track that has an incorrect address shows the error text in its tree row.
6. Make the Album page: the album fields, then a group with "Credits and links", "Recipients" and "Feed facts", then the tracks as rows that open the track page, then "Add a track".
7. Make the track page: the track fields, the audio file fields, the recipients section, then a group with credits, image and identity. Add the buttons for the track before and the track after, and Move up, Move down and Remove. Move the recipients of a track out of the disclosure group (ADR 0029 item 9).
8. Make the Recipients page and the Credits and links page from the current album sections.
9. Move the fields of "Add a track" into a sheet (ADR 0029 item 8). Studio continues to get the facts of the file from the host (ADR 0023).
10. Replace the hero of the editor page with the compact header (ADR 0029 item 10). Keep the hero for a feed that has no editor.
11. At 760 px or less, put the sidebar in a drawer behind a menu button (ADR 0029 item 15). Open each sheet from the bottom, and put each label above its field (ADR 0029 item 16).
12. Add the background test of the mechanical criteria below. When it fails, its message names ADR 0029 and the allowed tokens.
13. Remove `docs/notes/2026-10-22-editor-layout-mockup.html` and its line in `docs/README.md` when the visual check passes.

## Constraints

- The constraints of [task 011](studio-v1-task-011-album-editor.md) apply.
- Keep the sun and moon theme toggle as it is. The operator asked for no change to it.
- Keep the share bars, the share meters and the fee stripes of task 012.
- A split is a number of shares. The split of a fee is a percentage. Keep the labels "Split (shares)" and "Split (percent)". The mockup shows "%" only for its static data.
- Keep the theme toggle, the brand and the Hosts row in the sidebar.

## Escalation

Stop and ask the operator when:

- a notice of the Changes panel has no clear position in the Review sheet.
- a page needs a fourth background level.
- the `dialog` element fails in the WebView of the desktop shell.

## Acceptance Criteria

Mechanical:

- A test reads `styles.css`. A `background` or `background-color` value must contain no color literal and must not name `--bg`, `--surface` or `--surface2`. For each other value, the test fails and its message names ADR 0029.
- The tests of ADR 0016 and ADR 0028 pass with no change.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- The agent runs `tools/drive-ui.mjs` at 1280 px and at 390 px wide, on the Album page, a track page and the Review sheet. `$small()` gives an empty list, the page does not scroll to the side, and the console shows no error.

Visual:

- The operator opens a track from the tree, and the editor shows only that track.
- The operator edits a field at the end of a long page, and "Review" stays in view.
- The operator reviews and writes a change from the Review sheet. The diff and the write agree with the change.
- The operator puts an earlier version in the draft from the Versions sheet.
- The operator adds a track from the sheet, and the size and the type come from the host.
- On a phone width, the menu button opens the tree, and a tree item closes the drawer and opens its page.
- The operator finds the three background levels easy to read in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node tools/drive-ui.mjs "<studio-url-with-token>" .studio-dev/task-014/shots '<steps-json>'
```

## Final Report

Give the changed files and the result of each check command. Give the `drive-ui.mjs` results at each width. Give each escalation point that occurred.
