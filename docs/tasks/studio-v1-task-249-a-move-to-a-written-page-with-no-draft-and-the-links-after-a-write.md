# Studio v1 Task 249: A Move To A Written Page With No Draft, And The Links After A Write

Source: observations 14 and 15 of `docs/reviews/2026-11-23-desktop-check-walk.md`.

Status: Done. The test of step 2 failed with `no_guid` before the correction and passes after it. The two tests of step 4 pass, so the API keeps the links. The harness was the cause of observation 15.

Operator choices (defect corrections, no ADR):

- A move to a written page with no draft passes, as it does after `GET /api/v1/feeds/{id}/draft`.
- Observation 15 gets a test first. A correction follows only when the test fails.

## Problem

- Observation 14: `move_release_to_pages` (`crates/api/src/routes.rs` near line 3749) checks the GUID of each touched feed before it starts the drafts. A feed with a draft uses `draft_model`. A feed with no draft uses the catalog model from `files_feed` (near line 5677), which reads the library copy (`crates/api/src/workspace.rs` near line 2353). `ensure_draft` (near line 3401) starts a draft from the file of the connector. Thus the check and the draft can read two different sources. This is an inference from the code. The test proves or disproves it.
- Observation 15: the harness linked a written release to an artist page with `POST /api/v1/feeds/{page}/links`. A write of the release with the test helper `write_with_own_diff` (`crates/api/src/tests.rs` near line 17362) removed each link of the release. `publisher::link` (`crates/core/src/publisher.rs` near line 347) changes the two drafts. The cause is not known.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Release move | extend | crates/api/src/routes.rs | `move_release_to_pages` checks the GUID before it starts the drafts. |
| Draft start | reuse | crates/api/src/routes.rs | `ensure_draft` starts a draft from the connector file. |
| Move tests | extend | crates/api/src/tests.rs | Copy the form of `the_move_route_links_three_pages_and_unlinks_the_other_artist_page`. |
| Two-side write test | reuse | crates/api/src/tests.rs | `a_changed_link_of_two_written_feeds_gives_a_row_on_each_side` (near line 1837) writes the two feeds. |

## Changes

1. Read `move_release_to_pages`, `ensure_draft`, `files_feed` and the link route near line 3542 in `crates/api/src/routes.rs`. Read the tests named above and `write_with_own_diff` in `crates/api/src/tests.rs`.
2. Observation 14 test: write a hosting page, so that it has no draft. Move a written release with an artist page to that hosting page with `POST /api/v1/feeds/{id}/move-to`. The move must pass, and the drafts of the release and the hosting page must have the link.
3. If the test of step 2 fails with `no_guid`, correct the source of the GUID check. The check reads the same source that `ensure_draft` reads, or the route starts the drafts before the check. A failed check must still change no draft (test `the_move_route_changes_no_draft_when_a_page_fails_its_check`).
4. Observation 15 test: write a release and an artist page. Link them with `POST /api/v1/feeds/{page}/links`. Write the two feeds in the order of Review (ADR 0251 item 7): the release, then the page. After the writes, the written file of the release has the `podcast:remoteItem` of the page, and the written file of the page has the item of the release. Add a second case: write only the release with `write_with_own_diff`, and record what the file holds.
5. If the case in the order of Review fails, stop and report the cause with evidence. Do not correct it. If only the second case loses the links, report that the helper is the cause.
6. Each failure message names the observation of the walk of 2026-11-23 and the function to fix.

## Do not touch

`crates/core/`, `crates/api/ui/`, `crates/api/ui-tests/`, `docs/`, `AGENTS.md`. Task 248 changes the UI at the same time.

## Constraints

- Change only `crates/api/src/routes.rs` and `crates/api/src/tests.rs`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of the crate.
- Run `cargo test -p studio-api` and `tools/gate.sh` at the end, and report the results. A failure in the Node tests can come from task 248. Report it and do not correct it.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The test of step 2 passes.
- The test of step 4 in the order of Review passes, or the report gives the cause with evidence.
- `cargo test -p studio-api` passes.

## Escalation

Stop and report if the correction of step 3 needs a change in `crates/core/`, or a change of what the move writes.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-249-a-move-to-a-written-page-with-no-draft-and-the-links-after-a-write.md
- crates/api/src/routes.rs (the functions named in the task)
- crates/api/src/tests.rs (the tests named in the task)

Goal:
- Prove or correct observation 14. Prove or explain observation 15.

Constraints:
- The constraints of the task file.

Do not touch:
- crates/core/, crates/api/ui/, crates/api/ui-tests/, docs/, AGENTS.md.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo test -p studio-api
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
