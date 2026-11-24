# Studio v1 Task 214: A Link That Only Studio Keeps Puts A New Artist Below Its Label

Decision: [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) items 9 to 12.

Status: Done.

## Problem

A feed is below a label only when the two feeds share a release (ADR 0215 item 2). `link_levels` in `crates/core/src/publisher.rs` near line 514 compares only the shared releases. Thus a new artist that the user makes on the level of a label cannot show below the label. Stophammer ADR 0069 rejects a link in the feed files, so Studio keeps the link in the collection only.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Collection migrations | extend | crates/core/src/collection.rs | `MIGRATIONS` near line 26. Add one migration with the new table. |
| Levels of the publisher feeds | extend | crates/core/src/publisher.rs | `catalogs` near line 397 and `link_levels` near line 514. |
| Feed list route | extend | crates/api/src/routes.rs | The route near line 1426 gives `child_ids` and `parent_ids` near line 1512. |
| Make of a feed | extend | crates/api/src/routes.rs | `NewAlbumFeed` near line 1574, `new_album_feed`, `finish_new_feed`. |
| Link of a release | extend | crates/api/src/routes.rs | `add_link` near line 2933 and `link_new_release_pages` near line 1742. |
| Remove and undo of a feed | extend | crates/core/src/people.rs | `FEED_DELETE_TABLES` near line 3611 and the predicates of `delete_snapshot` near line 3652. |
| Core tests | extend | crates/core/tests/publisher.rs | `the_levels_follow_the_rank_and_the_shared_releases` near line 628. |
| API tests | extend | crates/api/src/tests.rs | The link tests near line 1561 and the create tests near line 1672. |

## Changes

1. Add a migration with the table `kept_levels (upper_id INTEGER NOT NULL, lower_id INTEGER NOT NULL, PRIMARY KEY (upper_id, lower_id))`. Each column names a row of `feeds`. Add `Collection` functions to add a kept link, to list all kept links, and to remove one kept link.
2. Item 10: `link_levels` also puts feed B below feed A when B has a lower rank and a kept link `(A, B)` exists. Keep the rank rule and the rule "directly below". Do not change the signature of `catalogs`. Add `catalogs_with_kept(feeds, kept: &[(FeedId, FeedId)])`, and let `catalogs` call it with an empty list. The feed list route gives the kept links of the collection. Also examine the other callers of `catalogs` that read `children` or `parents`, for example `crates/core/src/change_set.rs` near line 2075. Give them the kept links when they have the collection.
3. Item 9: `POST /api/v1/feeds/new` takes the new optional field `below_id`. It is valid only for `medium` `publisher`. The feed of `below_id` must be a publisher feed with a higher rank than the role of the new feed. Else the route gives HTTP 400 with the code `invalid_below` and changes nothing. After the make, the route adds the kept link `(below_id, new id)`.
4. Item 11, first condition: after Studio links a release to a publisher feed (`add_link` and `link_new_release_pages`), it removes each kept link `(A, B)` where A and B now share a release. Use the catalogs of the drafts after the link.
5. Item 11, second condition: the remove of a feed removes each kept link that names it, as upper or as lower. Add `("kept_levels", "upper_id")` to `FEED_DELETE_TABLES` with a predicate `upper_id = ?1 OR lower_id = ?1` in `delete_snapshot`. The undo of the remove then gives the rows back. The remove with no undo (`Collection::delete_feed` near line 3090 of `crates/core/src/collection.rs`) also removes the rows.
6. Item 12: do not write the kept link in a feed file or a draft.

## Checks

Mechanical:

- Core test: a kept link `(label, artist)` with no shared release gives the artist in `children` of the label and the label in `parents` of the artist. A kept link from an artist to a label gives nothing.
- Core test: a hosting feed, a label and an artist with kept links `(hosting, label)` and `(label, artist)` give the artist directly below the label only.
- API test: `/feeds/new` with `medium` `publisher`, `role` `artist` and `below_id` of a label gives the artist in `child_ids` of the label in `/api/v1/feeds`. A `below_id` of an artist for a new artist gives HTTP 400 `invalid_below`.
- API test: a link of a release to the label and to the artist removes the kept link.
- Core test: the remove of the artist removes the kept link, and the undo gives it back. Run it on a collection after `migrate::into_library` too.
- `cargo test -p studio-core`, `cargo test -p studio-api`, `cargo clippy --workspace --all-targets` and `bash tools/gate.sh` pass.

Visual: none. Task 215 gives the UI.

Human: none.

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Write code comments and the Result section in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it.
- Change only the files of the table above and this task file. Task 215 changes `crates/api/ui/`.
- Do not make an architecture decision. Stop and report a question.
- Write the section "Result" at the end of this file: each change with file and line, and each check with its result.

## Result

- `crates/core/src/collection.rs:702, 3115`: Added the kept level table and collection operations. Feed deletion removes matching links.
- `crates/core/src/publisher.rs:403, 524`: Added catalogs that include kept links. Rank and direct-level rules still apply.
- `crates/core/src/people.rs:3611, 3666`: Feed deletion snapshots kept links for undo.
- `crates/api/src/routes.rs:1591, 1694, 1844, 3027`: Added `below_id` validation, link storage, hierarchy output, and removal after feeds share a release.
- `crates/core/tests/publisher.rs:708`: Added coverage for kept links and invalid rank direction.
- `crates/api/src/tests.rs:1732`: Added route coverage for a new artist below a label and invalid rank direction.
- `cargo test -p studio-core`: Passed.
- `cargo test -p studio-api`: Passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: Passed.
- `bash tools/gate.sh`: Passed.

Main session changes after the review:

- `kept_levels` has a foreign key with `ON DELETE CASCADE` on each column. Before, `people::remove_feed` deleted only through the cascade of `feeds`, so a kept link stayed after the remove. The explicit delete in `Collection::delete_feed` is gone, because the cascade does it.
- `validate_below_page` uses `publisher::rank`, which is now public. The copy in the route used the lowest role, not the first role of ADR 0215 item 1.
- New core test `the_remove_of_a_feed_removes_its_kept_link_and_the_undo_gives_it_back`. It also proves that a chain of kept links gives "directly below" only.
- New API test `a_shared_release_removes_the_kept_level`. Red check: without `remove_shared_kept_levels` in `add_link`, the test fails.
