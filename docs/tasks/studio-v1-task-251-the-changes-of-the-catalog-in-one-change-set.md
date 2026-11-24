# Studio v1 Task 251: The Changes Of The Catalog In One Change Set

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 21, 26 to 28, 34 to 37, 40 and 41. [ADR 0257](../adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 14 and 15.

Status: Packets 251a and 251b are done. Packet 251c is not started.

Technical choices of the main session (no ADR, because ADR 0254 gives the rule and this is how the code does it):

- The page actions move from `move_publisher_page_to_pages` in `crates/api/src/routes.rs` into one core function. The route only reads the request and calls the core (ADR 0065).
- The change set of the Catalog is a change set with a new operation kind. Each change adds its feeds and its kept links to the one open change set. A feed that is already in it keeps its first `before` draft, so Discard puts back the state before the first change.
- Undo needs the state before each change. The collection gets three tables: `catalog_steps` (one row for each change), `catalog_step_feeds` (the drafts before one change) and `catalog_step_kept_levels` (the kept links before one change). This is schema version 53. Undo puts back the last step and deletes it. When no step stays, the change set has no change and closes.
- The Catalog Undo has no Redo (ADR 0254 item 37 gives only Undo).

## Goal

Each change of the Catalog uses music words: sign to a label, change label, go independent, change publisher. Each change goes into one open change set. Undo removes the last change, and Discard removes all.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page move logic | extend | crates/api/src/routes.rs | `move_publisher_page_to_pages` becomes a call of the new core function. |
| Link and unlink | reuse | crates/core/src/publisher.rs | `link` with roles (task 250) and `unlink`. |
| Kept links | reuse | crates/core/src/collection.rs | `add_kept_level` and `remove_kept_level`. |
| Page move change set | extend | crates/core/src/change_set.rs | `record_page_move_change_set` adds to the open change set of the Catalog. |
| Discard | reuse | crates/core/src/change_set.rs | `discard_change_set` already puts back drafts and kept links. |
| Migrations | extend | crates/core/src/collection.rs | `MIGRATIONS` gets entry 53 for the step tables. |
| Gather route | extend | crates/api/src/routes.rs | `gather_pages` does the Sign and Publish of the Add sheet. |
| Change set tests | extend | crates/core/tests/change_set.rs | Undo and Discard of two changes. |
| API move tests | extend | crates/api/src/tests.rs | `an_artist_page_move_updates_its_releases_and_kept_level` and the tests near it. |

## Constraints

- Each rule is in the core (ADR 0065). This task does not change `crates/api/ui/`.
- No change of the Catalog writes a file (ADR 0254 item 34).
- A migration keeps each row of an older collection. The test `a_new_database_gets_the_schema_and_opens_again_with_no_change` passes.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md`.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Files To Read First

- ADR 0254 items 21 to 28 and 34 to 41, ADR 0257 items 14 and 15, ADR 0207 items 24 to 33, ADR 0243.
- `crates/api/src/routes.rs`: `move_publisher_page_to_pages` (near line 4039), `gather_pages` (near line 3846), `check_page_move_feeds` (near line 3610), `page_move_response` (near line 3497).
- `crates/core/src/change_set.rs`: `record_page_move_change_set` (near line 1375), `discard_change_set` (near line 2183).
- `crates/core/src/collection.rs`: `MIGRATIONS`, `add_kept_level`, `remove_kept_level`, the `change_set_kept_levels` table.
- `crates/api/src/tests.rs`: `an_artist_page_move_updates_its_releases_and_kept_level`, `a_label_page_move_keeps_artist_links_and_a_page_without_releases_moves`, `a_page_move_records_one_change_set_and_discard_restores_drafts_and_kept_links`, `a_page_move_names_a_release_in_an_open_change_set`, `gather_moves_two_pages_with_no_role_in_one_change_set`.

## Packets

### 251a: The Page Actions (Core)

1. A new core function, for example `publisher::change_page`, takes the page, the action, the selected page (or none) and the list of selected releases. The actions are `SignToLabel`, `ChangeLabel`, `GoIndependent`, `ChangePublisher`. "Choose a publisher…" uses `ChangePublisher`, and "Not published" is `ChangePublisher` with no page. `LeaveAll` is the drop on the top area of ADR 0254 item 39: "Go independent…" and "Not published" as one change and one step of Undo.
2. For each selected release (ADR 0254 item 26, ADR 0257 item 15):
   - `SignToLabel` and `ChangeLabel` remove the link to each label page and each publisher page. They add a link to the selected label with the role `label`, and a link to the one publisher that follows the label, with the role `hosting`. The publisher that follows is the publisher page directly above the label, first from A to Z (ADR 0257 item 5).
   - `GoIndependent` removes the link to each label page.
   - `ChangePublisher` removes the link to each publisher page and adds the selected publisher with the role `hosting`.
   - A link that has a removed role and also a different role keeps the different role. It loses only the removed role.
   - "Each label page" and "each publisher page" include a page outside Studio: a release item with that role in its `rel` and no feed in the collection. If not, the link of the new page gives `RoleTaken` (packet 250b). Two API tests of the move routes of ADR 0240 prevent this condition with `rel="curator"` in their fixtures (`the_move_route_links_three_pages_and_unlinks_the_other_artist_page`, `a_page_move_names_a_release_in_an_open_change_set`). Give them back the `rel` that they had before, when the new route replaces the routes of ADR 0240.
   - The links to curator pages, to the page of the action and (on a label page) to the artist pages stay.
3. ADR 0254 item 27: the kept links of the page change in the same way. `SignToLabel`, `ChangeLabel` and `ChangePublisher` replace each kept link to a page of that role. `ChangePublisher` with no page removes each kept link to a publisher page. `GoIndependent` removes each kept link to a label page.
4. A function gives the start selection of ADR 0254 item 28, with the page above as an argument: the releases that link to the page above. With no such release, each release of the page.
5. The function gives the result of ADR 0254 item 25: the place after the change, the count of changed releases, the titles of the releases that keep their links, and `nothing_changes` when the links are the same.
6. Tests in `crates/core/tests/publisher.rs`:
   - A change of label for one of two releases: the selected release loses its label link and gets the new label and its publisher. The other release keeps its links.
   - A label page `label hosting` is the selected label. The new link has `rel="label hosting"` only when that page is also the publisher that follows. Otherwise `rel="label"`.
   - A page with no releases and a kept link gets a new kept link.
   - A change to the same label gives `nothing_changes`.
   - `LeaveAll` removes the label links and the publisher links. One Undo (packet 251b) puts back the two.

### 251b: One Change Set And Undo (Core)

1. Migration 53: the tables `catalog_steps`, `catalog_step_feeds` and `catalog_step_kept_levels`, with the change set ID and a step number.
2. A new operation kind of a change set for the Catalog. `record_page_move_change_set` (or a function that it calls) adds the feeds and the kept links of a change to the open change set of the Catalog, when one exists. Otherwise it makes one. A feed that is already in the change set keeps its first `before`.
3. Each change records one step: the draft of each changed feed before the change, and the kept links before the change.
4. ADR 0254 item 34a: a change refuses a feed that is in a different open change set, with the error of ADR 0243 item 5. The open change set of the Catalog does not refuse.
5. A new function `undo_catalog_step` puts back the drafts and the kept links of the last step, and deletes the step. When no step stays, it discards the change set.
6. Tests in `crates/core/tests/change_set.rs`:
   - Two changes give one change set with the feeds of the two changes.
   - Undo after two changes gives the state after the first change. A second Undo gives the state before the first change, and the change set closes.
   - Discard after two changes gives the state before the first change, also for the kept links.
   - A feed in a recipient change set refuses a Catalog change.
   - The migration test of `crates/core/src/collection.rs` gets version 53.

### 251c: The Routes (API)

1. A route for a page action, for example `POST /api/v1/feeds/{id}/page-change`, with the action, the selected page and the selected releases. It calls packet 251a and records with packet 251b. It gives the result of item 251a.5 and the change set.
2. A route for the preview of a sheet, for example `GET /api/v1/feeds/{id}/page-change?action=…&page=…&above=…&releases=…`. `above` is the page above of ADR 0254 item 12a: the parent of the tree row where the user opened the menu. `releases` is the selected releases. With no value, the route gives the start selection of item 28. The route gives the start selection and the result with no change. The sheet uses it for "After:" and "Changes {N} releases."
3. A route for Undo, for example `POST /api/v1/catalog/undo`, and the count of changes of the open Catalog change set in the feed list or in the change set list (ADR 0254 item 36).
4. `gather_pages` (the Add sheet, ADR 0254 item 21): it gives each selected page with no role the role of the sheet. Then it does `SignToLabel` (on a label page) or `ChangePublisher` (on a publisher page) for each selected page with all its releases. It records into the change set of the Catalog. A page with no releases gets a kept link (ADR 0254 item 40).
5. The old move route of a page is removed when no caller uses it. Its tests change to the new route. `move_release_to_pages` stays for task 252.
6. Tests in `crates/api/src/tests.rs`:
   - The tests of the old page move change to the new route and the music actions.
   - A preview changes no draft.
   - An artist below two labels: the preview with each label as `above` gives a different start selection. A preview with two of three releases gives "Changes 2 releases."
   - Two changes and an Undo through the routes.
   - `gather_moves_two_pages_with_no_role_in_one_change_set` gives the Catalog change set.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes.
- `cargo test -p studio-core` and `cargo test -p studio-api` pass. `tools/gate.sh` passes. A Node test failure from a route that the UI still calls is reported, not corrected: task 254 changes the UI.
- `rg 'fn move_publisher_page_to_pages' crates/api/src/routes.rs` gives no line, or the report names the caller that keeps it.

Visual: none.

## Stop And Ask When

- A change of the Catalog needs a write of a file.
- The UI calls the old page move route, and the removal stops a Node test that this task cannot change.
- A step of Undo cannot put back a draft because the user wrote the feed. Report what the core does now.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-251-the-changes-of-the-catalog-in-one-change-set.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (251a, 251b or 251c). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- crates/api/ui/, crates/api/ui-tests/, docs/, AGENTS.md, crates/desktop/, crates/desktop-app/.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- cargo test -p studio-core
- cargo test -p studio-api
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
