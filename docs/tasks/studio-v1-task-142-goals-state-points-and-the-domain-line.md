# Studio v1 Task 142: Goals, State Points And The Domain Line

Decision: [ADR 0208](../adr/0208-the-delete-and-move-sheets-give-goals-state-points-and-the-domain-line.md).

Status: Complete on 2026-11-14, with its screenshot visual check. The human gate of ADR 0208 is open.

## Result

- A subagent did the build, because ChatGPT gave "usage limit". The main session read the diff, ran the five checks and took the screenshots.
- `list_feeds` gives `address_host` on each feed row. `feeddelete.js` gives the goal rows with an `action` key, `domainLine`, `DELETE_POINTS` and `retireActions`. `DomainLine` in `editor.js` is in the choice sheet, the last sheet, the Retire block and the move sheet.
- The main session made four corrections. The choice sheet had the old title. The full domain line was bold, and now only its first sentence is the title. The line in the Retire block had no space at its sides. **Put the File Back** did not load the feed list, so Review showed the notice of a move.
- **Put the File Back** calls the restore route, loads the feed list, and opens Review through `OPEN_DELETE_REVIEW`. It does not call `draft.show`, because that event loads the draft.
- The prop of `DomainLine` is `domain`, because the words test refuses the attribute `host=`.
- The test of ADR 0198 for **Stop Retiring** now reads `retireActions`. The markup that it compared is gone.
- Criterion 7: the existing test `the_delete_draft_gives_the_summary_and_the_names_and_the_write_deletes_the_file` proves the restore.
- Walk: `setup-137.py` with `steps-142-{light,dark}.json`, and `setup-142-move.py` with `steps-142-move.json`. The walk ran at 1280 px light and at 390 px dark. It showed the choice sheet with three rows, the last sheet and the Retire block of a deleted feed. It also showed Review after **Put the File Back**. After the write, the file was on the test host again. The move sheet showed the line with `music.example`. Each view gave no edge and no overflow finding.
- The 390 px fit check gave one overlap of "Versions" with "Move Artist X up". The walk ended with the page scrolled down, as in task 139. This change does not cause it.
- The walk did not show the choice sheet with two rows. The Node test proves the two rows.
- Open defect, not corrected: after the write of the restore, step 1 of the Retire block says "Hide from apps is on in the draft." The bar says "No changes". The expected text is "Hide from apps is on in the last write." The cause is not known.

## Goal

1. The sheet "Delete from File Host" shows goals, with no paragraph and no tag (ADR 0208 items 5 to 8).
2. The sheet "Delete the Feed File" shows three state points in the place of "You lose these things:" (items 9 to 11).
3. The domain line shows in the two sheets, in the Retire block and in the move sheet (items 1 to 4, 12 and 16).
4. A deleted feed shows **Put the File Back** (items 13 to 15).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/feeddelete.js`: `DELETE_CHOICES`, `deleteChoices(state)` and `retireSteps(state)`.
- `crates/api/ui/editor.js`, `FeedDelete` (about line 2350). The sheet `delete-host-title` has two paragraphs and the rows. The sheet `delete-feed-title` has the list `delete-loss`. The Retire block has the head `retire-head` and its actions. The click handler of a row compares `choice.title` with the old titles.
- `crates/api/ui/editor.js`, the sheet `move-release-title` (about line 4236): the section "The plan" with `planRows`. `plan.feed.moves` is true when the feed file moves.
- `crates/api/ui/editor.js`, `ReturnWarning` (about line 1766): the markup of a warning, `notice warning` with the icon `alert`.
- `crates/api/src/routes.rs` (about line 1431): the feed row of the API, where `hidden` is set.
- `crates/core/src/web.rs:723` `host_of`, and `crates/core/src/outside.rs:159` `address_of`, which leaves out a placeholder address.
- `crates/api/src/lib.rs:460` and `crates/api/src/feed_retire.rs:100`: the route `POST /feeds/{id}/delete/restore`. Its answer is the diff of the restore draft.
- `OPEN_DELETE_REVIEW` in `editor.js`: the event that loads the draft and opens Review.
- Tests: `crates/api/ui-tests/feeddelete.test.mjs`, `crates/api/src/tests/feed_retire.rs`, and the tests of the feed list in `crates/api/src/tests.rs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Delete choices | extend | crates/api/ui/feeddelete.js | The rows get the goal titles and texts. A row gets a stable `action` key. |
| Retire steps | reuse | crates/api/ui/feeddelete.js | The steps do not change. |
| Domain line view | extend | crates/api/ui/feeddelete.js | A pure function gives the words of the line from `address_host`. |
| Warning notice | reuse | crates/api/ui/editor.js | The markup `notice warning` with the icon `alert` of `ReturnWarning`. |
| Host of an address | reuse | crates/core/src/web.rs | `host_of` gives the host. The UI gets no host from an address. |
| Address of a feed | reuse | crates/core/src/outside.rs | `address_of` leaves out a placeholder address. |
| Feed row of the API | extend | crates/api/src/routes.rs | The row gets `address_host`. |
| Restore route | reuse | crates/api/src/feed_retire.rs | `restore_draft` starts the draft. No change. |
| Delete sheets and Retire block | extend | crates/api/ui/editor.js | `FeedDelete` shows the new texts and the button. |
| Move sheet | extend | crates/api/ui/editor.js | The sheet shows the domain line when the feed file moves. |

