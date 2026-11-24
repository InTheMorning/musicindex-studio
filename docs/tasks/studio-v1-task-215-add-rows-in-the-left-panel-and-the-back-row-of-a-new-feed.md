# Studio v1 Task 215: Add Rows In The Left Panel And The Back Row Of A New Feed

Decision: [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) items 1 to 8.

Status: Done.

## Problem

Findings 1 and 2 of the [desktop check of the commits up to 2d4fb25](../reviews/2026-11-20-rc29-human-check.md#desktop-check-of-the-commits-up-to-2d4fb25).

- The groups of a level and of the outline of a publisher feed have no add row. `feedLevel` and `outlineItems` in `crates/api/ui/shell.js` give no add item. A group with no rows does not show.
- A new feed opens with `go({ name: 'feed', id: f.id })` and no `from`. This is in `addedFeed` of the shell, `crates/api/ui/app.js` near line 4160, and in `addedFeed` of `PublisherPage`, `crates/api/ui/editor.js` near line 4238. Thus the back row of the new page is **‹ Releases**.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Panel model | extend | crates/api/ui/shell.js | `feedLevel` near line 427, `headed` near line 363, `outlineItems` near line 173, `openFeedView`, `upView`. |
| Level panel | extend | crates/api/ui/app.js | `Level` near line 3379 and `LevelPage`. |
| Outline panel | extend | crates/api/ui/app.js | `Outline` near line 3420. Its add row `node add` for **＋ Add tracks** is the pattern for the new rows. |
| Make form sheet | extend | crates/api/ui/app.js | `AddFeedRowSheet` near line 3836 and the form near line 1472. `start` gives `role` and `publisher_id`. Add `below_id`. |
| Shell new feed | extend | crates/api/ui/app.js | `addedFeed` near line 4160. |
| Publisher page new feed | extend | crates/api/ui/editor.js | `addedFeed` near line 4238 of `PublisherPage`. |
| Start of a new release | reuse | crates/api/ui/welcome.js | `releaseAdd` near line 139. |
| UI tests | extend | crates/api/ui-tests/shell.test.mjs | The tests of `feedLevel` near line 630 and of `outlineItems` near line 695. |

## Changes

1. Items 2 and 6: `feedLevel` gives the groups of item 2 of ADR 0232 for the rank of the level feed. A group of the rank list shows also with no rows. Each group of the rank list ends with an add item: `{ add: 'artist' | 'label' | 'release', label: 'Add artist' | 'Add label' | 'Add release', start }`. The group Releases is a group of the model too, with its heading. A group of another role heading shows only with rows and has no add item. Give the rank from the first role, as `headingIndex` does.
2. Item 4: the start of **＋ Add release** is `releaseAdd(feed, title).start` for an artist page that `releaseAdd` accepts, else `{}`. The start of **＋ Add artist** is `{ role: 'artist', below_id: level feed ID }`. The start of **＋ Add label** is `{ role: 'label', below_id: level feed ID }`.
3. Item 3: `outlineItems` of a publisher feed always gives the heading Releases, then its releases, then the add item **Add release** with the start of change 2.
4. Items 1 and 7: `Level` and `Outline` show each add item as a row with `＋` and the label, at the end of its group, in the keyboard sequence of the tree. The row uses the class pattern of **＋ Add tracks**. `LevelPage` and `OutlinePage` show the same rows at narrow width.
5. The click on an add row opens `AddFeedRowSheet` with `open` `album` for a release and `publisher` for an artist or a label, and with the start of change 2. Keep the sheet in the shell, so that the left panel and the narrow pages can open it. With no connected file host, the click opens the page that connects one, as `AddMenu` does.
6. The form of a publisher page sends `below_id` from `start` in the body of `/feeds/new`. Task 214 gives the field in the API.
7. Item 8: add a pure function in `crates/api/ui/shell.js`, for example `newFeedView(id, from)`, that gives the view of a new feed. The shell `addedFeed` and the `PublisherPage` `addedFeed` take a `from`:
   - The add row of a level, and the add menu of the page Releases of a level group: `{ group: level feed ID, trail }`.
   - The add row of an outline of a publisher feed, and the add rows and add menu of `PublisherPage`: the level of that feed, as `outlineItems` gives to its release rows.
   - The top page Releases, "Not linked", "Other feeds", Welcome and **Add One You Have**: no `from`.
8. Keep the add row **＋ Add a Release** in the body of the artist page (ADR 0230 item 5).

## Checks

Mechanical:

- Node test: `feedLevel` of a hosting feed, of a label and of an artist with no children and no releases gives the groups and add items of ADR 0232 item 2.
- Node test: a label with a child of the role curator gives the heading Curators with no add item.
- Node test: `outlineItems` of an artist with no release gives the heading Releases and the add item with the start parent of the artist. A label gives no start parent.
- Node test: `newFeedView` for a level, for a publisher page that opened from a level, and for the top page gives the `from` of change 7. Then `outlineItems` of the new feed gives the back row with the name of that level.
- `tools/node --check` on a `.mjs` copy of each changed file of `crates/api/ui/`.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual (main session):

- drive-ui at 1100 px and 390 px: the level of a label with no artist shows Artists with **＋ Add artist**. The artist page shows **＋ Add release** at the end of the group Releases. A new release made from the level of an artist shows the back row with the name of the artist.

Human: none beyond the next desktop check.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only `crates/api/ui/`, `crates/api/ui-tests/` and this task file. Task 214 changes the Rust files at the same time.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/api/ui/shell.js:173` adds the Releases group and its add row to publisher outlines. `shell.js:430` gives each level its ranked groups and add rows.
- `crates/api/ui/shell.js:477` adds `newFeedView` for source-aware navigation.
- `crates/api/ui/app.js:1520` sends `below_id` from the publisher start values. `app.js:3381` adds rows to wide and narrow views.
- `crates/api/ui/app.js:4170` opens a new feed from its source level.
- `crates/api/ui/editor.js:4056` passes the publisher page level to the new feed view.
- `crates/api/ui-tests/shell.test.mjs` checks empty groups, add starts, outline rows and new feed views.
- Syntax checks of the three changed UI files passed. The Node tests passed all 940 tests.
- `bash tools/gate.sh` passed.
- The STE checker reports 15 findings in the task text.
- The visual check at 1100 px and 390 px was not run.

Main session changes after the review:

- `releaseStart` in `crates/api/ui/shell.js` gives the start values of a new release. `releaseAdd` in `welcome.js`, `outlineItems` and `feedLevel` use it. Before, the condition was in three places.
- `pageLevel` in `shell.js` gives the level of a publisher page. `outlineItems` and the `from` of `PublisherPage` in `editor.js` use it. Before, `editor.js` had a copy.
- The rank of `headingIndex` agrees with `publisher::rank`: the two use the role sequence artist, label, hosting, and the first role of that sequence that the feed has.
- Checks by the main session: syntax checks of a `.mjs` copy of `shell.js`, `app.js`, `editor.js` and `welcome.js`. The Node tests (940), `cargo test -p studio-core`, `cargo test -p studio-api`, clippy and `bash tools/gate.sh` pass.
- Visual check by the main session (drive-ui, `.studio-dev/v-207/out32` to `out34`): the level of a new label shows Artists with **＋ Add artist** and Releases with **＋ Add release**. The sheet starts with the role Artist.
- Visual check, continued: the new artist opens with **‹ Demo Label** and shows below the label. **＋ Add release** on the artist page starts with the artist as parent. The new release opens with **‹ Demo Artist**. At 390 px the label level and the artist page show the same rows. `$edges()` gives no finding.
