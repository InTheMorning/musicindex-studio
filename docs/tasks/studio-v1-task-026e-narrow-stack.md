# Studio v1 Task 026e: One Stack At Narrow Width

Parent: [task 026](studio-v1-task-026-shell.md). Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 9 to 11a, [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Shared rules: musicindex ADR 0030 R-SHELL-2, musicindex ADR 0027 R-NAV-2.
Sequence: the last packet of task 026. It depends on 026d.

## Goal

At 760 px or less, Studio has no menu button and no drawer. The left panel becomes two pages of the stack: the root page with the destinations, and the outline page of a feed. Back goes up the stack.

## Files

Inspect: `crates/api/ui/shell.js` (`leftPanel`, `outlineItems`, `startNav`, `pushNav`, `backNav`, `canBack`), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/app.js` (`Shell`, `Bar`, `LeftPanel`, `ReleasesPage`, `FeedDetail`, `EditableFeed`, the drawer code, `useNarrow`), `crates/api/ui/styles.css` (`dialog.drawer`, `.drawer-scrim`, `.bar .menu`, the rules at 760 px or less).

Change: `shell.js`, `shell.test.mjs`, `app.js`, `styles.css`.

Do not change: `editor.js` other than a call of the navigation, `values.js`, `draft.js`, `crates/core/`, the routes.

## Architecture

These choices are fixed. Do not change them.

### The Views

Two new view names:

- `{ name: 'root' }`: the root page "Studio" (ADR 0039 item 11).
- `{ name: 'outline', id }`: the outline page of a feed (ADR 0039 item 11a).

A wide window does not show these two views.

### The Model

`shell.js` gets these functions. Each one is pure, as ADR 0042 says.

- `narrowStack(view)`: the stack that a narrow window shows for one view. The entries go from the root to the view:
  - `root` gives `[root]`.
  - `releases` gives `[root, releases]`.
  - `hosts` and `add-feed` give `[root, view]`.
  - `outline` gives `[root, releases, outline]`.
  - `feed` gives `[root, releases, { name: 'outline', id }, view]`.
  - The index is the last entry.
- `openFeedView(id, narrow)`: the view of a selection of a feed on Releases. It is `{ name: 'outline', id }` when `narrow` is true, and `{ name: 'feed', id }` when `narrow` is false.
- `backNav` from an outline to a Releases entry marks the feed, as it does for a feed before this packet. `backNav` from a feed part to an outline entry keeps no mark.
- `canBack` is false on `root`.
- `leftPanel` accepts the view `outline` as an open feed. With the argument `narrow` true, the outline items have no "Releases" row, because Back opens Releases (ADR 0039 item 11a). With the view `root`, it gives the destinations, with no current row.

### The Shell

- The shell reads `narrow` from `useNarrow`. With `narrow` true, a selection of a feed on Releases pushes `openFeedView(id, true)`.
- A change of `narrow` makes a new navigation state: `narrowStack(current)` for narrow, and `startNav(current)` for wide. An `outline` view becomes `{ name: 'feed', id }` for wide. A `root` view becomes `{ name: 'releases' }` for wide.
- The root page shows the destinations of `leftPanel`, with Hosts and Lock Studio at the bottom of the page. Its bar has the title "Studio" and no back control.
- The outline page shows the outline of `leftPanel`: the cover, the title and the type label, then one row for each item. A track row shows its number and its title. "Add a track" and "Import files" open the album page, and then their sheet, as the left panel does. Its bar has the back control, the title of the feed and the draft toolbar of the feed. The shell loads the draft of the feed for the outline view too.
- The menu button, the drawer, the scrim, `drawerOpen`, `drawerTop` and their CSS go. The bar has no menu button at each width.
- The two rows of the bar of a feed at narrow width stay (ADR 0039 item 10a). The first row has the back control and the title.
- A wide window does not change.

## Steps

1. Add the model functions, with a test for each rule.
2. Add the root page and the outline page.
3. Remove the menu button and the drawer.
4. Run the `drive-ui.mjs` check below, and give the screenshots to the planning model.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step and no new UI file ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- All functions work from the keyboard. Each control has the target size of its profile.

## Acceptance Criteria

Mechanical:

- The Node tests examine `narrowStack` for each view, `openFeedView` for each width, the marks of `backNav` and `canBack` on `root`.
- A Node test examines the outline at narrow width: it has no "Releases" row.
- No UI file has the label "Show the panel", and `styles.css` has no rule for `dialog.drawer`.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs` at 375 px, with touch and with a mouse:

- Releases shows the back control. Back opens the root page "Studio" with Releases, Add a feed, Hosts and Lock Studio, and no back control.
- A release opens its outline page, with its tracks. Album opens the album page. Back returns to the outline page, and Back again returns to Releases with the row marked.
- The bar of the outline page and of the album page has the draft status, "Versions" and "Review".
- At 1100 px, the left panel shows at the side of the page, as before this packet.
- The pages in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

Run `tools/drive-ui.mjs` with `tools/node`.

## Escalation

Stop and report when a rule of this packet needs a change of the API. Also stop when it needs a change of a page of the editor.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-026e-narrow-stack.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- ../musicindex/docs/adr/0030-an-editor-app-is-one-stack-when-narrow.md
- The files that the Files section names.

Goal:
- At 760 px or less, Studio has no menu button and no drawer. The left panel becomes two pages of the stack: the root page with the destinations, and the outline page of a feed. Back goes up the stack.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step and no new UI file. All functions work from the keyboard.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- editor.js other than a call of the navigation, values.js, draft.js, crates/core/, the routes.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. Give the screenshots of the visual criteria to the planning model.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
