# Studio v1 Task 032b: The Playlist Models

Parent: [task 032](studio-v1-task-032-playlists.md). Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md) items 1, 4, 6 and 7, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the second packet of task 032. It depends on 032a.

## Goal

The UI model modules know that a `musicL` feed is editable, and give the rules of the playlist tracks. Node tests examine them. No Preact component changes in this packet.

## Files

Inspect: `crates/api/ui/shell.js` (`mediumKind`, `isEditable`, `isPublisher`, `leftPanel`), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/announce.js` (the form of a model module), `crates/api/src/ui.rs` (the list of UI files).

Change: `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, the new files `crates/api/ui/playlist.js` and `crates/api/ui-tests/playlist.test.mjs`, and `crates/api/src/ui.rs` (one entry for `playlist.js`).

Do not change: the Preact components in `app.js` and `editor.js`, the Rust code other than the `ui.rs` entry.

## Architecture

These choices are fixed. Do not change them.

### The Shell

- `isPlaylist(feed)` is true for the medium `musicL`, with no regard to case.
- `isEditable(feed)` is also true for a playlist with a connector.
- The outline of a playlist in `leftPanel` has the entries "Playlist", "Tracks", "Value routes" and "Files". It has no "Add a track" and no "Import files" entry.

### The Playlist Module

`playlist.js` has no Preact and no browser state. It exports:

- `pickKey(track)` gives `feedGuid + ' ' + itemGuid` for a remote item or a pick.
- `remoteItemFromPick(pick)` gives `{ feedGuid, itemGuid, feedUrl, medium: 'music', title }` from a pick of the two sources. A missing `feedUrl` or `title` stays out of the object.
- `addTrack(remoteItems, pick)` gives a new list with the new remote item last. When the list has the same `pickKey`, it gives the list with no change.
- `hasTrack(remoteItems, pick)` is true when the list has the same `pickKey`.
- `moveTrack(remoteItems, index, delta)` gives a new list with the entry at `index` moved by `delta` (`-1` or `1`). A move out of the list gives the list with no change.
- `removeTrack(remoteItems, index)` gives a new list without that entry.
- `trackRow(remoteItem)` gives `{ title, sub }`. `title` is the `title` attribute, or "Untitled track". `sub` is the feed GUID and the item GUID, separated by " · ".
- `tracksSummary(remoteItems)` gives "No tracks", "1 track" or "N tracks".

Each function keeps the other fields of each entry, such as the entry key of [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md).

## Steps

1. Change `shell.js` and its tests.
2. Add `playlist.js`, its tests and the `ui.rs` entry.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- The UI loads nothing from other servers, and it has no build step.

## Acceptance Criteria

Mechanical:

- The Node tests examine each function above, with a list that keeps the entry keys.
- The shell tests examine `isPlaylist`, `isEditable` for a playlist, and the outline of a playlist.
- The four commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a remote item of the draft model has different field names. The expected names are `feedGuid`, `itemGuid`, `feedUrl`, `medium` and `title`. Give the names that it has.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-032b-playlist-models.md
- docs/adr/0053-playlists-are-musicl-feeds-with-remote-items.md
- The files that the Files section names.

Goal:
- `shell.js` treats a `musicL` feed as an editable playlist with its own outline. The new model module `playlist.js` gives the rules of the playlist tracks, with Node tests.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The Preact components in `app.js` and `editor.js`, and the Rust code other than one entry in `crates/api/src/ui.rs`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

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
