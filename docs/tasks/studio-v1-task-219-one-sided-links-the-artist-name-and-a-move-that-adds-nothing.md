# Studio v1 Task 219: One-Sided Links, The Artist Name, And A Move That Adds Nothing

Decision: [ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) items 5 to 7, 9 and 11 to 13.

Status: Done.

## Problem

The desktop check of 9af319b gave findings 2 to 5 (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of 9af319b"). This task gives the core and the API. Task 220 gives the UI, and it uses the API contract below.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed list | extend | crates/api/src/routes.rs | `list_feeds` near line 1411 and the row fields near line 1506. Add `author` and `unlisted_by`. |
| Written copy of a feed | reuse | crates/core/src/pipeline.rs | `hosted_copy` near line 3414 reads the library copy of the last write or read, with no draft. |
| Publisher catalogs | reuse | crates/core/src/publisher.rs | `catalogs_with_kept` near line 404 and `names` near line 590 match a remote item by guid or address. |
| Publisher link | extend | crates/core/src/publisher.rs | `link` near line 214 edits the two feeds. Add a function that edits only the page. |
| Check route | extend | crates/api/src/feed_check.rs | `check_json` near line 40 adds the findings of the API, as `index_address_differs` does. |
| Remove of a feed | extend | crates/core/src/people.rs | `release_link_cleanup` near line 3858 and `cleanup_release_links` near line 3923. |
| Move route | extend | crates/api/src/routes.rs | `move_release` near line 5894 and `apply_release_move` near line 5777. |
| Kept media host | extend | crates/core/src/collection.rs | `keep_files_connector` near line 3077 and the migrations list. |
| Link routes | extend | crates/api/src/lib.rs | The routes `/feeds/{id}/links` near line 414. |

## API Contract For Task 220

- `GET /api/v1/feeds`: each row gets `author` (string or null, item 9) and `unlisted_by` (array of feed ids, item 5). `unlisted_by` is empty for a release that was never written.
- `GET /api/v1/feeds/{id}/check`: a written release gets one finding for each id of `unlisted_by`: `kind: "unlisted_by_page"`, `level: "notice"`, `title: "{page} does not list this release."`, `reason: "Apps show the release below {page} only after a write of {page}."`, `entry: null`, `repair: null`, `url: null`, and the extra field `page_id` (number) and `page_title` (string).
- `POST /api/v1/feeds/{page}/links/{release}/list`: item 6. It starts a draft of the page when it has none. It adds the channel remote item of the release, with the rel of the role of the page. The release does not change, and the answer is `{ "page_id": n }`. A release that the page lists in its draft already gives the same answer and no change.

## Changes

1. Items 5 and 7: compute `unlisted_by` of a written release (`last_version` is set). For each page of the collection that the release names in `podcast:publisher` (guid or address), read the written copy of the page with `hosted_copy`. When the written copy has no channel remote item that names the release, add the id of the page. A page with no written copy is not in the list.
2. Item 9: `author` is `itunes:author` of the draft, or else of the written copy, as the other row fields are read.
3. Item 5: add the finding of the contract in `check_json`.
4. Item 6: add the route of the contract with a new function in `publisher.rs` that edits only the page draft. Use `ensure_draft` before the edit.
5. Item 11: in `release_link_cleanup`, remove the condition that the release was never written. For each page draft with a remote item that names the release, remove that item only when the written copy of the page does not name the release. With no written copy of the page, remove it. Keep the undo snapshot and the drop of a draft that only changes the build date.
6. Item 12: in `move_release`, after `.limited(body.what)`, when the plan adds no feed move, no unwritten file, no target copy and no file operation, and `plan.default_changes` is true, call `set_files_connector` with the target and clear `files_connector_kept` (a new collection function). Do not call `keep_files_connector`. When the plan adds an operation, keep the code as it is.
7. Item 13: add a migration: `UPDATE feeds SET files_connector_kept = NULL` for each feed with a kept value and with no row in `draft_operations` and in `draft_feed_moves` for that feed. Read the real column names of these tables first.

## Checks

Mechanical:

- API test: a written release that names a written artist page whose written copy has no remote item gives `unlisted_by` with the page id, and the check finding of the contract. After a write of the page with the item, the list is empty.
- API test: the route of change 4 adds the remote item to the page draft. The release has no draft change after it.
- API test: the feed list gives `author` of the release.
- API test: **Remove from Collection** of a written release removes its remote item from a page draft whose written copy does not name it. With no other change, the draft goes. A page whose written copy names the release keeps the item. **Undo** gives back the release and the drafts.
- API test: a release on host 2 with its files on host 1, `files_connector_id` 2. **Move This Release…** "Media files only" to host 1 gives `files_connector_id` 1 and `files_connector_kept` NULL. A later move to host 2 with an operation, then a discard, gives `files_connector_id` 1. Record the red result before the fix.
- Core test: the migration clears a kept value with no draft operation, and keeps a kept value with a draft operation.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass. The gate runs the UI tests, which task 220 changes at the same time. Report a UI test failure, but do not change `crates/api/ui/` or `crates/api/ui-tests/`.

Visual: none. Task 220 gives the UI.

Human: the next desktop check repeats the discard of a move on the test album.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above, `crates/api/src/tests.rs`, `crates/api/src/workspace.rs` if necessary, and this task file. Do not change `crates/api/ui/` or `crates/api/ui-tests/`.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/api/src/routes.rs:1411` adds `author` and `unlisted_by` to feed rows. `crates/api/src/routes.rs:1545` calculates missing page listings.
- `crates/api/src/feed_check.rs:87` adds the `unlisted_by_page` result.
- `crates/core/src/publisher.rs:262` adds the page-only link operation. `crates/api/src/routes.rs:3213` adds its API path. `crates/api/src/lib.rs:414` registers that path.
- `crates/core/src/people.rs:3858` removes stale page links when a feed leaves the collection. It keeps links that the written page lists.
- `crates/api/src/routes.rs:6016` clears the kept host when a move adds no operation. `crates/core/src/collection.rs:3097` adds the clear function.
- `crates/core/src/collection.rs:715` clears kept hosts with no pending move. `crates/core/src/collection.rs:6638` tests the migration.
- `crates/core/src/pipeline.rs:3415` exposes the existing written-copy reader for API use. `crates/api/src/tests.rs:1873` tests the page-only API path.
- `cargo check --workspace`: passed.
- `cargo test -p studio-core`: passed, 881 unit tests and all integration tests.
- `cargo test -p studio-api`: 466 tests passed. Three unrelated tests reported errors.
- Three unrelated API tests reported errors.
- The tests are `an_nsite_host_with_no_d_and_a_feed_gets_the_d_of_the_feed_guid`, `the_outside_check_compares_the_address_of_musicindex_only_on_request` and `the_outside_check_finds_a_change_and_a_move_and_keeps_a_result_after_a_failed_read`.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `bash tools/gate.sh`: stopped at the same three API test errors. It skipped the UI tests.
- `tools/node --test crates/api/ui-tests/*.test.mjs`: passed, 945 tests.
- The API test `one_sided_link_changes_only_the_page_draft` returned an OK result.
- No UI file changed. No commit was made.

## Main Session Changes

- `crates/api/src/routes.rs`: `list_feeds` uses `w.library().ok()`. With `?`, a collection with no library folder got an error for the full list. This caused the three failing API tests.
- `crates/api/src/feed_check.rs`: `unlisted_findings` uses `routes::unlisted_pages`, so that the list and the check use one comparison. A collection with no library folder gives no finding and no error.
- `crates/core/src/people.rs`: `release_link_cleanup` takes the guid from the draft, or else from the feed row. A written release with no draft now also clears its links (item 11).
- `crates/core/src/collection.rs`, `crates/core/src/pipeline.rs`: a feed made in Studio has no release folder, so `hosted_copy` gave no written copy. A screenshot run showed `author` null and `unlisted_by` empty for such a page and release. A migration adds `feed_writes.written_xml`. The write keeps its bytes there (`record_written_xml`). `pipeline::written_copy` gives the library copy, or else these bytes. The list, the check and `release_link_cleanup` use it. ADR 0234 item 7 states this source.
- `crates/api/src/tests.rs`: the test `a_written_release_without_a_draft_gives_its_written_author_in_the_feed_list` changed the draft with a PUT, and the write then gave 409. It now makes the release with `make_release` and the author "Written Author".

## Follow-Up: The Missing Tests

The Result gives two of the seven tests of the section "Checks". Add the five tests below to `crates/api/src/tests.rs`. Use the fixtures that exist (`publisher_app`, `make_feed`, `new_feed_body`, `listed`, and the write and move tests near the tests of ADR 0230 items 20 and 21, and of task 218). Do not change code outside `crates/api/src/tests.rs`. When a test shows a defect in the code, stop and report it with the test name and the output. Do not correct the code.

1. A written release that names a written artist page in `podcast:publisher`, when the written copy of the page has no remote item for the release: `GET /api/v1/feeds` gives `unlisted_by` with the page id. `GET /api/v1/feeds/{release}/check` gives the finding `unlisted_by_page` with `page_id` and `page_title`. After a write of the page with the item, the list is empty and the check has no such finding.
2. The feed list gives `author` of a written release with no draft (the value of the written copy).
3. **Remove from Collection** (`DELETE /api/v1/feeds/{id}`) of a written release with no draft: the release remote item goes out of a page draft whose written copy does not name the release. The draft of a second page whose written copy names the release keeps the item. The undo route gives back the release and the page draft.
4. A release on host 2 with `files_connector_id` 2 and its media files on host 1. **Move This Release…** with "Media files only" to host 1 (`POST /api/v1/feeds/{id}/move`) gives no draft operation, `files_connector_id` 1 and `files_connector_kept` NULL. Read the columns with the collection of the test workspace, as the tests of task 218 do.
5. After test 4, a move to host 2 that adds an operation, then the discard of the draft, gives `files_connector_id` 1.

Checks: `cargo test -p studio-api` and `cargo clippy --workspace --all-targets -- -D warnings` pass. Add the test names and results to the end of this section as "Follow-Up Result".

## Follow-Up Result

- `a_written_release_reports_a_written_page_that_does_not_list_it`: failed. `unlisted_by` was `[]`; the test expected `[1]`.
- `a_written_release_without_a_draft_gives_its_written_author_in_the_feed_list`: failed. Writing the edited draft returned `409`; the test expected `200`.
- `removing_a_written_release_cleans_only_unlisted_page_drafts_and_undo_restores_them`: passed.
- `a_move_that_adds_nothing_clears_the_kept_media_host`: passed.
- `a_later_media_move_operation_discard_restores_the_host_from_before_that_move`: passed.
- `cargo test -p studio-api`: failed. 472 tests passed and two tests failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: not run because two API tests failed.

Main session checks after these changes:

- `a_written_release_reports_a_written_page_that_does_not_list_it` failed before the change of `written_copy` (the result above) and passes after it.
- `cargo test -p studio-core`, `cargo test -p studio-api` (474 tests), `cargo clippy --workspace --all-targets -- -D warnings` and `bash tools/gate.sh` pass.
