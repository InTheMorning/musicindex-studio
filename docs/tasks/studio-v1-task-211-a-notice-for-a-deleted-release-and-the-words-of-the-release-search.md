# Studio v1 Task 211: A Notice For A Deleted Release, And The Words Of The Release Search

Decision: [ADR 0231](../adr/0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 1 to 3 and item 6.

Status: Done.

## Problem

Findings 2 and 8 of the [desktop check of tasks 205 to 209](../reviews/2026-11-20-rc29-human-check.md#desktop-check-of-tasks-205-to-209).

- Finding 2: the actions of a deleted release are only in the Retire block, at the bottom of the tab **Release**.
- Finding 8: the sheet **Find on Music Index** of a new release has the group title "Feeds" and the row text "Feed · {artist}".
- ADR 0231 item 6: the parent list has artist pages only. Task 208 built this. No test pins it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Retire actions | reuse | crates/api/ui/feeddelete.js | `retireActions` near line 72 and `retireBlockShown` near line 66. |
| Retire block | extend | crates/api/ui/editor.js | `FeedFacts` near line 2439. The actions near line 2624 call `restoreFile` and the check again. The notice must call the same functions. |
| Head notices | extend | crates/api/ui/outsidecheck.js | The notice **Use This File Host** near line 65 shows under the head. Put the new notice in the same place and with the same notice classes. |
| Search sheet | extend | crates/api/ui/pickers.js | `FindOneSheet` near line 291. The row text near line 282 and the title "Feeds" near line 278. |
| Release search caller | extend | crates/api/ui/app.js | The form of a new release calls `FindOneSheet` near line 1562. |
| Parent sheet | reuse | crates/api/ui/app.js | `ParentSheet` near line 1446. |
| UI tests | extend | crates/api/ui-tests/ | The tests of `feeddelete.js`, `pickers.js` and the form of a release. |

## Changes

1. Item 1: a feed with `host_deleted` true shows a notice under the head of its page, on each tab. Text: "The feed file of this release is deleted from its file host." Buttons: **Put the File Back** and **Check Again**. They do the same actions as the Retire block.

   Put the condition and the text in a pure function in `feeddelete.js`, so that a node test can reach it.
2. Item 2: the Retire block stays in the section **Feed**, with no change.
3. Item 3: in the sheet of a new release only, the group title is "Releases" and each row says "Release · {artist}", or "Release" with no artist. Give `FindOneSheet` a parameter for this. The other callers (`values.js`, `peoplepages.js`, `editor.js`) keep "Feeds" and "Feed".
4. Item 6: add a node test that the parent choices of the form of a release have artist pages only. When the code gives a label page, stop and report.

## Checks

Mechanical:

- Node test: the notice function gives the notice for a feed with `host_deleted` true, and nothing for a feed with no done delete.
- Node test: the release caller of `FindOneSheet` gives "Releases" and "Release · {artist}". The default stays "Feeds" and "Feed · {artist}".
- Node test: the parent choices have no label page.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.

Visual (main session):

- drive-ui at 1280 px and 390 px: the notice on the tab **Release** and on one other tab of a deleted release. The sheet **Find on Music Index** with the new words.

Human:

- None. The walk of the next candidate covers it.

## Result

Changes:

- `crates/api/ui/feeddelete.js`: the new function `deletedNotice`. It gives the text and the actions `restore` and `check` for a feed with `host_deleted` true. It gives null for each other feed.
- `crates/api/ui/editor.js`: the new exports `restoreDeletedFile`, `checkDeletedAgain` and `RETIRE_STATE`. The restore action of the Retire block calls `restoreDeletedFile`. `FeedFacts` listens for `RETIRE_STATE` and shows the new retire state.
- `crates/api/ui/app.js`: `OutsideNotices` shows the notice from `deletedNotice` under the head, on each tab. It has the buttons **Put the File Back** and **Check Again**. `EditableFeed` passes `onListChanged` to it. The `FindOneSheet` of the form of a new release has `noun="Release"`.
- `crates/api/ui/pickers.js`: `FindOneSheet` has the parameter `noun`, with the default "Feed". `SearchResults` uses it through `feedWords`.
- `crates/api/ui/people.js`: the new function `feedWords`. It is here and not in `pickers.js`, because a node test cannot import `pickers.js`. The task did not list this file.
- `crates/api/ui-tests/feeddelete.test.mjs`, `people.test.mjs` and `welcome.test.mjs`: one new test each, for the notice, the words of the sheet and the parent choices (item 6).

Checks:

- Item 6: `releaseArtistChoices` already gave no label page, and the new test passes with no code change.
- `tools/node --check` on a copy of each changed ui file: passes.
- `tools/node --test crates/api/ui-tests/*.test.mjs`: 927 pass, 0 fail (924 before).
- `bash tools/gate.sh`: "gate: each check passed".
- Visual (2026-11-20, main session): drive-ui at 1280 px and 390 px on a copy of `.studio-dev/v-207/data-206`. The notice shows on the tab **Release** and on the tab **Settings**. **Check Again** gives no error. `$edges()` gives no finding.
- The sheet **Find on Music Index** has no screenshot, because the sandbox has no network. The node test covers its words.
- Note: on the tab **Settings**, **Check Again** shows no result. Only the Retire block on the tab **Release** shows the new state.
- Note: the test data `.studio-dev/v-207/data-del` has a done delete and no snapshot. Its draft does not load ("feed 1 has no snapshot of its missing last copy"). This is a defect of the test data. It is not examined on real data.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit and do not stage. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.
- Syntax check: copy each changed `crates/api/ui/*.js` file to `$TMPDIR/x.mjs`, then run `tools/node --check` on the copy.
