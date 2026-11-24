# Studio v1 Task 225: An Artist Page Or A Label Page Moves

Decision: [ADR 0240](../archive/adr/0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) items 20 to 25.

Status: Done. The main session checked the visual criteria with screenshots on 2026-11-21, in the dark theme at 1280 px. The sheet "Move Test Artist" opened from the row menu **⋯** and from the head button **Move To…**. It showed the change lines "+ Test Label" and "− Test Label". After the move, the artist page was below the label page. The row menu of a release opened the sheet of that release.

## Problem

Task 224 gives **Move To…** only to a release. ADR 0240 items 20 to 25 also move an artist page and a label page below a different label page or hosting page. The move links each release of the page, and moves the kept link.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Move route | extend | crates/api/src/routes.rs | The route `POST /api/v1/feeds/{id}/move-to` of task 224 also accepts an artist page or a label page as `id`. |
| Move sheet model | extend | crates/api/ui/movepages.js | The pure model of task 224 also gives the menus of item 21 and the change lines of item 22. |
| Move sheet | extend | crates/api/ui/editor.js | The sheet of task 224 also shows the sheet of a page. |
| Publisher link | reuse | crates/core/src/publisher.rs | `publisher::link` and `publisher::unlink` change the two drafts. |
| Kept levels | reuse | crates/core/src/publisher.rs | `catalogs_with_kept` and the `kept_levels` table of the collection give the pages above a page and the kept links of item 24. |
| Level and outline rows | extend | crates/api/ui/shell.js | The rows of `feedLevel` and `outlineItems` get the menu **⋯** of item 20. |
| Row menu | new | crates/api/ui/app.js | Searched crates/api/ui/*.js for a menu on a panel row: no row has a menu. The **＋ Add** menu of the Releases head is the pattern for the look and the keys. |

## Changes

1. Read ADR 0240 items 9 to 25, ADR 0215 item 2 and ADR 0232 items 9 to 12 first. Read the diff of task 224 in the log.
2. The route: when `id` is an artist page, the body has `label_id` and `hosting_id`, and no `artist_id`. When `id` is a label page, the body has only `hosting_id`. Refuse a hosting page, a page with a different role, and a body with a key of a different role.
3. For a page, the route finds the releases of the collection that link to the page. For each release, it does item 16 and item 17 only for the roles of the sheet. It keeps the link to the moved page, and on a label page it keeps the artist links.
4. The route removes each kept link from the moved page to a page of the roles of the sheet. It adds a kept link to each selected page. Check each page and each release before the first change. When one link fails, change no draft and no kept link.
5. The model gives the menus of item 21 and the start values from the pages above the moved page. It also gives the change lines of item 22: "Changes {N} releases:", then "+ {page}" or "− {page}" for each page that the releases get or lose.
6. The sheet "Move {name}": the menus, the change lines, **Cancel** and **Move**. After **Move**, open the level of the new upper page.
7. **Move To…** in the head of an artist page and of a label page.
8. The menu **⋯** on each release row, artist row and label row of a level and of an outline, with the accessible name "More for {name}". Its one value is **Move To…**. The menu uses the keys of the **＋ Add** menu. The button does not open the row.
9. Tests:
   - Node: the model for an artist page and a label page, the start values, and the change lines. The rows of the level and the outline give the menu of item 20 on release, artist and label rows, and not on a hosting row.
   - Rust in `crates/api/src/tests.rs`: an artist page with two releases moves from label A to label B: each release links B and not A, and keeps the artist link. A label page moves to a hosting page and keeps the artist links. An artist page with no releases moves through the kept link. A hosting page as `id` is refused. A failed link changes no draft and no kept link.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0240 and an item. A test failure message names ADR 0240, the item and the function to fix.
- Follow the `rust-dev` skill and the lints of each crate.
- Button labels use title case. The UI text does not use the word "feed".
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The Node tests of step 9 pass.
- The Rust tests of step 9 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- The sheet "Move {name}" of an artist page opens from its head and from the menu **⋯** of its panel row, and shows the change lines.
- After a move, the artist page shows on the level of the new label page.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