## Rules For The Delegate

- Read ADR 0208 first, then ADR 0131, ADR 0198 and ADR 0200.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not add a route. Do not change the schema of the collection.
- If a step needs an architecture decision that ADR 0208 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.
- Use the exact words of ADR 0208 for each text.

## Changes

1. `crates/api/src/routes.rs`: the feed row gets `address_host`. Its value is `web::host_of` of `outside::address_of(&row)`, or null. If more than one function builds a feed row, each of them gives the field through one helper.
2. `crates/api/ui/feeddelete.js`:
    - `DELETE_CHOICES` gets the titles and texts of ADR 0208 item 6, and an `action` key for each row: `move`, `retire` and `delete`. Remove `recommended`.
    - `deleteChoices(state)` gives three rows only when `removal_done` is true, and two rows in each other state.
    - Add `export function domainLine(host)`. It gives the text of ADR 0208 item 1, or null with no host.
    - Add `export const DELETE_POINTS`: the three points of item 10, each with its mark and its text. The third point gives the button name as a separate value, so that the sheet shows it in bold.
    - Add `export function retireActions(state)`. It gives the keys of the buttons of the Retire block in their sequence: `restore` and `check` for a done delete, `check` and `stop` in each other state.
3. `crates/api/ui/editor.js` `FeedDelete`:
    - The choice sheet: the title "What do you want to do?", no paragraph, the rows from `deleteChoices`, the click handler by `choice.action`, no tag, then the domain line.
    - The last sheet: remove the list `delete-loss`. Show `DELETE_POINTS`, then the domain line.
    - The Retire block: the head text is "Studio cannot check Podcast Index." The domain line is below the head.
    - The actions come from `retireActions`. **Put the File Back** calls `POST /feeds/${feed.id}/delete/restore`, shows the answer with `draft.show` when the answer is a draft, and opens Review with `OPEN_DELETE_REVIEW`. Examine how the route answer and `draft.show` agree before you use them. An error shows in the `ErrorNotice` of the block.
    - One small component `DomainLine` gives the warning markup. Each place uses it.
4. `crates/api/ui/editor.js`, the move sheet: when `plan.feed.moves` is true, show `DomainLine` with `feed.address_host` below the plan section.
5. `crates/api/ui/styles.css`: remove the rule of `delete-loss` if no markup uses it. Add a rule for the points only if the existing list styles do not fit. Use tokens only.
6. Update each test that reads the old titles or texts. Delete each test of a removed text.

## Acceptance Criteria

Mechanical:

1. A Node test of `deleteChoices`: the titles, texts and `action` keys of ADR 0208 item 6, no `recommended`, three rows only with `removal_done: true`. Each failure message starts with "ADR 0208" and gives the fix.
2. A Node test of `domainLine`: the exact text for `music.example`, and null for null and for an empty text.
3. A Node test of `DELETE_POINTS`: three points with the texts of item 10.
4. A Node test of `retireActions`: `restore` then `check` for `delete: 'done'`, and `check` then `stop` in each other state.
5. A Node scan test of `editor.js`: the text "You lose these things" is not in the file, and `DomainLine` is in the choice sheet, the last sheet, the Retire block and the move sheet.
6. An API test: the feed row gives `address_host` for a feed with a public address, and null for a feed with a placeholder address.
7. An API test: after a delete, `POST /feeds/{id}/delete/restore` gives a draft, and a write of that draft puts the file back. Use the existing test if it already proves this, and name it in the report.
8. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The choice sheet with two rows and with three rows, at 1280 px light and 390 px dark.
2. The last sheet, at 1280 px light and 390 px dark.
3. The Retire block of a deleted feed with **Put the File Back**, and Review after the button.
4. The move sheet with the domain line.

Human, by the operator: the gate of ADR 0208.

## Report

- The changed lines, with file and line.
- The test names, and the output of the four checks.
- Each point where the code did not agree with this task.
