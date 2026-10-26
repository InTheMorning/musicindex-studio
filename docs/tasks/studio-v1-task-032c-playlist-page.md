# Studio v1 Task 032c: The Playlist Page And The Picker

Parent: [task 032](studio-v1-task-032-playlists.md). Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0037](../adr/0037-button-roles.md) and [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md).
Sequence: the third packet of task 032. It depends on 032b.

## Goal

The user makes a new playlist from "Add a feed", edits it on the playlist page, and adds tracks with a picker sheet.

## Files

Inspect: `crates/api/ui/editor.js` (`AlbumPage`, `PublisherPage`, `FeedFacts`, the value editor of the album page, `useSheet`), `crates/api/ui/app.js` (`AddFeeds`, the new album form, `OutlinePage`, the view routing), `crates/api/ui/values.js` (the recipient picker sheet, as the form of a sheet with a search), `crates/api/ui/playlist.js`, `crates/api/ui/shell.js`, `crates/api/ui/styles.css`, `tools/drive-ui.mjs`.

Change: `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css` (only when a class is necessary).

Do not change: the Rust crates. Do not change `shell.js` and `playlist.js`, other than a small helper with its Node test.

## Architecture

These choices are fixed. Do not change them.

### The New Playlist

- The new feed form of "Add a feed" gets the kind "Playlist", after "Album" and "Publisher feed". It asks for the name, the author, the host and the path, and sends `medium` `musicL`.

### The Playlist Page

- A `PlaylistPage` shows:
  - The channel fields of the album page: title, author, description, image, link and language.
  - The value routes of the channel, with the value editor of the album page.
  - A "Tracks" group. Each remote item is one row with `trackRow` of `playlist.js`, the buttons "Move up" and "Move down" with the normal role, and "Remove" with the destructive role. The group ends with "Add a track" with the action role.
  - `FeedFacts`.
- Each change goes into the draft model as the album page does ([ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md)).
- The page has no import of music files.

### The Picker Sheet

- "Add a track" opens a sheet with two tabs: "Search musicindex" and "My releases".
- "Search musicindex" has a search field. It uses `GET /api/v1/search?q=`. Each result is a feed row. A click on a feed row loads `GET /api/v1/musicindex/feeds/{guid}/tracks` and shows its tracks below it.
- "My releases" uses `GET /api/v1/playlist/tracks`, grouped by feed title.
- Each track row has "Add" with the action role. A track that the playlist has already shows "In the playlist" in place of the button (`hasTrack`).
- "Add" uses `addTrack` and keeps the sheet open, so that the user can add more tracks. "Done" with the primary role closes the sheet.

## Steps

1. Add "Playlist" to the new feed form.
2. Add `PlaylistPage` and its routing from the view.
3. Add the picker sheet.
4. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the comments and the texts in the files are.
- The UI loads nothing from other servers, and it has no build step ([ADR 0016](../adr/0016-ui-without-a-build-step.md)). An image of a search result goes through the image route of Studio, as the recipient picker does.
- Each control has the target size of its profile. Each function works from the keyboard.
- The page uses a maximum of three background levels ([ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md)).

## Acceptance Criteria

Mechanical:

- The four commit checks pass.

Visual:

- The planning model serves Studio against a local musicindex test server with a temporary ignored test, and runs `tools/drive-ui.mjs`. The screenshots show these items in the dark theme and in the light theme:
  - The new playlist form.
  - The playlist page with three tracks, after a move and a removal.
  - The picker with search results and the tracks of one feed, and the "My releases" tab.
  - The review sheet of the playlist.
  - At 390 px, the playlist page and the picker with no small control.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `AlbumPage` cannot give its channel fields to a different page without a large change. Propose the smallest shared part.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-032c-playlist-page.md
- docs/adr/0053-playlists-are-musicl-feeds-with-remote-items.md
- The files that the Files section names.

Goal:
- "Add a feed" can make a playlist. The playlist page edits the channel fields, the value routes and the tracks. A picker sheet adds tracks from a search of musicindex and from the releases of the collection.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.
- The UI loads nothing from other servers, and it has no build step.

Do not touch:
- The Rust crates. `shell.js` and `playlist.js`, other than a small helper with its Node test.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
