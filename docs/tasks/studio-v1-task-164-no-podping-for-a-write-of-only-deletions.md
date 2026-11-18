# Studio v1 Task 164: No Podping For A Write Of Only Deletions

Decision: ADR 0198 items 16 and 17.

Status: Complete on 2026-11-17, with its screenshot visual check.

## Problem

A write of only deletions (ADR 0198 item 12) does not change the feed file. At this time, `plan_announcements` plans a podping for it when the announcements are on. An index then reads a feed that did not change.

## Work

1. In `crates/core/src/announce.rs`, add the variant `AnnounceSkip::FeedUnchanged` with the text `feed_unchanged`.
2. In `plan_announcements` (`crates/core/src/pipeline.rs`), give no announcement and `FeedUnchanged` for a write of only deletions. Use the same condition as ADR 0198 item 12: each operation of the diff is a deletion, and the feed file does not change and does not move. A refused deletion does not count (item 14). A granted write uses the same rule.
3. In `announceSkipLine` (`crates/api/ui/announce.js`), give "No podping: this write does not change the feed." for `feed_unchanged`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Announcement plan | extend | crates/core/src/pipeline.rs | One more skip condition in `plan_announcements`. |
| Skip causes | extend | crates/core/src/announce.rs | One more variant of `AnnounceSkip`. |
| Footer skip line | extend | crates/api/ui/announce.js | One more text in `announceSkipLine`. |
| Only-deletions view | reuse | crates/api/ui/feeddelete.js | `deleteWrite` keeps its condition. The UI does not write the condition again. |

## Do Not Touch

- `crates/api/ui/feeddelete.js`, `crates/api/ui/review.js` and the footer markup.
- The announcement queue and the podping sender.

## Acceptance Criteria

Mechanical:

1. A Rust test in `crates/core`: a write of only deletions with the announcements on gives no announcement and the skip cause `feed_unchanged`.
2. A Rust test in `crates/core`: a write that changes the feed and deletes a file keeps its announcement.
3. A Rust test in `crates/core`: a diff with only refused deletions gives no `feed_unchanged` skip.
4. A Node test in `crates/api/ui-tests/announce.test.mjs`: `announceSkipLine` gives the text of item 3 of the work for `feed_unchanged`.
5. `tools/gate.sh` passes.

Visual, by the main session:

1. The review of a write of only deletions at 1280 px Light and 390 px Dark. The footer shows the line of item 3 of the work, and no check box. `$edges()` gives no result.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-164-no-podping-for-a-write-of-only-deletions.md
- docs/adr/0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md, items 12 to 17
- crates/core/src/pipeline.rs (`plan_announcements`, `announce_bytes` and their callers)
- crates/core/src/announce.rs (`AnnounceSkip`)
- crates/api/ui/announce.js and crates/api/ui-tests/announce.test.mjs
- crates/api/ui/feeddelete.js (`deleteWrite`, for the condition only)
- /home/citizen/.agents/skills/asd-ste100/SKILL.md, for code comments

Goal:
- A write of only deletions plans no podping, and the review footer gives the cause.

Constraints:
- The core owns the condition. The UI shows only the skip cause of the core.
- Follow the `rust-dev` skill for Rust code.
- Write code comments in Simplified Technical English.
- Do not commit. Keep the change in the working tree. The main session reviews the diff and commits.

Do not touch:
- crates/api/ui/feeddelete.js, crates/api/ui/review.js, the announcement queue and the podping sender.

Acceptance criteria:
- Mechanical criteria 1 to 5 of the task file.

Test commands:
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Result

- Mechanical 1 to 4: met. `only_deletions_give_feed_unchanged_and_no_announcement`, `a_feed_change_and_deletion_keep_the_announcement` and `refused_deletions_do_not_give_feed_unchanged` in `crates/core/tests/pipeline.rs`, and the `feed_unchanged` line in `announce.test.mjs`. A refused deletion is in `blocked`, not in the operations, so it does not count.
- Mechanical 5: met.
- Visual, by the main session: met. The walk data `.studio-dev/walk/setup-164.py` gives Artist X one delete of a stray file and no feed change. At 1280 px Light and 390 px Dark, the footer shows the line and **Delete 1 File**, with no check box. `$edges()`, `$overflow()` and the fit check give no result.
