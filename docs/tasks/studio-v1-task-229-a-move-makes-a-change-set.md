# Studio v1 Task 229: A Move Makes A Change Set

Decision: [ADR 0243](../adr/0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) items 1, 2, 5, 6, 11 and 12.

Status: Open.

## Problem

The move route of ADR 0240 changes many drafts and gives no change set. The move of one artist page with 21 releases gives 22 drafts, and the user writes each one alone. When a release is read-only, the refusal does not name it. A page with no role cannot move. No route moves many pages below one page.

This task changes the core and the API. Task 230 changes the UI.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Change set of the core | extend | crates/core/src/change_set.rs | `make_move_change_set` and `RecipientOperation::MoveReleases` (ADR 0148) already record a change set that is not a payee operation. Add one operation for a move of pages. |
| Change set tables | extend | crates/core/src/collection.rs | `change_sets` and `change_set_feeds` hold the earlier draft of each feed. Add the kept links of item 2. |
| Discard | extend | crates/core/src/change_set.rs | `discard_change_set` puts back each draft. It also puts back the kept links of the change set. |
| Move route | extend | crates/api/src/routes.rs | `move_release_to_pages` and `move_publisher_page_to_pages` do the checks and the links. They also make the change set. |
| Publisher link and roles | reuse | crates/core/src/publisher.rs | `publisher::link`, `publisher::unlink` and the function that sets the roles (test `set_roles_changes_the_rel_in_the_publisher_draft_and_in_each_linked_release` in crates/core/tests/publisher.rs). |
| Error details | reuse | crates/api/src/error.rs | `ApiError` has `details` for typed data. |

## Changes

1. Read ADR 0243, ADR 0240 items 9 to 25, ADR 0207 items 24 to 33 and ADR 0148 items 16 to 19 first. Read `make_move_change_set`, `record_change_set`, `discard_change_set`, `move_release_to_pages`, `move_publisher_page_to_pages` and `check_move_release`.
2. Add one `RecipientOperation` variant for a move, for example `MovePages`. It holds what the review title of ADR 0243 item 3 needs: the names of the moved pages or the title of the moved release, the name of the target page, and the role. Give it serde names in the style of the other variants.
3. Before the first link, the route finds each feed that the move changes, and keeps the draft of each one (or "no draft"), as `record_change_set` does. It also keeps each `kept_levels` row that the move adds or removes. Then it does the links. Then it records one change set with each changed feed and the outcome "changed". Add the kept links to the change set through a schema migration in `crates/core/src/collection.rs`, in the style of the earlier migrations.
4. Discard of this change set puts back each draft (byte for byte, as today) and each kept link: it removes each kept link that the move added and adds back each kept link that the move removed. A feed written after the move does not change (ADR 0207 item 27).
5. Before the first link, the route refuses the move and changes nothing in these conditions:
   - A feed of the move is in an open change set. Status 409, code `in_change_set`.
   - A feed of the move is read-only. Status 409, code `read_only` (as today).
   The `details` of each refusal has `feeds`: a list of `{ "id", "title" }` for each feed of that condition, not only the first one. For `in_change_set`, each entry also has `change_set_id`.
6. The success response of the move route gives `change_set_id`, and the earlier fields.
7. The move of a release (ADR 0240 items 9 to 19) and the move of an artist page or a label page (ADR 0240 items 20 to 25) both make a change set through steps 3 to 6.
8. A new route `POST /api/v1/feeds/{id}/gather`. `{id}` is a hosting page or a label page. The body is `{ "role": "artist" | "label", "page_ids": [..] }`. A label page accepts only `artist`. Refuse a page of a different role, a page that is below `{id}` now, and an empty list. For each page with no role, set the role of the body, as the Role field of the page form does (ADR 0082). Then apply the move of an artist page or a label page to each page, with `{id}` as the selected page of its role. The links of the other roles do not change. One change set holds every changed feed (ADR 0243 items 11 and 12). Steps 3 to 6 apply.
9. A move of an artist page or a label page with no role (`page_ids` of step 8) needs a role change in the publisher feed and in each linked release. These drafts are also in the change set.
10. Tests in `crates/api/src/tests.rs`:
    - A move of an artist page with two releases below a hosting page gives one change set. It holds the two releases, the hosting page and the artist page if its draft changes. The response has `change_set_id`.
    - Discard of that change set puts back each draft byte for byte and the kept links of before the move.
    - A move with one read-only release changes no draft and no kept link, and `details.feeds` names that release. A move with two read-only releases names both.
    - A move with a release in an open payee change set changes no draft, and gives `in_change_set` with the name and the `change_set_id`.
    - The gather route with two pages with no role gives each page the role artist, links each release to the hosting page, and gives one change set. A label page refuses `role: label`. A page below `{id}` now is refused.
    - The move of a single release gives a change set.
11. Keep the earlier move tests. Change their expected values only where a change set is now part of the result.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Do not change a file in `crates/api/ui/`. Task 230 does the UI.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0243 and an item. A test failure message names ADR 0243, the item and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065).
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The Rust tests of step 10 pass.
- `cargo test -p studio-core -p studio-api` passes.
- `tools/gate.sh` passes.

Visual: none. Task 230 has the visual criteria.

## Report

Give the changed files, the new tests, the schema version, the result of `tools/gate.sh` and `git status --short`.
