# Studio v1 Task 187: Releases Has A Search Field

Decision: [ADR 0223](../adr/0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 5, 6 and 9.

Status: Done.

## Problem

Observation O13 of the [user journeys](../research/2026-11-18-user-journeys/observations.md). Releases has sort, view and add controls, and no search. A user with many releases must scroll or use the letter bar.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Search field | extend | crates/api/ui/peoplepages.js | The People search near line 192 uses `<div class="field search-field">`. Move it into one exported component `SearchField({ label, value, onInput })` in `app.js`, and use it on People and on Releases. |
| Release list model | extend | crates/api/ui/shell.js | `releasesList` near line 600 gives the rows and the letters. Add the parameter `query`. |
| Releases page | extend | crates/api/ui/app.js | `ReleasesPage` near line 3448. |

## Changes

1. `app.js`: add the exported component `SearchField({ label, value, onInput })`. It gives the markup of the People search: `<div class="field search-field"><div class="field-body"><input type="search" ...></div></div>`, with `aria-label` and `placeholder` from `label`.
2. `peoplepages.js`: the section `search` uses `SearchField` with the label "Search people, addresses and keys". The page must look the same as before.
3. `shell.js`: `releasesList` gets `query`. When `query.trim()` is not empty, a row stays only when its title, or the title of one of its publisher feeds (`publisher_ids`), has the text. The match ignores letter case. With a query, `letters` is `null`. The sort and the view do not change.
4. `app.js`: `ReleasesPage` shows `SearchField` with the label "Search releases" between the head and the list. It shows on Releases and on each feed group, when the list before the filter has rows. Keep the text in the state of the page. Do not put it in the address.
5. `app.js`: when the query gives no row, the page shows `<p class="hint">No release has “{text}”.</p>` in the place of the list.

## Files

- Change: `crates/api/ui/app.js`, `crates/api/ui/shell.js`, `crates/api/ui/peoplepages.js`.
- New test: `crates/api/ui-tests/release-search.test.mjs`.

## Checks

Mechanical:

- `release-search.test.mjs`:
    - `releasesList` with `query` "night" keeps a release "Night Drive" and removes "Album 2".
    - `query` "artist x" keeps each release of the publisher feed "Artist X".
    - "NIGHT" gives the same rows as "night".
    - A query with no match gives no row and `letters` `null`.
    - The source of `peoplepages.js` and of `app.js` (`ReleasesPage`) each uses `<${SearchField}`. `peoplepages.js` has no `class="field search-field"` of its own.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- Releases with a search that has rows, and with a search that has no row.
- People with its search field, with no change.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
