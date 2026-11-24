# Studio v1 Task 250: The Roles Of Each Link, And One Page For Each Role

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decisions: [ADR 0253](../adr/0253-the-roles-of-a-feed-and-of-each-link.md) items 15, 17, 19 to 23. [ADR 0257](../adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 7, 9, 10, 10a, 12 and 16a. [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 4, 40 and 41.

Status: Packets 250a and 250b are done. Packets 250c and 250d are not started. This is the first task of the code of ADR 0253, ADR 0254 and ADR 0257. Tasks 251 to 254 use it.

Technical choices of the main session (no ADR, because the ADRs give the rule and this is how the code does it):

- `publisher::link` gets the roles of the new link as an argument. With no roles, the link gets each level role of the set of the feed (ADR 0253 item 21). The function is extended. It is not copied.
- `publisher::set_roles` applies the difference between the old set and the new set to each link. It adds only a new role and removes only a removed role. Thus a role that the user removed from one link stays off that link (ADR 0253 item 22). The collection keeps no new value for it.

## Goal

Each link writes its own roles in `rel`. A release has at most one page of the role `artist`, `label` and `hosting`. The levels come from the roles of a link. `podcast:publisher` has the sequence publisher, label, artist.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link of two feeds | extend | crates/core/src/publisher.rs | `link` and `list_on_page` take the roles of the link. |
| Role set of a feed | extend | crates/core/src/publisher.rs | `set_roles` applies the difference of the set. |
| Role words and sequence | reuse | crates/core/src/collection.rs | `PublisherRole`, `PublisherRole::ALL`, `rel_of` and `roles_from_rel`. |
| Error kinds | extend | crates/core/src/publisher.rs | `PublisherErrorKind` gets `RoleTaken`. |
| API error map | extend | crates/api/src/error.rs | `From<PublisherError>` gives the code `role_taken`. |
| Levels and catalogs | extend | crates/core/src/publisher.rs | `rank`, `catalogs_with_kept` and `link_levels` use the roles of a link. |
| Feed list | extend | crates/api/src/routes.rs | `list_feeds` gives `roles_linked`, `child_ids` and `parent_ids` from the core. |
| Pages of a new release | extend | crates/api/src/routes.rs | `link_new_release_pages` takes the first page of each role. |
| Set role route | extend | crates/api/src/routes.rs | `set_publisher_role` gives the count of item 10a. |
| Core link tests | extend | crates/core/tests/publisher.rs | The tests named in each packet. |
| API tests | extend | crates/api/src/tests.rs | `a_new_release_keeps_its_kind_and_inherits_the_newest_artists_pages` and the link route tests. |

## Constraints

- Each rule is in the core. No feed rule in the UI (ADR 0065). This task does not change `crates/api/ui/`.
- A read and a write with no edit give the same bytes (ADR 0008, ADR 0012). The corpus tests do not change. Studio does not change the sequence of `podcast:publisher` in a file until the user changes a link of that release (ADR 0253 consequence 4).
- No test sends a request outside the computer (ADR 0045).
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of the crate.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- A test that asserts a rule of a superseded record changes or goes in the same packet (ADR 0253 "How This Decision Is Checked").

## Files To Read First

- ADR 0253, ADR 0257 items 7 to 12 and 16a, ADR 0254 items 4, 4a, 40 and 41.
- `crates/core/src/publisher.rs`: `link` (near line 322), `list_on_page`, `unlink`, `set_roles` (near line 481), `rank` (near line 578), `catalogs_with_kept` (near line 599), `link_levels` (near line 720) and `upsert` (near line 789).
- `crates/core/src/collection.rs`: `PublisherRole` (near line 965), `rel_of`, `roles_from_rel`, `inferred_roles`, `set_publisher_roles`.
- `crates/core/src/feed.rs`: `Publisher` (near line 591) and `Publisher::apply`.
- `crates/core/tests/publisher.rs`: the tests named in each packet.

## Packets

### 250a: The Roles Of A Link And The Role Set (Core)

1. `link` and `list_on_page` take `roles: Option<&[PublisherRole]>`. `None` gives each level role (hosting, label, curator, artist) of the set of the feed. `Some` gives these roles. A role that is not in the set of the feed gives the error `NoRole` (ADR 0253 item 23). Each caller in the workspace passes `None`. A packet of a subsequent task can give a value.
2. A second `link` of the same two feeds merges the roles into the `rel` of the one item. It does not add a second item (ADR 0257 item 7). The words keep the sequence of `PublisherRole::ALL`.
3. `set_roles` reads the old set. For each link of the feed, on the two sides, it adds each new role and removes each removed role. It does not change the other words of that `rel`.
4. A new function `set_link_roles(collection, publisher, release, roles)` replaces the roles of one link on its two sides (ADR 0253 item 21: the user adds a credit role to one link or removes a role from one link). Each role is in the set of the feed. An empty list gives the error `NoRole`. The user removes a link with `unlink`.
5. `changed_links` (near line 57) also gives a link whose `rel` in the draft is not the `rel` of the written file, as a changed link on each side. Review then writes the two sides after a change of roles only (ADR 0251).
6. ADR 0257 item 10a: `set_roles` does not add a role `artist`, `label` or `hosting` to a link when the release of that link has a link to a different page with that role. `set_roles` gives these releases with the role that each one keeps, for example as `Vec<(PublisherRole, FeedId)>`.
7. Tests in `crates/core/tests/publisher.rs`:
   - `link_writes_one_item_in_each_draft_with_the_same_rel` changes: a feed with the set `label producer` writes `rel="label"` on the two sides.
   - New: a link with `Some(&[Hosting])` of a feed `label hosting` writes `rel="hosting"` on the two sides.
   - New: a link of the same feeds with a second role gives one item with the two roles.
   - `set_roles_changes_the_rel_in_the_publisher_draft_and_in_each_linked_release` changes: a link with `rel="label"` of a feed `label hosting` keeps `rel="label"` when the set becomes `label hosting producer`, and gets `label producer`.
   - New: a release with a label page A. Page B gets the role `label` in its set. The link of B to that release does not get `label`, and `set_roles` gives that release with the role `label`.
   - New: `set_link_roles` adds `producer` to one link of a feed `label producer`. The other links do not change.
   - New: a change of the roles of one link only, with no other change, gives a changed link on the two sides in `changed_links`.

### 250b: One Page For Each Role And The Sequence (Core)

1. `PublisherErrorKind::RoleTaken`. `link` refuses a link that gives a release a second page of the role `artist`, `label` or `hosting` (ADR 0257 item 10). The message is "{release} already has the {artist, label or publisher} page {page}. Change it in the fields of the release." A role `curator` is not refused. The check reads the release draft, or the file when there is no draft. The check occurs before each draft change.
2. Each write of a link to the `podcast:publisher` of a release puts the items in this sequence: an item with the role `hosting`, then `label`, then `artist`, then each other item in its old sequence (ADR 0257 item 12). An item with two roles goes at the place of its first role in this sequence. Only a release that `link`, `unlink`, `set_roles` or `set_link_roles` changes gets the new sequence. A release that these functions do not change keeps its bytes.
3. A file with two pages of one role still reads and writes (ADR 0257 item 11). `link` refuses only a new second page.
4. Tests in `crates/core/tests/publisher.rs`:
   - New: a second artist page gives `RoleTaken`, and the two drafts do not change. The same for `label` and `hosting`. A second curator page passes.
   - `a_new_link_goes_last_after_the_links_of_the_release` changes to the sequence of item 12: a release with an artist link gets a hosting link, and the hosting item is first.
   - New: `set_roles` gives the role `hosting` to an artist page that is the second item of a release. The item becomes the first.
   - New: a release file with two label items reads, and a write with no edit gives the same bytes.

### 250c: The Levels Come From A Link (Core)

1. `rank` gives the rank of the first level role of a set: hosting, then label, then curator, then artist (ADR 0253 item 15). A set with only credit roles has no rank. Use `Option<u8>` or an equal type.
2. `link_levels` puts feed B below feed A when A and B link one release, and the `rel` of the link of A gives a level role with a higher rank than the rank of B. The set of A alone does not put B below A. A kept link also puts B below A (ADR 0254 item 40).
3. ADR 0254 item 41: a kept link goes when a shared release puts the lower feed below the upper feed by the rule of item 2.
4. `roles_linked` (ADR 0253 item 17) gives the first role of a feed only when one link gives it on the two sides: in the `rel` of the channel item of the feed and in the `rel` of the `podcast:publisher` item of the release.
5. Tests in `crates/core/tests/publisher.rs`:
   - `the_levels_follow_the_rank_and_the_shared_releases` gets a feed `artist hosting` that links the release of a different artist with `rel="hosting"`. The other artist is below it. Its own releases with `rel="artist hosting"` do not put a feed below it.
   - New: a label feed with a link that gives only `hosting` puts the artist below it at the publisher rank. A link with `rel="producer"` puts no feed below.
   - `a_role_that_the_two_sides_of_a_link_give_is_linked` gets a feed with two links, where only the release side of one link gives the role. That link does not count (ADR 0253 item 19).
   - `a_feed_with_the_roles_artist_and_label_is_one_feed_at_the_artist_rank` stays.
   - The kept link tests (`a_kept_link_places_a_new_feed_below_its_label`, `the_remove_of_a_feed_removes_its_kept_link_and_the_undo_gives_it_back`) pass.

### 250d: The Routes (API)

1. `crates/api/src/error.rs`: `RoleTaken` gives the HTTP status 409 and the code `role_taken`, with the message of the core.
2. `set_publisher_role` gives the list of ADR 0257 item 10a in its response, as `kept_role_releases`: for each role, the ID and the title of each release.
2a. A route for `set_link_roles`, for example `PUT /api/v1/feeds/{page}/links/{release}` with the roles.
2b. `link_found_release` (near line 1955) gives `RoleTaken` as the error of the route. It does not change it to `false` and add a channel item. The other link errors keep the fallback.
3. `link_new_release_pages` (ADR 0257 item 16a): the new release gets the artist page of the add, and the first label page and the first publisher page of the newest other release of that artist, in the sequence of its `podcast:publisher`. It passes the role of each link, so a label page `label hosting` gives the link `rel="label"` only, unless it is also the first publisher.
4. `list_feeds` gives `roles_linked`, `child_ids` and `parent_ids` from the core of packet 250c. The API has no copy of the rule.
5. Tests in `crates/api/src/tests.rs`:
   - `a_new_release_keeps_its_kind_and_inherits_the_newest_artists_pages` changes: the newest release has two label links. The new release gets only the first.
   - New: `POST /api/v1/feeds/{page}/links` with a second artist page gives 409 and `role_taken`.
   - New: the response of the set role route names a release that keeps its label page, with the role `label`.
   - New: an add of a found feed that gives a release a second artist page gives `role_taken`, and no draft changes.
   - New: the route of item 2a changes the roles of one link.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes.
- `cargo test -p studio-core` and `cargo test -p studio-api` pass.
- `tools/gate.sh` passes.
- `rg 'fn link\(' crates/core/src/publisher.rs` gives one function. No copy of `link` exists.

Visual: none. This task does not change the UI.

## Stop And Ask When

- A caller of `link` needs a role that this task does not name.
- A corpus test fails.
- The rule of item 250c.2 gives a cycle of two feeds that are each below the other.
- A route outside this task needs a change to compile, other than the argument `None`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-250-the-roles-of-each-link-and-one-page-for-each-role.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (250a, 250b, 250c or 250d). Do only that packet.

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
