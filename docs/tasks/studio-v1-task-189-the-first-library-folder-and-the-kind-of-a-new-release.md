# Studio v1 Task 189: The First Library Folder And The Kind Of A New Release

Decision: [ADR 0223](../adr/0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 10 to 12.

Status: Done. Open: the human check of "Choose Folder…" in the desktop app.

## Problem

Observations O10 and O12 of the [user journeys](../research/2026-11-18-user-journeys/observations.md). The first-run library sheet has a text field only. A new release with no track shows "ALBUM", and "SINGLE" after the first track.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Folder dialog route | extend | crates/api/src/routes.rs | `choose_folder` near line 4706 calls the `FolderPicker` with the library root. With no library, it must start at the home folder. |
| Desktop app check | reuse | crates/api/ui/editor.js | `inDesktopApp` and the button "Choose Folder…" of `LibraryFolderSheet` near line 4595. |
| Library sheet | extend | crates/api/ui/app.js | `LibrarySheet` near line 1774. |
| Kind rule | extend | crates/core/src/feed.rs | `release_kind_shown` near line 451, with its tests near line 890. |
| Kind label | extend | crates/api/ui/shell.js | `feedKind` near line 50. |
| Kind options | extend | crates/api/ui/addfeeds.js | The option list near line 36, and the menu "Kind" of the release page that uses it. |

## Changes

1. `routes.rs`: in `choose_folder`, when the library is not set, call the picker with the home folder of the user. Keep the current start folder when the library is set.
2. `app.js`: in the desktop app, `LibrarySheet` has the button "Choose Folder…" (class `button`) adjacent to the field "Folder". It calls `POST /choose-folder`. A returned `path` fills the field. A `null` path changes nothing. A different shell shows only the field.
3. `feed.rs`: `release_kind_shown` gets the count of live items, or the callers pass a count that includes live items. With no kept kind, no track and no live item, it gives `"release"`. Keep each other result. Add the case to the test near line 890. Do not change the feed file.
4. `shell.js`: `feedKind` gives the label "Release" for `kind_shown` `"release"`.
5. The menu "Kind": the option with the value `''` has the text "From the tracks ({label})", where `{label}` is the label of the current `kind_shown`, for example "From the tracks (Single)". With no track and no live item, it is "From the tracks (no tracks yet)". The other options do not change.

## Files

- Change: `crates/api/src/routes.rs`, `crates/api/ui/app.js`, `crates/core/src/feed.rs` and each caller of `release_kind_shown`, `crates/api/ui/shell.js`, `crates/api/ui/addfeeds.js` and the file of the menu "Kind".
- Tests: the core test of change 3. An API test of change 1 with a test picker that records its start folder. A UI test of changes 4 and 5. Change each test that asserts "album" for no track.

## Checks

Mechanical:

- `cargo test -p studio-core` and `cargo test -p studio-api` pass, with the new tests.
- UI test: `feedKind` gives "Release" for `kind_shown` `"release"`. The option text of change 5 for "single" and for no track.
- UI test: the source of `LibrarySheet` has "Choose Folder…" and `/choose-folder`.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- A new release with no track: the tag "RELEASE" and the option "From the tracks (no tracks yet)".

Human, by the operator: "Choose Folder…" in the first-run sheet of the desktop app opens the dialog of the system. The selected folder fills the field.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
