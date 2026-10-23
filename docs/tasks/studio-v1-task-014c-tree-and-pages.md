# Studio v1 Task 014c: The Sidebar Tree And The Editor Pages

Parent: [task 014](studio-v1-task-014-editor-layout.md). Decision: [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md) items 5 to 10.
Sequence: after [task 014b](studio-v1-task-014b-toolbar-and-sheets.md).

## Goal

Divide the editor into pages, show the pages as a tree in the sidebar, and give the editor page a compact header.

## Files

Inspect: `crates/api/ui/app.js` (`useNav`, `Shell`, `FeedDetail`), `crates/api/ui/editor.js`, `crates/api/ui/values.js`, `crates/api/ui/draft.js`, `crates/api/ui/styles.css`, and the mockup `docs/notes/2026-10-22-editor-layout-mockup.html` (`tree()`, `albumPage()`, `trackPage()`, `recipientsPage()`, `creditsPage()`, and the `add` dialog).

Change: `app.js`, `editor.js`, `values.js` and `styles.css`.

Do not change `crates/core`, the API routes or the tests. In `draft.js`, add one export only if necessary.

## Architecture

These choices are fixed. Do not change them.

- The view of a feed is `{ name: 'feed', id, page, track }`. `page` is `'album'`, `'credits'`, `'recipients'` or `'track'`. `track` is a track key. A view with no `page` is the album page. Each page change uses the `go` function of `useNav`, so that the browser Back button works.
- The key of a track is `item.guid` when it has one. When it has no GUID, the key is `` `cid-${item._cid}` ``. Put `trackKey(item)` in `editor.js` and export it.
- When the track key of the view is not in the model, the editor shows the album page. This occurs after a write, because the cids change.
- `Shell` keeps an `adding` state for the Add a track sheet. "Add a track" in the tree sets it, and goes to the album page of the feed. `AlbumEditor` gets `adding` and `onAdding`.
- The tree is the component `FeedTree` in `app.js`. It gets the feeds, the view, the draft and `go`. Only the selected feed shows its pages, and only when `draft.model` is not `null`.

## Steps

1. Make `FeedTree`. Each feed row is a `.node` button with `aria-expanded`. A click on a different feed goes to its album page and opens it. A click on the selected feed opens or closes its pages.
   - Below the open feed: "Album", "Credits and links", "Recipients", a "Tracks" label, a row for each track with its number and title, and "＋ Add a track". The current page has `aria-current="page"`.
2. A track row that has an incorrect address (a `draft.bad` field that starts with `items/{index}/`) shows the text "Address not correct" with the `field-error` class.
3. Make `AlbumEditor` show one page from `page` and `track`:
   - **album**: the album fields, then an "Album details" group with link rows for "Credits and links" (the current `creditSummary`), "Recipients" (the count and `ShareStack`) and "Feed facts". Then the tracks as link rows with number, title and duration, then a "＋ Add a track" row and the note about track numbers.
   - **credits**: the current `Persons`, `Fundings` and `LicenseFields`, each as a section.
   - **recipients**: the current album recipients, with no `details` around them.
   - **track**: a header with "Track N of M" and the buttons for the track before and the track after. Then the sections "Track" (title, description, publication date, duration), "Audio file" (`Enclosure`), "Recipients", and "More" (track image, and the identity facts in a `details.disclosure`). Then "Move up", "Move down" and "Remove the track". After a remove, go to the album page.
4. In `values.js`, change `AlbumPayments` and `TrackPayments` into sections with no `details` element (ADR 0029 item 9). Keep `Recipients` and `Payments` as they are.
5. Make "Feed facts" a `details.disclosure` row. It holds the facts of `FeedDetail` and the "Remove from collection" button.
6. Change `FeedDetail`: an editable album gets the compact header (cover, type, title, address, "Read again"). Other feeds keep the hero and the facts.
7. Move `AddTrack` into a sheet, `AddTrackSheet`. It opens from `adding`. After an add, it closes and goes to the page of the new track.
8. Make each page a sequence of sections. A section is a heading on level 1, then one `.group` on level 2. Copy the `.group`, `.row`, `.link-row`, `.section-head` and `.node` styles from the mockup. At more than 760 px, a `.field` in a `.group` puts its label at the left, in a column of 170 px.

## Constraints

- One draft for all the pages. A page change does not load the draft again.
- Each `details` element keeps a `name` or the class `disclosure` (ADR 0028).
- Keep the error text of each address field (ADR 0023).
- The current editor has no credits for a track. Do not add them. The mockup shows them only as an example.
- Each control is 44 px high or more, and each control works from the keyboard.

## Acceptance Criteria

Mechanical:

- The three commit checks pass.
- The agent runs `tools/drive-ui.mjs` at 1280 px. It clicks the second track in the tree. The page text contains "Track 2 of" and the title of that track, and does not contain the title of the first track. The agent clicks "Recipients" in the tree, and the page shows the recipients. `$small()` gives an empty list. The console shows no error.
- The agent goes back with `history.back()` and the album page shows again.

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

Stop and report when:

- a page needs a fourth background level.
- the track key is not stable during a save.
- `draft.js` needs more than one new export.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-014c-tree-and-pages.md
- docs/adr/0029-editor-pages-toolbar-and-three-levels.md
- docs/adr/0028-summary-lines-and-one-open-entry.md
- crates/api/ui/app.js
- crates/api/ui/editor.js
- crates/api/ui/values.js
- crates/api/ui/draft.js
- crates/api/ui/styles.css
- docs/notes/2026-10-22-editor-layout-mockup.html

Goal:
- The editor has one page for each tree item. The sidebar shows the tree. An editable album has a compact header.

Constraints:
- Follow the Architecture section of the task file fully.
- One draft for all pages. Keep each address error. Do not add track credits.
- Keep each control 44 px high or more.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, the API routes, the tests. draft.js only for one new export.
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
