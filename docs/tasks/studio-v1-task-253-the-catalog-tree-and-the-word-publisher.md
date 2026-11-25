# Studio v1 Task 253: The Catalog Tree And The Word "Publisher"

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0253](../adr/0253-the-roles-of-a-feed-and-of-each-link.md) items 7 to 14, 16 and 18. [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 1 to 17 and 43 to 46.
Prototype: `.studio-dev/design-brief/prototype/structure.html`. Git ignores it. Its words "Hosting" and "Catalog" follow the ADRs, not the prototype, where the two are different.

Status: Packets 253a to 253c are done. Packet 253d is not started.

Operator choice (main session, packet 253b): a feed with no level role shows the number of its releases. A feed with only the role `hosting` shows the number of the pages below it.

## Goal

The left panel is the Catalog tree. The role `hosting` has the screen word "Publisher". The page of a publisher feed shows its head, its path and its groups.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Role words and type label | extend | crates/api/ui/shell.js | `roleWord`, `mediumKind` and `feedKind`. |
| Role menu | extend | crates/api/ui/shell.js | `roleChoice` and `toggleRole`. |
| Word guard | extend | crates/api/ui-tests/words.test.mjs | The message of musicindex ADR 0097 R-WORD-15 item 6. |
| Role headings | extend | crates/api/ui/shell.js | `HEADINGS` and `headingIndex`. |
| Panel model | extend | crates/api/ui/shell.js | `leftPanel`, `groupDestinations`, `feedLevel`, `pageGroups`, `narrowStack`, `openFeedView`, `newFeedView`. |
| Panel rendering | extend | crates/api/ui/app.js | `LeftPanel`, `DestinationRow`, `LevelRow`, `Level`, `Outline`. |
| Row marks | reuse | crates/api/ui/outsidecheck.js | `panelSet` gives the set that a row covers. |
| Publisher page | extend | crates/api/ui/editor.js | `PublisherPage`. |
| Add menu | extend | crates/api/ui/addfeeds.js | `addMenuEntries`. |
| Welcome texts | extend | crates/api/ui/welcome.js | `WELCOME`, `welcomeSteps` and `FIRST_STEPS`. |
| Node tests | extend | crates/api/ui-tests/shell.test.mjs | The level tests change to the tree. |

## Constraints

- The UI reads `child_ids`, `parent_ids`, `release_ids`, `publisher_ids` and `roles_linked` from the feed list. It has no copy of the rule "below" (ADR 0065, ADR 0254 item 4).
- A test that asserts a rule of ADR 0215, ADR 0232 or ADR 0240 changes or goes in the same packet as the new code (ADR 0254 "How This Decision Is Checked").
- Each button has an accessible name. A row button gives `aria-expanded` and the current row gives `aria-current`.
- Use the tokens of `crates/api/ui/styles.css`. Compare the other two apps before a new mark or token (HitIt and musicindex-live-publisher).
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A UI module passes `node --check` on a `.mjs` copy.
- The agent does not commit and does not stage. The main session reads the diff, takes the screenshots, runs the gate and commits.

## Files To Read First

- ADR 0253 items 7 to 18, ADR 0254 items 1 to 17 and 43 to 46.
- The prototype `.studio-dev/design-brief/prototype/structure.html`.
- `crates/api/ui/shell.js` near lines 17 to 80, 195, 257, 319 to 600, 963 and 998.
- `crates/api/ui/app.js` near lines 2550 to 2610, 3568 to 3730, 3934 and 4043.
- `crates/api/ui/editor.js`: `PublisherPage` (near line 4326).
- `crates/api/ui/addfeeds.js`, `crates/api/ui/welcome.js`, `crates/api/ui/outsidecheck.js` (`panelSet`).
- `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui-tests/words.test.mjs`, `crates/api/ui-tests/welcome.test.mjs`, `crates/api/ui-tests/addfeeds.test.mjs`.

## Packets

### 253a: The Word "Publisher" (UI)

1. `roleWord` gives "Publisher" for `hosting`. The type label of `label hosting` is "Label · Publisher" (ADR 0253 items 7 and 11).
2. The dot is filled when `roles_linked` gives the first role. It is also filled when the set is confirmed and the feed has no link. In each other condition, it is hollow (ADR 0253 item 8).
3. The Role menu has "Artist", "Label", "Publisher" and "Custom" (item 10). The make sheet has "Artist", "Label" and "Publisher".
4. The headings are "Publishers", "Labels", "Curators", "Artists", then "No role" (item 16).
5. The Role section says "The feed and its releases give the same role." when `roles_linked` gives the first role (item 18).
5a. ADR 0257 item 10a: after a change of the Role field, when the response has `kept_role_releases`, the Role section says "{N} releases keep their {role} page." for each role, with the names in a fold.
6. The menu **＋ Add** of Releases has Release, Artist, Label, Publisher, Show, Playlist (ADR 0254 item 44). The Welcome row is "I publish music for other artists" with its line (ADR 0253 item 12, ADR 0254 item 45). The page of a new label has the rows "Add an artist" and "Add a release", and no row "Find an artist that is online" (ADR 0254 item 46).
7. The word guard of `words.test.mjs` permits "Publisher" only as the word of `hosting` and in "published by", with the message of musicindex ADR 0097 R-WORD-15 item 6 (ADR 0253 items 13 and 14). Its failure message names ADR 0253 item 13 and the fix.
8. Node tests: the text of item 5a. `words.test.mjs`, `welcome.test.mjs`, `addfeeds.test.mjs` and the Role menu tests of `shell.test.mjs` change to these texts.

### 253b: The Tree Model (UI)

1. The panel model gives, in this sequence: Releases, the tree, "No artist yet" and "Other feeds" (ADR 0254 item 2). "No artist yet" holds each release with no link to an artist page of the collection.
2. The tree gives the top rows (no `parent_ids`) in the headings of packet 253a, A to Z in each (item 3). Below an open row, it gives the feeds of `child_ids`, in the rank of ADR 0253 item 15, then A to Z (item 5). A feed below two feeds shows below each one.
3. Each row gives the art, the dot and the title. It gives the count of releases, the count of pages below, or the two counts (item 6). It also gives the marks of `panelSet` (item 6a) and the second line "Also on {names}" or "Also published by {names}" (item 8).
4. The model keeps the set of open rows and the current row. The outline of a release or a playlist starts with the row "‹ Catalog". A publisher feed has no outline (item 7).
5. The path of a page (item 12) and the page above (item 12a): the tree row that the user selected, or else the feed directly above with the highest rank, then A to Z.
6. A new publisher feed: its row is current and each row above it is open (item 43).
7. At 760 px or less, the tree is a page of the narrow stack, and "‹ Catalog" goes back to it (item 10).
8. Node tests in `shell.test.mjs`: the top rows in their headings, the rows below an open row, a feed below two feeds, the counts and the second line. Also "No artist yet", the path and the page above, a new feed, and the narrow stack. The tests of the levels of ADR 0215 and of the outline of a publisher feed go.

### 253c: The Tree (UI Rendering)

1. `LeftPanel` has the heading and the accessible name "Catalog". It renders the model of packet 253b. Each depth adds an indent of 16 px.
2. A row with rows below it has a button before its title, named "Open {name}" or "Close {name}", with `aria-expanded`. A selection opens the page and keeps the open rows.
3. Each row ends with the button **⋯** named "More for {name}". Task 254 fills its menu. Until then, it has "Open".
4. The level views (`Level`, `LevelPage`) and their styles go when no caller uses them.

### 253d: The Page Of A Publisher Feed (UI)

1. The head: art, title, type label, the place in music words from the core or the feed list (for example "On Northwind Records, published by Riverside"), the state of the feed of the page only (item 11a), the button **⋯** and "Open the feed" (item 11).
2. The path above the head (item 12): each name opens its page. "Top level" when no feed is above.
3. The groups of item 13 for each level role of the feed, from the highest rank (item 6b), with the add rows. The add rows of the Add sheet show and do nothing until task 254. **＋ Add release** opens the form of a release with that artist (item 15).
4. The group Releases has the sorts "Feed order", "Newest" and "A to Z" (item 13a).
5. The empty texts of item 14.
6. The menu **＋ Add** of a page head (item 17).
7. Node tests in `pages.test.mjs` or `shell.test.mjs`: the groups of each rank, a feed `artist hosting`, the empty texts and the menu of a page head.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes. `node --test crates/api/ui-tests/` passes. `tools/gate.sh` passes.
- `rg -i '"Hosting"' crates/api/ui` gives no screen text.

Visual (the main session with screenshots and `$edges()`, then the operator):

- The tree at 1280 px and at 390 px, in Light and Dark, with an open publisher, a label and an artist below two labels.
- The page of a label with its path, its head and its groups.

## Stop And Ask When

- The place in music words of item 11 needs a rule that the feed list does not give. Report the missing value. Do not write the rule in the UI.
- The prototype and the ADR give different texts.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-253-the-catalog-tree-and-the-word-publisher.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (253a, 253b, 253c or 253d). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- crates/core/, crates/api/src/, docs/, AGENTS.md, crates/desktop/, crates/desktop-app/.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- node --test crates/api/ui-tests/
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
