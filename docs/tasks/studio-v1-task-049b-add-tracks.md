# Studio v1 Task 049b: Add Tracks

Parent: [task 049](studio-v1-task-049-add-pages.md). Decision: [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md) items 11 to 14.

Status: Complete - 2026-10-28.

## Goal

The outline and the Tracks section have one row "Add tracks". It opens the file sheet. A link goes to the address sheet, where "Properties" holds the size and the type.

## Files

Inspect: `crates/api/ui/shell.js` (the outline items near line 146), `crates/api/ui-tests/shell.test.mjs` (near line 120), `crates/api/ui/app.js` (`Outline`, the click of an outline item near line 2320), `crates/api/ui/editor.js` (`AddTrackSheet`, `ImportSheet`, `AlbumPage` Tracks section near line 1829, `AlbumEditor` near line 3356, the "Properties" `details` near line 198), `crates/api/ui/audio.js` (`propertiesSummary`, `propertiesOpen`), the guide pages below.

Change: `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/app.js` (the outline click only), `crates/api/ui/editor.js`, `docs/user-guide/tracks.md`, `docs/user-guide/edit-album.md`, `docs/user-guide/find-your-way.md`.

Do not change: `audio.js`, the import of files in `ImportSheet` (its queue, its upload and its rows), `AddLiveSheet`, the "Add a track" of a playlist (`PlaylistPickerSheet` and its button), the Rust crates, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- The outline model gives one item `{ id: 'add-tracks', label: 'Add tracks', action: 'add-tracks' }`. It replaces the items `add-track` and `import`.
- The outline click: the action `add-tracks` calls `onImporting(true)`. The action `add-live` calls `onAdding('live')`, as in the current code.
- The Tracks section of `AlbumPage` has one `link-row add` "＋ Add tracks". It calls `onImporting(true)`.
- `ImportSheet`:
  - The title is "Add tracks". The `id` of the heading stays.
  - While `rows` is empty, the body shows `<button class="button link" type="button">The audio is already online</button>` after the file chooser. It calls a new prop `onAddress`.
- `AddTrackSheet`:
  - The title is "Add a track from an address".
  - The body shows Title, then Audio file address, then the status line of the lookup, as in the current sheet.
  - The size and the type are in `<details class="disclosure">` with `<summary class="link-row"><span class="label">Properties</span><span class="summary">${propertiesSummary({ length, media_type: type })}</span></summary>`, and the two current fields in its body.
  - The `details` is open when `propertiesOpen({ length, media_type: type }, [])` is true and the lookup state is `partial` or `failed`. It stays closed while the state is `none` or `reading`. The user can open it at each time.
  - After the fields, the body shows `<button class="button link" type="button">Choose files from this computer</button>`. It calls a new prop `onFiles`.
- `AlbumEditor` gives `onAddress={() => { onImporting(false); onAdding(true); }}` to `ImportSheet`, and `onFiles={() => { onAdding(false); onImporting(true); }}` to `AddTrackSheet`.

## Steps

1. Change the outline model and its test. The test expects `add-tracks` at index 3, and no item with the action `add-track` or `import`. Its message names ADR 0073 item 11.
2. Change the outline click in `app.js`.
3. Change `editor.js`.
4. Change the guide pages. `tracks.md` gives "Add tracks" first, then "The audio is already online". `edit-album.md` and `find-your-way.md` name the row "Add tracks".
5. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on the changed guide lines.
6. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). This packet adds no function that changes the model.
- Each comment, each test message, each UI text and each guide line is in ASD-STE100 Simplified Technical English.
- Each labeled button has its role of ADR 0037.

## Acceptance Criteria

Mechanical:

- The outline test passes with one item `add-tracks`.
- `grep -n "'import'\|'add-track'" crates/api/ui/shell.js crates/api/ui/app.js` finds no outline action.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when a different part of the UI reads the outline actions `add-track` or `import`.
- Stop and report when the switch between the two sheets leaves a file on the host.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0073-the-add-pages-ask-one-question-at-a-time.md
- docs/tasks/studio-v1-task-049-add-pages.md
- docs/tasks/studio-v1-task-049b-add-tracks.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- One row "Add tracks" opens the file sheet, a link goes to the address sheet, and "Properties" holds the size and the type in the address sheet.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- audio.js, the import queue of ImportSheet, AddLiveSheet, the playlist "Add a track", the Rust crates, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
