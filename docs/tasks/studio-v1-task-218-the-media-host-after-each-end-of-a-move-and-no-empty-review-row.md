# Studio v1 Task 218: The Media Host After Each End Of A Move, And No Empty Review Row

Decision: [ADR 0230](../archive/adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 20 and 21. This task corrects defects. It makes no new decision.

Status: Done.

## Problem

Finding 3 of the desktop check of rc.30 (`docs/reviews/2026-11-20-rc29-human-check.md`): after the discard of a move, **Media files** of the test album said CloudFlare. The feed uses bastone. After the check, the collection had `files_connector_id` 2 and `files_connector_kept` NULL.

A code search found these causes. Each is an inference from the code, and no test proves it now:

- **Move All Releases** does not set `files_connector_id` to the target at its write. Its plan sets no default change (`crates/api/src/routes.rs` near line 5836). `set_feed_location` keeps the earlier value (`crates/core/src/collection.rs` near line 2924).
- These paths remove a move from a draft, but do not put back `files_connector_id` and do not clear `files_connector_kept`:
  - The undo of a feed move in Review (`crates/api/src/routes.rs` near line 5005, `crates/core/src/pipeline.rs` near line 3086).
  - The removal of the last draft operation of a move (`crates/api/src/routes.rs` near line 6556).
  - The discard of a change set on a feed that had a draft before (`crates/core/src/change_set.rs` near line 2057).
  - The restart or the rebase of a draft that removes a feed move (`crates/core/src/pipeline.rs` near lines 807 and 945).

Finding 6: Review **Simplified** shows a row "Remote items" with no text below it. The label comes from `crates/api/ui/review.js` near line 175.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Kept media host | reuse | crates/core/src/collection.rs | `keep_files_connector` near line 3028 and `restore_files_connector` near line 3045. |
| Move routes | extend | crates/api/src/routes.rs | `move_release` near line 5806 and the move of all releases near line 5836. |
| Write of a draft | extend | crates/core/src/pipeline.rs | The location update near line 4907. |
| Change sets | extend | crates/core/src/change_set.rs | The discard near line 2057. |
| Review rows | extend | crates/api/ui/review.js | The simplified rows and their labels near line 175. |
| API tests | extend | crates/api/src/tests.rs | `a_discard_puts_back_the_file_host_that_a_move_changed` near line 16147 and the helpers near lines 15848, 16133 and 16416. |

## Changes

1. Write a failing API test first for each cause above. Then correct the code. Record the red result of each test in the section "Result".
2. **Move All Releases** with media files sets `files_connector_id` to the target, as **Move This Release…** does, with `keep_files_connector` first.
3. Each path that removes the last move from a draft calls `restore_files_connector`. The discard of a change set keeps the values of the two columns in its snapshot and puts them back.
4. Finding 6: find the change that gives a simplified row with no text. Give it a text, or join it to the row of the same field. Add a UI test with that change.

## Checks

Mechanical:

- The API tests of change 1 fail before the correction and pass after it.
- Test `a_written_move_of_all_releases_sets_the_media_host`: Move All Releases from host A to host B, write, then `files_connector_id` is B and `files_connector_kept` is NULL. Then a move of the media files back to A, a discard, and `files_connector_id` is B.
- UI test: the simplified rows of the change of finding 6 have no row with an empty text.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `node --test crates/api/ui-tests/`, `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass.

Visual: none.

Human: a repair of the test album with **Move This Release…**, "Media files only" to bastone, then a write. **Media files** then says bastone.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above and this task file.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/api/src/routes.rs:5996` saves the media host before Move All Releases changes it.
- `crates/api/src/routes.rs:6661` restores the saved media host when the last draft operation is removed.
- `crates/core/src/pipeline.rs:3278` restores the saved media host when a feed move leaves a draft.
- `crates/core/src/pipeline.rs:6324` restores the saved media host after confirmation removes a move.
- `crates/core/src/change_set.rs:1845` saves the two media host columns. `crates/core/src/change_set.rs:2075` restores them on discard.
- `crates/core/src/collection.rs:711` adds snapshot columns. Its methods read and restore those columns near line 3027.
- `crates/api/ui/review.js:334` omits empty groups from the Simplified view.
- `crates/api/src/tests.rs:16633` checks move removal, change set discard, Move All Releases and a canceled feed move.
- `crates/api/ui-tests/review.test.mjs:214` checks that an empty Remote items group does not show.
- Before the fix, the Move All Releases API test showed a mismatch. The host value was 1. The target was 2.
- Before the fix, the empty Remote items UI test showed a label-only group.
- `cargo test -p studio-core`: completed with no errors.
- `cargo test -p studio-api`: completed with no errors, 467 tests.
- `tools/node --test crates/api/ui-tests/review.test.mjs`: completed with no errors, 27 tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: completed with no errors.
- `bash tools/gate.sh`: stopped at UI tests. The tests report unmet task 217 expectations in `addfeeds.test.mjs`, `people.test.mjs` and `shell.test.mjs`.
- No commit was made.

Main session changes after the review:

- `remove_draft_move` in `crates/core/src/pipeline.rs` puts back the media host only when the draft has no host operation. Before, the cancel of a feed move put back the media host while a move of the media files stayed in the draft. The test `a_cancelled_feed_move_keeps_the_media_host_of_a_pending_media_move` replaces the test that asked for that result.
- `keep_draft_after_failed_confirmation` in `crates/core/src/pipeline.rs` does not put back the media host. That write put the feed on the new host, and the next write requests the files again.
- Finding 6: the core removes each group with no change (`crates/core/src/review.rs` near line 660), so a filter of empty groups did not correct the finding. The cause: a list group has the heading "Remote items" with no summary, and each row below it has the same label. `fieldRows` in `crates/api/ui/review.js` now gives no heading for such a group. The UI test uses a real remote item change.
- The red results of the tests for the removal of the last operation and for the discard of a change set are not recorded. The main session did not run them before the correction.
- `cargo test -p studio-core`, `cargo test -p studio-api` (467 tests), `cargo clippy --workspace --all-targets -- -D warnings` and `tools/node --test crates/api/ui-tests/review.test.mjs` pass after these changes.
