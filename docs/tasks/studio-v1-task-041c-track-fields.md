# Studio v1 Task 041c: The Track Fields

Parent: [task 041](studio-v1-task-041-msp-fields.md). Decisions: [ADR 0064](../adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) items 3, 4 and 8, [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md).
Sequence: the third packet of task 041. It depends on 041b.

Status: Not started.

## Goal

The track page gets "Explicit", the credits selection, the lyrics list and the season.

## Files

Inspect: `crates/api/ui/editor.js` (`TrackPage`, `Persons`, `EntryRow`, `EntryActions`, `addOpen`, `PickedImageField`, `useFilePick`, `ChooseFile`, `FIELD_NOUN`), `crates/api/ui/explicit.js`, `crates/api/ui/values.js` (the confirmation of the track recipients, line 520), `crates/api/ui/draft.js` (`withCids`), `crates/api/ui/picker.js`, `crates/api/src/ui.rs`.

Change: a new file `crates/api/ui/credits.js`, a new file `crates/api/ui-tests/credits.test.mjs`, `crates/api/ui/editor.js`, `crates/api/src/ui.rs`.

Do not change: the core, the API routes, `crates/api/ui/explicit.js`, `crates/api/ui/picker.js`.

## Architecture

These choices are fixed. Do not change them.

### "Explicit"

- The section "Track" gets `ExplicitField` of packet 041b, for `item.explicit`, after "Duration". It uses `explicitText` of `explicit.js`. A track with no `itunes:explicit` shows the switch in the off position, and Studio sets no value until the user changes it.

### The Model Module `credits.js`

- `usesAlbumCredits(item)` gives true when `item.persons` is empty.
- `withTrackCredits(item, albumPersons)` gives an item whose `persons` is a copy of the album persons. Each copied entry has no `_cid`, so that the editor gives it a new key. The values of the entries are the same as in the album.
- `withAlbumCredits(item)` gives an item with an empty `persons` list.
- The module has no Preact and no browser state.

### "Credits"

- The track page gets a section "Credits" after the section "Audio file".
- A select menu "Credits" has two values: "Use the album credits" and "Credits for this track". Its value comes from `usesAlbumCredits`.
- A change to "Credits for this track" applies `withTrackCredits` with `model.persons`, then gives each new entry its client ID with `withCids`. The section then shows `Persons` with the prefix `track-${cid}`, the list of the item and the address path `items/${index}/persons/…`.
- A change to "Use the album credits" asks for a confirmation first: "Remove the credits of this track? The track then uses the album credits." When the user confirms, the change applies `withAlbumCredits`. When the user cancels, the select keeps "Credits for this track".
- When the user removes the last person with "Remove", the track uses the album credits again. No confirmation is necessary, because the list is already empty.

### "Lyrics"

- The track page gets a section "Lyrics" after "Credits". It lists the `podcast:transcript` entries of the item, with `EntryRow`, `EntryActions` and `addOpen`, as `Persons` does. One entry is open at a time (ADR 0028).
- The summary line of an entry gives the file name of the address, the type and the language, such as "song.srt · application/srt · en".
- An entry has these fields:
  - "Lyrics file address", for `url`. It uses `PickedImageField` with the kind of the picker for a text file, or a `TextField` with the button "Choose…" of `ChooseFile`. When the picker has no kind for a text file, use a `TextField` with no picker, and report it.
  - "Type", for `media_type`: a select menu with `text/plain`, `text/html`, `text/vtt`, `application/json` and `application/srt`. When the feed has a different value, the menu also offers that value.
  - "Language", for `language`. The hint says: "For example en or de-AT."
  - "Captions", a check box for `rel`. On gives `captions`. Off gives `null`.
- A new entry has `url` empty, `media_type` `text/plain`, `language` `null` and `rel` `null`.

### "Season"

- The disclosure "Identity" gets "Season number" for `season.value` and "Season name" for `season.name`, before the facts.
- When both fields are empty, `season` becomes `null`. A new track has no season (ADR 0064 item 3).
- The summary line of "Identity" gives "Number 3 · Season 1" when the track has a season, and "Number 3" when it has none.

### The Served Files

- `crates/api/src/ui.rs` serves `credits.js`.

## Steps

1. Add `credits.js` with its Node tests.
2. Add "Explicit" to the section "Track".
3. Add the section "Credits".
4. Add the section "Lyrics".
5. Add the season to "Identity".
6. Serve the new module.
7. Run the four checks.
8. Take the screenshots of the visual criteria with `tools/drive-ui.mjs`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the files are.
- Each control follows the durable rules of `AGENTS.md`.
- A setting with a few values is a select menu, not a radio group.
- No logic in two places: the explicit form rule stays in `explicit.js`, and the credits rule is in `credits.js`.

## Acceptance Criteria

Mechanical:

- A Node test: `withTrackCredits` copies two album persons. The copies have the same values and no `_cid`, and the album list does not change.
- A Node test: `withAlbumCredits` gives an empty list, and `usesAlbumCredits` then gives true.
- A Node test: `usesAlbumCredits` gives false for an item with one person.
- An API test saves a draft whose track has persons, then saves it with an empty list. The diff removes each item `podcast:person` and no channel `podcast:person`.
- The four commit checks pass.

Visual, with screenshots, in the dark theme, in the light theme and at 390 px:

- The track page with "Explicit" on, "Credits for this track" with two persons, two lyrics entries with one open, and "Identity" open with a season.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `Persons` cannot take an address path for the persons of an item.
- Stop and report when the picker has no kind for a lyrics file.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-041c-track-fields.md
- docs/adr/0064-the-editor-edits-the-fields-that-msp-2-0-edits.md
- The files that the Files section names.

Goal:
- Add "Explicit", the credits selection, the lyrics list and the season to the track page.

Constraints:
- Follow the Architecture section of the task file fully.
- The credits rule is in `credits.js` only, with Node tests. Use `explicit.js` for the switch.
- A change back to the album credits asks for a confirmation.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The core, the API routes, `explicit.js` and `picker.js`.
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
