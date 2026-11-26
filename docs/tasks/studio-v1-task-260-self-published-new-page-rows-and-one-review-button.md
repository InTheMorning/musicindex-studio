# Studio v1 Task 260: Self-Published, New Page Rows And One Review Button

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md).
Decision: [ADR 0267](../adr/0267-a-page-sheet-can-make-a-new-page-and-review-has-one-button.md) items 1 to 10.
Findings: [walk of 2026-11-26](../reviews/2026-11-26-human-check-walk.md), findings 1, 3, 11 and 12.

Status: not started.

Operator choices (the main session, technical choices inside ADR 0267):

- Item 7 applies to Overview also. Overview has no draft of its own, so its top bar shows only the count and **Review** while a change set is open.
- Item 9 gives the API a value `index_warning` for each feed that Review writes. It uses the same paths as `page_refusal`: the feed diff (`diff_json`) and the feed rows of a change set.
- Item 9 applies to a feed with the medium `music` only. A feed with live items and no items gets the warning also, because the rule names items.

## Goal

Build ADR 0267. It gives the words "self-published", the group "New" in each page sheet with a list, and the button **Sign**. It also gives one **Review** button in the top bar, and the warning in Review before a write that Music Index refuses.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Place words of the core | extend | crates/core/src/publisher/page_change.rs | The texts near lines 622 and 633 give "Self-published" and "self-published" (item 1). |
| Publisher value of a release | extend | crates/core/src/publisher/credits.rs | The doc comment near line 39 and each text of "Not published" (item 1). |
| Notice text of the API | extend | crates/api/src/routes.rs | The text near line 3962 gives "{title} is self-published." (item 1). |
| Page rule check | extend | crates/core/src/feed_check.rs | A new function `index_warning(feed: &Feed)` adjacent to `page_refusal` (line 153), with the same shape: an enum with `code()` (item 9). |
| Feed diff answer | extend | crates/api/src/routes.rs | `diff_json` (line 5048) adds `"index_warning"` adjacent to `"page_refusal"` (line 5115). |
| Change set feed rows | extend | crates/api/src/routes.rs | The feed row near line 5184 adds `"index_warning"` adjacent to `"refusal"`. |
| API tests | extend | crates/api/src/tests.rs | The place texts near line 21307 change to "self-published". New tests for `index_warning`. |
| Place words of the UI | extend | crates/api/ui/shell.js | `placeText` near line 512 (item 1). |
| Page sheet model | extend | crates/api/ui/movepages.js | `PAGE_SHEETS` (line 42): the button **Sign** for `sign_to_label`. `pageChoiceRows` (line 129): the group "New" with the row of item 2. The texts at lines 120, 147 and 294 (item 1). |
| Page sheet | extend | crates/api/ui/editor.js | `PageChangeSheet` (line 2946) shows the group "New". The row opens `AddFeedRowSheet` with `ARTIST_PAGE_FORM` and `start={{ role }}`, as `ReleaseCreditFields` does near line 3288. After the page is made, the sheet selects it (item 3). |
| Change notice | extend | crates/api/ui/undonotice.js | `CatalogUndoNotice` (line 104) has no **Review** button (item 6). Remove the `onReview` property from each caller in `app.js` (line 4818) and `editor.js` (line 3287). |
| Top bar status | extend | crates/api/ui/shell.js | Move `toolbarStatus` from `editor.js` (line 2587) to `shell.js` and export it. It takes the open change set also and gives "{N} changes not written" from `catalogChangeCount` (`movepages.js` line 316) (item 7). |
| Top bar | extend | crates/api/ui/editor.js | `DraftToolbar` (line 2601): with an open change set, **Review** opens `{ name: 'change-set', id }`. |
| Top bar of Overview | extend | crates/api/ui/app.js | The Overview top bar shows the count and **Review** while a change set is open (operator choice above). |
| Panel count | extend | crates/api/ui/app.js | Remove `CatalogChangeCount` (line 3637) and its two uses in `LeftPanel` (item 8). |
| Review warning | extend | crates/api/ui/editor.js | The view adjacent to `pageRefusalView` (near line 2325) shows `index_warning` with the class `notice warning` and the icon `alert` (ADR 0191 item 1). The change set Review (`app.js` near line 3592) shows it on the feed row. |
| UI Node tests | extend | crates/api/ui-tests/movepages.test.mjs | The group "New", the button **Sign** and the words of item 1. |
| Shell Node tests | extend | crates/api/ui-tests/shell.test.mjs | `toolbarStatus` with and without an open change set. The words of `placeText`. |

## Constraints

- No feed rule in the UI (ADR 0065). The core gives `index_warning`. The UI shows it and does not read the items or `podcast:value`.
- Item 10: copy only the rule of item 9 from Stophammer. Name the owner in the doc comment: the Stophammer verifier `v4v_payment`.
- The warning does not stop the write. The button **Write** stays available.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- A test message names the owner and the fix, for example: `ADR 0267 item 1: a page with no publisher page is "self-published". Change the text in placeText.`
- Do not commit. Leave the change in the working tree and report it.

## Packets

### 260a: The Words And The Warning (core and API)

- Item 1 in the core and the API.
- Item 9: `index_warning` in `feed_check.rs`, in `diff_json` and in the change set feed rows.

### 260b: The Sheets, The Top Bar And The Warning View (UI)

- Items 1 to 8 in the UI, and the view of item 9. Starts after 260a is committed.

## Acceptance Criteria

Mechanical:

1. 260a: an API test: the place texts and the notice give "self-published" (item 1).
2. 260a: a core test of `index_warning`: a `music` feed with no item and no channel `podcast:value` gives the warning. A feed with one item, a feed with a channel `podcast:value` and a `publisher` feed give none (item 9).
3. 260a: an API test: the feed diff and the change set feed row give `index_warning` (item 9).
4. 260b: a Node test: the rows of a label sheet and of a publisher sheet start with the group "New", also when pages of that role exist. The sheet `sign_to_label` has the button **Sign** (items 2 and 5). The sheet `go_independent` has no group "New" (item 4).
5. 260b: a Node test: `toolbarStatus` gives "{N} changes not written" with an open change set, and the draft state with no change set (item 7).
6. `cargo test -p studio-core`, `cargo test -p studio-api`, `tools/node --test crates/api/ui-tests/*.test.mjs` and `tools/gate.sh` pass.

Visual (the main session with screenshots and `$edges()`):

- The sheet "Sign to a label…" with the group "New" and the button **Sign**, in Light and Dark.
- The row "＋ Make a new label page" opens the form. After the page is made, the sheet shows again with the new label selected.
- After a change: the notice has **Undo** and the close button only. The top bar has "{N} changes not written" and **Review**, also on Overview. The Catalog panel has no **Review**.
- The warning of item 9 in Review.

Human check (the operator): ADR 0267 §How This Decision Is Checked.

## Stop And Ask When

- A correction needs a change of an ADR rule.
- The top bar of a page has a draft of its own and an open change set at the same time, and **Review** of the change set does not write the draft of the page. Report the facts.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-260-self-published-new-page-rows-and-one-review-button.md
- docs/adr/0267-a-page-sheet-can-make-a-new-page-and-review-has-one-button.md
- The locations of "Base components"

Goal:
- The packet that the main session names (260a or 260b). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- docs/, AGENTS.md, crates/desktop/, crates/desktop-app/. Change crates/core/ and crates/api/src/ only in packet 260a. Change crates/api/ui/ only in packet 260b.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- cargo test -p studio-core (packet 260a)
- cargo test -p studio-api (packet 260a)
- tools/node --test crates/api/ui-tests/*.test.mjs
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
