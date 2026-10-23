# Studio v1 Task 014d: The Phone Width

Parent: [task 014](studio-v1-task-014-editor-layout.md). Decision: [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 15 and 16.
Sequence: after [task 014c](studio-v1-task-014c-tree-and-pages.md). It is the last packet.

## Goal

At 760 px or less, put the sidebar in a drawer, open the sheets from the bottom, and put each label above its field.

## Files

Inspect: `crates/api/ui/app.js` (`Shell`, `FeedTree`, `FeedDetail`, `Hosts`, `AddFeeds`), `crates/api/ui/styles.css`, and the mockup `docs/notes/2026-10-22-editor-layout-mockup.html` (`dialog.drawer`, `.bar .menu`, and the `@media (max-width: 760px)` block).

Change: `app.js` and `styles.css`.

Do not change: `editor.js`, `values.js`, `draft.js`, `crates/core`, the API routes, and the tests.

## Architecture

These choices are fixed. Do not change them.

- The hook `useNarrow()` in `app.js` gives `true` when `matchMedia('(max-width: 760px)')` matches. It follows each change of the match.
- The sidebar content is one component, `SidebarContent`. At wide width, `Shell` shows it in the `nav.sidebar`. At narrow width, `Shell` shows it in a `dialog.drawer`, and shows no `nav.sidebar`.
- A component `Bar` in `app.js` makes each `.bar`. It has a menu button with the label "Show the feeds". CSS hides the button at more than 760 px. `FeedDetail`, `Hosts` and `AddFeeds` use `Bar`. `Shell` gives each of them an `onMenu` function.
- A selection in the drawer closes the drawer. A click on the backdrop closes the drawer.

## Steps

1. Add `useNarrow`, `SidebarContent` and `Bar`, and use them in `Shell`, `FeedDetail`, `Hosts` and `AddFeeds`.
2. Add the drawer styles and the menu button styles from the mockup.
3. In the `@media (max-width: 760px)` block: remove the current rules that put the sidebar above the editor. Open each `dialog.sheet` from the bottom, with the full width and rounded top corners. Put each label above its field. Hide the long status text and the "Versions" label, and keep their buttons.
4. Keep the theme toggle, the brand, the Hosts row and "Lock Studio" in `SidebarContent`.

## Constraints

- The page does not scroll to the side at 390 px.
- Each control is 44 px high or more, and each control works from the keyboard.
- The menu button has a name that a screen reader can read.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `tools/drive-ui.mjs` at 390 px. To set the width, the agent uses a copy of the tool in the scratchpad. The copy sends `Emulation.setDeviceMetricsOverride`.
- At 390 px, the agent clicks "Show the feeds", then the third track. The drawer closes and the page text contains "Track 3 of". `document.documentElement.scrollWidth` is 390. `$small()` gives an empty list. The console shows no error.
- The same check at 1280 px shows no menu button.

Visual:

- None in this packet. Task 014 has the visual check.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p studio-desktop -- --data-dir .studio-dev/task-014/data
node tools/drive-ui.mjs "<address from the run command>" .studio-dev/task-014/shots '<steps-json>'
```

## Escalation

Stop and report when the drawer needs a change to `editor.js`. Stop and report when a screen that the Architecture section does not name has a `.bar`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-014d-phone-width.md
- docs/adr/0029-editor-pages-toolbar-and-three-levels.md
- crates/api/ui/app.js
- crates/api/ui/styles.css
- docs/notes/2026-10-22-editor-layout-mockup.html
- tools/drive-ui.mjs

Goal:
- At 760 px or less: the sidebar is a drawer, the sheets open from the bottom, and each label is above its field.

Constraints:
- Follow the Architecture section of the task file fully.
- No side scroll at 390 px. Keep each control 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- editor.js, values.js, draft.js, crates/core, the API routes, the tests.
- Do not commit.

Acceptance criteria:
- The criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- The drive-ui.mjs check of the task file.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
