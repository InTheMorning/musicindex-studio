# Studio v1 Task 188: The Narrow Root Page And One Tap To A Release

Decision: [ADR 0223](../adr/0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 7 and 8.

Status: Done.

## Problem

Observations O14 and O15 of the [user journeys](../research/2026-11-18-user-journeys/observations.md). On a phone, People, File hosts and Settings are at the end of a long list. A release needs two taps to open: the first tap opens the outline page.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Left panel model | reuse | crates/api/ui/shell.js | `leftPanel` near line 270 gives `top` and `bottom`. The model does not change. |
| Root page | extend | crates/api/ui/app.js | `RootPage` near line 3387 and `LeftPanelFooter` near line 3371. |
| Feed view at a width | extend | crates/api/ui/shell.js | `openFeedView` near line 839, `narrowStack`, `widthNav` and `navAtWidth` near line 852. |
| Editor bar | extend | crates/api/ui/app.js | `Bar` near line 1983. `FeedDetail` renders the feed page. |

## Changes

1. `app.js`: `RootPage` shows, in this sequence: the header, the row Releases, the rows People, File hosts and Settings, the feed groups, then the collection mark and "Lock Studio" or "Unlock Studio". The rows People, File hosts and Settings are the rows of `panel.bottom` with these IDs. Use the row markup of `LeftPanelFooter` for each row. The wide left panel does not change.
2. `shell.js`: `openFeedView(id, narrow, from)` gives `{ name: 'feed', id }` at each width. The narrow stack of a feed view is the root page, Releases (or the level of `from`), then the feed view. The narrow stack of an outline view is that stack, then the outline view.
3. `app.js`: at a narrow width, the bar of the feed page has the button "Sections" (class `button`) at its trailing end, before the draft toolbar. It opens `{ name: 'outline', id }` as a new history entry. Back from the outline page opens the feed page. A selection on the outline page opens its part, and Back from the part opens the outline page.
4. Keep each other route of the outline page. The wide layout does not change.

## Files

- Change: `crates/api/ui/app.js`, `crates/api/ui/shell.js`.
- Change tests: each test in `crates/api/ui-tests/` that asserts the old narrow stack. Give the new stack of change 2.
- New test: `crates/api/ui-tests/narrow-root.test.mjs`.

## Checks

Mechanical:

- `narrow-root.test.mjs`:
    - `openFeedView(5, true)` and `openFeedView(5, false)` give `{ name: 'feed', id: 5 }`.
    - `widthNav({ name: 'feed', id: 5 }, true)` gives the entries root, releases, feed.
    - `widthNav({ name: 'outline', id: 5 }, true)` gives the entries root, releases, feed, outline.
    - The source of `RootPage` renders the rows People, File hosts and Settings before `destinationRows`.
    - The source of the feed page bar has "Sections".
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 390 px, and at 1280 px for no change:

- The root page with the fixed rows at the top.
- A release page after one tap, with "Sections". The outline page after "Sections".

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
