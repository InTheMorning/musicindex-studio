# Studio v1 Task 230: The Move Review And The Gather Sheet

Decision: [ADR 0243](../adr/0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) items 3 to 10.

Status: Done. ChatGPT built it. The main session read the diff, made the corrections below, and took the screenshots of the visual criteria on a copy of the tutorial data. The shots are dark, at 1280 px and at 390 px. The gather sheet is also in the light theme. It also ran `tools/gate.sh`.

## Problem

Task 229 makes a change set for each move, gives the names of the feeds in a refusal, and adds the route `POST /api/v1/feeds/{id}/gather`. The UI still closes the move sheet with no review, shows "It is read-only" with no name, and has no sheet that moves many pages below a hosting page or a label page.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Move sheet | extend | crates/api/ui/editor.js | `MoveSheet` opens the review after **Move**, and shows the refusals of items 5 and 6. |
| Move sheet model | extend | crates/api/ui/movepages.js | `pageMoveLines` gives no heading when it has no change lines. The model of the sheet of item 8 goes here. |
| Change set review | reuse | crates/api/ui/app.js | `ChangeSetPage` shows the review, **Write {N} Feeds** and **Discard**. |
| Change row | extend | crates/api/ui/people.js | `changeRow`, `changeSetView` and `afterDiscard` know the new move operation. |
| Level add rows | extend | crates/api/ui/shell.js | `feedLevel` and `outlineItems` give the add rows. A hosting level and a label level give a menu. |
| Add menu | reuse | crates/api/ui/app.js | The **＋ Add** menu and the row menu **⋯** of task 225 give the look and the keys of a menu. |
| Error notice | reuse | crates/api/ui/app.js | `ErrorNotice` and `ERROR_STEPS` in crates/api/ui/errors.js. |

## Changes

1. Read ADR 0243, ADR 0240 items 12 to 25 and ADR 0232 items 1 to 7 first. Read the diff of task 229 (`git log -1 -p` of its commit) for the response fields and the operation name.
2. After **Move** in `MoveSheet`, close the sheet and open the review of the change set `change_set_id` (ADR 0243 item 3). Remove the old `onMoved` navigation to the level, or keep it only for a response with no change set.
3. The title of the review and of the row in "Changes to write" is "Move {name} to {page}", or "Move {N} artists to {page}" / "Move {N} labels to {page}" (items 3 and 4). Put this text in `changeTitle` or `changeRow` of `people.js`. `changeSetView` opens the review in the area Releases for a move of pages. `afterDiscard` opens the level of the target page.
4. A refusal with code `read_only` shows "Studio cannot change {title}. It is read-only." for each entry of `details.feeds`, and the button **Add a File Host**. The button closes the sheet, opens File hosts and opens the sheet **＋ Add a File Host** (item 6).
5. A refusal with code `in_change_set` shows "{title} has changes to write. Write or discard them first." for each entry, with a button **Open Changes** that opens that change set (item 5). Add the code to `ERROR_STEPS` in `errors.js`.
6. `pageMoveLines`: with no change line, give no "Changes {N} releases:" heading. Give the line "Select a label or a hosting page." Finding 5 of the [tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md). Technical choice (main session): this text.
7. Add rows (item 7): on a hosting level, **＋ Add artist** opens a menu with "New Artist Page…" and "Move Artists Here…". **＋ Add label** opens a menu with "New Label Page…" and "Move Labels Here…". On a label level, **＋ Add artist** opens the artist menu. A "New … Page…" value does what the add row does today. The add row of an artist level does not change. The same applies to the outline and at 760 px or less.
8. The sheet "Move artists to {page}" or "Move labels to {page}" (items 8 to 10). Put its model in `movepages.js` as a pure function: from the feed list of the collection and the level page, give the rows (each writable publisher page with the role of the sheet or with no role, not below the level page now), the start value of each check box (selected for "No role", else not selected), the button text **Move {N} Artists** / **Move {N} Labels**, the state "not available" with no selection, and the note "A page with no role gets the role Artist." / "… Label." when a selected row has no role. Each row shows the name, the number of releases and the role or "No role".
9. **Move** in that sheet sends `POST /api/v1/feeds/{id}/gather` with the role and the selected IDs. Then it opens the review as step 2. A refusal shows as steps 4 and 5.
10. Node tests in `crates/api/ui-tests/`:
    - `movepages.test.mjs`: the model of step 8 (rows, start values, button text, not available with no selection, the note). `pageMoveLines` with no change gives the line of step 6 and no heading.
    - `shell.test.mjs`: a hosting level gives the two menus of step 7, a label level gives the artist menu, an artist level does not change.
    - `people.test.mjs`: `changeRow` gives the titles of step 3 for a move of one page and of three pages. `changeSetView` and `afterDiscard` give the views of step 3.
    - `errors.test.mjs`: `in_change_set` has its step.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Do not change Rust files, except when the response of task 229 lacks a field that step 3 needs. Then stop and report.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0243 and an item. A test failure message names ADR 0243, the item and the function to fix.
- Button labels use title case. The UI text does not use the word "feed", except the existing **Write {N} Feeds**.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The Node tests of step 10 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots, in the dark theme at 1280 px and at 390 px):

- After **Move** of an artist page, the review "Move {name} to {page}" opens with **Write {N} Feeds** and **Discard**.
- The menu of **＋ Add artist** on a hosting level, and the sheet "Move artists to {page}".
- The refusal with a read-only release, with the name and **Add a File Host**.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

## Corrections Of The Main Session

- The entries of the add menu kept the page role in `role`. `MenuControl` gives `entry.role` to the button as its ARIA role, so the entries had `role="artist"`. The field is now `pageRole`, and a test in `shell.test.mjs` stops a `role` field.
- The add row with a menu was centered. It now has the "＋" column and the label of the other add rows. It is not a tree item and does not take a row number, because its button has `aria-expanded` (musicindex ADR 0061 R-LISTKEY-2 item 1). The outline of an open feed also shows the menu.
- The refusal had a frame around a second card. It now has the shape of `ErrorNotice`, with one line for each feed and one **Open Changes** for each change set.
- The gather sheet copied the check row. It now uses `FeedCheckRow` in `group rows`, as the Use sheet does.
- The core recorded each feed of the move as changed, also an artist page with no change. `record_page_move_change_set` now leaves out a feed whose draft did not change, and removes a draft that the move started with no change. Test `gather_leaves_an_unchanged_page_out_of_the_change_set`. On the tutorial data, the gather of two artist pages gives **Write 14 Feeds** and not 16.
- Operator choice (main session, in ADR 0243 item 9): one page gives **Move 1 Artist**.
- Known limit: Discard of a move opens the page whose title is the target name of the change set. With two pages of the same title, it opens one of them.
