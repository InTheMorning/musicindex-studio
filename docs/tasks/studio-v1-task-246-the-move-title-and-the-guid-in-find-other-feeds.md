# Studio v1 Task 246: The Move Title, And The GUID In Find Other Feeds

Source: observations 10 and 13 of `docs/reviews/2026-11-23-desktop-check-walk.md`.

Status: Done.

Operator choices (defect corrections, no ADR):

- The title of a release move names the page that the move adds. When the move adds no page, the title names the artist page, as now.
- **Find Other Feeds** marks a file as in the collection when a feed of the collection has the same GUID, also on another file host.

## Problem

- `move_release_to_pages` in `crates/api/src/routes.rs` sets `target_name` to the title of the selected artist page. A move that keeps the artist page and adds a hosting page gets the title "Move {release} to {artist}".
- `connector_feeds` in `crates/api/src/routes.rs` sets `in_collection` only when a feed of the same file host has the same path. The discover route uses `collection_keys` and `in_collection`, which also compare the GUID. **Add to Studio** can then make a second copy of a release.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Release move | extend | crates/api/src/routes.rs | `move_release_to_pages` selects `target_name`. |
| Collection match | reuse | crates/api/src/routes.rs | `collection_keys` and `in_collection` of the discover route compare the URL and the GUID. |
| Host feed list | extend | crates/api/src/routes.rs | `connector_feeds` uses the match above in addition to the path. |

## Changes

1. Read `move_release_to_pages`, `connector_feeds`, `collection_keys` and `in_collection` in `crates/api/src/routes.rs`, and the tests `the_move_route_links_three_pages_and_unlinks_the_other_artist_page`, `the_host_lists_its_feed_files_and_marks_the_ones_in_the_collection` and `discovery_follows_a_publisher_and_import_adds_read_only_feeds` in `crates/api/src/tests.rs`.
2. Observation 10: before the links change, read the pages that the release links. `target_name` is the title of the first selected page that the release does not link yet, in the order of the request. When each selected page is linked already, keep the artist page title, as now.
3. Observation 13: in `connector_feeds`, a candidate is `in_collection` when the path matches as now, or when `in_collection` of the discover route matches its `podcast_guid` (case-insensitive) or its address. Do not read more from the host.
4. Tests in `crates/api/src/tests.rs`:
   - A release that links an artist page moves to the artist page and a new hosting page. The title of the change set names the hosting page.
   - A file host has a file with the GUID of a release of the collection on another file host. The list gives `in_collection` `true` for it.
   - When an existing test fixture shares a GUID by accident, give it its own GUID. Do not change what the test proves.
   - Each failure message names the observation of the walk of 2026-11-23 and the function to fix.

## Constraints

- Change only `crates/api/src/routes.rs` and `crates/api/src/tests.rs`. Do not change `crates/core/`, a file in `docs/` or `AGENTS.md`. A second task changes `crates/core/` at the same time.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of the crate.
- Run `cargo test -p studio-api` and `tools/gate.sh` at the end, and report the results. A failure in `crates/core` can come from the other task. Report it and do not correct it.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 4 pass.
- `cargo test -p studio-api` passes.
