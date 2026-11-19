# Studio v1 Task 190: The Level Stop And Add Tracks After A New Release

Decision: [ADR 0224](../adr/0224-a-finding-that-stops-the-write-and-a-new-release-opens-add-tracks.md) items 1 to 8.

Status: Done. The screenshot visual checks passed.

## Problem

Observations O3 and O11 of the [user journeys](../research/2026-11-18-user-journeys/observations.md). The Check tab shows a missing artist page as "Notice", but Review stops the write. The release checklist has no row for the artist page. After **Make the Release**, the user must find "Add tracks".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Finding level | extend | crates/core/src/feed_check.rs | `FindingLevel` near line 66, and the page refusal finding near line 236. |
| Check rows | extend | crates/api/ui/feedcheck.js | `LEVELS` near line 26 and the sort rank near line 172. |
| Check row mark | extend | crates/api/ui/editor.js | The level mark near line 5006 uses the class `kind warning`. |
| Release checklist | extend | crates/api/ui/feedcheck.js | `releaseChecklist` near line 123. It uses `checkRows` for the button of each row. |
| Release form | reuse | crates/api/ui/app.js | `MakeFeedForm` near line 1435 sends `POST /feeds/new`. `AddFeedSheet` near line 2976 gives the answer to `onAdded`. |
| Import sheet | reuse | crates/api/ui/editor.js | `ImportSheet` near line 1293, opened by the `importing` state of the shell (`app.js` near line 3760). |
| Start from field | reuse | crates/api/ui/app.js | `from` and `showFrom` of `MakeFeedForm` near line 1443. A release with no `from` has no track. |

## Changes

1. `feed_check.rs`: add the level `Stop` to `FindingLevel`, first in the enum, with the doc comment "Review refuses the write until the user repairs it." It serializes as `"stop"`. The finding of `page_refusal` (`no_artist_page` and `no_publisher_page`) gets `FindingLevel::Stop`. Do not change its title, its reason or its repair. Change the tests that assert `Notice` for these two kinds.
2. `feedcheck.js`: `LEVELS` gets `stop: 'Stops the write'`. The sort rank puts `stop` before `warning`. A row of a stop finding has `stop: true`.
3. `editor.js`: the level mark of a row with `stop: true` has the class `kind stop`. Add the CSS rule `.kind.stop { color: var(--error); }` next to the rule of `.kind.warning` for check rows. Use the token `--error` only. Do not add a color value.
4. `releaseChecklist`: add `no_artist_page` to the kinds, and the fifth row `{ label: 'Artist page', finding: first('no_artist_page') }` after "Payment recipient". The button of the row comes from `checkRows`, as for the other rows. Its click must do the same action as **Link a Page** on the Check tab. If the release page does not handle that action, route it through the same handler that the Check tab uses. Do not write a second handler.
5. After **Make the Release**, open the release page and then the sheet "Add tracks" on it in one condition. The form type is `album`, and the body has no `from` (field "Start from"). Such a release has no track and no live item. The answer of `POST /feeds/new` is the feed row and has no `kind_shown`, so do not use it.

   The reset at `app.js` near line 4022 closes the import sheet when the feed changes. Open the sheet after that reset, for example with a pending flag that the shell reads when `draftFeedId` is the new feed. A release with `from`, a playlist and each other form open the page with no sheet.
6. A close of the sheet keeps the release page open. The sheet does not open again by itself.

Do not change the route `POST /feeds/new`.

## Files

- Change: `crates/core/src/feed_check.rs`, `crates/api/ui/feedcheck.js`, `crates/api/ui/editor.js`, `crates/api/ui/app.js` and the CSS file of the check rows.
- Tests: the tests in `crates/core/src/feed_check.rs` and `crates/api/ui-tests/feedcheck.test.mjs`. Add a UI test of the rule of change 5 in a new small test file. Put the rule of change 5 in a small exported function, so that a test can call it.

## Checks

Mechanical:

- Core test: `no_artist_page` and `no_publisher_page` have the level `stop`. The level of each other kind does not change.
- UI test: `checkRows` puts a stop finding first and gives it the level "Stops the write" and `stop: true`.
- UI test: `releaseChecklist` gives five rows in the order of ADR 0219 item 7 and then "Artist page". With only `no_artist_page`, it gives a checklist with the title "This release needs 1 thing" and a cross on "Artist page".
- UI test: the rule of change 5 is true for the type `album` with no `from`. It is false for the type `album` with a `from`, for a playlist and for each other type.
- `cargo test -p studio-core`, `cargo test -p studio-api` and `tools/node --test crates/api/ui-tests/*.test.mjs` pass.
- `bash tools/gate.sh` passes.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- The Check tab of a release with no artist page: the mark "Stops the write" in the error color, first in the list.
- The release page of that release: the checklist row "Artist page" with a cross and **Link a Page**. A click opens the same place as on the Check tab.
- After **Make the Release**: the sheet "Add tracks" over the new release page. After a close: the release page with its checklist.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

- The delegate run opened the sheet "Add a track from an address" (`adding`). The main session changed it to the import sheet "Add tracks" (`importing`).
- The artist page has its own `addedFeed` in `editor.js`, and the delegate run did not change it. The main session added `addTracksNext` to the navigation context. The shell and the artist page call it before they open the new release.
- Walks: `.studio-dev/journeys/v190a.json` and `v190b.json` (mode `fresh`, 1280 px and 390 px), and `v190c.json` (mode `base`, release "Loose Single").

- Defect correction after the commit of this task: the checklist button **Add Tracks** opened the sheet "Add a track from an address". It now opens the import sheet "Add tracks", as the outline does (ADR 0219 item 3). Walk: `.studio-dev/journeys/v191a.json` (mode `fresh`, 1280 px).
