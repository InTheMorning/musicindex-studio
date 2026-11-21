# Studio v1 Task 224: Hosting Is A First Role, And A Stray Release Moves To Its Pages

Decision: [ADR 0240](../adr/0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) items 1 to 19. Task 225 does items 20 to 25.

Status: Open.

## Problem

The page of a hosting feed shows only the group Releases and **＋ Add release**. The add menu and the Welcome page have no Hosting. A release in "Not linked" has no clear path to an artist page, a label page or a hosting page. ADR 0240 gives the new behavior.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Add menu model | extend | crates/api/ui/addfeeds.js | `addMenuEntries` gets Hosting and the values of a page head (ADR 0240 items 1 and 4). |
| Add menu | extend | crates/api/ui/app.js | `AddMenu` and the Releases head give the role of the head page to the model. |
| Welcome model | extend | crates/api/ui/welcome.js | `WELCOME.rows`, the steps and `FIRST_STEPS` get the hosting path (ADR 0240 items 2 and 3). |
| Welcome page and start sheet | extend | crates/api/ui/app.js | `WelcomePage` and `StartSheet` get the path `hosting`. |
| First steps of a new page | extend | crates/api/ui/editor.js | The rows of a new label page (`first === 'label'`) get a hosting variant. |
| Level and outline | extend | crates/api/ui/shell.js | `feedLevel` and `outlineItems` give the groups of ADR 0240 items 5 to 7. |
| Link section of a release | extend | crates/api/ui/editor.js | `ArtistsAndLabels` gets **Move To…** and the line of item 11. |
| Release list of Not linked | extend | crates/api/ui/app.js | The rows of the group `none` get **Move To…**. |
| Move sheet | new | crates/api/ui/editor.js | Searched crates/api/ui/*.js for a sheet with three page menus. `ParentSheet` (app.js) has one artist menu with no "None". `ArtistsAndLabels` links one page at a time. Neither fits. Put the pure model in crates/api/ui/addfeeds.js or a new module, with Node tests. |
| Publisher form | reuse | crates/api/ui/app.js | `AddFeedRowSheet` with `open: 'publisher'` and a start role makes the new page of item 14. |
| Publisher link | reuse | crates/core/src/publisher.rs | `publisher::link` and `publisher::unlink` change the two drafts. |
| Levels rule | reuse | crates/core/src/publisher.rs | `catalogs_with_kept` gives the pages above an artist page for item 15. The feed list already gives `parent_ids`. |
| Move route | new | crates/api/src/routes.rs | Searched routes.rs: `add_link` and `remove_link` change one link each and have no check of all links before the first change. Add one route for item 18. |

## Changes

1. Read these records first: ADR 0240, ADR 0232, ADR 0230 items 1 to 4, ADR 0221 items 2 to 9 and ADR 0239 item 4.
2. `addfeeds.js`, `addMenuEntries`: add `{ id: 'hosting', label: 'Hosting', open: 'publisher', start: { role: 'hosting' } }` after Label. Add a second parameter, the head page (null on the page Releases). On a label page or a hosting page, leave out Release and each role with the same rank or a higher rank. Give the remaining role entries `below_id` of the head page. Keep the ADR 0230 tests that still hold, and change the ones that ADR 0240 item 4 changes.
3. `app.js`: the Releases head passes the head feed to `addMenuEntries`.
4. `welcome.js` and `app.js`: add the row "I'm hosting for other artists" with the line "Start with your hosting page. The artists and labels that you host go on it." Add the path `hosting` to the start sheet, with the role `hosting`. Its steps are "Your name", "Where it lives" and "Your first artist". The page of the new hosting feed shows the rows "Add an artist", "Add a label" and "Find an artist that is online".
5. `shell.js`, `feedLevel`: a hosting feed gives Labels and Artists with add rows and no Releases group. A label feed gives Artists with its add row, then Releases with no add row. An artist feed gives Releases with **＋ Add release**. Represent "no add row" as `releaseAdd: null`, and make `LevelPanel` in `app.js` skip a null add row and a missing Releases group.
6. `shell.js`, `outlineItems`: a publisher feed gives the same groups and add rows as its level. Reuse the code of step 5. Do not copy it.
7. New route `POST /api/v1/feeds/{id}/move-to` with the body `{ "artist_id": N, "label_id": N|null, "hosting_id": N|null }`. `id` is the release. Check each page before the first change: medium, role, connector and name, as `publisher::link` does. Then link each selected page. Then unlink each other page of the collection with the role artist, label or hosting. Return the changed feeds, as `add_link` does (`changed_json`). Call `remove_shared_kept_levels` after the links.
8. The model of the sheet: a pure function that gives the three menus from the feed list and the release. Artist has no "None". Label and Hosting start with "None". Each menu ends with "New {Artist, Label or Hosting} Page…". The start values follow ADR 0240 item 15.
9. The sheet "Move {title}" in `editor.js`, with the button **Move** and **Cancel**. "New … Page…" opens `AddFeedRowSheet` with `open: 'publisher'` and the role. After the make, the menu selects the new page.
10. **Move To…** on each row of the group "Not linked" and in the section "Linked artist and label pages" of a release. With no linked artist page, the section shows "A release needs an artist page."
11. Tests:
    - Node: `addMenuEntries` for the page Releases, an artist page, a label page and a hosting page. The Welcome row and steps. The level and the outline of a hosting, a label and an artist feed. The model of the sheet, with the start values of item 15.
    - Rust in `crates/api/src/tests.rs`: the move route links three pages; it unlinks the other artist page; it keeps a link to a page that is not in the collection; it changes no draft when a page is not writable; it refuses a label page in `artist_id`.

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

- The Node tests of step 11 pass.
- The Rust tests of step 11 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots):

- The panel of a hosting page shows Labels and Artists with their add rows, and no Releases group.
- The panel of a label page shows Artists with its add row, and Releases with no add row.
- The Welcome page shows three rows in its first group.
- The sheet "Move {title}" opens from "Not linked" and from a release page, and a move puts the release on the level of its artist.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.
