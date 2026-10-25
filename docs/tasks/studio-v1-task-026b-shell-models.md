# Studio v1 Task 026b: The Shell Models And The Node Tests

Parent: [task 026](studio-v1-task-026-shell.md). Decisions: [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 1 to 9 and the Verification section, musicindex ADR 0027 R-NAV-2 items 2 to 4 and R-LIST-1.
Sequence: the second packet of task 026. It depends on 026a.

## Goal

The logic of the left panel, of the Releases list and of the back control is in a plain model module. Node tests examine it, and CI runs them.

## Files

Inspect: `crates/api/ui/app.js` (`mediumKind`, `isEditable`, `FeedTree`, `useNav`), `crates/api/ui/editor.js` (`trackKey`, `isTrack`, `isPublisher`), `crates/api/src/ui.rs`, `crates/api/src/tests.rs` (`each_ui_address_in_the_ui_files_is_served`), `.github/workflows/ci.yml`.

Change: a new model module `crates/api/ui/shell.js`, a new test file `crates/api/ui-tests/shell.test.mjs`, `ui.rs`, `ci.yml`, and `app.js` and `editor.js` only to import the moved helpers from `shell.js`.

Do not change: the components and the look of the UI. Packet 026c uses the model. Do not change `crates/core/` or the routes.

## Architecture

These choices are fixed. Do not change them.

### The Module

`crates/api/ui/shell.js` is a model module of ADR 0042. It imports nothing. It exports:

- `mediumKind`, `trackKey`, `isTrack`, `isPublisher` and `isEditable`, moved from `app.js` and `editor.js` with no change of behavior. The two files import them from `shell.js`.
- `leftPanel({ view, feeds, model, vaultOpen })`.
- `releasesList({ feeds, connectors, sort, view, marked })`.
- `listQuery(search)` and `queryText({ sort, view })`.
- `startNav(view)`, `pushNav(nav, view)`, `backNav(nav)`, `forwardNav(nav)`, `canBack(nav)`, `canForward(nav)` and `setScroll(nav, y)`.

### The Left Panel

`leftPanel` gives one of two values (ADR 0039 items 1, 5 and 6):

- With no open feed: `{ kind: 'destinations', top, bottom }`. `top` is Releases and Add a feed. `bottom` is Hosts, and Lock Studio when `vaultOpen` is true. Each entry is `{ id, label, current }`. The ids are `releases`, `add-feed`, `hosts` and `lock`.
- With an open feed (`view.name === 'feed'`): `{ kind: 'outline', feed, items, bottom }`. `feed` is `{ id, title, image, typeLabel }`. `items` starts with the destination link `{ id: 'releases', label: 'Releases' }`. Then:
  - an editable album: Album, one entry for each track `{ page: 'track', track, number, label }`, "Add a track", "Import files", "Value routes" (page `recipients`), "Credits and links", "Files".
  - an editable publisher feed: Publisher, Files.
  - a read-only feed: Feed (page `album`).
  - Each page entry has `current`. "Add a track" and "Import files" are `{ action: 'add-track' }` and `{ action: 'import' }`.
- With an open album before its draft model loads, the outline has the Releases link, Album and Files only.

### The Releases List

`releasesList` gives `{ sort, view, rows, letters }`:

- `sort` is `newest` or `az`, and `view` is `rows` or `tiles` (ADR 0039 item 2a).
- `newest` puts the greatest `added_at` first, and the greatest `id` first when two are equal. `az` compares the title, or the path with no title, with `localeCompare` and the base sensitivity.
- Each row is `{ id, title, typeLabel, host, image, draftLabel, marked }`. `host` is the name of the connector, or "Read-only" for a feed with no connector. `draftLabel` comes from `draft_state`: `new` gives "New, not written", `changed` gives "Saved, not written", and `unchanged` and `none` give "No changes". `marked` is true for the row of `marked`.
- `letters` is null for `newest`. For `az` it is the list of the first letters in sequence, each `{ letter, firstId }`. A title that does not start with a letter from A to Z goes to "#", after Z.
- `listQuery` reads `sort` and `view` from a query text, with the defaults for a value that is missing or not known. `queryText` gives the query with only the values that are not the default, or an empty text.

### The Navigation

The navigation state is `{ entries, index }`. Each entry is a view, such as `{ name: 'releases' }` or `{ name: 'feed', id, page }`, with an optional `scroll` and `marked` (musicindex ADR 0027 R-NAV-2):

- `pushNav` drops each entry after `index`, and adds the view.
- `backNav` from a feed to the Releases entry before it sets `marked` of that entry to the feed id, and keeps its `scroll` (item 2).
- `backNav` at index 0 on a feed puts `{ name: 'releases', marked: id }` before the feed, at index 0 (item 3). Forward then opens the feed again.
- `canBack` is true when `index` is above 0, or when the current view is a feed. `canForward` is true when an entry is after `index` (item 4).
- `setScroll` keeps a scroll position on the current entry.
- Each function gives a new state, and does not change its argument.

### The Tests And CI

- `crates/api/ui-tests/shell.test.mjs` uses only `node:test` and `node:assert/strict`. It has a test for each rule of this packet. It imports `../ui/shell.js`.
- `ui.rs` serves `shell.js`. It does not serve the test file.
- `ci.yml` gets `actions/setup-node@v4` with Node.js 22, and a step `node --test crates/api/ui-tests/*.test.mjs`.

## Steps

1. Make `shell.js` with the moved helpers, and change the imports.
2. Add the left panel, the Releases list, the query and the navigation functions, with their tests.
3. Serve the module, and add the CI step.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No package, no `package.json` and no build step ([ADR 0016](../adr/0016-ui-without-a-build-step.md), ADR 0042).
- The UI looks and works as before this packet.

## Acceptance Criteria

Mechanical:

- `node --test crates/api/ui-tests/*.test.mjs` passes, with a test for each rule above. When Node.js is not on the PATH, `tools/node --test crates/api/ui-tests/*.test.mjs` passes.
- With no open feed, the left panel model gives the four destinations. For an open album, it gives the outline in the sequence of ADR 0039 item 6.
- After a feed and Back, the Releases entry has the marked feed and its scroll.
- `canForward` is false with no subsequent entry.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

When Node.js is not on the PATH, run `tools/node` in place of `node`.

## Escalation

Stop and report when a rule of this packet needs a change of a component, or of the API.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-026b-shell-models.md
- docs/adr/0042-ui-models-with-node-tests.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- The files that the Files section names.

Goal:
- The logic of the left panel, of the Releases list and of the back control is in a plain model module. Node tests examine it, and CI runs them.

Constraints:
- Follow the Architecture section of the task file fully.
- No package, no package.json and no build step. The UI looks and works as before this packet.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The components and the look of the UI, crates/core/, the routes.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
