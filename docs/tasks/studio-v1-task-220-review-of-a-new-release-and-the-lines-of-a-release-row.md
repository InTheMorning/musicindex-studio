# Studio v1 Task 220: Review Of A New Release, And The Lines Of A Release Row

Decision: [ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) items 1 to 3, 5, 6, 8 and 10.

Status: Done.

## Problem

The desktop check of 9af319b gave findings 1 to 3 (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of 9af319b"). This task gives the UI. Task 219 gives the API at the same time. Use the API contract of task 219 (`docs/tasks/studio-v1-task-219-one-sided-links-the-artist-name-and-a-move-that-adds-nothing.md`, section "API Contract For Task 220"). Do not wait for it. The UI tests use their own data.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Rows of "Also write" | extend | crates/api/ui/writeresult.js | `alsoRows` near line 32. |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` near line 1996 and the check boxes near line 2323. The discard is near line 2175, its button near line 2379 and the error notice near line 2268. |
| Remove from Collection | reuse | crates/api/ui/app.js | The remove of `ReadOnlyFeed` near line 2202: `/remove-info`, the confirmation, then `DELETE /api/feeds/{id}`. Move it into a shared function when two places need it. |
| Confirmations | reuse | crates/api/ui/confirms.js | `CONFIRMS.removeFeed` near line 15. |
| Check rows | extend | crates/api/ui/feedcheck.js | `checkRows` near line 170. The kinds `copy_other_recipients` and `index_address_differs` have their own action. |
| Check page | extend | crates/api/ui/editor.js | `CheckPage` near line 5093 runs the actions of a row. |
| Level rows | extend | crates/api/ui/shell.js | `releasesList` near line 672 builds the rows. |
| Releases page | extend | crates/api/ui/app.js | `ReleasesPage` near line 3673 and the line of a row near line 3736. |
| UI tests | extend | crates/api/ui-tests/ | The tests of `writeresult`, `feedcheck`, `shell` and the Review models. |

## Changes

1. Item 3: `alsoRows` gives each row with `checked: true` and `locked: true`. The Review sheet shows the check boxes on and disabled, and the text "A new release is written with its pages." below the rows. `rowsToWrite` gives each row.
2. Item 1: for a feed with `draft_state` "new", Review shows **Remove from Collection…** in the place of **Discard the Draft**. It runs the remove of **Feed facts** with the same confirmation, closes Review, and gives the notice with **Undo**. Put the choice of the button in a pure function, so that a test can prove it.
3. Item 2: when the discard fails, the error notice title is "Studio could not discard the draft." The write keeps its own titles.
4. Items 5 and 6: `checkRows` gives the kind `unlisted_by_page` the button "Write {page_title}…" and the action `write-page` with `pageId`. The action calls `POST /api/v1/feeds/{pageId}/links/{release}/list`, then opens the page with its Review open. Use the way that the app opens a feed and its Review now. If no such way exists, open the page and report this in the Result.
5. Item 8: `releasesList` gives each row the lines of `unlisted_by`. In the level of a page, a row whose `unlisted_by` has the id of that page gets the line "⚠ {page title} does not list it yet". `ReleasesPage` shows it as its own line below the kind line.
6. Item 10: in the level of an artist page, `releasesList` gives the line "Artist: {author}" when `author` differs from the title of the artist page. Compare with no change of case and with no space at the two ends. `ReleasesPage` shows it as its own line above the kind line. A row with no `author` gets no line.

## Checks

Mechanical:

- UI test: `alsoRows` gives each row on and locked.
- UI test: the function of change 2 gives **Remove from Collection…** for `draft_state` "new" and **Discard the Draft** for other drafts.
- UI test: the title of a failed discard is "Studio could not discard the draft."
- UI test: `checkRows` gives "Write HeyCitizen…" and the action `write-page` for a finding `unlisted_by_page` with `page_title` "HeyCitizen".
- UI test: `releasesList` in the level of HeyCitizen gives "Artist: Matt Finlay" for a release with that author, no line for author "heycitizen " and no line for no author. It gives "⚠ HeyCitizen does not list it yet" for a row with the page id in `unlisted_by`, and no line in a different level.
- `tools/node --test crates/api/ui-tests/*.test.mjs` passes. A syntax check of each changed `ui/*.js` file as a `.mjs` copy passes (`node --check` misses template errors in a `.js` file).

Visual, by the main session with the screenshot harness: the visual checks of ADR 0234.

Human: none in this task.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments, UI text and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above, `crates/api/ui/style.css` if necessary, and this task file. Do not change Rust files.
- `node` is not on the PATH. Use `tools/node`.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/api/ui/writeresult.js:32`: each page row starts checked and locked. `:58` selects the removal label and the discard error title.
- `crates/api/ui/editor.js:1929`: one function removes a feed with the shared confirmation and undo. Review uses it for a new release at `:2189`.
- `crates/api/ui/editor.js:2349`: Review locks each page check box and shows the new release note. Discard errors use the required title at `:2292`.
- `crates/api/ui/editor.js:5129`: the page write action calls the API route, opens the page and opens Review.
- `crates/api/ui/feedcheck.js:176`: `checkRows` gives the page write action and page ID.
- `crates/api/ui/shell.js:742`: release rows give the artist name and the missing page link line.
- `crates/api/ui/app.js:3727`: the release row shows the artist name above its kind and the missing link below it. `:2208` uses the shared remove function.
- `crates/api/ui-tests/writeresult.test.mjs:56`, `feedcheck.test.mjs:123`, and `shell.test.mjs:721` cover the new models. `feeddelete.test.mjs:155` checks the changed Review button.
- `docs/tasks/studio-v1-task-220-review-of-a-new-release-and-the-lines-of-a-release-row.md`: this Result records the work and checks.

Checks:

- `tools/node --test crates/api/ui-tests/*.test.mjs`: 945 tests pass.
- Syntax checks pass for `.mjs` copies of `writeresult.js`, `feedcheck.js`, `shell.js`, `app.js` and `editor.js`.
- `cargo build -p studio-desktop`: passes.
- The STE checker reports 10 dictionary findings in this Result. They identify technical terms such as `Review`, `action` and `route`.
- The screenshot harness captured the Releases page and the new release form. Its fit checks found no issue at 390 px or 761 px.
- The visual check of the Review rows and the two release row lines remains open. The copied collection has no new release, differing artist name or unlisted page.
- No human check was required. No Rust file was changed for this task.

## Main Session Changes

- `crates/api/ui/editor.js`: each open `DraftToolbar` answered the request to open Review, also the toolbar of a different feed. The toolbar now compares `event.detail.feedId` with its own feed. The action `write-page` sends `feedId: result.page_id`, and it tries for 3 seconds (60 tries) until the page opens Review.

Main session checks, with the fixture `.studio-dev/v-207/data-220` and the steps `s43.json` and `s44.json` (screenshots `out43` and `out44`):

- Review of a new release at 1100 px and 390 px: "Also write Demo Artist" is on and disabled, the text "A new release is written with its pages." is below the rows, and **Remove from Collection…** is in the place of **Discard the Draft**.
- The level of Demo Artist at 1100 px: the row of Demo Single has "Artist: Matt Finlay" above the kind line and "⚠ Demo Artist does not list it yet" below it.
- **Issues** of Demo Single has "Demo Artist does not list this release." with **Write Demo Artist…**. The button opens Review of Demo Artist with "Added: Demo Single".
- `$edges()` gives no finding at 1100 px and at 390 px.

Open:

- At 390 px the level is the left panel. Its release rows show only the title, so the two lines of items 8 and 10 do not show on a phone. **Issues** of the release still shows the finding.
