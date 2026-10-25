# Studio v1 Task 026c: The Shell UI

Parent: [task 026](studio-v1-task-026-shell.md). Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 1 to 11, [ADR 0037](../adr/0037-button-roles.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 1 to 4 and 7 to 9.
Shared rules: musicindex ADR 0027 R-SHELL-1, R-NAV-2 and R-LIST-1, ADR 0024 R-PROFILE-1, ADR 0023 R-CASE-1.
Sequence: the third packet of task 026. It depends on 026b.

## Goal

The UI uses the models of 026b. Studio starts on Releases. The left panel shows the destinations or the outline. The bar has the back control, and the menu button at narrow width.

## Files

Inspect: `crates/api/ui/app.js` (`Shell`, `SidebarContent`, `FeedTree`, `Bar`, `useNav`, `useNarrow`, `FeedDetail`, `EditableFeed`, `Crumb`, `Hosts`, `AddFeeds`), `crates/api/ui/shell.js` from 026b, `crates/api/ui/editor.js` (`AlbumEditor`, the use of `go`), `crates/api/ui/styles.css` (`.sidebar`, `.tree`, `.drawer`, `.bar`), `crates/api/src/tests.rs` (the UI tests), `tools/drive-ui.mjs`.

Change: `app.js`, `styles.css` and the UI tests in `tests.rs` that examine the tree. Change `editor.js` only where it calls the navigation, and `tools/drive-ui.mjs` only to add a step that sets the window size.

Do not change: `shell.js`, other than a defect that a test shows. Do not change `crates/core/`, the routes, `values.js` and `draft.js`.

## Architecture

These choices are fixed. Do not change them.

### The Left Panel

- A new component `LeftPanel` replaces `SidebarContent` and `FeedTree`. It shows the value of `leftPanel()`: the brand and the theme button at the top, then the scroll area, then `bottom` pinned at the bottom of the panel (ADR 0039 items 1 and 5).
- The destinations are rows: "Releases" and "Add a feed". The outline is the row "Releases", the cover and the title of the feed with its type label, then one row for each item. A track row shows its number and its title, and the mark of a bad address as the tree does now.
- The row "Releases" opens Releases, with the row of the open feed marked (ADR 0039 item 7). It is a destination link. It is not the back control.
- The current row has `aria-current`. The mark of a change in the library folder shows on the title of the outline, and on the row of the feed in Releases.

### Releases

- The view `{ name: 'releases' }` is the start view. The view name `feeds` goes. No screen shows "Select a feed" (ADR 0039 item 4).
- A new component `ReleasesPage` shows the value of `releasesList()`. The bar title is "Releases". The title row has the heading "Releases" and, at its right end, the sort button and the two view buttons (R-LIST-1 item 4).
- The sort button is a normal button with the name of the current sort and "⌄", for example "Newest ⌄". It opens a menu with "Newest" and "A to Z", and a check mark on the current sort (R-LIST-1 item 1). The menu works from the keyboard, and Escape closes it.
- The view buttons are two icon buttons, "Rows" and "Tiles", with `aria-pressed`. The current one has the accent tint (R-LIST-1 item 3).
- With "A to Z", a letter bar shows each letter of `letters`. A letter scrolls its first row into view (R-LIST-1 item 2).
- A row shows the cover, the title, the type label, the host and the draft label (ADR 0039 item 2). A tile shows the cover, the title and the type label. A click opens the feed over Releases. The marked row has the fill of an open entry ([ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) item 12).
- The sort and the view come from the query of the address with `listQuery`. A change writes `queryText` into the address with `history.replaceState`, and keeps the hash.
- With no feed, the page shows "No releases yet." and the action button "Add a Feed".

### The Navigation

- The shell keeps the navigation state of 026b. Each new view is `pushNav`, with `setScroll` of the current scroll of `main` first. The shell also calls `history.pushState` with the index, so that the browser Back button works. A `popstate` event calls `backNav` or `forwardNav` until the index agrees.
- The shell keeps the state in `sessionStorage`, in a `try` block, so that a reload keeps it. A failed read starts a new state.
- After a change of view, `main` scrolls to the `scroll` of the entry. With no `scroll`, it scrolls to the top, or to the marked row of Releases.
- `useNav('shell', …)` goes. The wizard and the other screens keep their own `useNav`.

### The Bar

- The bar shows, in this sequence: the menu button at 760 px or less, the back control, the title (R-NAV-2 item 6, ADR 0039 items 9 and 10).
- The profile comes from `matchMedia('(pointer: coarse)')` (R-PROFILE-1 item 2). Desktop shows two icon buttons `‹` and `›`, with the names and the tooltips "Back" and "Forward". Touch shows one button `‹ Back`. Each button is disabled when `canBack` or `canForward` gives false.
- The menu button has the name "Show the panel". It opens the left panel as a drawer below the bar (ADR 0039 item 11). The drawer has the destinations or the outline, and Hosts and Lock Studio at its bottom. A selection in the drawer closes it.

### Labels

Each new button label is in title case (musicindex ADR 0023 R-CASE-1).

## Steps

1. Add `LeftPanel`, and remove `SidebarContent` and `FeedTree`.
2. Add `ReleasesPage`, and make it the start view.
3. Change the navigation and the bar.
4. Change the UI tests that examine the tree.
5. Run the `drive-ui.mjs` check below, and give the screenshots to the planning model.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step and no new UI file other than `shell.js` ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- All functions work from the keyboard. The page uses the three background levels of ADR 0029.

## Acceptance Criteria

Mechanical:

- No UI file has the text "Select a feed".
- The Node tests of 026b pass with no change.
- The three commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`:

- Studio starts on Releases with rows. "A to Z" shows the letter bar, and "Tiles" shows tiles. A reload keeps the sort and the view.
- A feed opens over Releases, and the left panel shows its outline in the sequence of ADR 0039 item 6.
- The row "Releases" and the back control each return to Releases with the row of the feed marked, at the same scroll position.
- Hosts and Lock Studio are at the bottom of the left panel in a window 1100 px high and in a window 600 px high.
- At 390 px, the menu button shows at the top left on Releases, on a feed and on Hosts. It opens and closes the drawer below the bar.
- The pages in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

When Node.js is not on the PATH, run `tools/node` in place of `node`. Run Studio with a data folder in `.studio-dev/`, as `AGENTS.md` says. Run `tools/drive-ui.mjs` with `tools/node` when Node.js is not on the PATH.

## Escalation

Stop and report when a rule of ADR 0039 needs a change of the API. Also stop when it needs a change of `shell.js` that is not a defect.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-026c-shell-ui.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The UI uses the models of 026b. Studio starts on Releases. The left panel shows the destinations or the outline. The bar has the back control, and the menu button at narrow width.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step and no new UI file other than shell.js. All functions work from the keyboard.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- shell.js (only a defect that a test shows), crates/core/, the routes, values.js, draft.js.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. Give the screenshots of the visual criteria to the planning model.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs (or tools/node in place of node when Node.js is not on the PATH)

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
